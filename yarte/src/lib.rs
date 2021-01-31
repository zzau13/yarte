//! Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine,
//! is the fastest template engine. Uses a Handlebars-like syntax,
//! well known and intuitive for most developers. Yarte is an optimized, and easy-to-use
//! rust crate, can create logic around their templates using conditionals, loops,
//! rust code and templates composition.
//!
//! [Yarte book](https://yarte.netlify.com)
//!
use std::fmt::{self, Write};

pub use yarte_derive::auto;
#[cfg(all(
    any(feature = "bytes-buf", feature = "bytes-buf-tokio3"),
    feature = "html-min"
))]
pub use yarte_derive::ywrite_min;
pub use yarte_derive::{yformat, yformat_html};
#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
pub use yarte_derive::{ywrite, ywrite_html};
pub use yarte_helpers::at_helpers::*;
pub use yarte_helpers::{
    helpers::{
        display_fn::DisplayFn, io_fmt::IoFmt, Aligned256, IntoCopyIterator, Render, RenderA,
    },
    recompile, Error, Result,
};

/// Template trait, will implement by derive `fmt::Display`
pub trait TemplateTrait: fmt::Display {
    /// which will write this template
    fn call(&self) -> Result<String> {
        let mut buf = String::with_capacity(Self::size_hint());
        write!(buf, "{}", self).map(|_| buf)
    }

    /// Approximation of output size used in method `call`.
    /// Yarte implements an heuristic algorithm of allocation.
    fn size_hint() -> usize;
}

pub use yarte_derive::{Template, TemplateText};
pub use TemplateTrait as Template;
pub use TemplateTrait as TemplateText;

#[cfg(feature = "html-min")]
pub use yarte_derive::TemplateMin;
#[cfg(feature = "html-min")]
pub use TemplateTrait as TemplateMin;

#[cfg(feature = "wasm")]
pub use yarte_derive::TemplateWasmServer;
#[cfg(feature = "wasm")]
pub use TemplateBytesTrait as TemplateWasmServer;

#[cfg(feature = "fixed")]
/// Template trait
pub trait TemplateFixedTrait {
    /// Writes to buffer
    ///
    /// # Safety
    /// Not respect the lifetime bounds it's possible borrow mut when it's borrow
    /// ```rust,ignore
    /// # const N: usize = 1;
    /// let buf = TemplateFixedTrait::call(&mut [MaybeUninit::uninit(); N]).expect("buffer overflow");
    /// ```
    unsafe fn call<'call>(
        &self,
        buf: &'call mut [std::mem::MaybeUninit<u8>],
    ) -> Option<&'call [u8]>;

    /// Writes to buffer and drop
    ///
    /// # Safety
    /// Not respect the lifetime bounds it's possible borrow mut when it's borrow
    /// ```rust,ignore
    /// # const N: usize = 1;
    /// let buf = TemplateFixedTrait::ccall(&mut [MaybeUninit::uninit(); N]).expect("buffer overflow");
    /// ```
    unsafe fn ccall(self, buf: &mut [std::mem::MaybeUninit<u8>]) -> Option<&[u8]>;
}

#[cfg(feature = "fixed")]
pub use yarte_derive::{TemplateFixed, TemplateFixedText};
#[cfg(feature = "fixed")]
pub use yarte_helpers::helpers::{RenderFixed, RenderFixedA, RenderSafe, RenderSafeA};
#[cfg(feature = "fixed")]
pub use TemplateFixedTrait as TemplateFixed;
#[cfg(feature = "fixed")]
pub use TemplateFixedTrait as TemplateFixedText;

#[cfg(all(feature = "fixed", feature = "html-min"))]
pub use yarte_derive::TemplateFixedMin;
#[cfg(all(feature = "fixed", feature = "html-min"))]
pub use TemplateFixedTrait as TemplateFixedMin;

#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
/// Template trait
pub trait TemplateBytesTrait {
    /// Writes to buffer and return it freeze
    ///
    /// # Panics
    /// Render length overflows usize
    fn call<B: Buffer>(&self, capacity: usize) -> B::Freeze;
    /// Writes to buffer and return it freeze and drop
    ///
    /// # Panics
    /// Render length overflows usize
    fn ccall<B: Buffer>(self, capacity: usize) -> B::Freeze;
    /// Writes to buffer
    ///
    /// # Panics
    /// Render length overflows usize
    fn write_call<B: Buffer>(&self, buf: &mut B);
    /// Writes to buffer and drop
    ///
    /// # Panics
    /// Render length overflows usize
    fn write_ccall<B: Buffer>(self, buf: &mut B);
}

#[cfg(all(
    any(feature = "bytes-buf", feature = "bytes-buf-tokio3"),
    feature = "html-min"
))]
pub use yarte_derive::TemplateBytesMin;
#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
pub use yarte_derive::{TemplateBytes, TemplateBytesText};
#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
pub use TemplateBytesTrait as TemplateBytes;
#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
pub use TemplateBytesTrait as TemplateBytesText;
#[cfg(all(
    any(feature = "bytes-buf", feature = "bytes-buf-tokio3"),
    feature = "html-min"
))]
pub use TemplateBytesTrait as TemplateBytesMin;

#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3"))]
pub use yarte_helpers::helpers::{RenderBytes, RenderBytesA, RenderBytesSafe, RenderBytesSafeA};

#[cfg(feature = "bytes-buf")]
pub use buf_min::t2::{Bytes, BytesMut};
#[cfg(any(feature = "bytes-buf", feature = "bytes-buf-tokio3", feature = "json"))]
pub use buf_min::Buffer;

#[cfg(all(feature = "bytes-buf-tokio3", not(feature = "bytes-buf")))]
pub use buf_min::t3::{Bytes, BytesMut};

#[cfg(feature = "json")]
pub use yarte_derive::Serialize;
#[cfg(feature = "json")]
pub use yarte_helpers::helpers::json::{Serialize, *};
