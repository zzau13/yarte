#![no_std]
extern crate alloc;

use core::any::TypeId;
use core::cell::{Cell, UnsafeCell};
use core::default::Default;
use core::ptr;

use alloc::alloc::{dealloc, Layout};
use alloc::boxed::Box;
use alloc::vec::Vec;

pub use serde_json::from_str;
pub use wasm_bindgen::JsCast;
pub use web_sys as web;

pub use yarte_derive::App;
pub use yarte_helpers::helpers::{big_num_32::*, IntoCopyIterator};

mod queue;

use self::queue::Queue;

/// App are object which encapsulate state and behavior
///
/// App communicate exclusively by directional exchanging messages
/// Not implement `Clone` trait
///
/// It is recommended not to implement out of WASM Single Page Application context.
// TODO: derive
pub trait App: Default + Sized {
    type BlackBox;
    type Message: 'static;
    /// Private: empty for overridden in derive
    #[doc(hidden)]
    fn __render(&mut self, _addr: &'static Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    fn __hydrate(&mut self, _addr: &'static Addr<Self>) {}

    /// Private: empty for overridden in derive
    #[doc(hidden)]
    fn __dispatch(&mut self, _msg: Self::Message, _addr: &'static Addr<Self>) {}
}

/// The address of App
#[repr(transparent)]
pub struct Addr<A: App>(Context<A>);

#[doc(hidden)]
/// Register new Addr<`ty`> singleton
///
/// # Safety
/// Not use, can broke singleton register
pub unsafe fn __insert_singleton(ty: TypeId) -> bool {
    static mut __IS_INIT: Vec<TypeId> = Vec::new();
    let x = &mut __IS_INIT;
    if x.iter().copied().any(|x| x == ty) {
        false
    } else {
        x.push(ty);
        true
    }
}

/// Macro to create a `Addr<A>` reference to a statically allocated `App`.
/// `$ty`: App
///
/// This macro returns a value with type `&'static Addr<$ty>`.
/// # Panics
/// Have one instance
/// Only construct to target arch `wasm32`
/// ```
#[macro_export]
macro_rules! run {
    ($ty:ty) => {
        unsafe {
            if $crate::__insert_singleton(core::any::TypeId::of::<$ty>()) {
                $crate::Addr::new(<$ty as core::default::Default>::default())
            } else {
                panic!(concat!(
                    "Addr<",
                    stringify!($ty),
                    "> is a Singleton. Only have one instance"
                ));
            }
        }
    };
}

/// Constructor and destructor
impl<A: App> Addr<A> {
    /// Make new Address for App
    ///
    /// Use at `run!` macro and for testing
    ///
    /// # Safety
    /// Produce memory leaks if return reference and its copies hasn't owner
    ///
    /// # Panics
    /// Only construct in target arch `wasm32`
    pub unsafe fn new(a: A) -> &'static Addr<A> {
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
    pub unsafe fn dealloc(&'static self) {
        let p = stc_to_ptr(self);
        ptr::drop_in_place(p);
        dealloc(p as *mut u8, Layout::new::<Addr<A>>());
    }
}

/// Static ref to mutable ptr
///
/// # Safety
/// Broke `'static` lifetime and mutability
const unsafe fn stc_to_ptr<T>(t: &'static T) -> *mut T {
    t as *const T as *mut T
}

impl<A: App> Addr<A> {
    /// Sends a message
    ///
    /// The message is always queued
    pub fn send(&'static self, msg: A::Message) {
        self.0.q.push(msg);
        self.update();
    }

    /// Hydrate app
    /// Link events and save closures
    ///
    /// # Safety
    /// Produce **unexpected behaviour** if launched more than one time
    #[inline]
    pub unsafe fn hydrate(&'static self) {
        debug_assert!(!self.0.ready.get());
        // Only run one time
        self.0.app.get().as_mut().unwrap().__hydrate(self);
        self.0.ready.replace(true);
        self.update();
    }

    #[inline]
    fn update(&'static self) {
        if self.0.ready.get() {
            self.0.ready.replace(false);
            // UB is checked by ready Cell
            let app = unsafe { self.0.app.get().as_mut().unwrap() };
            while let Some(msg) = self.0.q.pop() {
                app.__dispatch(msg, self);
                while let Some(msg) = self.0.q.pop() {
                    app.__dispatch(msg, self);
                }
                app.__render(self);
            }
            self.0.ready.replace(true);
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
}
