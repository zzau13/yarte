// based on https://github.com/miloyip/itoa-benchmark
use std::ptr::addr_of;

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
mod v_integer;

static DIGITS_LUT: &[u8] = b"\
      0001020304050607080910111213141516171819\
      2021222324252627282930313233343536373839\
      4041424344454647484950515253545556575859\
      6061626364656667686970717273747576777879\
      8081828384858687888990919293949596979899";

#[inline(always)]
fn dig(n: usize) -> u8 {
    debug_assert!(n < DIGITS_LUT.len());
    unsafe { *DIGITS_LUT.as_ptr().add(n) }
}

#[inline(always)]
fn sum_0(n: u8) -> u8 {
    n.sum(b'0')
}

trait UnsafeInteger: Copy {
    fn sum(self, a: Self) -> Self;
    fn dib(self, a: Self) -> Self;
    fn ren(self, a: Self) -> Self;
    fn m2(self) -> usize;
}

macro_rules! impl_unsafe_integers {
    ($($t:ty)+) => {
    $(
    impl UnsafeInteger for $t {
        #[inline(always)]
        fn sum(self, b: Self) -> Self {
            debug_assert!(self.checked_add(b).is_some());
            self.wrapping_add(b)
        }

        #[inline(always)]
        fn dib(self, b: Self) -> Self {
            debug_assert!(self.checked_div(b).is_some());
            self.wrapping_div(b)
        }

        #[inline(always)]
        fn ren(self, b: Self) -> Self {
            debug_assert!(self.checked_rem(b).is_some());
            self.wrapping_rem(b)
        }

        #[inline(always)]
        fn m2(self) -> usize {
            debug_assert!(self.checked_shl(1).is_some());
            (self as usize).wrapping_shl(1)
        }
    }
    )+
    };
}

impl_unsafe_integers!(u8 u16 u32 u64 usize);

#[inline]
unsafe fn write_less10k(value: u16, buf: *mut u8) -> usize {
    debug_assert!(value < 10_000);

    if value > 1_000 - 1 {
        let d1 = value.dib(100).m2();
        let d2 = value.ren(100).m2();
        *buf = dig(d1);
        *buf.add(1) = dig(d1.sum(1));
        *buf.add(2) = dig(d2);
        *buf.add(3) = dig(d2.sum(1));
        4
    } else if value < 100 {
        if value > 10 - 1 {
            let d2 = value.m2();
            *buf = dig(d2);
            *buf.add(1) = dig(d2.sum(1));
            2
        } else {
            *buf = sum_0(value as u8);
            1
        }
    } else {
        let d2 = value.ren(100).m2();
        *buf = sum_0(value.dib(100) as u8);
        *buf.add(1) = dig(d2);
        *buf.add(2) = dig(d2.sum(1));
        3
    }
}

#[inline]
#[allow(dead_code)]
unsafe fn write_10kk_100kk(value: u32, buf: *mut u8) {
    debug_assert!(value < 100_000_000);
    // value = bbbbcccc
    let b = value.dib(10_000);
    let c = value.ren(10_000);

    let d1 = b.dib(100).m2();
    let d2 = b.ren(100).m2();
    let d3 = c.dib(100).m2();
    let d4 = c.ren(100).m2();

    *buf = dig(d1);
    *buf.add(1) = dig(d1.sum(1));
    *buf.add(2) = dig(d2);
    *buf.add(3) = dig(d2.sum(1));
    *buf.add(4) = dig(d3);
    *buf.add(5) = dig(d3.sum(1));
    *buf.add(6) = dig(d4);
    *buf.add(7) = dig(d4.sum(1));
}

#[inline]
unsafe fn write_10k_100kk(value: u32, buf: *mut u8) -> usize {
    debug_assert!(value < 100_000_000);
    debug_assert!(value >= 10_000);
    // value = bbbb_cccc
    let b = value.dib(10_000);
    let c = value.ren(10_000);

    let d3 = c.dib(100).m2();
    let d4 = c.ren(100).m2();

    if value < 10_000_000 {
        if value > 1_000_000 - 1 {
            let d2 = b.ren(100).m2();

            *buf = sum_0(b.dib(100) as u8);
            *buf.add(1) = dig(d2);
            *buf.add(2) = dig(d2.sum(1));
            *buf.add(3) = dig(d3);
            *buf.add(4) = dig(d3.sum(1));
            *buf.add(5) = dig(d4);
            *buf.add(6) = dig(d4.sum(1));
            7
        } else if value > 100000 - 1 {
            let d2 = b.ren(100).m2();

            *buf = dig(d2);
            *buf.add(1) = dig(d2.sum(1));
            *buf.add(2) = dig(d3);
            *buf.add(3) = dig(d3.sum(1));
            *buf.add(4) = dig(d4);
            *buf.add(5) = dig(d4.sum(1));
            6
        } else {
            *buf = sum_0(b.ren(100) as u8);
            *buf.add(1) = dig(d3);
            *buf.add(2) = dig(d3.sum(1));
            *buf.add(3) = dig(d4);
            *buf.add(4) = dig(d4.sum(1));
            5
        }
    } else {
        let d1 = b.dib(100).m2();
        let d2 = b.ren(100).m2();

        *buf = dig(d1);
        *buf.add(1) = dig(d1.sum(1));
        *buf.add(2) = dig(d2);
        *buf.add(3) = dig(d2.sum(1));
        *buf.add(4) = dig(d3);
        *buf.add(5) = dig(d3.sum(1));
        *buf.add(6) = dig(d4);
        *buf.add(7) = dig(d4.sum(1));
        8
    }
}

