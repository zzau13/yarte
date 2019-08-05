// Based on https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/lib.rs
use std::fmt::{self, Display};

use v_htmlescape::escape;

pub trait Render: Display {
    fn render(&self, f: &mut fmt::Formatter) -> fmt::Result;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl Render for &$ty {
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    escape(self).fmt(f)
                }
            }
        )*
    };
}

#[rustfmt::skip]
str_display!(str &str &&str);

macro_rules! string_display {
    ($($ty:ty)*) => {
        $(
            impl Render for $ty {
                fn render(&self, f: &mut fmt::Formatter) -> fmt::Result {
                    escape(self.as_str()).fmt(f)
                }
            }
        )*
    };
}

#[rustfmt::skip]
string_display!(String &String &&String);

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
    &bool
    &char
    &u8 &u16 &u32 &u64 &u128 &usize
    &i8 &i16 &i32 &i64 &i128 &isize
    &f32 &f64
    &&bool
    &&char
    &&u8 &&u16 &&u32 &&u64 &&u128 &&usize
    &&i8 &&i16 &&i32 &&i64 &&i128 &&isize
    &&f32 &&f64
}
