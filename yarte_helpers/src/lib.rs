#![allow(unused_imports)]

#[macro_use]
extern crate cfg_if;

pub use std::fmt::Error;
pub type Result<I> = ::std::result::Result<I, Error>;
pub mod helpers;
