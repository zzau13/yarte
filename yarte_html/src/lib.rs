//! Adapted from [`html5ever`](https://github.com/servo/html5ever)
#![allow(
    clippy::match_single_binding,
    clippy::unknown_clippy_lints,
    clippy::match_on_vec_items,
    clippy::match_like_matches_macro,
    clippy::collapsible_match,
    clippy::unused_unit,
    clippy::suspicious_else_formatting,
    unreachable_patterns
)]
#[macro_use]
mod macros;
pub mod driver;
#[macro_use]
pub mod interface;
pub mod serializer;
pub mod tokenizer;
pub mod tree_builder;
pub mod utils;
