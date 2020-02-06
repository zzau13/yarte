use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign};

pub trait YNumber: Copy {
    fn zero() -> Self;
    fn neq_zero(&self) -> bool;
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
                fn neq_zero(&self) -> bool {
                    *self != 0
                }
            }
        )*
    };
}

// TODO: study not use u64 and u128
impl_ynumber!(u8 u16 u32 u64 u128);

#[derive(Clone, Copy)]
pub struct U256([u32; 8]);

impl YNumber for U256 {
    #[inline(always)]
    fn zero() -> Self {
        U256([0, 0, 0, 0, 0, 0, 0, 0])
    }

    #[inline(always)]
    fn neq_zero(&self) -> bool {
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

impl BitAndAssign for U256 {
    fn bitand_assign(&mut self, rhs: Self) {
        *self = *self & rhs;
    }
}
