use alloc::alloc::{dealloc, Layout};
use alloc::boxed::Box;
use core::default::Default;
use core::future::Future;
use core::pin::Pin;
use core::task::{self, Poll};
use core::{mem, ptr};

// Are auto implement. Is for document
#[cfg(nightly)]
use core::marker::{Send, Sync};

use crate::{stc_to_ptr, Context};

pub type Return<T> = Pin<Box<dyn Future<Output = T> + 'static>>;

/// App are object which encapsulate state and behavior
///
/// App communicate exclusively by directional exchanging messages.
// TODO: derive
pub trait App: Default + 'static {
    type BlackBox;
    type Output: 'static;
    type Message: 'static;

    /// TODO
    fn __render(&mut self, _addr: A<Self>) {}

    /// TODO
    fn __hydrate(&mut self, addr: A<Self>) -> Return<Self::Output>;

    /// TODO
    fn __dispatch(&mut self, msg: Self::Message, addr: A<Self>);
}

/// The address of App
pub struct Addr<A: App>(pub(crate) Context<A>);

/// Macro to create a `Runtime<App>`
///
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
    /// # Safety
    /// Can broke needed atomicity of unique references and queue pop
    pub unsafe fn run(a: I) -> Runtime<I> {
        let addr = A(Addr::new(a));
        // SAFETY: only run one time
        let hydrate = addr.hydrate();
        Runtime(hydrate, Dispatch(addr))
    }

    /// Dealloc Address
    /// Probably if you are here, you do NOT need this method.
    ///
    /// Use in testing or for awesome fail recovery features
    ///
    /// # Safety
    /// Broke `'static` lifetime and all copies are nothing.
    /// Make sure your runtime is ended BEFORE dealloc app.
    /// World could explode
    pub unsafe fn dealloc(self) {
        self.0.dealloc();
    }

    /// Sends a message
    ///
    /// The message is always queued
    pub fn send(self, msg: I::Message) {
        self.ctx().push(msg);
    }

    /// Hydrate app
    #[inline]
    unsafe fn hydrate(self) -> Hydrate<I> {
        let ctx = self.ctx();
        debug_assert!(!ctx.is_ready());
        // SAFETY: only use one time
        let fut = ctx.app().__hydrate(self);
        ctx.ready(true);
        Hydrate(fut)
    }

    #[inline]
    fn ctx(&self) -> &Context<I> {
        &(self.0).0
    }
}

/// Recover the owner of app and dealloc Context
///
/// Basically you can recovery your App with a version of your current state
///
/// # Safety
/// Ultra unsafe function, NEVER call inside your App, Only Outside.
/// Current is replace for `Default::default()`, check it's correctly with yours expected behaviour
/// Make sure to ALL your runtime is ended.
///
/// Drop all your old `DeLorean<I>` references
///
pub unsafe fn recovery<I: App>(addr: A<I>) -> I {
    let app = addr.0;
    let app = (app.0).app();
    let app = mem::take(app);

    addr.dealloc();
    app
}

/// TODO
pub struct Runtime<A: App>(Hydrate<A>, Dispatch<A>);

#[cfg(nightly)]
impl<A: App> !Send for Runtime<A> {}
#[cfg(nightly)]
impl<A: App> !Sync for Runtime<A> {}

impl<A: App> Future for Runtime<A> {
    type Output = A::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let Runtime(hydrate, dispatch) = self.get_mut();
        match Pin::new(hydrate).poll(cx) {
            // Dispatch ever return Pending
            Poll::Pending => Pin::new(dispatch).poll(cx),
            Poll::Ready(a) => Poll::Ready(a),
        }
    }
}

/// TODO
pub struct Hydrate<A: App>(Return<A::Output>);

impl<A: App> Future for Hydrate<A> {
    type Output = A::Output;

    fn poll(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        (self.get_mut().0).as_mut().poll(cx)
    }
}

/// TODO
pub struct Dispatch<A: App>(DeLorean<A>);

impl<A: App> Future for Dispatch<A> {
    type Output = A::Output;

    fn poll(self: Pin<&mut Self>, _cx: &mut task::Context<'_>) -> Poll<Self::Output> {
        let addr = self.0;
        let ctx = &(addr.0).0;
        if ctx.is_ready() {
            ctx.ready(false);
            // SAFETY: UB is checked by ready Cell
            unsafe {
                while let Some(msg) = ctx.pop() {
                    ctx.app().__dispatch(msg, addr);
                    while let Some(msg) = ctx.pop() {
                        ctx.app().__dispatch(msg, addr);
                    }
                    ctx.app().__render(addr);
                }
            }
            ctx.ready(true);
        }
        Poll::Pending
    }
}

// TODO: NEW with Reference Counter for use outside of main thread
/// Constructor and destructor
impl<I: App> Addr<I> {
    /// Make new Address for App
    #[inline]
    fn new(a: I) -> &'static Addr<I> {
        Box::leak(Box::new(Addr(Context::new(a))))
    }

    /// Dealloc Address
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