unsafe fn write_u8(value: u8, buf: *mut u8) -> usize {
    if value < 10 {
        *buf = sum_0(value);
        1
    } else if value < 100 {
        let d2 = value.m2();
        *buf = dig(d2);
        *buf.add(1) = dig(d2.sum(1));
        2
    } else {
        let d2 = value.ren(100).m2();
        *buf = sum_0(value.dib(100));
        *buf.add(1) = dig(d2);
        *buf.add(2) = dig(d2.wrapping_add(1));
        3
    }
}

unsafe fn write_u16(value: u16, buf: *mut u8) -> usize {
    if value < 100 {
        if value < 10 {
            *buf = sum_0(value as u8);
            1
        } else {
            let d = value.m2();
            *buf = dig(d);
            *buf.add(1) = dig(d.sum(1));
            2
        }
    } else if value < 10_000 {
        let d2 = value.ren(100).m2();
        if value < 1_000 {
            *buf = sum_0(value.dib(100) as u8);
            *buf.add(1) = dig(d2);
            *buf.add(2) = dig(d2.sum(1));
            3
        } else {
            let d1 = value.dib(100).m2();
            *buf = dig(d1);
            *buf.add(1) = dig(d1.sum(1));
            *buf.add(2) = dig(d2);
            *buf.add(3) = dig(d2.sum(1));
            4
        }
    } else {
        let c = value.ren(10_000);

        let d1 = c.dib(100).m2();
        let d2 = c.ren(100).m2();

        *buf = sum_0(value.dib(10_000) as u8);
        *buf.add(1) = dig(d1);
        *buf.add(2) = dig(d1.sum(1));
        *buf.add(3) = dig(d2);
        *buf.add(4) = dig(d2.sum(1));
        5
    }
}

mod fallback {
    use super::*;

    pub unsafe fn write_u32(value: u32, buf: *mut u8) -> usize {
        if value < 10_000 {
            write_less10k(value as u16, buf)
        } else if value < 100_000_000 {
            write_10k_100kk(value, buf)
        } else {
            // value = aabbbbbbbb in decimal
            let a = value.dib(100_000_000); // 1 to 42
            let value = value.ren(100_000_000);

            let o = if a >= 10 {
                let i = a.m2();
                *buf = dig(i);
                *buf.add(1) = dig(i.sum(1));
                2
            } else {
                *buf = sum_0(a as u8);
                1
            };
            write_10kk_100kk(value, buf.add(o));
            o.sum(8)
        }
    }

