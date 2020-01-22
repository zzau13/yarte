//! A mostly lock-free single-producer, single consumer queue.
//!
// http://www.1024cores.net/home/lock-free-algorithms
//                         /queues/non-intrusive-mpsc-node-based-queue

// NOTE: this implementation is lifted from the standard library and
//      modified for single thread
// Unsafe only use in single thread environment
use std::{cell::UnsafeCell, ptr};

#[derive(Debug)]
struct Node<T> {
    next: UnsafeCell<*mut Node<T>>,
    value: Option<T>,
}

/// This Queue is unsafe because only one thread can use it at a time
#[derive(Debug)]
pub struct Queue<T> {
    head: UnsafeCell<*mut Node<T>>,
    tail: UnsafeCell<*mut Node<T>>,
}

impl<T> Node<T> {
    unsafe fn new(v: Option<T>) -> *mut Node<T> {
        Box::into_raw(Box::new(Node {
            next: UnsafeCell::new(ptr::null_mut()),
            value: v,
        }))
    }
}

impl<T> Queue<T> {
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
            let n = Node::new(Some(t));
            let prev = self.head.get().replace(n);
            *(*prev).next.get() = n;
        }
    }

    /// Pops some data from this queue.
    pub fn pop(&self) -> Option<T> {
        unsafe {
            let tail = *self.tail.get();
            let next = *(*tail).next.get();

            if next.is_null() {
                None
            } else {
                *self.tail.get() = next;
                debug_assert!((*tail).value.is_none());
                debug_assert!((*next).value.is_some());
                let ret = (*next).value.take().unwrap();
                drop(Box::from_raw(tail));
                Some(ret)
            }
        }
    }
}

impl<T> Drop for Queue<T> {
    fn drop(&mut self) {
        unsafe {
            let mut cur = *self.tail.get();
            while !cur.is_null() {
                let next = (*cur).next.get();
                drop(Box::from_raw(cur));
                cur = *next;
            }
        }
    }
}

#[cfg(test)]
mod test {
    use std::rc::Rc;
    use wasm_bindgen_futures::spawn_local;
    use wasm_bindgen_test::*;

    use super::*;

    #[wasm_bindgen_test]
    fn test() {
        let q = Rc::new(Queue::new());
        let q1 = Rc::clone(&q);

        spawn_local(async move {
            for _ in 0..100_000 {
                loop {
                    match q1.pop() {
                        Some(1) => break,
                        Some(_) => panic!(),
                        None => {}
                    }
                }
            }
        });
        for _ in 0..100_000 {
            q.push(1);
        }
    }
}
