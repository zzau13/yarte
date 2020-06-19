// based on https://github.com/miloyip/itoa-benchmark
#![allow(dead_code)]
use std::ptr;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod v_integer;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
use v_integer::{write_u32, write_u64};

static DIGITS_LUT: &[u8] = b"\
      0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

macro_rules! lookup {
    ($idx:expr) => {
        DIGITS_LUT.as_ptr().add(($idx as usize) << 1)
    };
}

macro_rules! write_ascii {
    ($b:ident, $n:expr) => {
        *$b = $n as u8 + 0x30;
    };
}

/// write integer smaller than 10000
#[inline]
unsafe fn write_small(n: u16, buf: *mut u8) -> usize {
    debug_assert!(n < 10000);

    if n < 100 {
        if n < 10 {
            write_ascii!(buf, n);
            1
        } else {
            ptr::copy_nonoverlapping(lookup!(n), buf, 2);
            2
        }
    } else if n < 1000 {
        write_ascii!(buf, n / 100);
        ptr::copy_nonoverlapping(lookup!(n % 100), buf.add(1), 2);
        3
    } else {
        write_small_pad(n, buf);
        4
    }
}

/// write integer smaller with 0 padding
#[inline]
unsafe fn write_small_pad(n: u16, buf: *mut u8) {
    debug_assert!(n < 10000);

    ptr::copy_nonoverlapping(lookup!(n / 100), buf, 2);
    ptr::copy_nonoverlapping(lookup!(n % 100), buf.add(2), 2);
}

unsafe fn write_u8(value: u8, buf: *mut u8) -> usize {
    if value < 10 {
        *buf = value as u8 + b'0';
        1
    } else if value < 100 {
        let d2 = (value << 1) as usize;
        *buf = DIGITS_LUT[d2];
        *buf.add(1) = DIGITS_LUT[d2 + 1];
        2
    } else {
        let d2 = ((value % 100) << 1) as usize;
        *buf = (value / 100) as u8 + b'0';
        *buf.add(1) = DIGITS_LUT[d2];
        *buf.add(2) = DIGITS_LUT[d2 + 1];
        3
    }
}

unsafe fn write_u16(value: u16, buf: *mut u8) -> usize {
    if value < 10 {
        *buf = value as u8 + b'0';
        1
    } else if value < 100 {
        let d2 = (value << 1) as usize;
        *buf = DIGITS_LUT[d2];
        *buf.add(1) = DIGITS_LUT[d2 + 1];
        2
    } else if value < 1000 {
        let d2 = ((value % 100) << 1) as usize;
        *buf = (value / 100) as u8 + b'0';
        *buf.add(1) = DIGITS_LUT[d2];
        *buf.add(2) = DIGITS_LUT[d2 + 1];
        3
    } else if value < 10000 {
        let d1 = ((value / 100) << 1) as usize;
        let d2 = ((value % 100) << 1) as usize;
        *buf = DIGITS_LUT[d1];
        *buf.add(1) = DIGITS_LUT[d1 + 1];
        *buf.add(2) = DIGITS_LUT[d2];
        *buf.add(3) = DIGITS_LUT[d2 + 1];
        4
    } else {
        let c = value % 10000;
        let d1 = ((c / 100) << 1) as usize;
        let d2 = ((c % 100) << 1) as usize;

        *buf = (value / 10000) as u8 + b'0';
        *buf.add(1) = DIGITS_LUT[d1];
        *buf.add(2) = DIGITS_LUT[d1 + 1];
        *buf.add(3) = DIGITS_LUT[d2];
        *buf.add(4) = DIGITS_LUT[d2 + 1];
        5
    }
}

