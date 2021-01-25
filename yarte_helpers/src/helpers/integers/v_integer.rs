#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_ptr_alignment)]

#[cfg(target_arch = "x86")]
use std::arch::x86::{
    __m128i, _mm_add_epi8, _mm_cmpeq_epi8, _mm_cvtsi32_si128, _mm_movemask_epi8, _mm_mul_epu32,
    _mm_mulhi_epu16, _mm_mullo_epi16, _mm_packus_epi16, _mm_setzero_si128, _mm_slli_epi64,
    _mm_srli_epi64, _mm_srli_si128, _mm_storel_epi64, _mm_storeu_si128, _mm_sub_epi16,
    _mm_sub_epi32, _mm_unpacklo_epi16, _mm_unpacklo_epi32,
};
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m128i, _mm_add_epi8, _mm_cmpeq_epi8, _mm_cvtsi32_si128, _mm_movemask_epi8, _mm_mul_epu32,
    _mm_mulhi_epu16, _mm_mullo_epi16, _mm_packus_epi16, _mm_setzero_si128, _mm_slli_epi64,
    _mm_srli_epi64, _mm_srli_si128, _mm_storel_epi64, _mm_storeu_si128, _mm_sub_epi16,
    _mm_sub_epi32, _mm_unpacklo_epi16, _mm_unpacklo_epi32,
};

use std::mem::transmute;

use super::{dig, sum_0, write_10k_100kk, write_less10k, UnsafeInteger};

#[repr(align(16))]
struct A16<T>(pub T);

const K_DIV10000: u32 = 0xd1b71759;
const K_DIV10000VECTOR: A16<[u32; 4]> = A16([K_DIV10000; 4]);
const K10000VECTOR: A16<[u32; 4]> = A16([10_000; 4]);
const K_DIV_POWERS_VECTOR: A16<[u16; 8]> =
    A16([8389, 5243, 13108, 32768, 8389, 5243, 13108, 32768]); // 10^3, 10^2, 10^1, 10^0
const K_SHIFT_POWERS_VECTOR: A16<[u16; 8]> = A16([
    1 << (16 - (23 + 2 - 16)),
    1 << (16 - (19 + 2 - 16)),
    1 << (16 - 1 - 2),
    1 << (15),
    1 << (16 - (23 + 2 - 16)),
    1 << (16 - (19 + 2 - 16)),
    1 << (16 - 1 - 2),
    1 << (15),
]);
const K10VECTOR: A16<[u16; 8]> = A16([10; 8]);
const K_ASCII_ZERO: A16<[u8; 16]> = A16([b'0'; 16]);

#[inline]
unsafe fn convert8digits_sse2(value: u32) -> __m128i {
    debug_assert!(value <= 99999999);

    // abcd, efgh = abcdefgh divmod 10000
    let abcdefgh = _mm_cvtsi32_si128(value as i32);
    let abcd = _mm_srli_epi64(_mm_mul_epu32(abcdefgh, transmute(K_DIV10000VECTOR)), 45);
    let efgh = _mm_sub_epi32(abcdefgh, _mm_mul_epu32(abcd, transmute(K10000VECTOR)));

    // v1 = [ abcd, efgh, 0, 0, 0, 0, 0, 0 ]
    let v1 = _mm_unpacklo_epi16(abcd, efgh);

    // v1a = v1 * 4 = [ abcd * 4, efgh * 4, 0, 0, 0, 0, 0, 0 ]
    let v1a = _mm_slli_epi64(v1, 2);

    // v2 = [ abcd * 4, abcd * 4, abcd * 4, abcd * 4, efgh * 4, efgh * 4, efgh * 4, efgh * 4 ]
    let v2a = _mm_unpacklo_epi16(v1a, v1a);
    let v2 = _mm_unpacklo_epi32(v2a, v2a);

    // v4 = v2 div 10^3, 10^2, 10^1, 10^0 = [ a, ab, abc, abcd, e, ef, efg, efgh ]
    let v3 = _mm_mulhi_epu16(v2, transmute(K_DIV_POWERS_VECTOR));
    let v4 = _mm_mulhi_epu16(v3, transmute(K_SHIFT_POWERS_VECTOR));

    // v5 = v4 * 10 = [ a0, ab0, abc0, abcd0, e0, ef0, efg0, efgh0 ]
    let v5 = _mm_mullo_epi16(v4, transmute(K10VECTOR));

    // v6 = v5 << 16 = [ 0, a0, ab0, abc0, 0, e0, ef0, efg0 ]
    let v6 = _mm_slli_epi64(v5, 16);

    // v7 = v4 - v6 = { a, b, c, d, e, f, g, h }
    _mm_sub_epi16(v4, v6)
}

