// Adapted from [`simd-json-derive`](https://github.com/simd-lite/simd-json-derive)

//! Serialize a Rust data structure into JSON data.

use buf_min::Buffer;
use v_jsonescape::{b_escape, b_escape_char, fallback};

use crate::helpers::bytes::render_bool;
use crate::helpers::ryu::{Sealed, MAX_SIZE_FLOAT};

mod array;
mod chrono;
mod collections;
mod deref;
mod tpl;

pub trait Serialize {
    fn to_bytes_mut<B: Buffer>(&self, buf: &mut B);

    fn to_bytes<B: Buffer + Sized>(&self, capacity: usize) -> B::Freeze {
        let mut buf: B = Buffer::with_capacity(capacity);
        self.to_bytes_mut(&mut buf);
        Buffer::freeze(buf)
    }
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl Serialize for $ty {
                #[inline]
                 fn to_bytes_mut<B: Buffer>(&self, buf: &mut B)  {
                    begin_string(buf);
                    b_escape(self.as_bytes(), buf);
                    end_string(buf);
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
            impl Serialize for $ty {
                #[inline(always)]
                 fn to_bytes_mut<B: Buffer>(&self, buf: &mut B)  {
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

impl Serialize for char {
    #[inline(always)]
    fn to_bytes_mut<B: Buffer>(&self, buf: &mut B) {
        begin_string(buf);
        b_escape_char(*self, buf);
        end_string(buf);
    }
}

impl Serialize for bool {
    #[inline(always)]
    fn to_bytes_mut<B: Buffer>(&self, buf: &mut B) {
        render_bool(*self, buf)
    }
}

macro_rules! ryu_display {
    ($($t:ty)+) => {
    $(
        impl Serialize for $t {
            #[inline(always)]
            fn to_bytes_mut<B: Buffer>(&self, buf: &mut B)  {
                if self.is_nonfinite() {
                    render_null(buf);
                } else {
                    buf.reserve(MAX_SIZE_FLOAT);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to_ryu_buffer(buf.buf_ptr()) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance(b) };
                }
            }
        }
    )+
    }
}

ryu_display!(f32 f64);

#[inline]
pub fn render_null<B: Buffer>(buf: &mut B) {
    buf.extend_from_slice(b"null");
}

#[inline]
pub fn begin_string<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf.buf_ptr() = b'"' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline(always)]
pub fn end_string<B: Buffer>(buf: &mut B) {
    begin_string(buf);
}

#[inline]
pub fn empty_array<B: Buffer>(buf: &mut B) {
    buf.reserve(2);
    // Safety: previous reserve 2
    unsafe { *buf.buf_ptr() = b'[' };
    unsafe { *buf.buf_ptr().add(1) = b']' };
    // Safety: previous write 2
    unsafe { buf.advance(2) };
}

#[inline]
pub fn begin_array<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf.buf_ptr() = b'[' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline]
pub fn end_array<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf.buf_ptr() = b']' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline]
pub fn write_comma<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1
    unsafe { *buf.buf_ptr() = b',' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline]
pub fn empty_object<B: Buffer>(buf: &mut B) {
    buf.reserve(2);
    // Safety: previous reserve 2
    unsafe { *buf.buf_ptr() = b'{' };
    unsafe { *buf.buf_ptr().add(1) = b'}' };
    // Safety: previous write 2
    unsafe { buf.advance(2) };
}

#[inline]
pub fn begin_object<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1
    unsafe { *buf.buf_ptr() = b'{' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline]
pub fn end_object<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1
    unsafe { *buf.buf_ptr() = b'}' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline]
pub fn write_colon<B: Buffer>(buf: &mut B) {
    buf.reserve(1);
    // Safety: previous reserve 1
    unsafe { *buf.buf_ptr() = b':' };
    // Safety: previous write 1
    unsafe { buf.advance(1) };
}

#[inline]
pub fn end_array_object<B: Buffer>(buf: &mut B) {
    buf.reserve(2);
    // Safety: previous reserve 2
    unsafe { *buf.buf_ptr() = b']' };
    unsafe { *buf.buf_ptr().add(1) = b'}' };
    // Safety: previous write 2
    unsafe { buf.advance(2) };
}

#[inline]
pub fn end_object_object<B: Buffer>(buf: &mut B) {
    buf.reserve(2);
    // Safety: previous reserve 2
    unsafe { *buf.buf_ptr() = b'}' };
    unsafe { *buf.buf_ptr().add(1) = b'}' };
    // Safety: previous write 2
    unsafe { buf.advance(2) };
}

#[inline]
fn serialize_str_short<B: Buffer>(value: &str, buf: &mut B) {
    begin_string(buf);
    fallback::b_escape(value.as_bytes(), buf);
    end_string(buf);
}

#[inline]
pub fn to_bytes_mut<B, T>(value: &T, buf: &mut B)
where
    B: Buffer,
    T: ?Sized + Serialize,
{
    value.to_bytes_mut(buf)
}

pub fn to_bytes<B, T>(value: &T, capacity: usize) -> B::Freeze
where
    B: Buffer,
    T: ?Sized + Serialize,
{
    Serialize::to_bytes::<B>(value, capacity)
}
