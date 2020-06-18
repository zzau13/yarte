pub mod integers;
pub mod io_fmt;

#[repr(align(32))]
#[cfg(target_pointer_width = "64")]
pub struct Aligned256<T>(pub T);

#[cfg(not(target_pointer_width = "64"))]
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
pub use markup::{Render, RenderA};
#[cfg(feature = "ryu-ad")]
pub mod ryu;

pub trait IntoCopyIterator {
    type Item;
    type Iterator: Iterator<Item = Self::Item>;
    fn __into_citer(self) -> Self::Iterator;
}

impl<I: IntoIterator + Sized> IntoCopyIterator for I {
    type Item = <Self as IntoIterator>::Item;
    type Iterator = <Self as IntoIterator>::IntoIter;

    #[inline]
    fn __into_citer(self) -> Self::Iterator {
        self.into_iter()
    }
}
