#![allow(clippy::transmute_ptr_to_ptr)]

use std::io;
use std::slice::from_raw_parts_mut;

use buf_min::Buffer;
use v_htmlescape::{b_escape, b_escape_char};

use super::ryu::{Sealed, MAX_SIZE_FLOAT};

/// Render trait, used for wrap  expressions `{{ ... }}` when it's in a html template
pub trait RenderBytes {
    /// Render in buffer will html escape the string type
    ///
    /// # Panics
    /// With an new buffer, render length overflows usize
    fn render<B: Buffer>(self, buf: &mut B);
}

/// Auto copy/deref trait
pub trait RenderBytesA {
    /// Render in buffer will html escape the string type
    ///
    /// # Panics
    /// With an empty buffer, render length overflows usize
    fn __render_itb<B: Buffer>(self, buf: &mut B);
}

impl<T: RenderBytes + Copy> RenderBytesA for T {
    #[inline(always)]
    fn __render_itb<B: Buffer>(self, buf: &mut B) {
        self.render(buf)
    }
}

/// Auto copy/deref trait
pub trait RenderBytesSafeA {
    /// Render in buffer
    ///
    /// # Panics
    /// With an empty buffer, render length overflows usize
    fn __render_itb_safe<B: Buffer>(self, buf: &mut B);
}

impl<T: RenderBytesSafe + Copy> RenderBytesSafeA for T {
    #[inline(always)]
    fn __render_itb_safe<B: Buffer>(self, buf: &mut B) {
        self.render(buf)
    }
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytes for $ty {
                 fn render<B: Buffer>(self, buf: &mut B)  {
                    b_escape(self.as_bytes(), buf)
                 }
            }
        )*
    };
}

#[rustfmt::skip]
str_display!(
    &str &String
);

