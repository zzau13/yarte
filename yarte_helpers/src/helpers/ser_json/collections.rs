// Adapted from [`simd-json-derive`](https://github.com/simd-lite/simd-json-derive)

use super::*;
use std::collections;

macro_rules! vec_like {
    ($t:ty) => {
        impl<T> Serialize for $t
        where
            T: Serialize,
        {
            #[inline]
            fn to_bytes_mut(&self, buf: &mut BytesMut) {
                let mut i = self.iter();
                if let Some(first) = i.next() {
                    begin_array(buf);
                    first.to_bytes_mut(buf);
                    for e in i {
                        write_comma(buf);
                        e.to_bytes_mut(buf);
                    }
                    end_array(buf);
                } else {
                    empty_array(buf)
                }
            }
        }
    };
}

vec_like!(Vec<T>);
vec_like!([T]);
vec_like!(collections::VecDeque<T>);
vec_like!(collections::BinaryHeap<T>);
vec_like!(collections::BTreeSet<T>);
vec_like!(collections::LinkedList<T>);
impl<T, H> Serialize for collections::HashSet<T, H>
where
    T: Serialize,
    H: std::hash::BuildHasher,
{
    #[inline]
    fn to_bytes_mut(&self, buf: &mut BytesMut) {
        let mut i = self.iter();
        if let Some(first) = i.next() {
            begin_array(buf);
            first.to_bytes_mut(buf);
            for e in i {
                write_comma(buf);
                e.to_bytes_mut(buf);
            }
            end_array(buf);
        } else {
            empty_array(buf);
        }
    }
}

/// PRIVATE: Not implement
/// Bounded Object keys types
#[doc(hidden)]
pub trait SerObjKey: Serialize {
    fn ser_obj_key(&self, buf: &mut BytesMut);
}

impl SerObjKey for str {
    #[inline]
    fn ser_obj_key(&self, buf: &mut BytesMut) {
        serialize_str_short(self, buf);
    }
}

impl SerObjKey for String {
    #[inline]
    fn ser_obj_key(&self, buf: &mut BytesMut) {
        serialize_str_short(self, buf);
    }
}

// Taken from https://docs.serde.rs/src/serde/ser/impls.rs.html#378
macro_rules! deref_impl {
    (
        $(#[doc = $doc:tt])*
        <$($desc:tt)+
    ) => {
        $(#[doc = $doc])*
        impl <$($desc)+ {
            #[inline]
            fn ser_obj_key(&self, buf: &mut BytesMut) {
                (**self).ser_obj_key(buf)
            }
        }
    };
}

deref_impl!(<'a, T> SerObjKey for &'a T where T: ?Sized + SerObjKey);
deref_impl!(<'a, T> SerObjKey for &'a mut T where T: ?Sized + SerObjKey);
deref_impl!(<T: ?Sized> SerObjKey for Box<T> where T: SerObjKey);
deref_impl!(<'a, T: ?Sized> SerObjKey for std::borrow::Cow<'a, T> where T: SerObjKey + ToOwned);

impl SerObjKey for char {
    fn ser_obj_key(&self, buf: &mut BytesMut) {
        begin_string(buf);
        b_escape_char(*self, buf);
        end_string(buf);
    }
}

// TODO: Ser Obj keys integers

impl<K, V, H> Serialize for collections::HashMap<K, V, H>
where
    K: SerObjKey,
    V: Serialize,
    H: std::hash::BuildHasher,
{
    #[inline]
    fn to_bytes_mut(&self, buf: &mut BytesMut) {
        let mut i = self.iter();
        if let Some((k, v)) = i.next() {
            begin_object(buf);
            k.ser_obj_key(buf);
            write_colon(buf);
            v.to_bytes_mut(buf);
            for (k, v) in i {
                write_comma(buf);
                k.ser_obj_key(buf);
                write_colon(buf);
                v.to_bytes_mut(buf);
            }
            end_object(buf);
        } else {
            empty_object(buf);
        }
    }
}

impl<K, V> Serialize for collections::BTreeMap<K, V>
where
    K: SerObjKey,
    V: Serialize,
{
    #[inline]
    fn to_bytes_mut(&self, buf: &mut BytesMut) {
        let mut i = self.iter();
        if let Some((k, v)) = i.next() {
            begin_object(buf);
            k.ser_obj_key(buf);
            write_colon(buf);
            v.to_bytes_mut(buf);
            for (k, v) in i {
                write_comma(buf);
                k.ser_obj_key(buf);
                write_colon(buf);
                v.to_bytes_mut(buf);
            }
            end_object(buf);
        } else {
            empty_object(buf);
        }
    }
}
