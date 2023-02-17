use std::{ops::{Mul, Sub, Add, AddAssign, MulAssign, SubAssign, Div, DivAssign}, fmt::Display};

pub trait Numeric: Sized + Copy + Mul<Output = Self> + Sub<Output = Self> + Add<Output = Self> + Div<Output = Self> + AddAssign + MulAssign + SubAssign + DivAssign + Display {
    const ZERO: Self;
    const ONE: Self;
}


macro_rules! impl_numeric {
    ($t:ty, $z:literal, $o:literal) => {
        impl Numeric for $t {
            const ZERO: Self = $z;
            const ONE: Self = $o;
        }
    };
    ($z:literal, $o:literal => $($t:ty),+) => {
        $(impl_numeric!($t, $z, $o);)+
    }
}

impl_numeric!(0, 1 => i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128,  usize);
impl_numeric!(0.0, 1.0 => f32, f64);
