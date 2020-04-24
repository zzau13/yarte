#[cfg(feature = "big-num-32")]
pub mod big_num_32;
#[cfg(feature = "display-fn")]
pub mod display_fn;
#[cfg(feature = "io-fmt")]
pub mod io_fmt;
#[cfg(feature = "markup")]
mod markup;

pub use markup::Render;
