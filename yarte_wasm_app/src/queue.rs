/* Copyright (c) 2010-2011 Dmitry Vyukov. All rights reserved.
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions are
 * met: */
//
//    1. Redistributions of source code must retain the above copyright notice,
//       this list of conditions and the following disclaimer.
//
//    2. Redistributions in binary form must reproduce the above copyright
//       notice, this list of conditions and the following disclaimer in the
//       documentation and/or other materials provided with the distribution.
//
// THIS SOFTWARE IS PROVIDED BY DMITRY VYUKOV "AS IS" AND ANY EXPRESS OR IMPLIED
// WARRANTIES, INCLUDING, BUT NOT LIMITED TO, THE IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS FOR A PARTICULAR PURPOSE ARE DISCLAIMED. IN NO
// EVENT SHALL DMITRY VYUKOV OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
// INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES (INCLUDING, BUT NOT
// LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR SERVICES; LOSS OF USE, DATA,
// OR PROFITS; OR BUSINESS INTERRUPTION) HOWEVER CAUSED AND ON ANY THEORY OF
// LIABILITY, WHETHER IN CONTRACT, STRICT LIABILITY, OR TORT (INCLUDING
// NEGLIGENCE OR OTHERWISE) ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE,
// EVEN IF ADVISED OF THE POSSIBILITY OF SUCH DAMAGE.
//
// The views and conclusions contained in the software and documentation are
// those of the authors and should not be interpreted as representing official
// policies, either expressed or implied, of Dmitry Vyukov.
//

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