#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
unsafe fn write_u32(value: u32, buf: *mut u8) -> usize {
    if value >= 100_000_000 {
        // value = aabbbbbbbb in decimal
        let a = value / 100_000_000; // 1 to 42
        let value = value % 100_000_000;

        let o = if a >= 10 {
            let i = (a << 1) as usize;
            *buf = DIGITS_LUT[i];
            *buf.add(1) = DIGITS_LUT[i + 1];
            2
        } else {
            *buf = a as u8 + b'0';
            1
        };

        // value = bbbbcccc
        let b = value / 10000;
        let c = value % 10000;

        let d1 = ((b / 100) << 1) as usize;
        let d2 = ((b % 100) << 1) as usize;
        let d3 = ((c / 100) << 1) as usize;
        let d4 = ((c % 100) << 1) as usize;

        *buf.add(o) = DIGITS_LUT[d1];
        *buf.add(1 + o) = DIGITS_LUT[d1 + 1];
        *buf.add(2 + o) = DIGITS_LUT[d2];
        *buf.add(3 + o) = DIGITS_LUT[d2 + 1];
        *buf.add(4 + o) = DIGITS_LUT[d3];
        *buf.add(5 + o) = DIGITS_LUT[d3 + 1];
        *buf.add(6 + o) = DIGITS_LUT[d4];
        *buf.add(7 + o) = DIGITS_LUT[d4 + 1];
        8 + o
    } else if value >= 10_000_000 {
        // value = bbbbcccc
        let b = value / 10000;
        let c = value % 10000;

        let d1 = ((b / 100) << 1) as usize;
        let d2 = ((b % 100) << 1) as usize;
        let d3 = ((c / 100) << 1) as usize;
        let d4 = ((c % 100) << 1) as usize;
        *buf = DIGITS_LUT[d1];
        *buf.add(1) = DIGITS_LUT[d1 + 1];
        *buf.add(2) = DIGITS_LUT[d2];
        *buf.add(3) = DIGITS_LUT[d2 + 1];
        *buf.add(4) = DIGITS_LUT[d3];
        *buf.add(5) = DIGITS_LUT[d3 + 1];
        *buf.add(6) = DIGITS_LUT[d4];
        *buf.add(7) = DIGITS_LUT[d4 + 1];
        8
    } else if value >= 1_000_000 {
        // value = bbbbcccc
        let b = value / 10000;
        let c = value % 10000;

        let d2 = ((b % 100) << 1) as usize;
        let d3 = ((c / 100) << 1) as usize;
        let d4 = ((c % 100) << 1) as usize;
        *buf = (b / 100) as u8 + 0x30;
        *buf.add(1) = DIGITS_LUT[d2];
        *buf.add(2) = DIGITS_LUT[d2 + 1];
        *buf.add(3) = DIGITS_LUT[d3];
        *buf.add(4) = DIGITS_LUT[d3 + 1];
        *buf.add(5) = DIGITS_LUT[d4];
        *buf.add(6) = DIGITS_LUT[d4 + 1];
        7
    } else if value >= 100000 {
        // value = bbbbcccc
        let b = value / 10000;
        let c = value % 10000;

        let d2 = ((b % 100) << 1) as usize;
        let d3 = ((c / 100) << 1) as usize;
        let d4 = ((c % 100) << 1) as usize;
        *buf = DIGITS_LUT[d2];
        *buf.add(1) = DIGITS_LUT[d2 + 1];
        *buf.add(2) = DIGITS_LUT[d3];
        *buf.add(3) = DIGITS_LUT[d3 + 1];
        *buf.add(4) = DIGITS_LUT[d4];
        *buf.add(5) = DIGITS_LUT[d4 + 1];
        6
    } else if value >= 10000 {
        // value = bbbbcccc
        let b = value / 10000;
        let c = value % 10000;

        let d3 = ((c / 100) << 1) as usize;
        let d4 = ((c % 100) << 1) as usize;
        *buf = (b % 100) as u8 + 0x30;
        *buf.add(1) = DIGITS_LUT[d3];
        *buf.add(2) = DIGITS_LUT[d3 + 1];
        *buf.add(3) = DIGITS_LUT[d4];
        *buf.add(4) = DIGITS_LUT[d4 + 1];
        5
    } else if value >= 1_000 {
        let d1 = ((value / 100) << 1) as usize;
        let d2 = ((value % 100) << 1) as usize;
        *buf = DIGITS_LUT[d1];
        *buf.add(1) = DIGITS_LUT[d1 + 1];
        *buf.add(2) = DIGITS_LUT[d2];
        *buf.add(3) = DIGITS_LUT[d2 + 1];
        4
    } else if value >= 100 {
        let d2 = ((value % 100) << 1) as usize;
        *buf = (value / 100) as u8 + b'0';
        *buf.add(1) = DIGITS_LUT[d2];
        *buf.add(2) = DIGITS_LUT[d2 + 1];
        3
    } else if value >= 10 {
        let d2 = (value << 1) as usize;
        *buf = DIGITS_LUT[d2];
        *buf.add(1) = DIGITS_LUT[d2 + 1];
        2
    } else {
        *buf = value as u8 + b'0';
        1
    }
}

