//! A mostly lock-free single producer, single consumer queue.
//!
// http://www.1024cores.net/home/lock-free-algorithms/queues/non-intrusive-mpsc-node-based-queue

// TODO: test grow array implementation
use alloc::boxed::Box;

use core::{cell::UnsafeCell, ptr};

use super::unwrap;

#[derive(Debug)]
struct Node<T: Sized> {
    next: UnsafeCell<*mut Node<T>>,
    value: Option<T>,
}

/// This Queue is unsafe because only one thread can use it and it need checks atomicy at `.pop()`
#[derive(Debug)]
pub struct Queue<T: Sized> {
    head: UnsafeCell<*mut Node<T>>,
    tail: UnsafeCell<*mut Node<T>>,
}

impl<T: Sized> Node<T> {
    /// Make node
    ///
    /// Non volatile operation because Box::new is a allocation
    unsafe fn new(v: Option<T>) -> *mut Node<T> {
        Box::into_raw(Box::new(Node {
            next: UnsafeCell::new(ptr::null_mut()),
            value: v,
        }))
    }
}

impl<T: Sized> Queue<T> {
    /// Creates a new queue
    pub fn new() -> Queue<T> {
        let stub = unsafe { Node::new(None) };
        Queue {
            head: UnsafeCell::new(stub),
            tail: UnsafeCell::new(stub),
        }
    }

    /// Pushes a new value onto this queue.
    pub fn push(&self, t: T) {
        unsafe {
            // SAFETY: Non volatile operation guarantee order in operations
            let n = Node::new(Some(t));

            // SAFETY: Operate over just created and just bellow of non volatile operation
            let prev = self.head.get().replace(n);
            *(*prev).next.get() = n;
        }
    }

    /// Pops some data from this queue.
    ///
    /// # Safety
    /// Volatile.It does NOT guarantee order in operations.
    /// Need atomicity in the process
    pub unsafe fn pop(&self) -> Option<T> {
        let tail = *self.tail.get();
        let next = *(*tail).next.get();

        if next.is_null() {
            None
        } else {
            *self.tail.get() = next;
            debug_assert!((*tail).value.is_none());
            debug_assert!((*next).value.is_some());
            let ret = unwrap((*next).value.take());
            let _ = Box::from_raw(tail);
            Some(ret)
        }
    }
}

impl<T: Sized> Drop for Queue<T> {
    fn drop(&mut self) {
        unsafe {
            let mut cur = *self.tail.get();
            while !cur.is_null() {
                let next = *(*cur).next.get();
                let _ = Box::from_raw(cur);
                cur = next;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_e() {
        let q = &Queue::new();

        for i in 0..1_000_000 {
            q.push(i);
        }
        for i in 0..1_000_000 {
            loop {
                match unsafe { q.pop() } {
                    Some(j) if i == j => break,
                    Some(_) => panic!(),
                    None => {}
                }
            }
        }
        assert!(unsafe { q.pop() }.is_none());
        let q = Queue::new();

        for i in 0..1_000_000 {
            q.push(i);
        }
        drop(q);
    }
}
