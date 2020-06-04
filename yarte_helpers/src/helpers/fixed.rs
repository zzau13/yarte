// Based on https://github.com/utkarshkukreti/markup.rs/blob/master/markup/src/lib.rs
use v_htmlescape::{v_escape, v_escape_char};

/// Render trait, used for wrap unsafe expressions `{{ ... }}` when it's in a html template
pub trait RenderFixed {
    fn render(&self, buf: &mut [u8]) -> Option<usize>;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderFixed for &$ty {
                #[inline(always)]
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    v_escape(self.as_bytes(), buf)
                }
            }
        )*
    };
}

impl RenderFixed for String {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(*self, buf)
    }
}

impl RenderFixed for &char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(**self, buf)
    }
}

impl RenderFixed for &&char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(***self, buf)
    }
}

impl RenderFixed for &&&char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(****self, buf)
    }
}

impl RenderFixed for &&&&char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        v_escape_char(*****self, buf)
    }
}

impl RenderFixed for bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*self, buf)
    }
}

impl RenderFixed for &bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(**self, buf)
    }
}

impl RenderFixed for &&bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(***self, buf)
    }
}

impl RenderFixed for &&&bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(****self, buf)
    }
}

impl RenderFixed for &&&&bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*****self, buf)
    }
}

/// Render trait, used for wrap unsafe expressions `{{ ... }}` when it's in a html template
pub trait RenderSafe {
    fn render(&self, buf: &mut [u8]) -> Option<usize>;
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for &$ty {
                #[inline(always)]
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
                    if buf.len() < self.len() {
                        None
                    } else {
                        buf.copy_from_slice(self.as_bytes());
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
    String &String &&String &&&String &&&&String
);

macro_rules! itoa_display_0 {
    ($($ty:ty)*) => {
        $(
            impl RenderSafe for $ty {
                #[inline(always)]
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
                fn render(&self, buf: &mut [u8]) -> Option<usize> {
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
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(*self, buf)
    }
}

impl RenderSafe for &char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(**self, buf)
    }
}

impl RenderSafe for &&char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(***self, buf)
    }
}

impl RenderSafe for &&&char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(****self, buf)
    }
}

impl RenderSafe for &&&&char {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_char(*****self, buf)
    }
}

impl RenderSafe for bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*self, buf)
    }
}

impl RenderSafe for &bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(**self, buf)
    }
}

impl RenderSafe for &&bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(***self, buf)
    }
}

impl RenderSafe for &&&bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(****self, buf)
    }
}

impl RenderSafe for &&&&bool {
    #[inline(always)]
    fn render(&self, buf: &mut [u8]) -> Option<usize> {
        render_bool(*****self, buf)
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

#[inline(always)]
fn render_bool(b: bool, buf: &mut [u8]) -> Option<usize> {
    const T: &[u8] = b"true";
    const F: &[u8] = b"false";
    if b {
        if buf.len() < T.len() {
            None
        } else {
            buf.copy_from_slice(T);
            Some(T.len())
        }
    } else if buf.len() < F.len() {
        None
    } else {
        buf.copy_from_slice(F);
        Some(F.len())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn r_bool() {
        let b = &mut [0; 4];
        assert!(render_bool(true, b).is_some());
        assert_eq!(&b[..], b"true");
        let b = &mut [0; 5];
        assert!(render_bool(false, b).is_some());
        assert_eq!(&b[..], b"false");
        let b = &mut [0; 4];
        assert!(render_bool(false, b).is_none());
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
