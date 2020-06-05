// Based on https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/lib.rs
use std::fmt::{self, Display};

use dtoa::write;
use itoa::fmt;
use v_htmlescape::{escape, escape_char};

use super::io_fmt::IoFmt;

/// Render trait, used for wrap unsafe expressions `{{ ... }}` when it's in a html template
pub trait Render {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl Render for &$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    escape(self).fmt(f)
                }
            }
        )*
    };
}

#[rustfmt::skip]
str_display!(str &str &&str &&&str &&&&str);

macro_rules! string_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    escape(self.as_str()).fmt(f)
                }
            }
        )*
    };
}

#[rustfmt::skip]
string_display!(String &String &&String &&&String &&&&String);

macro_rules! itoa_display_0 {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt(f, *self)
                }
            }
        )*
    };
}

macro_rules! itoa_display_1 {
    ($($ty:ty)*) => {
        $(
            impl Render for &$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt(f, **self)
                }
            }
        )*
    };
}

macro_rules! itoa_display_2 {
    ($($ty:ty)*) => {
        $(
            impl Render for &&$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt(f, ***self)
                }
            }
        )*
    };
}

macro_rules! itoa_display_3 {
    ($($ty:ty)*) => {
        $(
            impl Render for &&&$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt(f, ****self)
                }
            }
        )*
    };
}

macro_rules! itoa_display_4 {
    ($($ty:ty)*) => {
        $(
            impl Render for &&&&$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    fmt(f, *****self)
                }
            }
        )*
    };
}

macro_rules! itoa_display {
    ($($ty:ty)*) => {
        itoa_display_0!($($ty)*);
        itoa_display_1!($($ty)*);
        itoa_display_2!($($ty)*);
        itoa_display_3!($($ty)*);
        itoa_display_4!($($ty)*);
    };
}

#[rustfmt::skip]
itoa_display! {
    u8 u16 u32 u64 u128 usize
    i8 i16 i32 i64 i128 isize
}

macro_rules! dtoa_display_0 {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write(IoFmt::new(f), *self)
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

macro_rules! dtoa_display_1 {
    ($($ty:ty)*) => {
        $(
            impl Render for &$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write(IoFmt::new(f), **self)
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

macro_rules! dtoa_display_2 {
    ($($ty:ty)*) => {
        $(
            impl Render for &&$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write(IoFmt::new(f), ***self)
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

macro_rules! dtoa_display_3 {
    ($($ty:ty)*) => {
        $(
            impl Render for &&&$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write(IoFmt::new(f), ****self)
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

macro_rules! dtoa_display_4 {
    ($($ty:ty)*) => {
        $(
            impl Render for &&&&$ty {
                #[inline(always)]
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    write(IoFmt::new(f), *****self)
                    .map(|_| ())
                    .map_err(|_| fmt::Error)
                }
            }
        )*
    };
}

macro_rules! dtoa_display {
    ($($ty:ty)*) => {
        dtoa_display_0!($($ty)*);
        dtoa_display_1!($($ty)*);
        dtoa_display_2!($($ty)*);
        dtoa_display_3!($($ty)*);
        dtoa_display_4!($($ty)*);
    };
}

#[rustfmt::skip]
dtoa_display! {
    f32 f64
}

impl Render for char {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape_char(*self).fmt(f)
    }
}

impl Render for &char {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape_char(**self).fmt(f)
    }
}

impl Render for &&char {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape_char(***self).fmt(f)
    }
}

impl Render for &&&char {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape_char(****self).fmt(f)
    }
}

impl Render for &&&&char {
    #[inline(always)]
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
        escape_char(*****self).fmt(f)
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
    &bool
    &&bool
    &&&bool
    &&&&bool
}

#[cfg(feature = "json")]
mod json {
    use super::*;
    use crate::at_helpers::{Json, JsonPretty};
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