// TODO: check
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
unsafe fn write_u64(mut n: u64, buf: *mut u8) -> usize {
    if n < 10000 {
        write_small(n as u16, buf)
    } else if n < 100000000 {
        let n = n as u32;
        let b = n / 10000;
        let c = n % 10000;

        let l = write_small(b as u16, buf);
        write_small_pad(c as u16, buf.add(l));
        l + 4
    } else if n < 10000000000000000 {
        let v0 = n / 100000000;
        let v1 = (n % 100000000) as u32;

        let l = if v0 < 10000 {
            write_small(v0 as u16, buf)
        } else {
            let b0 = v0 / 10000;
            let c0 = v0 % 10000;
            let l = write_small(b0 as u16, buf);
            write_small_pad(c0 as u16, buf.add(l));
            l + 4
        };

        let b1 = v1 / 10000;
        let c1 = v1 % 10000;

        write_small_pad(b1 as u16, buf.add(l));
        write_small_pad(c1 as u16, buf.add(l + 4));

        l + 8
    } else {
        let a = n / 10000000000000000; // 1 to 1844
        n %= 10000000000000000;

        let v0 = (n / 100000000) as u32;
        let v1 = (n % 100000000) as u32;

        let b0 = v0 / 10000;
        let c0 = v0 % 10000;

        let b1 = v1 / 10000;
        let c1 = v1 % 10000;

        let l = write_small(a as u16, buf);
        write_small_pad(b0 as u16, buf.add(l));
        write_small_pad(c0 as u16, buf.add(l + 4));
        write_small_pad(b1 as u16, buf.add(l + 8));
        write_small_pad(c1 as u16, buf.add(l + 12));
        l + 16
    }
}

pub trait Integer {
    const MAX_LEN: usize;
    /// Write
    ///
    /// # Safety
    /// Internal library NOT USE
    unsafe fn write_to(self, buf: *mut u8) -> usize;
}

macro_rules! impl_integer {
    ($unsigned:ty, $signed:ty, $conv:ty, $func:ident, $max_len:expr) => {
        impl Integer for $unsigned {
            const MAX_LEN: usize = $max_len;

            #[inline]
            unsafe fn write_to(self, buf: *mut u8) -> usize {
                $func(self as $conv, buf)
            }
        }

        impl Integer for $signed {
            const MAX_LEN: usize = $max_len + 1;

            #[inline]
            unsafe fn write_to(self, buf: *mut u8) -> usize {
                if self >= 0 {
                    $func(self as $conv, buf)
                } else {
                    *buf = b'-';
                    $func((!(self as $conv)).wrapping_add(1), buf.add(1)) + 1
                }
            }
        }
    };
}

impl_integer!(u8, i8, u8, write_u8, 3);
impl_integer!(u16, i16, u16, write_u16, 5);
impl_integer!(u32, i32, u32, write_u32, 10);
impl_integer!(u64, i64, u64, write_u64, 20);

#[cfg(target_pointer_width = "16")]
impl_integer!(usize, isize, u16, write_u16, 5);

#[cfg(target_pointer_width = "32")]
impl_integer!(usize, isize, u32, write_u32, 10);

