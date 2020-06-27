// adapted from [`serde-json`](https://github.com/serde-rs/json)

//! Serialize a Rust data structure into JSON data.

use std::fmt::{self, Display};
use std::mem::MaybeUninit;

use serde::ser::{self, Impossible, Serialize};
use serde::serde_if_integer128;

use bytes::{BufMut, Bytes, BytesMut};

use v_jsonescape::{b_escape, b_escape_char};

use crate::helpers::bytes::RenderBytesSafe;
use crate::helpers::ryu::{Sealed, MAX_SIZE_FLOAT};

macro_rules! buf_ptr {
    ($buf:expr) => {
        $buf as *mut _ as *mut u8
    };
}

#[derive(Debug)]
pub struct Error(String);

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

impl ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error(msg.to_string())
    }
}

impl std::error::Error for Error {}

/// A structure for serializing Rust values into JSON.
pub struct Serializer {
    buf: BytesMut,
}

impl Serializer {
    /// Creates a new JSON serializer.
    #[inline]
    pub fn new(capacity: usize) -> Self {
        Serializer {
            buf: BytesMut::with_capacity(capacity),
        }
    }

    pub fn freeze(self) -> Bytes {
        self.buf.freeze()
    }
}

// TODO: any error? or not?
type Result<T> = std::result::Result<T, Error>;

impl<'a> ser::Serializer for &'a mut Serializer {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Compound<'a>;
    type SerializeTuple = Compound<'a>;
    type SerializeTupleStruct = Compound<'a>;
    type SerializeTupleVariant = Compound<'a>;
    type SerializeMap = Compound<'a>;
    type SerializeStruct = Compound<'a>;
    type SerializeStructVariant = Compound<'a>;

    #[inline]
    fn serialize_bool(self, value: bool) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_i8(self, value: i8) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_i16(self, value: i16) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_i32(self, value: i32) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_i64(self, value: i64) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    serde_if_integer128! {
        fn serialize_i128(self, _value: i128) -> Result<()> {
            todo!();
        }
    }

    #[inline]
    fn serialize_u8(self, value: u8) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_u16(self, value: u16) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_u32(self, value: u32) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_u64(self, value: u64) -> Result<()> {
        value.render(&mut self.buf);
        Ok(())
    }

    serde_if_integer128! {
        fn serialize_u128(self, _value: u128) -> Result<()> {
            todo!();
        }
    }

    #[inline]
    fn serialize_f32(self, value: f32) -> Result<()> {
        if value.is_nonfinite() {
            render_null(&mut self.buf);
        } else {
            self.buf.reserve(MAX_SIZE_FLOAT);
            // Safety: Previous reserve MAX length
            let b = unsafe { value.write_to_ryu_buffer(buf_ptr!(self.buf.bytes_mut())) };
            // Safety: Wrote `b` bytes
            unsafe { self.buf.advance_mut(b) };
        }
        Ok(())
    }

    #[inline]
    fn serialize_f64(self, value: f64) -> Result<()> {
        if value.is_nonfinite() {
            render_null(&mut self.buf);
        } else {
            self.buf.reserve(MAX_SIZE_FLOAT);
            // Safety: Previous reserve MAX length
            let b = unsafe { value.write_to_ryu_buffer(buf_ptr!(self.buf.bytes_mut())) };
            // Safety: Wrote `b` bytes
            unsafe { self.buf.advance_mut(b) };
        }
        Ok(())
    }

    #[inline]
    fn serialize_char(self, value: char) -> Result<()> {
        begin_string(&mut self.buf);
        b_escape_char(value, &mut self.buf);
        end_string(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        begin_string(&mut self.buf);
        b_escape(value.as_bytes(), &mut self.buf);
        end_string(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_bytes(self, value: &[u8]) -> Result<()> {
        use serde::ser::SerializeSeq;
        let mut seq = self.serialize_seq(Some(value.len()))?;
        for byte in value {
            seq.serialize_element(byte)?;
        }
        seq.end()
    }

    #[inline]
    fn serialize_none(self) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_some<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_unit(self) -> Result<()> {
        render_null(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        self.serialize_unit()
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.serialize_str(variant)
    }

    /// Serialize newtypes without an object wrapper.
    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    #[inline]
    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        begin_object(&mut self.buf);
        begin_object_key(true, &mut self.buf);
        self.serialize_str(variant)?;
        end_object_key(&mut self.buf);
        begin_object_value(&mut self.buf);
        value.serialize(&mut *self)?;
        end_object_value(&mut self.buf);
        end_object(&mut self.buf);
        Ok(())
    }

    #[inline]
    fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
        if len == Some(0) {
            begin_array(&mut self.buf);
            end_array(&mut self.buf);
            Ok(Compound {
                ser: self,
                state: State::Empty,
            })
        } else {
            begin_array(&mut self.buf);
            Ok(Compound {
                ser: self,
                state: State::First,
            })
        }
    }

    #[inline]
    fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        begin_object(&mut self.buf);
        begin_object_key(true, &mut self.buf);
        self.serialize_str(variant)?;
        end_object_key(&mut self.buf);
        begin_object_value(&mut self.buf);
        self.serialize_seq(Some(len))
    }

    #[inline]
    fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
        if len == Some(0) {
            begin_object(&mut self.buf);
            end_object(&mut self.buf);
            Ok(Compound {
                ser: self,
                state: State::Empty,
            })
        } else {
            begin_object(&mut self.buf);
            Ok(Compound {
                ser: self,
                state: State::First,
            })
        }
    }

