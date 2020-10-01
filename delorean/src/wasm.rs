use alloc::alloc::{dealloc, Layout};
use alloc::boxed::Box;
use core::default::Default;
use core::ptr;

#[cfg(nightly)]
use core::marker::{Send, Sync};

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

    // TODO: Future backpressure
    /// Private: empty for overridden in derive
    #[doc(hidden)]
    #[inline]
    fn __hydrate(&mut self, _addr: A<Self>) {}

    // TODO: Future backpressure
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

/// Macro to create a `DeLorean<App>` reference to a statically allocated `App`.
///
/// This macro returns a value with type `DeLorean<$ty>`.
/// Use in the main thread
#[macro_export]
macro_rules! run {
    ($ty:ty) => {
        unsafe { $crate::A::run(<$ty as core::default::Default>::default()) }
    };
}

/// DeLorean for your app. Easy and safe traveling to the future in your thread and the nightly
///
/// No Send and No Sync wrapper static reference
pub struct A<I: App>(&'static Addr<I>);
pub use self::A as DeLorean;
use crate::{stc_to_ptr, Context};

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

// TODO: NEW with Reference Counter for use outside of main thread
/// Constructor and destructor
impl<I: App> Addr<I> {
    /// Make new Address for App
    ///
    /// Use at `run!` macro and for testing
    #[inline]
    fn new(a: I) -> &'static Addr<I> {
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
