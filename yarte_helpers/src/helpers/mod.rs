pub mod io_fmt;

#[repr(align(32))]
pub struct Aligned256<T>(pub T);

#[cfg(feature = "big-num-32")]
pub mod big_num_32;
#[cfg(feature = "display-fn")]
pub mod display_fn;

#[cfg(feature = "fixed")]
mod fixed;
#[cfg(feature = "markup")]
mod markup;

#[cfg(feature = "fixed")]
pub use fixed::{RenderFixed, RenderFixedA, RenderSafe, RenderSafeA};
#[cfg(feature = "markup")]
pub use markup::Render;
