use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

pub trait CapacityValue:
    Copy + Ord + Default + Add<Output = Self> + AddAssign + Sub<Output = Self> + SubAssign
{
}

macro_rules! impl_capacity_value {
    ($($type:ty),* $(,)?) => {
        $(
            impl CapacityValue for $type {}
        )*
    };
}

impl_capacity_value!(i32, i64, i128, isize);

pub trait CostValue:
    Copy
    + Ord
    + Default
    + Add<Output = Self>
    + AddAssign
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Neg<Output = Self>
{
    fn max_value() -> Self;
}

macro_rules! impl_cost_value {
    ($($type:ty),* $(,)?) => {
        $(
            impl CostValue for $type {
                fn max_value() -> Self {
                    <$type>::MAX
                }
            }
        )*
    };
}

impl_cost_value!(i32, i64, i128, isize);
