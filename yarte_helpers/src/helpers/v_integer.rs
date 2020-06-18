#![allow(dead_code)]
#![allow(clippy::missing_safety_doc)]
#![allow(clippy::cast_ptr_alignment)]
#![cfg(target_arch = "x86_64")]
use std::arch::x86_64::{
    __m128i, _mm_add_epi8, _mm_cvtsi32_si128, _mm_mul_epu32, _mm_mulhi_epu16, _mm_mullo_epi16,
    _mm_packus_epi16, _mm_setzero_si128, _mm_slli_epi64, _mm_srli_epi64, _mm_srli_si128,
    _mm_storel_epi64, _mm_sub_epi16, _mm_sub_epi32, _mm_unpacklo_epi16, _mm_unpacklo_epi32,
};
use std::mem::transmute;

use super::integers::DIGITS_LUT;

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

pub unsafe fn u32toa_sse2(value: u32, buf: *mut u8) -> usize {
    if value < 10000 {
        if value >= 1000 {
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
    } else if value < 100000000 {
        // value = bbbbcccc
        let b = value / 10000;
        let c = value % 10000;

        let d3 = ((c / 100) << 1) as usize;
        let d4 = ((c % 100) << 1) as usize;

        if value >= 10000000 {
            let d1 = ((b / 100) << 1) as usize;
            let d2 = ((b % 100) << 1) as usize;
            *buf = DIGITS_LUT[d1];
            *buf.add(1) = DIGITS_LUT[d1 + 1];
            *buf.add(2) = DIGITS_LUT[d2];
            *buf.add(3) = DIGITS_LUT[d2 + 1];
            *buf.add(4) = DIGITS_LUT[d3];
            *buf.add(5) = DIGITS_LUT[d3 + 1];
            *buf.add(6) = DIGITS_LUT[d4];
            *buf.add(7) = DIGITS_LUT[d4 + 1];
            8
        } else if value >= 1000000 {
            let d2 = ((b % 100) << 1) as usize;
            *buf = (b / 100) as u8 + 0x30;
            *buf.add(1) = DIGITS_LUT[d2];
            *buf.add(2) = DIGITS_LUT[d2 + 1];
            *buf.add(3) = DIGITS_LUT[d3];
            *buf.add(4) = DIGITS_LUT[d3 + 1];
            *buf.add(5) = DIGITS_LUT[d4];
            *buf.add(6) = DIGITS_LUT[d4 + 1];
            7
        } else if value >= 100000 {
            let d2 = ((b % 100) << 1) as usize;
            *buf = DIGITS_LUT[d2];
            *buf.add(1) = DIGITS_LUT[d2 + 1];
            *buf.add(2) = DIGITS_LUT[d3];
            *buf.add(3) = DIGITS_LUT[d3 + 1];
            *buf.add(4) = DIGITS_LUT[d4];
            *buf.add(5) = DIGITS_LUT[d4 + 1];
            6
        } else {
            *buf = (b % 100) as u8 + 0x30;
            *buf.add(1) = DIGITS_LUT[d3];
            *buf.add(2) = DIGITS_LUT[d3 + 1];
            *buf.add(3) = DIGITS_LUT[d4];
            *buf.add(4) = DIGITS_LUT[d4 + 1];
            5
        }
    } else {
        // value = aabbbbbbbb in decimal
        let a = value / 100000000; // 1 to 42
        let value = value % 100000000;

        let o = if a >= 10 {
            let i = (a << 1) as usize;
            *buf = DIGITS_LUT[i];
            *buf.add(1) = DIGITS_LUT[i + 1];
            2
        } else {
            *buf = a as u8 + b'0';
            1
        };

        let b = convert8digits_sse2(value);
        let ba = _mm_add_epi8(
            _mm_packus_epi16(_mm_setzero_si128(), b),
            transmute(K_ASCII_ZERO),
        );
        let result = _mm_srli_si128(ba, 8);
        _mm_storel_epi64(buf.add(o) as *mut __m128i, result);
        8 + o
    }
}
