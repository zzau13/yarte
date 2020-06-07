// Based on https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/lib.rs
#![allow(dead_code)]

use std::ptr::copy_nonoverlapping;
use v_htmlescape::{v_escape, v_escape_char};

macro_rules! buf_ptr {
    ($buf:expr) => {
        $buf as *mut [u8] as *mut u8
    };
}

macro_rules! src_ptr {
    ($buf:expr) => {
        $buf as *const [u8] as *const u8
    };
}

/// Render trait, used for wrap unsafe expressions `{{ ... }}` when it's in a html template
pub trait RenderFixed {
    /// Render in buffer will html escape the string type
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize>;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    v_escape(self.as_bytes(), buf)
                }
            }
        )*
    };
}

impl RenderFixed for String {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape(self.as_bytes(), buf)
    }
}

#[rustfmt::skip]
str_display!(
    str &str &&str &&&str &&&&str
    String &String &&String &&&String
);

macro_rules! itoa_display_0 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, *self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_1 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, **self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_2 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, ***self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_3 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, ****self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_4 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &&&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, *****self).ok()
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
            impl RenderFixed for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, *self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_1 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, **self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_2 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, ***self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_3 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, ****self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_4 {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &&&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, *****self).ok()
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

impl RenderFixed for char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(*self, buf)
    }
}

impl RenderFixed for &char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(**self, buf)
    }
}

impl RenderFixed for &&char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(***self, buf)
    }
}

impl RenderFixed for &&&char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(****self, buf)
    }
}

impl RenderFixed for &&&&char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(*****self, buf)
    }
}

impl RenderFixed for bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*self, buf)
    }
}

impl RenderFixed for &bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(**self, buf)
    }
}

impl RenderFixed for &&bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(***self, buf)
    }
}

impl RenderFixed for &&&bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(****self, buf)
    }
}

impl RenderFixed for &&&&bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*****self, buf)
    }
}

/// Render trait, used for wrap safe expressions `{{{ ... }}}` or text
pub trait RenderSafe {
    /// Render in buffer
    ///
    /// # Safety
    /// Possible overlap if you have a chance to implement:
    /// have a buffer reference in your data type
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize>;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
    str &str &&str &&&str &&&&str
    String &String &&String &&&String
);

impl RenderSafe for String {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
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

macro_rules! itoa_display_0 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, *self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_1 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, **self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_2 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, ***self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_3 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, ****self).ok()
                }
            }
        )*
    };
}

macro_rules! itoa_display_4 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &&&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    itoa::write(buf, *****self).ok()
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
            impl RenderSafe for $ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, *self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_1 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, **self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_2 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, ***self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_3 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, ****self).ok()
                }
            }
        )*
    };
}

macro_rules! dtoa_display_4 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &&&&$ty {
                #[inline(always)]
                unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    dtoa::write(buf, *****self).ok()
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

impl RenderSafe for char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(*self, buf)
    }
}

impl RenderSafe for &char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(**self, buf)
    }
}

impl RenderSafe for &&char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(***self, buf)
    }
}

impl RenderSafe for &&&char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(****self, buf)
    }
}

impl RenderSafe for &&&&char {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(*****self, buf)
    }
}

impl RenderSafe for bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*self, buf)
    }
}

impl RenderSafe for &bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(**self, buf)
    }
}

impl RenderSafe for &&bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(***self, buf)
    }
}

impl RenderSafe for &&&bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(****self, buf)
    }
}

impl RenderSafe for &&&&bool {
    #[inline(always)]
    unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*****self, buf)
    }
}

use std::io;

struct Writer<'a> {
    buf: &'a mut [u8],
    len: usize,
}

impl<'a> Writer<'a> {
    #[inline]
    fn new(buf: &mut [u8]) -> Writer {
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
        unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }

    impl<'a, D: Serialize> RenderFixed for JsonPretty<'a, D> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer_pretty(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }

    impl<'a, S: Serialize> RenderSafe for Json<'a, S> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }

    impl<'a, D: Serialize> RenderSafe for JsonPretty<'a, D> {
        #[inline(always)]
        unsafe fn render(&self, buf: &mut [u8]) -> Option<usize> {
            let mut buf = Writer::new(buf);
            to_writer_pretty(&mut buf, self.0).ok()?;
            Some(buf.consume())
        }
    }
}

#[inline(always)]
fn render_char(c: char, buf: &mut [u8]) -> Option<usize> {
    if buf.len() < c.len_utf8() {
        None
    } else {
        Some(c.encode_utf8(buf).len())
    }
}

/// fast boolean render
unsafe fn render_bool(b: bool, buf: &mut [u8]) -> Option<usize> {
    macro_rules! buf_ptr_u32 {
        ($buf:ident) => {
            $buf as *mut [u8] as *mut u32
        };
    }
    if b {
        if buf.len() < 4 {
            None
        } else {
            if (buf_ptr!(buf) as usize).trailing_zeros() < 2 {
                buf_ptr!(buf).write(b't');
                buf_ptr!(buf).add(1).write(b'r');
                buf_ptr!(buf).add(2).write(b'u');
                buf_ptr!(buf).add(3).write(b'e');
            } else {
                //                             e  u  r  t
                buf_ptr_u32!(buf).write(0x65_75_72_74);
            }
            Some(4)
        }
    } else if buf.len() < 5 {
        None
    } else {
        if (buf_ptr!(buf) as usize).trailing_zeros() < 2 {
            buf_ptr!(buf).write(b'f');
            buf_ptr!(buf).add(1).write(b'a');
            buf_ptr!(buf).add(2).write(b'l');
            buf_ptr!(buf).add(3).write(b's');
            buf_ptr!(buf).add(4).write(b'e');
        } else {
            //                             s  l  a  f
            buf_ptr_u32!(buf).write(0x73_6C_61_66);
            buf_ptr!(buf).add(4).write(b'e');
        }
        Some(5)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn r_bool() {
        unsafe {
            let b = &mut [0; 4];
            assert!(render_bool(true, b).is_some());
            assert_eq!(&b[..], b"true");
            let b = &mut [0; 5];
            assert!(render_bool(false, b).is_some());
            assert_eq!(&b[..], b"false");
            let b = &mut [0; 4];
            assert!(render_bool(false, b).is_none());
        }
    }

    #[test]
    fn r_char() {
        let b = &mut [0; 1];
        assert!(render_char('a', b).is_some());
        assert_eq!(&b[..], b"a");

        let b = &mut [0; 4];
        assert!(render_char('𝄞', b).is_some());
        assert_eq!(&b[..], "𝄞".as_bytes());
        let b = &mut [0; 3];
        assert!(render_char('𝄞', b).is_none());
    }
}
