#![allow(clippy::transmute_ptr_to_ptr)]

use std::io;
use std::mem::transmute;

use bytes::{BufMut, BytesMut};
use v_htmlescape::{b_escape, b_escape_char};

use super::ryu::{Sealed, MAX_SIZE_FLOAT};

macro_rules! buf_ptr {
    ($buf:expr) => {
        $buf as *mut _ as *mut u8
    };
}

/// Render trait, used for wrap  expressions `{{ ... }}` when it's in a html template
pub trait RenderBytes {
    /// Render in buffer will html escape the string type
    ///
    /// # Panics
    /// With an new buffer, render length overflows usize
    fn render(self, buf: &mut BytesMut);
}

/// Auto copy/deref trait
pub trait RenderBytesA {
    /// Render in buffer will html escape the string type
    ///
    /// # Panics
    /// With an empty buffer, render length overflows usize
    fn __render_itb(self, buf: &mut BytesMut);
}

impl<T: RenderBytes + Copy> RenderBytesA for T {
    #[inline(always)]
    fn __render_itb(self, buf: &mut BytesMut) {
        self.render(buf)
    }
}

/// Auto copy/deref trait
pub trait RenderBytesSafeA {
    /// Render in buffer
    ///
    /// # Panics
    /// With an empty buffer, render length overflows usize
    fn __render_itb_safe(self, buf: &mut BytesMut);
}

impl<T: RenderBytesSafe + Copy> RenderBytesSafeA for T {
    #[inline(always)]
    fn __render_itb_safe(self, buf: &mut BytesMut) {
        self.render(buf)
    }
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytes for $ty {
                #[inline(always)]
                 fn render(self, buf: &mut BytesMut)  {
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
                 fn render(self, buf: &mut BytesMut)  {
                    use super::integers::Integer;
                    buf.reserve(Self::MAX_LEN);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to(buf_ptr!(buf.bytes_mut())) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance_mut(b) };
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
                 fn render(self, buf: &mut BytesMut)  {
                    // Infallible, can panics overflows usize
                    let _ = itoa::write(Writer::new(buf), self);
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
    #[inline(always)]
    fn render(self, buf: &mut BytesMut) {
        b_escape_char(self, buf)
    }
}

impl RenderBytes for bool {
    #[inline(always)]
    fn render(self, buf: &mut BytesMut) {
        render_bool(self, buf)
    }
}

/// Render trait, used for wrap safe expressions `{{{ ... }}}` or text
pub trait RenderBytesSafe {
    /// Render in buffer
    ///
    /// # Panics
    /// With an empty buffer, render length overflows usize
    fn render(self, buf: &mut BytesMut);
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderBytesSafe for $ty {
                #[inline(always)]
                 fn render(self, buf: &mut BytesMut)  {
                    buf.extend_from_slice(self.as_bytes());
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
                 fn render(self, buf: &mut BytesMut)  {
                    use super::integers::Integer;
                    buf.reserve(Self::MAX_LEN);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to(buf_ptr!(buf.bytes_mut())) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance_mut(b) };
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
                 fn render(self, buf: &mut BytesMut)  {
                    let _ = itoa::write(Writer::new(buf), self);
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
     fn render(self, buf: &mut BytesMut)  {
        if self.is_nonfinite() {
            buf.extend_from_slice(self.format_nonfinite().as_bytes());
        } else {
            buf.reserve(MAX_SIZE_FLOAT);
            // Safety: Previous reserve MAX length
            let b = unsafe { self.write_to_ryu_buffer(buf_ptr!(buf.bytes_mut())) };
            // Safety: Wrote `b` bytes
            unsafe { buf.advance_mut(b) };
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
    fn render(self, buf: &mut BytesMut) {
        render_char(self, buf)
    }
}

impl RenderBytesSafe for bool {
    #[inline(always)]
    fn render(self, buf: &mut BytesMut) {
        render_bool(self, buf)
    }
}

struct Writer<'a> {
    buf: &'a mut BytesMut,
}

impl<'a> Writer<'a> {
    #[inline]
    fn new(buf: &mut BytesMut) -> Writer {
        Writer { buf }
    }
}

impl<'a> io::Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.buf.extend_from_slice(buf);
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
        fn render(self, buf: &mut BytesMut) {
            to_bytes_mut(self.0, buf)
        }
    }

    impl<'a, S: json::Serialize> RenderBytesSafe for Json<'a, S> {
        #[inline(always)]
        fn render(self, buf: &mut BytesMut) {
            to_bytes_mut(self.0, buf)
        }
    }
}

#[inline(always)]
fn render_char(c: char, buf: &mut BytesMut) {
    let len = c.len_utf8();
    buf.reserve(len);
    // Safety: Has same layout and encode_utf8 NOT read buf
    c.encode_utf8(unsafe { transmute(buf.bytes_mut()) });
    // Safety: Just write this len
    unsafe { buf.advance_mut(len) }
}

pub(crate) fn render_bool(b: bool, buf: &mut BytesMut) {
    const T: &[u8] = b"true";
    const F: &[u8] = b"false";
    if b {
        buf.extend_from_slice(T);
    } else {
        buf.extend_from_slice(F);
    }
}
