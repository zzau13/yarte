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
#[cfg(any(not(target_arch = "wasm32"), debug_assertions))]
pub use yarte_helpers::{helpers::Render, Error, Result};
#[cfg(not(target_arch = "wasm32"))]
pub use yarte_template::Template;
#[cfg(target_arch = "wasm32")]
pub use yarte_wasm_app::{Addr, App as Template};

#[cfg(feature = "client")]
pub use yarte_helpers::helpers::big_num::*;

pub mod recompile;

#[cfg(all(feature = "with-actix-web", not(target_arch = "wasm32")))]
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

#[cfg(target_arch = "wasm32")]
pub mod wasm {
    pub use serde_json::from_str;
    pub use wasm_bindgen::JsCast;
    pub use web_sys as web;
}

#[cfg(target_arch = "wasm32")]
pub use self::wasm::*;
