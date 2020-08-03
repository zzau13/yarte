//! # Yarte wasm application
//! A simple single thread reactor pattern with a doubly-linked list as dequeue.
//!
//! Intended to be used as a singleton and static.
//!
//! ## Architecture
//! ### Cycle
//! The cycle of App methods is:
//! - `init`:
//!     - `__hydrate(&mut self, _addr: &'static Addr<Self>)`
//! - `on message`:
//!     - enqueue message
//!     - is ready? -> `update`
//! - `update`
//!     - pop message? -> `__dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>)`
//!     - is queue empty?  -> `__render(&mut self, _addr: &'static Addr<Self>)`
//!     - is queue not empty? -> `update`
//!
//! ### Virtual DOM and differences in tree
//! The virtual DOM model and the difference calculation in the node tree
//! It differs from all previous implementations in many ways.
//! The main aspect that is not found in any other implementation
//! is that it accumulates the changes in a tree of differences
//! with dimension less or equal to the html node tree.
//!
//! It is simply built through a static link of the modules that allows us
//! detect local fixed points at compile time and change the base of the
//! difference application. That is, instead of making the difference in the html nodes,
//! we make the difference on a tree of differences in the process line.
//!
//! With what allows a reduction of the dimension of the domain and strong optimizations in parallel.
//!
//! ### Unsafe code and controversies
//! #### Why no RC?
//! Because you don't need it because it is thinking to be implemented as singleton and static.
//!
//! ### Whe no RefCell?
//! Because you don't need it because all uniques (mutable) references are made in atomic functions,
//! `run!` is designed for assure **unique** owner of **all** `App` is `Addr` and the unique safe method
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
extern crate alloc;

// TODO: core_intrinsics
use std::hint::unreachable_unchecked;

use core::cell::{Cell, UnsafeCell};
use core::default::Default;
#[cfg(debug_assertions)]
use core::ptr;

#[cfg(debug_assertions)]
use alloc::alloc::{dealloc, Layout};
use alloc::boxed::Box;

mod queue;

use self::queue::Queue;

/// App are object which encapsulate state and behavior
///
/// App communicate exclusively by directional exchanging messages.
///
/// It is recommended not to implement out of WASM Single Page Application context.
// TODO: derive
pub trait App: Default + Sized {
    type BlackBox;
    type Message: 'static;
    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __render(&mut self, _addr: &'static Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, _addr: &'static Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>) {}
}

/// The address of App
#[repr(transparent)]
pub struct Addr<A: App>(Context<A>);

/// Macro to create a `Addr<A: App>` reference to a statically allocated `App`.
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
        unsafe { $crate::Addr::run(<$ty as core::default::Default>::default()) }
    };
}

/// Constructor and destructor
impl<A: App> Addr<A> {
    /// Make new Address for App
    ///
    /// Use at `run!` macro and for testing
    ///
    /// # Panics
    /// Only construct in target arch `wasm32`
    #[inline]
    fn new(a: A) -> &'static Addr<A> {
        if cfg!(not(target_arch = "wasm32")) {
            panic!("Only construct in 'wasm32'");
        }
        Box::leak(Box::new(Addr(Context::new(a))))
    }

    /// Make new Address for App and run it
    ///
    /// # Panics
    /// Only run it in target arch `wasm32`
    ///
    /// # Safety
    /// Can broke needed atomicity of unique references and queue pop
    pub unsafe fn run(a: A) -> &'static Addr<A> {
        let addr = Self::new(a);
        // SAFETY: only run one time at the unique constructor
        addr.hydrate();
        addr
    }

    /// Dealloc Address
    ///
    /// Use for testing
    ///
    /// # Safety
    /// Broke `'static` lifetime
    #[cfg(debug_assertions)]
    pub unsafe fn dealloc(&'static self) {
        let p = stc_to_ptr(self);
        ptr::drop_in_place(p);
        dealloc(p as *mut u8, Layout::new::<Addr<A>>());
    }

    /// Sends a message
    ///
    /// The message is always queued
    pub fn send(&'static self, msg: A::Message) {
        self.0.push(msg);
        self.update();
    }

    /// Hydrate app
    /// Link events and save closures
    ///
    /// # Safety
    /// Produce **unexpected behaviour** if launched more than one time
    #[inline]
    unsafe fn hydrate(&'static self) {
        debug_assert!(!self.0.ready.get());
        self.0.app().__hydrate(self);
        self.0.ready(true);
    }

    #[inline]
    fn update(&'static self) {
        if self.0.is_ready() {
            self.0.ready(false);
            // SAFETY: UB is checked by ready Cell
            unsafe {
                while let Some(msg) = self.0.pop() {
                    self.0.app().__dispatch(msg, self);
                    while let Some(msg) = self.0.pop() {
                        self.0.app().__dispatch(msg, self);
                    }
                    self.0.app().__render(self);
                }
            }
            self.0.ready(true);
        }
    }
}

/// Encapsulate inner context of the App
pub struct Context<A: App> {
    app: UnsafeCell<A>,
    q: Queue<A::Message>,
    ready: Cell<bool>,
}

impl<A: App> Context<A> {
    fn new(app: A) -> Self {
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
    unsafe fn app(&self) -> &mut A {
        &mut *self.app.get()
    }

    /// Set ready
    #[inline]
    fn ready(&self, r: bool) {
        self.ready.replace(r);
    }

    /// Is ready
    #[inline]
    fn is_ready(&self) -> bool {
        self.ready.get()
    }

    /// Enqueue message
    #[inline]
    fn push(&self, msg: A::Message) {
        self.q.push(msg);
    }

    /// Pop message
    ///
    /// # Safety
    /// Only call in a atomic function
    #[inline]
    unsafe fn pop(&self) -> Option<A::Message> {
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
unsafe fn unwrap<T>(o: Option<T>) -> T {
    o.unwrap_or_else(|| unreachable_unchecked())
}
