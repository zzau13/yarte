//! # Yarte wasm application
//! A simple reactor pattern
//!
//! Intended to be used as a singleton and static with single state
//!
//! Only 101% rust safe in nightly
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
//! ### Virtual DOM and differences in tree
//! The virtual DOM model and the difference calculation in the node tree
//! differs from all previous implementations in many ways.
//! The main aspect that is not found in any other implementation
//! is that it accumulates the changes in a tree of differences
//! equivalent space dimension is less or equal than
//! html node tree equivalent space dimension.
//!
//! It's compiled with static linking of the modules that allows to
//! detect local fixed points at compile time and change the base of the
//! difference linear map. That is, instead of making the difference in the html nodes,
//! it make the difference on a tree of differences write at process line.
//!
//! With what allows a reduction of the dimension of the domain and strong optimizations in parallel.
//!
//! ### Unsafe code and controversies
//! #### Why no RC?
//! Because you don't need it because it is thinking to be implemented as singleton and static.
//!
//! ### Whe no RefCell?
//! Because you don't need it because all uniques (mutable) references are made in atomic functions,
//! `run!` is designed for assure **unique** owner of **all** `App` is `Addr` and its unique safe method
//! is `send`
//!
//! #### Why no backpressure?
//! Because you don't need it because you don't need a runtime to poll wasm futures.
//! Backpressure can be implemented for future it is needed and carry static reference to the
//! Address of the App.
//!
//! #### Why doubly-linked list?
//! Is simpler than grow array implementation and it will never be a bottleneck in a browser.
//! But in the future it can be implemented.
//!
#![no_std]
#![cfg_attr(nightly, feature(core_intrinsics, negative_impls))]

extern crate alloc;

use alloc::boxed::Box;
use core::cell::{Cell, UnsafeCell};
use core::default::Default;
#[cfg(nightly)]
use core::marker::{Send, Sync};

#[cfg(debug_assertions)]
use alloc::alloc::{dealloc, Layout};
#[cfg(debug_assertions)]
use core::ptr;

mod queue;

use self::queue::Queue;

/// App are object which encapsulate state and behavior
///
/// App communicate exclusively by directional exchanging messages.
///
/// It is recommended not to implement out of WASM Single Page Application context.
// TODO: derive
pub trait App: Default + 'static {
    type BlackBox;
    type Message: 'static;
    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __render(&mut self, _addr: A<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, _addr: A<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __dispatch(&mut self, _msg: Self::Message, _addr: A<Self>) {}
}

/// The address of App
pub struct Addr<A: App>(pub(crate) Context<A>);

#[cfg(not(debug_assertions))]
impl<A: App> Drop for Addr<A> {
    fn drop(&mut self) {
        panic!("drop app")
    }
}

/// Macro to create a `A<App: App>` reference to a statically allocated `App`.
///
/// This macro returns a value with type `&'static Addr<$ty>`.
///
/// # Panics
/// Have one type instance
/// Only construct to target arch `wasm32`
///
/// ```ignore
/// #[derive(App)]
/// #[template(path = "index")
/// #[msg(enum Msg { Inc, Reset })]
/// struct MyApp {
///     count: usize,
///     bb: <Self as App>::BlackBox,
/// }
///
/// fn inc(app: &mut MyApp, _addr: &Addr<MyApp>) {
///     set_count!(app, app.count + 1);
/// }
///
/// fn reset(app: &mut MyApp, _addr: &Addr<MyApp>) {
///     if app.count != 0 {
///         set_count!(app, 0);
///     }
/// }
///
/// let addr = run!(MyApp);
/// addr.send(Msg::Reset);
/// ```
#[macro_export]
macro_rules! run {
    ($ty:ty) => {
        unsafe { $crate::A::run(<$ty as core::default::Default>::default()) }
    };
}

/// No Send and No Sync wrapper static reference
pub struct A<I: App>(&'static Addr<I>);

#[cfg(nightly)]
impl<I: App> !Send for A<I> {}
#[cfg(nightly)]
impl<I: App> !Sync for A<I> {}

impl<I: App> Clone for A<I> {
    #[inline(always)]
    fn clone(&self) -> Self {
        A(self.0)
    }
}

impl<I: App> Copy for A<I> {}

impl<I: App> A<I> {
    /// Make new Address for App and run it
    ///
    /// # Panics
    /// Only run it in target arch `wasm32`
    ///
    /// # Safety
    /// Can broke needed atomicity of unique references and queue pop
    pub unsafe fn run(a: I) -> A<I> {
        let addr = A(Addr::new(a));
        // SAFETY: only run one time
        addr.hydrate();
        addr
    }

    /// Dealloc Address
    ///
    /// Use for testing
    ///
    /// # Safety
    /// Broke `'static` lifetime and all copies are nothing,
    /// World could explode
    #[cfg(debug_assertions)]
    pub unsafe fn dealloc(self) {
        self.0.dealloc();
    }

    /// Sends a message
    ///
    /// The message is always queued
    pub fn send(self, msg: I::Message) {
        self.ctx().push(msg);
        self.update();
    }

    /// Hydrate app
    /// Link events and save closures
    ///
    /// # Safety
    /// Produce **unexpected behaviour** if launched more than one time
    #[inline]
    unsafe fn hydrate(self) {
        let ctx = self.ctx();
        debug_assert!(!ctx.is_ready());
        ctx.app().__hydrate(self);
        ctx.ready(true);
    }

    #[inline]
    fn update(self) {
        let ctx = self.ctx();
        if ctx.is_ready() {
            ctx.ready(false);
            // SAFETY: UB is checked by ready Cell
            unsafe {
                while let Some(msg) = ctx.pop() {
                    ctx.app().__dispatch(msg, self);
                    while let Some(msg) = ctx.pop() {
                        ctx.app().__dispatch(msg, self);
                    }
                    ctx.app().__render(self);
                }
            }
            ctx.ready(true);
        }
    }

    #[inline]
    fn ctx(&self) -> &Context<I> {
        &(self.0).0
    }
}

/// Constructor and destructor
impl<I: App> Addr<I> {
    /// Make new Address for App
    ///
    /// Use at `run!` macro and for testing
    ///
    /// # Panics
    /// Only construct in target arch `wasm32`
    #[inline]
    fn new(a: I) -> &'static Addr<I> {
        if cfg!(not(target_arch = "wasm32")) {
            panic!("Only construct in 'wasm32'");
        }
        Box::leak(Box::new(Addr(Context::new(a))))
    }

    /// Dealloc Address
    ///
    /// Use for testing
    ///
    /// # Safety
    /// Broke `'static` lifetime
    #[cfg(debug_assertions)]
    pub(crate) unsafe fn dealloc(&'static self) {
        let p = stc_to_ptr(self);
        ptr::drop_in_place(p);
        dealloc(p as *mut u8, Layout::new::<Addr<I>>());
    }
}

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
#[cfg(debug_assertions)]
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