#[inline]
unsafe fn shift_digits_sse2(a: __m128i, digit: u8) -> __m128i {
    debug_assert!(digit <= 8);
    match digit {
        1 => _mm_srli_si128(a, 1),
        2 => _mm_srli_si128(a, 2),
        3 => _mm_srli_si128(a, 3),
        4 => _mm_srli_si128(a, 4),
        5 => _mm_srli_si128(a, 5),
        6 => _mm_srli_si128(a, 6),
        7 => _mm_srli_si128(a, 7),
        8 => _mm_srli_si128(a, 8),
        _ => a,
    }
}

#[inline]
pub unsafe fn write_u32(value: u32, buf: *mut u8) -> usize {
    if value < 10_000 {
        write_less10k(value as u16, buf)
    } else if value < 100_000_000 {
        write_10k_100kk(value, buf)
    } else {
        // value = aabbbbbbbb in decimal
        let a = value.dib(100_000_000) as u8; // 1 to 42
        let value = value.ren(100_000_000);

        let o = if a >= 10 {
            let i = a.m2();
            *buf = dig(i);
            *buf.add(1) = dig(i.sum(1));
            2
        } else {
            *buf = sum_0(a);
            1
        };

        _mm_storel_epi64(
            buf.add(o) as *mut __m128i,
            _mm_srli_si128(
                _mm_add_epi8(
                    _mm_packus_epi16(_mm_setzero_si128(), convert8digits_sse2(value)),
                    transmute(K_ASCII_ZERO),
                ),
                8,
            ),
        );

        o.sum(8)
    }
}

#[inline]
pub unsafe fn write_u64(value: u64, buf: *mut u8) -> usize {
    if value < 10_000 {
        write_less10k(value as u16, buf)
    } else if value < 100_000_000 {
        write_10k_100kk(value as u32, buf)
    } else if value < 10_000_000_000_000_000 {
        // value = aabbbbbbbb in decimal

        // Convert to ascii
        let va = _mm_add_epi8(
            _mm_packus_epi16(
                convert8digits_sse2(value.dib(100_000_000) as u32),
                convert8digits_sse2(value.ren(100_000_000) as u32),
            ),
            transmute(K_ASCII_ZERO),
        );

        // Count number of digit
        let digits = (!_mm_movemask_epi8(_mm_cmpeq_epi8(va, transmute(K_ASCII_ZERO))) | 0x80_00)
            .trailing_zeros();
        debug_assert!(digits <= 8);

        _mm_storeu_si128(buf as *mut __m128i, shift_digits_sse2(va, digits as u8));

        16u32.wrapping_sub(digits) as usize
    } else {
        let o = write_less10k(value.dib(10_000_000_000_000_000) as u16, buf); // 1 to 1844
        let value = value.ren(10_000_000_000_000_000);

        // value = aaaa_aaaa_bbbb_bbbb in decimal
        _mm_storeu_si128(
            buf.add(o) as *mut __m128i,
            _mm_add_epi8(
                _mm_packus_epi16(
                    convert8digits_sse2(value.dib(100_000_000) as u32),
                    convert8digits_sse2(value.ren(100_000_000) as u32),
                ),
                transmute(K_ASCII_ZERO),
            ),
        );
        o.sum(16)
    }
}
