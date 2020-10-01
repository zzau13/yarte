//! # DeLorean
//!
//! A simple single thread reactor pattern implementation.
//! Intended to be used as a singleton and static with single state
//!
//! ## Architecture
//! ### Cycle
//! The cycle of App methods is:
//! - `init`:
//!     - `__hydrate(&mut self, _addr: A<Self>)`
//! - `on message`:
//!     - enqueue message
//!     - is ready? -> `update`
//! - `update`
//!     - pop message? -> `__dispatch(&mut self, _msg: Self::Message, _addr: A<Self>)`
//!     - is queue empty?  -> `__render(&mut self, _addr: A<Self>)`
//!     - is queue not empty? -> `update`
//!
//! ### Unsafe code and controversies
//! #### Why no RC?
//! Because it is thinking to be implemented as singleton and static.
//!
//! ### Whe no RefCell?
//! Because all uniques (mutable) references are made in atomic instructions,
//! `run!` is designed for assure **unique** owner of **all** `App` is `Runtime<A>`
//!
#![no_std]
#![cfg_attr(nightly, feature(core_intrinsics, negative_impls))]

extern crate alloc;

use core::cell::{Cell, UnsafeCell};

use crate::queue::Queue;

mod queue;

#[cfg(target_arch = "wasm32")]
#[path = "wasm.rs"]
mod inner;

#[cfg(not(target_arch = "wasm32"))]
#[path = "native.rs"]
mod inner;

pub use inner::*;

/// Encapsulate inner context of the App
pub struct Context<A: App> {
    app: UnsafeCell<A>,
    q: Queue<A::Message>,
    ready: Cell<bool>,
}

impl<A: App> Context<A> {
    pub(crate) fn new(app: A) -> Self {
        Self {
            app: UnsafeCell::new(app),
            q: Queue::new(),
            ready: Cell::new(false),
        }
    }

    /// Get unsafe mutable reference of A
    ///
    /// # Safety
    /// Unchecked null pointer and broke mutability
    #[inline]
    #[allow(clippy::mut_from_ref)]
    pub(crate) unsafe fn app(&self) -> &mut A {
        &mut *self.app.get()
    }

    /// Set ready
    #[inline]
    pub(crate) fn ready(&self, r: bool) {
        self.ready.replace(r);
    }

    /// Is ready
    #[inline]
    pub(crate) fn is_ready(&self) -> bool {
        self.ready.get()
    }

    /// Enqueue message
    #[inline]
    pub(crate) fn push(&self, msg: A::Message) {
        self.q.push(msg);
    }

    /// Pop message
    ///
    /// # Safety
    /// Only call in a atomic function
    #[inline]
    pub(crate) unsafe fn pop(&self) -> Option<A::Message> {
        self.q.pop()
    }
}

/// Static ref to mutable ptr
///
/// # Safety
/// Broke `'static` lifetime and mutability
const unsafe fn stc_to_ptr<T>(t: &'static T) -> *mut T {
    t as *const T as *mut T
}

/// unchecked unwrap
///
/// # Safety
/// `None` produce UB
#[inline]
#[cfg(not(nightly))]
unsafe fn unwrap<T>(o: Option<T>) -> T {
    o.unwrap()
}

/// unchecked unwrap
///
/// # Safety
/// `None` produce UB
#[inline]
#[cfg(nightly)]
unsafe fn unwrap<T>(o: Option<T>) -> T {
    o.unwrap_or_else(|| core::intrinsics::unreachable())
}