    #[inline]
    fn serialize_struct(self, _name: &'static str, len: usize) -> Result<Self::SerializeStruct> {
        self.serialize_map(Some(len))
    }

    #[inline]
    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
        len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        begin_object(&mut self.buf);
        begin_object_key(true, &mut self.buf);
        self.serialize_str(variant)?;
        end_object_key(&mut self.buf);
        begin_object_value(&mut self.buf);
        self.serialize_map(Some(len))
    }

    fn collect_str<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Display,
    {
        use self::fmt::Write;

        struct Adapter<'ser> {
            buf: &'ser mut BytesMut,
        }

        impl<'ser> std::fmt::Write for Adapter<'ser> {
            fn write_str(&mut self, s: &str) -> std::fmt::Result {
                b_escape(s.as_bytes(), self.buf);
                Ok(())
            }
        }

        begin_string(&mut self.buf);
        {
            write!(Adapter { buf: &mut self.buf }, "{}", value)
                .map_err(|_| ser::Error::custom("Writer"))?;
        }
        end_string(&mut self.buf);
        Ok(())
    }
}

// Not public API. Should be pub(crate).
#[doc(hidden)]
#[derive(Eq, PartialEq)]
pub enum State {
    Empty,
    First,
    Rest,
}

// Not public API. Should be pub(crate).
#[doc(hidden)]
pub struct Compound<'a> {
    ser: &'a mut Serializer,
    state: State,
}

impl<'a> ser::SerializeSeq for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        begin_array_value(self.state == State::First, &mut self.ser.buf);
        self.state = State::Rest;
        value.serialize(&mut *self.ser)?;
        end_array_value(&mut self.ser.buf);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => end_array(&mut self.ser.buf),
        }
        Ok(())
    }
}

impl<'a> ser::SerializeTuple for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_element<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleStruct for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        ser::SerializeSeq::end(self)
    }
}

impl<'a> ser::SerializeTupleVariant for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeSeq::serialize_element(self, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => end_array(&mut self.ser.buf),
        }
        end_object_value(&mut self.ser.buf);
        end_object(&mut self.ser.buf);
        Ok(())
    }
}

impl<'a> ser::SerializeMap for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_key<T>(&mut self, key: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        begin_object_key(self.state == State::First, &mut self.ser.buf);
        self.state = State::Rest;

        key.serialize(MapKeySerializer { ser: self.ser })?;

        end_object_key(&mut self.ser.buf);
        Ok(())
    }

    #[inline]
    fn serialize_value<T>(&mut self, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        begin_object_value(&mut self.ser.buf);

        value.serialize(&mut *self.ser)?;
        end_object_value(&mut self.ser.buf);
        Ok(())
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => end_object(&mut self.ser.buf),
        }
        Ok(())
    }
}

impl<'a> ser::SerializeStruct for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeMap::serialize_entry(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        ser::SerializeMap::end(self)
    }
}

impl<'a> ser::SerializeStructVariant for Compound<'a> {
    type Ok = ();
    type Error = Error;

    #[inline]
    fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        ser::SerializeStruct::serialize_field(self, key, value)
    }

    #[inline]
    fn end(self) -> Result<()> {
        match self.state {
            State::Empty => {}
            _ => end_object(&mut self.ser.buf),
        }
        end_object_value(&mut self.ser.buf);
        end_object(&mut self.ser.buf);
        Ok(())
    }
}

struct MapKeySerializer<'a> {
    ser: &'a mut Serializer,
}

#[inline]
fn key_must_be_a_string() -> Error {
    ser::Error::custom("key must be a string")
}

impl<'a> ser::Serializer for MapKeySerializer<'a> {
    type Ok = ();
    type Error = Error;

    type SerializeSeq = Impossible<(), Error>;

    type SerializeTuple = Impossible<(), Error>;

    type SerializeTupleStruct = Impossible<(), Error>;

