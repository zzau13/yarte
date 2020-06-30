// Adapted from [`simd-json-derive`](https://github.com/simd-lite/simd-json-derive)

//! Serialize a Rust data structure into JSON data.

use bytes::{BufMut, Bytes, BytesMut};

use v_jsonescape::{b_escape, b_escape_char, fallback};

use crate::helpers::bytes::{buf_ptr, render_bool};
use crate::helpers::ryu::{Sealed, MAX_SIZE_FLOAT};

mod array;
mod chrono;
mod collections;
mod deref;
mod tpl;

pub trait Serialize {
    fn to_bytes_mut(&self, buf: &mut BytesMut);

    fn to_bytes(&self, capacity: usize) -> Bytes {
        let mut buf = BytesMut::with_capacity(capacity);
        self.to_bytes_mut(&mut buf);
        buf.freeze()
    }
}

macro_rules! str_display {
    ($($ty:ty)*) => {
        $(
            impl Serialize for $ty {
                #[inline(always)]
                 fn to_bytes_mut(&self, buf: &mut BytesMut)  {
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
                 fn to_bytes_mut(&self, buf: &mut BytesMut)  {
                    use super::integers::Integer;
                    buf.reserve(Self::MAX_LEN);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to(buf_ptr(buf)) };
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

impl Serialize for char {
    #[inline(always)]
    fn to_bytes_mut(&self, buf: &mut BytesMut) {
        begin_string(buf);
        b_escape_char(*self, buf);
        end_string(buf);
    }
}

impl Serialize for bool {
    #[inline(always)]
    fn to_bytes_mut(&self, buf: &mut BytesMut) {
        render_bool(*self, buf)
    }
}

macro_rules! ryu_display {
    ($($t:ty)+) => {
    $(
        impl Serialize for $t {
            #[inline(always)]
            fn to_bytes_mut(&self, buf: &mut BytesMut)  {
                if self.is_nonfinite() {
                    render_null(buf);
                } else {
                    buf.reserve(MAX_SIZE_FLOAT);
                    // Safety: Previous reserve MAX length
                    let b = unsafe { self.write_to_ryu_buffer(buf_ptr(buf)) };
                    // Safety: Wrote `b` bytes
                    unsafe { buf.advance_mut(b) };
                }
            }
        }
    )+
    }
}

ryu_display!(f32 f64);

#[inline]
pub fn render_null(buf: &mut bytes::BytesMut) {
    buf.extend_from_slice(b"null");
}

#[inline]
pub fn begin_string(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'"' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn end_string(buf: &mut BytesMut) {
    begin_string(buf);
}

#[inline]
pub fn empty_array(buf: &mut BytesMut) {
    buf.reserve(2);
    // Safety: previous reserve 2 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'[' };
    unsafe { *buf_ptr(buf).add(1) = b']' };
    // Safety: previous write 2
    unsafe { buf.advance_mut(2) };
}

#[inline]
pub fn begin_array(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'[' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn end_array(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b']' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn write_comma(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b',' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn empty_object(buf: &mut BytesMut) {
    buf.reserve(2);
    // Safety: previous reserve 2 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'{' };
    unsafe { *buf_ptr(buf).add(1) = b'}' };
    // Safety: previous write 2
    unsafe { buf.advance_mut(2) };
}

#[inline]
pub fn begin_object(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'{' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn end_object(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'}' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn write_colon(buf: &mut BytesMut) {
    buf.reserve(1);
    // Safety: previous reserve 1 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b':' };
    // Safety: previous write 1
    unsafe { buf.advance_mut(1) };
}

#[inline]
pub fn end_array_object(buf: &mut BytesMut) {
    buf.reserve(2);
    // Safety: previous reserve 2 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b']' };
    unsafe { *buf_ptr(buf).add(1) = b'}' };
    // Safety: previous write 2
    unsafe { buf.advance_mut(2) };
}

#[inline]
pub fn end_object_object(buf: &mut BytesMut) {
    buf.reserve(2);
    // Safety: previous reserve 2 and size_t of MaybeUninit<u8> it's equal to size_t of u8
    unsafe { *buf_ptr(buf) = b'}' };
    unsafe { *buf_ptr(buf).add(1) = b'}' };
    // Safety: previous write 2
    unsafe { buf.advance_mut(2) };
}

#[inline]
fn serialize_str_short(value: &str, buf: &mut BytesMut) {
    begin_string(buf);
    fallback::b_escape(value.as_bytes(), buf);
    end_string(buf);
}

#[inline]
pub fn to_bytes_mut<T>(value: &T, buf: &mut BytesMut)
where
    T: ?Sized + Serialize,
{
    value.to_bytes_mut(buf)
}

pub fn to_bytes<T>(value: &T, capacity: usize) -> Bytes
where
    T: ?Sized + Serialize,
{
    value.to_bytes(capacity)
}
