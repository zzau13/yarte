pub use std::fmt::Error;
pub type Result<I> = ::std::result::Result<I, Error>;

pub mod config;
pub mod helpers;
pub mod recompile;
