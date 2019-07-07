// Based on https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/lib.rs
use std::fmt::{self, Display};

use v_htmlescape::escape;

pub trait Render {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

impl<'a, T: Render + ?Sized> Render for &'a T {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        (*self).render(f)
    }
}

impl<T: Render> Render for Option<T> {
    fn render(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Some(t) => t.render(f),
            None => Ok(()),
        }
    }
}

impl Render for str {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape(self).fmt(f)
    }
}

impl Render for String {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.as_str().render(f)
    }
}

macro_rules! raw_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    self.fmt(f)
                }
            }
        )*
    };
}

raw_display! {
    bool
    char
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
    f32 f64
}
