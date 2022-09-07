// Based on https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/lib.rs
use std::fmt::{self, Display};

use v_htmlescape::escape;

/// Render trait, used for wrap unsafe expressions `{{ ... }}` when it's in a html template
pub trait Render {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

/// Auto ref trait
pub trait RenderA {
    /// Render in buffer will html escape the string type
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    fn __renders_it(&self, buf: &mut fmt::Formatter) -> fmt::Result;
}

impl<T: Render + ?Sized> RenderA for T {
    #[inline(always)]
    fn __renders_it(&self, buf: &mut fmt::Formatter) -> fmt::Result {
        Render::render(self, buf)
    }
}
impl Render for str {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape(self).fmt(f)
    }
}

impl Render for String {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape(self.as_str()).fmt(f)
    }
}

macro_rules! itoa_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(itoa::Buffer::new().format(*self))
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa_display! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}

macro_rules! dtoa_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    f.write_str(dtoa::Buffer::new().format(*self))
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

#[rustfmt::skip]
dtoa_display! {
    f32 f64
}

// TODO: in the future, your future.
impl Render for char {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape(&self.to_string()).fmt(f)
    }
}

macro_rules! raw_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.fmt(f)
                }
            }
        )*
    };
}

#[rustfmt::skip]
raw_display! {
    bool
}

#[cfg(feature = "json")]
mod json {
    use super::*;
    use crate::at_helpers::{Json, JsonPretty};
    use crate::helpers::io_fmt::IoFmt;
    use serde::Serialize;
    use serde_json::{to_writer, to_writer_pretty};

    impl<'a, S: Serialize> Render for Json<'a, S> {
        #[inline(always)]
        fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
            to_writer(IoFmt::new(f), self.0).map_err(|_| fmt::Error)
        }
    }

    impl<'a, D: Serialize> Render for JsonPretty<'a, D> {
        #[inline(always)]
        fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
            to_writer_pretty(IoFmt::new(f), self.0).map_err(|_| fmt::Error)
        }
    }
}
