//! ## Only run in nightly
//!
//! ## [Our book](https://yarte.netlify.app/)
//!
//! ## Example
//! ```rust
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
pub use yarte_derive::{yformat, yformat_html};
pub use yarte_helpers::{
    at_helpers::*,
    helpers::{display_fn::DisplayFn, Render, RenderA},
    recompile, Error, Result,
};