    pub unsafe fn write_u64(value: u64, buf: *mut u8) -> usize {
        if value < 10_000 {
            write_less10k(value as u16, buf)
        } else if value < 100_000_000 {
            write_10k_100kk(value as u32, buf)
        } else if value < 10_000_000_000_000_000 {
            // value = aaaa_aaaa_bbbb_bbbb in decimal
            let a = value.dib(100_000_000) as u32;
            let o = if a < 10_000 {
                write_less10k(a as u16, buf)
            } else {
                write_10k_100kk(a, buf)
            };

            write_10kk_100kk(value.ren(100_000_000) as u32, buf.add(o));
            o.sum(8)
        } else {
            let a = value.dib(10_000_000_000_000_000) as u16; // 1 to 1844
            let value = value.ren(10_000_000_000_000_000);

            let o = write_less10k(a, buf);
            write_10kk_100kk(value.dib(100_000_000) as u32, buf.add(o));
            write_10kk_100kk(value.ren(100_000_000) as u32, buf.add(o.sum(8)));
            o.sum(16)
        }
    }
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
macro_rules! detect_fn {
    ($name:ident, $t:ty) => {
        // https://github.com/BurntSushi/rust-memchr/blob/master/src/x86/mod.rs#L9-L29
        unsafe fn $name(value: $t, buf: *mut u8) -> usize {
            use std::mem;
            use std::sync::atomic::{AtomicUsize, Ordering};
            static mut FN: fn($t, *mut u8) -> usize = detect;

            fn detect(value: $t, buf: *mut u8) -> usize {
                let fun = if is_x86_feature_detected!("sse2") {
                    v_integer::$name as usize
                } else {
                    fallback::$name as usize
                };

                let slot = unsafe { &*(addr_of!(FN) as *const _ as *const AtomicUsize) };
                slot.store(fun as usize, Ordering::Relaxed);
                unsafe { mem::transmute::<usize, fn($t, *mut u8) -> usize>(fun)(value, buf) }
            }

            let slot = &*(addr_of!(FN) as *const _ as *const AtomicUsize);
            let fun = slot.load(Ordering::Relaxed);
            mem::transmute::<usize, fn($t, *mut u8) -> usize>(fun)(value, buf)
        }
    };
}

#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
detect_fn!(write_u32, u32);
#[cfg(any(target_arch = "x86_64", target_arch = "x86"))]
detect_fn!(write_u64, u64);
#[cfg(not(any(target_arch = "x86_64", target_arch = "x86")))]
use fallback::*;

pub trait Integer {
    const MAX_LEN: usize;
    /// Write
    ///
    /// # Safety
    /// Internal library NOT USE
    unsafe fn write_to(self, buf: *mut u8) -> usize;
}

macro_rules! implement {
    ($([$($attr:meta)*] $uint:ty, $int:ty, $ty:ty, $fun:path, $len:literal)+) => {
        $(
        $(#[$attr])*
        impl Integer for $uint{
            const MAX_LEN: usize = $len;

            #[inline]
            unsafe fn write_to(self, buf: *mut u8) -> usize {
                $fun(self as $ty, buf)
            }
        }

        $(#[$attr])*
        impl Integer for $int{
            const MAX_LEN: usize = $len + 1;

            #[inline]
            unsafe fn write_to(self, buf: *mut u8) -> usize {
                if self < 0 {
                    *buf = b'-';
                    $fun((!(self as $ty)).wrapping_add(1), buf.add(1)) + 1
                } else {
                    $fun(self as $ty, buf)
                }
            }
        }
        )+
    };
}

implement! {
    [] u8, i8, u8, write_u8, 3
    [] u16, i16, u16, write_u16, 5
    [] u32, i32, u32, write_u32, 10
    [] u64, i64, u64, write_u64, 20
    [cfg(target_pointer_width = "16")]
    usize, isize, u16, write_u16, 5
    [cfg(target_pointer_width = "32")]
    usize, isize, u32, write_u32, 10
    [cfg(target_pointer_width = "64")]
    usize, isize, u64, write_u64, 20
}

#[cfg(test)]
mod tests {
    use super::*;
    use random_fast_rng::{FastRng, Random};

    #[test]
    fn test_i8_all() {
        let mut buf = String::with_capacity(i8::MAX_LEN);

        for n in std::i8::MIN..=std::i8::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.as_mut_vec().set_len(l);
                assert_eq!(buf, n.to_string());
            }
        }
        for n in std::u8::MIN..=std::u8::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.as_mut_vec().set_len(l);
                assert_eq!(buf, n.to_string());
            }
        }
    }

    #[test]
    fn test_16_all() {
        let mut buf = String::with_capacity(i16::MAX_LEN);

        for n in std::i16::MIN..=std::i16::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.as_mut_vec().set_len(l);
                assert_eq!(buf, n.to_string());
            }
        }
        for n in std::u16::MIN..=std::u16::MAX {
            unsafe {
                let l = n.write_to(buf.as_mut_ptr());
                buf.as_mut_vec().set_len(l);
                assert_eq!(buf, n.to_string());
            }
        }
    }

    #[test]
    fn test_u64_random() {
        let mut buf = String::with_capacity(u64::MAX_LEN);
        let mut rng = FastRng::new();

        for _ in 0..1_000_000 {
            unsafe {
                let n = rng.get_u64();
                let l = n.write_to(buf.as_mut_ptr());
                buf.as_mut_vec().set_len(l);
                assert_eq!(buf, n.to_string());
            }
        }
    }

    #[test]
    fn test_u32_random() {
        use super::Integer;
        let mut buf = String::with_capacity(u64::MAX_LEN);
        let mut rng = FastRng::new();

        for _ in 0..1_000_000 {
            unsafe {
                let n = rng.get_u32();
                let l = n.write_to(buf.as_mut_ptr());
                buf.as_mut_vec().set_len(l);
                assert_eq!(buf, n.to_string());
            }
        }
    }
}
