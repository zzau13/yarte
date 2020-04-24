pub use std::fmt::Error;
pub type Result<I> = ::std::result::Result<I, Error>;

#[cfg(feature = "config")]
pub mod config;
#[cfg(feature = "config")]
pub mod recompile;

#[cfg(any(
    feature = "big-num-32",
    feature = "display-fn",
    feature = "io-fmt",
    feature = "markup",
))]
pub mod helpers;
