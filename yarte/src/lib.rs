//!
//! Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine,
//! is the fastest template engine. Uses a Handlebars-like syntax,
//! well known and intuitive for most developers. Yarte is an optimized, and easy-to-use
//! rust crate, can create logic around their templates using using conditionals, loops,
//! rust code and templates composition.
//!
//! Also Yarte incorporates feature `with-actix-web`, an implementation of `actix-web`'s
//! trait Responder for those using this framework.
//!
//! Yarte book at https://yarte.netlify.com/
//!

use std::fmt::{self, Write};

pub use yarte_derive::Template;
pub use yarte_helpers::{helpers::Render, Error, Result};

pub mod recompile;

/// Template trait, will implement by derive like `Display` or `actix_web::Responder` (with feature)
///
/// ```rust
/// use yarte::Template;
///
/// #[derive(Template)]
/// #[template(src="Hello, {{ name }}!")]
/// struct HelloTemplate<'a> {
///     name: &'a str,
/// }
///
///# fn main() {
/// println!("{}", HelloTemplate { name: "world" })
///# }
/// ```
///
pub trait Template: fmt::Display {
    /// which will write this template
    fn call(&self) -> Result<String> {
        let mut buf = String::with_capacity(Self::size_hint());
        write!(buf, "{}", self).map(|_| buf)
    }

    /// https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types
    fn mime() -> &'static str
    where
        Self: Sized;

    /// Approximation of output size used in method `call`.
    /// Yarte implements an heuristic algorithm of allocation.
    fn size_hint() -> usize;
}

#[cfg(feature = "with-actix-web")]
pub mod aw {
    pub use actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
    pub use futures::future::{err, ok, FutureResult};
}