    type SerializeTupleVariant = Impossible<(), Error>;
    type SerializeMap = Impossible<(), Error>;
    type SerializeStruct = Impossible<(), Error>;
    type SerializeStructVariant = Impossible<(), Error>;
    fn serialize_bool(self, _value: bool) -> Result<()> {
        Err(key_must_be_a_string())
    }
    fn serialize_i8(self, value: i8) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }
    fn serialize_i16(self, value: i16) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    fn serialize_i32(self, value: i32) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    fn serialize_i64(self, value: i64) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    fn serialize_u8(self, value: u8) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    fn serialize_u16(self, value: u16) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    fn serialize_u32(self, value: u32) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    serde_if_integer128! {
        fn serialize_i128(self, _value: i128) -> Result<()> {
            todo!()
        }
    }

    fn serialize_u64(self, value: u64) -> Result<()> {
        begin_string(&mut self.ser.buf);
        value.render(&mut self.ser.buf);
        end_string(&mut self.ser.buf);
        Ok(())
    }

    fn serialize_f32(self, _value: f32) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_f64(self, _value: f64) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_char(self, value: char) -> Result<()> {
        self.ser.serialize_str(&value.to_string())
    }

    serde_if_integer128! {
        fn serialize_u128(self, _value: u128) -> Result<()> {
            todo!()
        }
    }

    #[inline]
    fn serialize_str(self, value: &str) -> Result<()> {
        self.ser.serialize_str(value)
    }

    fn serialize_bytes(self, _value: &[u8]) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_none(self) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_some<T>(self, _value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_unit(self) -> Result<()> {
        Err(key_must_be_a_string())
    }

    fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
        Err(key_must_be_a_string())
    }

    #[inline]
    fn serialize_unit_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        variant: &'static str,
    ) -> Result<()> {
        self.ser.serialize_str(variant)
    }

    #[inline]
    fn serialize_newtype_struct<T>(self, _name: &'static str, value: &T) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        value.serialize(self)
    }

    fn serialize_newtype_variant<T>(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _value: &T,
    ) -> Result<()>
    where
        T: ?Sized + Serialize,
    {
        Err(key_must_be_a_string())
    }

    fn serialize_seq(self, _len: Option<usize>) -> Result<Self::SerializeSeq> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple(self, _len: usize) -> Result<Self::SerializeTuple> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_struct(
        self,
        _name: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_tuple_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeTupleVariant> {
        Err(key_must_be_a_string())
    }

    fn serialize_map(self, _len: Option<usize>) -> Result<Self::SerializeMap> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct(self, _name: &'static str, _len: usize) -> Result<Self::SerializeStruct> {
        Err(key_must_be_a_string())
    }

    fn serialize_struct_variant(
        self,
        _name: &'static str,
        _variant_index: u32,
        _variant: &'static str,
        _len: usize,
    ) -> Result<Self::SerializeStructVariant> {
        Err(key_must_be_a_string())
    }

    fn collect_str<T>(self, value: &T) -> Result<()>
    where
        T: ?Sized + Display,
    {
        self.ser.collect_str(value)
    }
}

#[inline]
fn render_null(buf: &mut bytes::BytesMut) {
    buf.extend_from_slice(b"null");
}

#[inline]
fn begin_string(buf: &mut BytesMut) {
    buf.reserve(1);
    buf.bytes_mut()[0] = MaybeUninit::new(b'"');
    unsafe { buf.advance_mut(1) };
}

#[inline]
fn end_string(buf: &mut BytesMut) {
    begin_string(buf);
}

#[inline]
fn begin_array(buf: &mut BytesMut) {
    buf.reserve(1);
    buf.bytes_mut()[0] = MaybeUninit::new(b'[');
    unsafe { buf.advance_mut(1) };
}

#[inline]
fn end_array(buf: &mut BytesMut) {
    buf.reserve(1);
    buf.bytes_mut()[0] = MaybeUninit::new(b']');
    unsafe { buf.advance_mut(1) };
}

/// Called before every array value.  Writes a `,` if needed to
/// the specified writer.
#[inline]
fn begin_array_value(first: bool, buf: &mut BytesMut) {
    if !first {
        buf.reserve(1);
        buf.bytes_mut()[0] = MaybeUninit::new(b',');
        unsafe { buf.advance_mut(1) };
    }
}

/// Called after every array value.
#[inline]
fn end_array_value(_buf: &mut BytesMut) {}

/// Called before every object.  Writes a `{` to the specified
/// writer.
#[inline]
fn begin_object(buf: &mut BytesMut) {
    buf.reserve(1);
    buf.bytes_mut()[0] = MaybeUninit::new(b'{');
    unsafe { buf.advance_mut(1) };
}

#[inline]
fn end_object(buf: &mut BytesMut) {
    buf.reserve(1);
    buf.bytes_mut()[0] = MaybeUninit::new(b'}');
    unsafe { buf.advance_mut(1) };
}

#[inline]
fn begin_object_key(first: bool, buf: &mut BytesMut) {
    if !first {
        buf.reserve(1);
        buf.bytes_mut()[0] = MaybeUninit::new(b',');
        unsafe { buf.advance_mut(1) };
    }
}

#[inline]
fn end_object_key(_buf: &mut BytesMut) {}

#[inline]
fn begin_object_value(buf: &mut BytesMut) {
    buf.reserve(1);
    buf.bytes_mut()[0] = MaybeUninit::new(b':');
    unsafe { buf.advance_mut(1) };
}

#[inline]
fn end_object_value(_buf: &mut BytesMut) {}

pub fn to_bytes<T: Serialize + ?Sized>(capacity: usize, value: &T) -> Result<Bytes> {
    let mut ser = Serializer::new(capacity);
    value.serialize(&mut ser)?;
    Ok(ser.freeze())
}