macro_rules! itoa_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytes for $ty {
                #[inline(always)]
                fn render<B: Buffer>(self, buf: &mut B)  {
                    use super::integers::Integer;
                    buf.reserve(Self::MAX_LEN);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to(buf.buf_ptr()) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance(b) };
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa_display! {
    u8 u16 u32 u64 usize
    i8 i16 i32 i64 isize
}

macro_rules! itoa128_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytes for $ty {
                #[inline(always)]
                 fn render<B: Buffer>(self, buf: &mut B)  {
                    // Safety: iota only write valid utf-8 bytes
                    let _ = itoa::write(UnsafeWriter::new(buf), self);
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa128_display! {
    u128 i128
}

impl RenderBytes for char {
    fn render<B: Buffer>(self, buf: &mut B) {
        b_escape_char(self, buf)
    }
}

impl RenderBytes for bool {
    #[inline(always)]
    fn render<B: Buffer>(self, buf: &mut B) {
        render_bool(self, buf)
    }
}

/// Render trait, used for wrap safe expressions `{{{ ... }}}` or text
pub trait RenderBytesSafe {
    /// Render in buffer
    ///
    /// # Panics
    /// With an empty buffer, render length overflows usize
    fn render<B: Buffer>(self, buf: &mut B);
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytesSafe for $ty {
                #[inline(always)]
                 fn render<B: Buffer>(self, buf: &mut B)  {
                    buf.extend(self);
                }
            }
        )*
    };
}

#[rustfmt::skip]
str_display!(
    &str &String
);

macro_rules! itoa_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytesSafe for $ty {
                #[inline(always)]
                 fn render<B: Buffer>(self, buf: &mut B)  {
                    use super::integers::Integer;
                    buf.reserve(Self::MAX_LEN);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to(buf.buf_ptr()) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance(b) };
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa_display! {
    u8 u16 u32 u64 usize
    i8 i16 i32 i64 isize
}

macro_rules! itoa128_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytesSafe for $ty {
                #[inline(always)]
                 fn render<B: Buffer>(self, buf: &mut B)  {
                    // Safety: iota only write valid utf-8 bytes
                    let _ = itoa::write(UnsafeWriter::new(buf), self);
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa128_display! {
    u128 i128
}

macro_rules! ryu_display {
    ($f:ty, $t:ty) => {
        impl $t for $f {
            #[inline(always)]
            fn render<B: Buffer>(self, buf: &mut B)  {
                if self.is_nonfinite() {
                    buf.extend(&self.format_nonfinite());
                } else {
                    buf.reserve(MAX_SIZE_FLOAT);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to_ryu_buffer(buf.buf_ptr()) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance(b) };
                }
            }
        }
    };
    ($f:ty, $t:ty, $($r:tt)+) => {
        ryu_display!($f, $t);
        ryu_display!($($r)+);
    };
}

#[rustfmt::skip]
ryu_display!(
    f32, RenderBytes,
    f64, RenderBytes,
    f32, RenderBytesSafe,
    f64, RenderBytesSafe
);

impl RenderBytesSafe for char {
    #[inline(always)]
    fn render<B: Buffer>(self, buf: &mut B) {
        render_char(self, buf)
    }
}

impl RenderBytesSafe for bool {
    #[inline(always)]
    fn render<B: Buffer>(self, buf: &mut B) {
        render_bool(self, buf)
    }
}

struct UnsafeWriter<'a, B> {
    buf: &'a mut B,
}

impl<'a, B: Buffer> UnsafeWriter<'a, B> {
    #[inline]
    fn new(buf: &mut B) -> UnsafeWriter<'_, B> {
        UnsafeWriter { buf }
    }
}

impl<'a, B: Buffer> io::Write for UnsafeWriter<'a, B> {
    #[inline]
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        // SAFETY: Writer only use for print checked utf-8
        unsafe { self.buf.extend_from_slice(buf) };
        Ok(buf.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "json")]
mod json {
    use super::*;
    use crate::at_helpers::Json;
    use crate::helpers::json::{self, to_bytes_mut};

    impl<'a, S: json::Serialize> RenderBytes for Json<'a, S> {
        #[inline(always)]
        fn render<B: Buffer>(self, buf: &mut B) {
            to_bytes_mut(self.0, buf)
        }
    }

    impl<'a, S: json::Serialize> RenderBytesSafe for Json<'a, S> {
        #[inline(always)]
        fn render<B: Buffer>(self, buf: &mut B) {
            to_bytes_mut(self.0, buf)
        }
    }
}

#[inline(always)]
fn render_char<B: Buffer>(c: char, buf: &mut B) {
    let len = c.len_utf8();
    buf.reserve(len);
    // Safety:  Previous reserve length
    unsafe {
        // TODO: char encode with length by argument
        c.encode_utf8(from_raw_parts_mut(buf.buf_ptr(), len));
    }
    // Safety: Just write this len
    unsafe { buf.advance(len) }
}

#[inline(always)]
pub(crate) fn render_bool<B: Buffer>(b: bool, buf: &mut B) {
    const T: &str = "true";
    const F: &str = "false";
    if b {
        buf.extend(T);
    } else {
        buf.extend(F);
    }
}

#[cfg(feature = "render-uuid")]
mod render_uuid {
    use crate::helpers::{RenderBytes, RenderBytesSafe};
    use buf_min::Buffer;
    use std::slice::from_raw_parts_mut;
    use uuid::adapter::HyphenatedRef;
    use uuid::Uuid;

    macro_rules! imp {
        ($($ty:ty)+) => {
            $(impl $ty for &Uuid {
                fn render<B: Buffer>(self, buf: &mut B) {
                    const LEN: usize = HyphenatedRef::LENGTH;
                    buf.reserve(LEN);
                    // Safety: previous reserve length
                    self.to_hyphenated_ref().encode_lower(unsafe {
                        from_raw_parts_mut(buf.buf_ptr(), LEN)
                    });
                    // Safety: previous write length
                    unsafe { buf.advance(LEN); }
                }
            })+
        };
    }

    imp!(RenderBytes RenderBytesSafe);

    #[cfg(test)]
    mod test {
        use super::*;

        #[test]
        fn test() {
            let mut buf = String::new();
            let u = Uuid::from_u128(0x1a);
            let res = u.to_string();
            RenderBytes::render(&u, &mut buf);

            assert_eq!(res, buf);
            buf.clear();
            RenderBytesSafe::render(&u, &mut buf);
            assert_eq!(res, buf);
        }
    }
}
