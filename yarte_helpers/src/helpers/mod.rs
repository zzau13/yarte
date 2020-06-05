pub mod io_fmt;

#[cfg(feature = "big-num-32")]
pub mod big_num_32;
#[cfg(feature = "display-fn")]
pub mod display_fn;

#[cfg(feature = "fixed")]
mod fixed;
#[cfg(feature = "markup")]
mod markup;

#[cfg(feature = "fixed")]
pub use fixed::{RenderFixed, RenderSafe};
#[cfg(feature = "markup")]
pub use markup::Render;
