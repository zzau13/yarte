// Adapted from [`simd-json-derive`](https://github.com/simd-lite/simd-json-derive)
use std::fmt;

use bytes::BytesMut;
use chrono::{DateTime, TimeZone};

use super::{begin_string, end_string, Serialize};

impl<Tz: TimeZone> Serialize for &DateTime<Tz> {
    /// Serialize into a rfc3339 time string
    ///
    /// See [the `serde` module](./serde/index.html) for alternate
    /// serializations.
    fn to_bytes_mut(&self, buf: &mut BytesMut) {
        struct FormatWrapped<'a, D: 'a> {
            inner: &'a D,
        }

        impl<'a, D: fmt::Debug> fmt::Display for FormatWrapped<'a, D> {
            fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
                self.inner.fmt(f)
            }
        }

        begin_string(buf);
        // Debug formatting is correct RFC3339, and it allows Zulu.
        Serialize::to_bytes_mut(&format!("{}", FormatWrapped { inner: &self }), buf);
        end_string(buf);
    }
}
