#![allow(dead_code)]

use core::mem::MaybeUninit;
use core::ptr::copy_nonoverlapping;
use core::slice::from_raw_parts_mut;
use std::io;

use v_htmlescape::{v_escape, v_escape_char};

use super::ryu::Sealed;

macro_rules! buf_ptr {
    ($buf:expr) => {
        $buf as *mut _ as *mut u8
    };
}

macro_rules! src_ptr {
    ($buf:expr) => {
        $buf as *const _ as *const u8
    };
}

// TODO: bound to Copy
/// Render trait, used for wrap unsafe expressions `{{ ... }}` when it's in a html template
pub trait RenderFixed {
    /// Render in buffer will html escape the string type
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize>;
}

/// Auto ref trait
pub trait RenderFixedA {
    /// Render in buffer will html escape the string type
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    unsafe fn __render_it(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize>;
}

impl<T: RenderFixed + ?Sized> RenderFixedA for T {
    #[inline(always)]
    unsafe fn __render_it(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        RenderFixed::render(self, buf)
    }
}

/// Auto ref trait
pub trait RenderSafeA {
    /// Render in buffer will html escape the string type
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    unsafe fn __render_it_safe(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize>;
}

impl<T: RenderSafe + ?Sized> RenderSafeA for T {
    #[inline(always)]
    unsafe fn __render_it_safe(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        RenderSafe::render(self, buf)
    }
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
                    v_escape(self.as_bytes(), buf)
                }
            }
        )*
    };
}

#[rustfmt::skip]
str_display!(
    str String
);

macro_rules! itoa_display {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
                    use super::integers::Integer;
                    if buf.len() < Self::MAX_LEN {
                        None
                    } else {
                        Some((*self).write_to(buf as *mut _ as *mut u8))
                    }
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
            impl RenderFixed for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
                    itoa::write(from_raw_parts_mut(buf_ptr!(buf), buf.len()), *self).ok()
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa128_display! {
    u128 i128
}

impl RenderFixed for char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        v_escape_char(*self, buf)
    }
}

impl RenderFixed for bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        render_bool(*self, buf)
    }
}

/// Render trait, used for wrap safe expressions `{{{ ... }}}` or text
pub trait RenderSafe {
    /// Render in buffer
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize>;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
                    if buf.len() < self.len() {
                        None
                    } else {
                        // Not use copy_from_slice for elide double checked
                        // Make sure move buf pointer in next render
                        copy_nonoverlapping(src_ptr!(self.as_bytes()), buf_ptr!(buf), self.len());
                        Some(self.len())
                    }
                }
            }
        )*
    };
}

#[rustfmt::skip]
str_display!(
    str String
);

macro_rules! itoa_display {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
                    use super::integers::Integer;
                    if buf.len() < Self::MAX_LEN {
                        None
                    } else {
                        Some((*self).write_to(buf as *mut _ as *mut u8))
                    }
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
            impl RenderSafe for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
                    itoa::write(from_raw_parts_mut(buf_ptr!(buf), buf.len()), *self).ok()
                }
            }
        )*
    };
}

#[rustfmt::skip]
itoa128_display! {
    u128 i128
}

// https://github.com/dtolnay/ryu/blob/1.0.5/src/buffer/mod.rs#L23
const MAX_SIZE_FLOAT: usize = 24;

macro_rules! ryu_display {
    ($f:ty, $t:ty) => {
impl $t for $f {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        let b = *self;
        if b.is_infinite() {
            let b = b.format_nonfinite();
            if buf.len() < b.len() {
                None
            } else {
                copy_nonoverlapping(b.as_ptr(), buf_ptr!(buf), b.len());
                Some(buf.len())
            }
        } else if buf.len() < MAX_SIZE_FLOAT {
            None
        } else {
            Some(b.write_to_ryu_buffer(buf_ptr!(buf)))
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
    f32, RenderFixed,
    f64, RenderFixed,
    f32, RenderSafe,
    f64, RenderSafe
);

impl RenderSafe for char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        render_char(*self, buf)
    }
}

impl RenderSafe for bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
        render_bool(*self, buf)
    }
}

struct Writer<'a> {
    buf: &'a mut [MaybeUninit<u8>],
    len: usize,
}

impl<'a> Writer<'a> {
    #[inline]
    fn new(buf: &mut [MaybeUninit<u8>]) -> Writer {
        Writer { buf, len: 0 }
    }

    #[inline]
    fn consume(self) -> usize {
        self.len
    }
}

impl<'a> io::Write for Writer<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        if self.buf.len() < buf.len() + self.len {
            Err(io::Error::from(io::ErrorKind::Other))
        } else {
            // Not use copy_from_slice for elide double checked
            // Make sure move buf pointer in next render
            unsafe {
                copy_nonoverlapping(src_ptr!(buf), buf_ptr!(self.buf).add(self.len), buf.len());
            }
            self.len += buf.len();
            Ok(buf.len())
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

#[cfg(feature = "json")]
mod json {
    use super::*;
    use crate::at_helpers::{Json, JsonPretty};
    use serde::Serialize;
    use serde_json::{to_writer, to_writer_pretty};

    impl<'a, S: Serialize> RenderFixed for Json<'a, S> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }

    impl<'a, D: Serialize> RenderFixed for JsonPretty<'a, D> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer_pretty(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }

    impl<'a, S: Serialize> RenderSafe for Json<'a, S> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }

    impl<'a, D: Serialize> RenderSafe for JsonPretty<'a, D> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer_pretty(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }
}

#[inline(always)]
unsafe fn render_char(c: char, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
    let len = c.len_utf8();
    if buf.len() < len {
        None
    } else {
        Some(c.encode_utf8(from_raw_parts_mut(buf_ptr!(buf), len)).len())
    }
}

unsafe fn render_bool(b: bool, buf: &mut [MaybeUninit<u8>]) -> Option<usize> {
    const T: &[u8] = b"true";
    const F: &[u8] = b"false";
    if b {
        if buf.len() < T.len() {
            None
        } else {
            copy_nonoverlapping(T.as_ptr(), buf_ptr!(buf), T.len());
            Some(T.len())
        }
    } else if buf.len() < F.len() {
        None
    } else {
        copy_nonoverlapping(F.as_ptr(), buf_ptr!(buf), F.len());
        Some(F.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::slice::from_raw_parts;

    macro_rules! slice {
        ($b:ident, $len:expr) => {
            from_raw_parts(src_ptr!($b), $len)
        };
    }

    #[test]
    fn r_bool() {
        unsafe {
            let b = &mut [MaybeUninit::uninit(); 4];
            assert!(render_bool(true, b).is_some());
            assert_eq!(slice!(b, 4), b"true");
            let b = &mut [MaybeUninit::uninit(); 5];
            assert!(render_bool(false, b).is_some());
            assert_eq!(slice!(b, 5), b"false");
            let b = &mut [MaybeUninit::uninit(); 4];
            assert!(render_bool(false, b).is_none());
        }
    }

    #[test]
    fn r_char() {
        unsafe {
            let b = &mut [MaybeUninit::uninit(); 1];
            assert!(render_char('a', b).is_some());
            assert_eq!(slice!(b, 1), b"a");

            let b = &mut [MaybeUninit::uninit(); 4];
            assert!(render_char('𝄞', b).is_some());
            assert_eq!(slice!(b, 4), "𝄞".as_bytes());
            let b = &mut [MaybeUninit::uninit(); 3];
            assert!(render_char('𝄞', b).is_none());
        }
    }
}
