use super::*;
// Adapted from https://docs.serde.rs/src/serde/ser/impls.rs.html#378

impl<T> Serialize for [T; 0] {
    #[inline]
    fn to_mut_bytes<B: Buffer>(&self, buf: &mut B) {
        empty_array(buf)
    }
}

macro_rules! array_impls {
    ($($len:tt)+) => {
        $(
            impl<T> Serialize for [T; $len]
            where
                T: Serialize,
            {
                #[inline]
                fn to_mut_bytes<B: Buffer>(&self, buf: &mut B) {
                    let mut i = self.iter();
                    if let Some(first) = i.next() {
                        begin_array(buf);
                        first.to_mut_bytes(buf);
                        for e in i {
                            write_comma(buf);
                            e.to_mut_bytes(buf);
                        }
                        end_array(buf);
                    } else {
                        unreachable!()
                    }
                }
            }
        )+
    }
}

#[rustfmt::skip]
array_impls! {
        1  2  3  4  5  6  7  8  9
    10 11 12 13 14 15 16 17 18 19
    20 21 22 23 24 25 26 27 28 29
    30 31 32
}
