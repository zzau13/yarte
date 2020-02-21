//! Yarte stands for **Y**et **A**nother **R**ust **T**emplate **E**ngine,
//! is the fastest template engine. Uses a Handlebars-like syntax,
//! well known and intuitive for most developers. Yarte is an optimized, and easy-to-use
//! rust crate, can create logic around their templates using using conditionals, loops,
//! rust code and templates composition.
//!
//! Also Yarte incorporates feature `with-actix-web`, an implementation of `actix-web`'s
//! trait Responder for those using this framework.
//!
//! [Yarte book](https://yarte.netlify.com)
//!

pub use yarte_derive::Template;
pub use yarte_helpers::{helpers::Render, recompile, Error, Result};
pub use yarte_template::Template;

#[cfg(feature = "with-actix-web")]
pub mod aw {
    pub use actix_web::{
        error::ErrorInternalServerError, Error, HttpRequest, HttpResponse, Responder,
    };
    pub use futures::future::{err, ok, Ready};
}

#[cfg(feature = "wasm")]
pub mod serde_json {
    pub use serde_json::to_string;
}
