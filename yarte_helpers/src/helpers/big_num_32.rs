use std::ops::{BitAnd, BitOr, BitOrAssign};
// TODO: to 1024

// TODO: trait const zero
pub trait YNumber: Copy + PartialEq + BitOr + BitOrAssign + BitAnd {
    fn zero() -> Self;
    fn neq_zero(self) -> bool;
}

macro_rules! impl_ynumber {
    ($($ty:ty)*) => {
        $(
            impl YNumber for $ty {
                #[inline(always)]
                fn zero() -> $ty {
                    0
                }

                #[inline(always)]
                fn neq_zero(self) -> bool {
                    self != 0
                }
            }
        )*
    };
}

impl_ynumber!(u8 u16 u32);

#[derive(Clone, Copy, PartialEq)]
pub struct U64([u32; 2]);

impl YNumber for U64 {
    #[inline(always)]
    fn zero() -> Self {
        U64([0, 0])
    }

    #[inline(always)]
    fn neq_zero(self) -> bool {
        self.0[0] != 0 || self.0[1] != 0
    }
}

impl BitOr for U64 {
    type Output = U64;

    fn bitor(self, rhs: Self) -> Self::Output {
        U64([self.0[0] | rhs.0[0], self.0[1] | rhs.0[1]])
    }
}

impl BitAnd for U64 {
    type Output = U64;

    fn bitand(self, rhs: Self) -> Self::Output {
        U64([self.0[0] & rhs.0[0], self.0[1] & rhs.0[1]])
    }
}

impl BitOrAssign for U64 {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct U128([u32; 4]);

impl YNumber for U128 {
    #[inline(always)]
    fn zero() -> Self {
        U128([0, 0, 0, 0])
    }

    #[inline(always)]
    fn neq_zero(self) -> bool {
        self.0[0] != 0 || self.0[1] != 0 || self.0[2] != 0 || self.0[3] != 0
    }
}

impl BitOr for U128 {
    type Output = U128;

    fn bitor(self, rhs: Self) -> Self::Output {
        U128([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
        ])
    }
}

impl BitAnd for U128 {
    type Output = U128;

    fn bitand(self, rhs: Self) -> Self::Output {
        U128([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
        ])
    }
}

impl BitOrAssign for U128 {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}

#[derive(Clone, Copy, PartialEq)]
pub struct U256([u32; 8]);

impl YNumber for U256 {
    #[inline(always)]
    fn zero() -> Self {
        U256([0, 0, 0, 0, 0, 0, 0, 0])
    }

    #[inline(always)]
    fn neq_zero(self) -> bool {
        self.0[0] != 0
            || self.0[1] != 0
            || self.0[2] != 0
            || self.0[3] != 0
            || self.0[4] != 0
            || self.0[5] != 0
            || self.0[6] != 0
            || self.0[7] != 0
    }
}

impl BitOr for U256 {
    type Output = U256;

    fn bitor(self, rhs: Self) -> Self::Output {
        U256([
            self.0[0] | rhs.0[0],
            self.0[1] | rhs.0[1],
            self.0[2] | rhs.0[2],
            self.0[3] | rhs.0[3],
            self.0[4] | rhs.0[4],
            self.0[5] | rhs.0[5],
            self.0[6] | rhs.0[6],
            self.0[7] | rhs.0[7],
        ])
    }
}

impl BitAnd for U256 {
    type Output = U256;

    fn bitand(self, rhs: Self) -> Self::Output {
        U256([
            self.0[0] & rhs.0[0],
            self.0[1] & rhs.0[1],
            self.0[2] & rhs.0[2],
            self.0[3] & rhs.0[3],
            self.0[4] & rhs.0[4],
            self.0[5] & rhs.0[5],
            self.0[6] & rhs.0[6],
            self.0[7] & rhs.0[7],
        ])
    }
}

impl BitOrAssign for U256 {
    fn bitor_assign(&mut self, rhs: Self) {
        *self = *self | rhs;
    }
}
