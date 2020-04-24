//! ## Only run in nightly
//!
//! ## [Our book](https://yarte.netlify.app/)
//!
//! ## Example
//! ```rust
//! #![feature(proc_macro_hygiene)]
//! use yarte_format::{yformat, yformat_html};
//!
//! let foo = "World";
//! assert_eq!("Hello, World!", yformat!("Hello, {{ foo }}!"));
//!
//! let foo = "&";
//! assert_eq!("Hello, &amp;!", yformat_html!("Hello, {{ foo }}!"));
//!
//! ```
//!
#![cfg(nightly)]
#![feature(proc_macro_hygiene)]

pub use yarte_derive::{yformat, yformat_html};
pub use yarte_helpers::{
    helpers::{display_fn::DisplayFn, Render},
    recompile, Error, Result,
};