#[cfg(target_pointer_width = "64")]
impl_integer!(usize, isize, u64, write_u64, 20);

#[cfg(test)]
mod tests {
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn test_i8_all() {
        use super::Integer;
        let mut buf = Vec::with_capacity(i8::MAX_LEN);

        for n in std::i8::MIN..=std::i8::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", n));
            }
        }
        for n in std::u8::MIN..=std::u8::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", n));
            }
        }
    }

    #[test]
    fn test_16_all() {
        use super::Integer;
        let mut buf = Vec::with_capacity(i16::MAX_LEN);

        for n in std::i16::MIN..=std::i16::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", n));
            }
        }
        for n in std::u16::MIN..=std::u16::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", n));
            }
        }
    }

    #[test]
    fn test_u64_random() {
        use super::Integer;
        let mut buf = Vec::with_capacity(u64::MAX_LEN);
        let mut state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u64;

        for _ in 0..10_000_000 {
            // xorshift
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            unsafe {
                let l = state.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", state));
            }
        }

        let mut state = 88172645463325252u64;
        for _ in 0..10_000_000 {
            // xorshift
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            unsafe {
                let l = state.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", state));
            }
        }
    }

    #[test]
    fn test_u32_random() {
        use super::Integer;
        let mut buf = Vec::with_capacity(u32::MAX_LEN);

        let mut state = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos() as u32;
        for _ in 0..10_000_000u32 {
            // xorshift
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            unsafe {
                let l = state.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", state));
            }
        }

        let mut state = 88172645463325252u64 as u32;
        for _ in 0..10_000_000u32 {
            // xorshift
            state ^= state << 13;
            state ^= state >> 7;
            state ^= state << 17;

            unsafe {
                let l = state.write_to(buf.as_mut_ptr());
                buf.set_len(l);
                assert_eq!(std::str::from_utf8_unchecked(&*buf), format!("{}", state));
            }
        }
    }

    macro_rules! make_test {
        ($name:ident, $type:ty, $($value:expr),*) => {
            #[test]
            fn $name() {
                use super::Integer;

                let mut buf = Vec::with_capacity(<$type>::MAX_LEN);
                $(
                    unsafe {
                        let l = ($value as $type).write_to(buf.as_mut_ptr());
                        buf.set_len(l);
                        assert_eq!(
                            std::str::from_utf8_unchecked(&*buf),
                            format!("{}", $value as $type)
                        );
                    }
                )*
            }
        }
    }

    // boundary tests
    make_test!(test_u8, u8, 0, 1, 9, 10, 99, 100, 254, 255);
    make_test!(test_u16, u16, 0, 9, 10, 99, 100, 999, 1000, 9999, 10000, 65535);
    make_test!(
        test_u32, u32, 0, 9, 10, 99, 100, 999, 1000, 9999, 10000, 99999, 100000, 999999, 1000000,
        9999999, 10000000, 99999999, 100000000, 999999999, 1000000000, 4294967295
    );
    make_test!(
        test_u64,
        u64,
        0,
        9,
        10,
        99,
        100,
        999,
        1000,
        9999,
        10000,
        99999,
        100000,
        999999,
        1000000,
        9999999,
        10000000,
        99999999,
        100000000,
        999999999,
        1000000000,
        9999999999,
        10000000000,
        99999999999,
        100000000000,
        999999999999,
        1000000000000,
        9999999999999,
        10000000000000,
        99999999999999,
        100000000000000,
        999999999999999,
        1000000000000000,
        9999999999999999,
        10000000000000000,
        99999999999999999,
        100000000000000000,
        999999999999999999,
        1000000000000000000,
        9999999999999999999,
        10000000000000000000,
        18446744073709551615
    );

    make_test!(test_i8, i8, std::i8::MIN, std::i8::MAX);
    make_test!(test_i16, i16, std::i16::MIN, std::i16::MAX);
    make_test!(test_i32, i32, std::i32::MIN, std::i32::MAX);
    make_test!(test_i64, i64, std::i64::MIN, std::i64::MAX);
}
