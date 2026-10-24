use std::fmt::{self, Display};
use std::ops::{Add, AddAssign, Sub, SubAssign, Mul, MulAssign, Div, DivAssign, Neg};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ModInt {
    val: u32,
    modulus: u32,
}

impl ModInt {

    pub fn new(v: i64, modulus: u32) -> Self {

        let val = v.rem_euclid(i64::from(modulus)) as u32;

        ModInt {
            val,
            modulus,
        }
    }

    #[inline(always)]
    pub fn new_998244353(v: i64) -> Self {
        const MODULUS : u32 = 998244353;

        Self::new(v, MODULUS)
    }

    #[inline(always)]
    pub fn new_1000000007(v: i64) -> Self {
        const MODULUS : u32 = 1000000007;

        Self::new(v, MODULUS)
    }

    #[inline(always)]
    pub fn value(&self) -> u32 {
        self.val
    }

    #[inline(always)]
    pub fn modulus(&self) -> u32 {
        self.modulus
    }

    pub fn pow(&self, n: i64) -> Self {

        if n < 0 {
            return self.inv().pow(-n);
        }

        let mut n = n;
        let mut base = *self;
        let mut res = ModInt::new(1, self.modulus);

        while 0 < n {

            if (n & 1) == 1 {
                res *= base;
            }

            base *= base;
            n >>= 1;
        }

        res
    }

    pub fn inv(&self) -> Self {

        let (mut a, mut b) = (self.val as i32, self.modulus as i32);
        let (mut p, mut q) = (1, 0);

        while b != 0 {
            let t = a / b;
            a -= t * b;
            std::mem::swap(&mut a, &mut b);
            p -= t * q;
            std::mem::swap(&mut p, &mut q);
        }

        if a != 1 {
            panic!("No modular inverse exists (gcd != 1): {}", a);
        }

        Self::new(p.into(), self.modulus)
    }

}

impl Add for ModInt {

    type Output = Self;
    fn add(mut self, rhs: Self) -> Self::Output {
        self.val = (self.value() + rhs.value()) % self.modulus();

        self
    }
}

impl AddAssign for ModInt {
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}

impl Sub for ModInt {

    type Output = Self;
    fn sub(mut self, rhs: Self) -> Self::Output {
        self.val = (self.modulus() + self.value() - rhs.value()) % self.modulus();

        self
    }
}

impl SubAssign for ModInt {
    fn sub_assign(&mut self, rhs: Self) {
        *self = *self - rhs;
    }
}

impl Mul for ModInt {

    type Output = Self;
    fn mul(mut self, rhs: Self) -> Self::Output {

        // u32 * u32
        self.val = ((self.val as u64) * (rhs.val as u64) % (self.modulus as u64)) as u32;

        self

    }
}

impl MulAssign for ModInt {
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs;
    }
}

impl Div for ModInt {

    type Output = Self;
    fn div(self, rhs: Self) -> Self::Output {
        self * rhs.inv()
    }
}

impl DivAssign for ModInt {
    fn div_assign(&mut self, rhs: Self) {
        *self = *self / rhs;
    }
}

impl Neg for ModInt {

    type Output = Self;
    fn neg(mut self) -> Self::Output {

        self.val = if self.val == 0 { 0 } else { self.modulus() - self.value() };

        self
    }
}

impl Display for ModInt {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.val)
    }
}

macro_rules! impl_modint_ops {

    ($($t:ty)*) => ($(

        // ModInt + T
        impl Add<$t> for ModInt {
            type Output = Self;
            fn add(self, rhs: $t) -> Self::Output {
                self + ModInt::new(rhs as i64, self.modulus)
            }
        }

        // ModInt += T
        impl AddAssign<$t> for ModInt {
            fn add_assign(&mut self, rhs: $t) {
                *self = *self + rhs;
            }
        }

        // ModInt - T
        impl Sub<$t> for ModInt {
            type Output = Self;
            fn sub(self, rhs: $t) -> Self::Output {
                self - ModInt::new(rhs as i64, self.modulus)
            }
        }

        // ModInt -= T
        impl SubAssign<$t> for ModInt {
            fn sub_assign(&mut self, rhs: $t) {
                *self = *self - rhs;
            }
        }

        // ModInt * T
        impl Mul<$t> for ModInt {
            type Output = Self;
            fn mul(self, rhs: $t) -> Self::Output {
                self * ModInt::new(rhs as i64, self.modulus)
            }
        }

        // ModInt *= T
        impl MulAssign<$t> for ModInt {
            fn mul_assign(&mut self, rhs: $t) {
                *self = *self * rhs;
            }
        }

        // ModInt / T
        impl Div<$t> for ModInt {
            type Output = Self;
            fn div(self, rhs: $t) -> Self::Output {
                self / ModInt::new(rhs as i64, self.modulus)
            }
        }

        // ModInt /= T
        impl DivAssign<$t> for ModInt {
            fn div_assign(&mut self, rhs: $t) {
                *self = *self / rhs;
            }
        }

        // T + ModInt
        impl Add<ModInt> for $t {
            type Output = ModInt;
            fn add(self, rhs: ModInt) -> ModInt {
                rhs + self
            }
        }

        // T - ModInt
        impl Sub<ModInt> for $t {
            type Output = ModInt;
            fn sub(self, rhs: ModInt) -> ModInt {
                ModInt::new(self as i64, rhs.modulus) - rhs
            }
        }

        // T * ModInt
        impl Mul<ModInt> for $t {
            type Output = ModInt;
            fn mul(self, rhs: ModInt) -> ModInt {
                rhs * self
            }
        }

        // T / ModInt
        impl Div<ModInt> for $t {
            type Output = ModInt;
            fn div(self, rhs: ModInt) -> ModInt {
                ModInt::new(self as i64, rhs.modulus) / rhs
            }
        }

    )*)
}

impl_modint_ops!(i8 i16 i32 i64 isize u8 u16 u32 u64 usize);

#[cfg(test)]
mod tests {
    use super::ModInt;
    use std::hint::black_box;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::time::Instant;

    fn gcd(mut a: u32, mut b: u32) -> u32 {
        while b != 0 {
            let remainder = a % b;
            a = b;
            b = remainder;
        }
        a
    }

    fn panics(f: impl FnOnce()) -> bool {
        catch_unwind(AssertUnwindSafe(f)).is_err()
    }

    #[test]
    fn new_998244353_normalizes_value() {
        // Arrange
        let value = -1;
        let expected = 998_244_352;

        // Act
        let actual = ModInt::new_998244353(value).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn new_998244353_retains_modulus() {
        // Arrange
        let value = 1;
        let expected = 998_244_353;

        // Act
        let actual = ModInt::new_998244353(value).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn new_1000000007_normalizes_value() {
        // Arrange
        let value = -1;
        let expected = 1_000_000_006;

        // Act
        let actual = ModInt::new_1000000007(value).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn new_1000000007_retains_modulus() {
        // Arrange
        let value = 1;
        let expected = 1_000_000_007;

        // Act
        let actual = ModInt::new_1000000007(value).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn value_equal_to_modulus_is_normalized_to_zero() {
        // Arrange
        let modulus = 7;
        let value = i64::from(modulus);
        let expected = 0;

        // Act
        let actual = ModInt::new(value, modulus).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn positive_value_multiple_periods_above_modulus_is_normalized() {
        // Arrange
        let modulus = 5;
        let value = 17;
        let expected = 2;

        // Act
        let actual = ModInt::new(value, modulus).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_value_multiple_periods_below_zero_is_normalized() {
        // Arrange
        let modulus = 5;
        let value = -17;
        let expected = 3;

        // Act
        let actual = ModInt::new(value, modulus).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn modulus_one_normalizes_every_value_to_zero() {
        // Arrange
        let modulus = 1;
        let values = [i64::MIN, -1, 0, 1, i64::MAX];
        let expected = [0; 5];

        // Act
        let actual = values.map(|value| ModInt::new(value, modulus).value());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_modulus_panics() {
        // Arrange
        let value = 1;
        let modulus = 0;

        // Act
        let actual_result = panics(|| {
            ModInt::new(value, modulus);
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn negative_modulus_multiple_is_normalized_to_zero() {
        // Arrange
        let modulus = 7;
        let value = -i64::from(modulus);
        let expected = 0;

        // Act
        let actual = ModInt::new(value, modulus).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn i64_max_is_normalized_with_u32_max_modulus() {
        // Arrange
        let modulus = u32::MAX;
        let value = i64::MAX;
        let expected = 2_147_483_647;

        // Act
        let actual = ModInt::new(value, modulus).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn i64_min_is_normalized_without_overflow() {
        // Arrange
        let modulus = 7;
        let value = i64::MIN;
        let expected = 6;

        // Act
        let actual = ModInt::new(value, modulus).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_signed_integer_on_right_is_normalized() {
        // Arrange
        let value = ModInt::new(3, 7);
        let right = -2_i64;
        let expected = 1;

        // Act
        let actual = (value + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_signed_integer_on_left_is_normalized() {
        // Arrange
        let left = -2_i64;
        let value = ModInt::new(3, 7);
        let expected = 2;

        // Act
        let actual = (left - value).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn u64_max_on_right_is_cast_to_negative_i64() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = u64::MAX;
        let expected = 1;

        // Act
        let actual = (left + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn u64_max_on_left_is_cast_to_negative_i64() {
        // Arrange
        let left = u64::MAX;
        let right = ModInt::new(2, 7);
        let expected = 4;

        // Act
        let actual = (left - right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn usize_max_on_right_is_cast_to_negative_i64() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = usize::MAX;
        let expected = 1;

        // Act
        let actual = (left + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn usize_max_on_left_is_cast_to_negative_i64() {
        // Arrange
        let left = usize::MAX;
        let right = ModInt::new(2, 7);
        let expected = 4;

        // Act
        let actual = (left - right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn modulus_accessor_returns_constructor_modulus() {
        // Arrange
        let value = 12;
        let modulus = 17;
        let expected = modulus;

        // Act
        let actual = ModInt::new(value, modulus).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn pow_one_returns_original_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let exponent = 1;
        let expected = value;

        // Act
        let actual = value.pow(exponent);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_power_of_non_invertible_value_panics() {
        // Arrange
        let value = ModInt::new(2, 4);
        let exponent = -1;

        // Act
        let actual_result = panics(|| {
            value.pow(exponent);
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn pow_retains_modulus() {
        // Arrange
        let value = ModInt::new(3, 7);
        let exponent = 5;
        let expected = value.modulus();

        // Act
        let actual = value.pow(exponent).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn pow_zero_returns_multiplicative_identity() {
        // Arrange
        let value = ModInt::new(3, 7);
        let exponent = 0;
        let expected = 1;

        // Act
        let actual = value.pow(exponent).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn large_positive_power_matches_repeated_multiplication() {
        // Arrange
        let value = ModInt::new(2, 1_000_000_007);
        let exponent = 100;
        let mut expected = ModInt::new(1, value.modulus());
        for _ in 0..exponent {
            expected *= value;
        }

        // Act
        let actual = value.pow(exponent);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negative_power_matches_positive_power_of_inverse() {
        // Arrange
        let value = ModInt::new(3, 7);
        let exponent = -2;
        let expected = value.inv().pow(2);

        // Act
        let actual = value.pow(exponent);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    #[cfg(debug_assertions)]
    fn i64_min_power_panics_when_negated() {
        // Arrange
        let value = ModInt::new(1, 7);
        let exponent = i64::MIN;

        // Act
        let actual_result = panics(|| {
            value.pow(exponent);
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn exhaustive_small_nonnegative_powers_match_reference() {
        // Arrange
        let moduli = [1_u32, 2, 3, 5, 8, 11];
        let exponents = 0_i64..=10;
        let cases: Vec<(u32, u32, i64, ModInt)> = moduli
            .into_iter()
            .flat_map(|modulus| {
                (0..modulus).flat_map({
                    let exponents = exponents.clone();
                    move |raw_value| {
                        exponents.clone().map(move |exponent| {
                            let value = ModInt::new(i64::from(raw_value), modulus);
                            let expected =
                                (0..exponent).fold(ModInt::new(1, modulus), |acc, _| acc * value);
                            (modulus, raw_value, exponent, expected)
                        })
                    }
                })
            })
            .collect();
        let expected_values: Vec<ModInt> =
            cases.iter().map(|(_, _, _, expected)| *expected).collect();

        // Act
        let actual_values: Vec<ModInt> = cases
            .iter()
            .map(|(modulus, raw_value, exponent, _)| {
                ModInt::new(i64::from(*raw_value), *modulus).pow(*exponent)
            })
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn inverse_under_prime_modulus_matches_known_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = 5;

        // Act
        let actual = value.inv().value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn inverse_under_composite_modulus_exists_for_coprime_value() {
        // Arrange
        let value = ModInt::new(3, 10);
        let expected = 7;

        // Act
        let actual = value.inv().value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn inverse_of_non_coprime_value_panics() {
        // Arrange
        let value = ModInt::new(2, 4);

        // Act
        let actual_result = panics(|| {
            value.inv();
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn inverse_under_modulus_one_returns_zero() {
        // Arrange
        let value = ModInt::new(0, 1);
        let expected = 0;

        // Act
        let actual = value.inv().value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn inverse_of_one_with_modulus_above_i32_max_follows_build_overflow_behavior() {
        // Arrange
        let value = ModInt::new(1, 2_147_483_648);
        let expected_release = 1;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| value.inv().value()));

        // Assert
        if cfg!(debug_assertions) {
            assert!(actual_result.is_err());
        } else {
            assert_eq!(actual_result.unwrap(), expected_release);
        }
    }

    #[test]
    fn inverse_of_two_under_u32_max_modulus_panics() {
        // Arrange
        let value = ModInt::new(2, u32::MAX);

        // Act
        let actual_result = panics(|| {
            value.inv();
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn inverse_retains_modulus() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = value.modulus();

        // Act
        let actual = value.inv().modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn exhaustive_small_modulus_inverses_satisfy_identity() {
        // Arrange
        let moduli = [1_u32, 2, 3, 5, 8, 11];
        let cases: Vec<(u32, u32, u32)> = moduli
            .into_iter()
            .flat_map(|modulus| {
                (0..modulus)
                    .filter(move |raw_value| gcd(*raw_value, modulus) == 1)
                    .map(move |raw_value| (modulus, raw_value, ModInt::new(1, modulus).value()))
            })
            .collect();
        let expected_values: Vec<u32> = cases.iter().map(|(_, _, expected)| *expected).collect();

        // Act
        let actual_values: Vec<u32> = cases
            .iter()
            .map(|(modulus, raw_value, _)| {
                let value = ModInt::new(i64::from(*raw_value), *modulus);
                (value * value.inv()).value()
            })
            .collect();
        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn inverse_of_zero_panics() {
        // Arrange
        let value = ModInt::new(0, 7);

        // Act
        let actual_result = panics(|| {
            value.inv();
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn reversed_cross_modulus_addition_uses_new_left_modulus() {
        // Arrange
        let left = ModInt::new(4, 7);
        let right = ModInt::new(1, 5);
        let expected = 5;

        // Act
        let actual = (left + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn modint_plus_all_integer_types_returns_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [5; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (value + operand_i8).value(),
            (value + operand_i16).value(),
            (value + operand_i32).value(),
            (value + operand_i64).value(),
            (value + operand_isize).value(),
            (value + operand_u8).value(),
            (value + operand_u16).value(),
            (value + operand_u32).value(),
            (value + operand_u64).value(),
            (value + operand_usize).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_integer_types_plus_modint_return_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [5; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (operand_i8 + value).value(),
            (operand_i16 + value).value(),
            (operand_i32 + value).value(),
            (operand_i64 + value).value(),
            (operand_isize + value).value(),
            (operand_u8 + value).value(),
            (operand_u16 + value).value(),
            (operand_u32 + value).value(),
            (operand_u64 + value).value(),
            (operand_usize + value).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn addition_below_modulus_is_not_reduced() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = ModInt::new(3, 7);
        let expected = 5;

        // Act
        let actual = (left + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn addition_equal_to_modulus_is_reduced_to_zero() {
        // Arrange
        let left = ModInt::new(3, 7);
        let right = ModInt::new(4, 7);
        let expected = 0;

        // Act
        let actual = (left + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn addition_retains_modulus() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = ModInt::new(3, 7);
        let expected = left.modulus();

        // Act
        let actual = (left + right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn addition_near_u32_max_modulus_follows_build_overflow_behavior() {
        // Arrange
        let modulus = u32::MAX;
        let left = ModInt::new(i64::from(modulus - 1), modulus);
        let right = ModInt::new(i64::from(modulus - 1), modulus);
        let expected_release = modulus - 3;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| (left + right).value()));

        // Assert
        if cfg!(debug_assertions) {
            assert!(actual_result.is_err());
        } else {
            assert_eq!(actual_result.unwrap(), expected_release);
        }
    }

    #[test]
    fn cross_modulus_addition_uses_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = 0;

        // Act
        let actual = (left + right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_addition_retains_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = left.modulus();

        // Act
        let actual = (left + right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn exhaustive_small_modulus_addition_matches_reference() {
        // Arrange
        let moduli = [1_u32, 2, 3, 5, 8, 11];
        let cases: Vec<(u32, u32, u32, u32)> = moduli
            .into_iter()
            .flat_map(|modulus| {
                (0..modulus).flat_map(move |left| {
                    (0..modulus).map(move |right| {
                        (
                            modulus,
                            left,
                            right,
                            ((u64::from(left) + u64::from(right)) % u64::from(modulus)) as u32,
                        )
                    })
                })
            })
            .collect();
        let expected_values: Vec<u32> = cases.iter().map(|(_, _, _, expected)| *expected).collect();

        // Act
        let actual_values: Vec<u32> = cases
            .iter()
            .map(|(modulus, left, right, _)| {
                (ModInt::new(i64::from(*left), *modulus) + ModInt::new(i64::from(*right), *modulus))
                    .value()
            })
            .collect();
        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    #[ignore = "manual performance comparison"]
    fn benchmark_repeated_addition() {
        // Arrange
        let iterations = 100_000_usize;
        let modulus = 998_244_353;
        let addend = ModInt::new(1, modulus);
        let mut value = ModInt::new(0, modulus);
        let expected = iterations as u32;

        // Act
        let start = Instant::now();
        for _ in 0..iterations {
            value = black_box(value + addend);
        }
        let elapsed = start.elapsed();
        let actual = value.value();

        // Assert
        eprintln!("repeated addition: {elapsed:?}");
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore = "manual performance comparison"]
    fn benchmark_repeated_primitive_addition() {
        // Arrange
        let iterations = 100_000_usize;
        let modulus = 998_244_353;
        let addend = 1_i64;
        let mut value = ModInt::new(0, modulus);
        let expected = iterations as u32;

        // Act
        let start = Instant::now();
        for _ in 0..iterations {
            value = black_box(value + addend);
        }
        let elapsed = start.elapsed();
        let actual = value.value();

        // Assert
        eprintln!("repeated primitive addition: {elapsed:?}");
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_assign_modint_matches_addition() {
        // Arrange
        let mut actual = ModInt::new(5, 7);
        let right = ModInt::new(4, 7);
        let expected = actual + right;

        // Act
        actual += right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn add_assign_supports_all_integer_types() {
        // Arrange
        let mut values = [ModInt::new(3, 7); 10];
        let expected = [5; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        values[0] += operand_i8;
        values[1] += operand_i16;
        values[2] += operand_i32;
        values[3] += operand_i64;
        values[4] += operand_isize;
        values[5] += operand_u8;
        values[6] += operand_u16;
        values[7] += operand_u32;
        values[8] += operand_u64;
        values[9] += operand_usize;
        let actual = values.map(|value| value.value());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn reversed_cross_modulus_subtraction_uses_new_left_modulus() {
        // Arrange
        let left = ModInt::new(4, 7);
        let right = ModInt::new(1, 5);
        let expected = 3;

        // Act
        let actual = (left - right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn modint_minus_all_integer_types_returns_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [1; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (value - operand_i8).value(),
            (value - operand_i16).value(),
            (value - operand_i32).value(),
            (value - operand_i64).value(),
            (value - operand_isize).value(),
            (value - operand_u8).value(),
            (value - operand_u16).value(),
            (value - operand_u32).value(),
            (value - operand_u64).value(),
            (value - operand_usize).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_integer_types_minus_modint_return_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [6; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (operand_i8 - value).value(),
            (operand_i16 - value).value(),
            (operand_i32 - value).value(),
            (operand_i64 - value).value(),
            (operand_isize - value).value(),
            (operand_u8 - value).value(),
            (operand_u16 - value).value(),
            (operand_u32 - value).value(),
            (operand_u64 - value).value(),
            (operand_usize - value).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_subtraction_with_large_right_value_follows_build_overflow_behavior() {
        // Arrange
        let left = ModInt::new(0, 5);
        let right = ModInt::new(6, 7);
        let expected_release = 0;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| (left - right).value()));

        // Assert
        if cfg!(debug_assertions) {
            assert!(actual_result.is_err());
        } else {
            assert_eq!(actual_result.unwrap(), expected_release);
        }
    }

    #[test]
    fn subtraction_below_zero_wraps_by_modulus() {
        // Arrange
        let left = ModInt::new(0, 7);
        let right = ModInt::new(1, 7);
        let expected = 6;

        // Act
        let actual = (left - right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtraction_retains_modulus() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = ModInt::new(3, 7);
        let expected = left.modulus();

        // Act
        let actual = (left - right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn subtraction_near_u32_max_modulus_follows_build_overflow_behavior() {
        // Arrange
        let modulus = u32::MAX;
        let left = ModInt::new(1, modulus);
        let right = ModInt::new(0, modulus);
        let expected_release = 0;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| (left - right).value()));

        // Assert
        if cfg!(debug_assertions) {
            assert!(actual_result.is_err());
        } else {
            assert_eq!(actual_result.unwrap(), expected_release);
        }
    }

    #[test]
    fn cross_modulus_subtraction_uses_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = 2;

        // Act
        let actual = (left - right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_subtraction_retains_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = left.modulus();

        // Act
        let actual = (left - right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn exhaustive_small_modulus_subtraction_matches_reference() {
        // Arrange
        let moduli = [1_u32, 2, 3, 5, 8, 11];
        let cases: Vec<(u32, u32, u32, u32)> = moduli
            .into_iter()
            .flat_map(|modulus| {
                (0..modulus).flat_map(move |left| {
                    (0..modulus).map(move |right| {
                        (modulus, left, right, (left + modulus - right) % modulus)
                    })
                })
            })
            .collect();
        let expected_values: Vec<u32> = cases.iter().map(|(_, _, _, expected)| *expected).collect();

        // Act
        let actual_values: Vec<u32> = cases
            .iter()
            .map(|(modulus, left, right, _)| {
                (ModInt::new(i64::from(*left), *modulus) - ModInt::new(i64::from(*right), *modulus))
                    .value()
            })
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    #[ignore = "manual performance comparison"]
    fn benchmark_repeated_subtraction() {
        // Arrange
        let iterations = 100_000_usize;
        let modulus = 998_244_353;
        let subtrahend = ModInt::new(1, modulus);
        let mut value = ModInt::new(0, modulus);
        let expected = modulus - iterations as u32;

        // Act
        let start = Instant::now();
        for _ in 0..iterations {
            value = black_box(value - subtrahend);
        }
        let elapsed = start.elapsed();
        let actual = value.value();

        // Assert
        eprintln!("repeated subtraction: {elapsed:?}");
        assert_eq!(actual, expected);
    }

    #[test]
    fn sub_assign_modint_matches_subtraction() {
        // Arrange
        let mut actual = ModInt::new(2, 7);
        let right = ModInt::new(5, 7);
        let expected = actual - right;

        // Act
        actual -= right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn sub_assign_supports_all_integer_types() {
        // Arrange
        let mut values = [ModInt::new(3, 7); 10];
        let expected = [1; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        values[0] -= operand_i8;
        values[1] -= operand_i16;
        values[2] -= operand_i32;
        values[3] -= operand_i64;
        values[4] -= operand_isize;
        values[5] -= operand_u8;
        values[6] -= operand_u16;
        values[7] -= operand_u32;
        values[8] -= operand_u64;
        values[9] -= operand_usize;
        let actual = values.map(|value| value.value());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn reversed_cross_modulus_multiplication_uses_new_left_modulus() {
        // Arrange
        let left = ModInt::new(4, 7);
        let right = ModInt::new(1, 5);
        let expected = 4;

        // Act
        let actual = (left * right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiplication_by_one_returns_original_value() {
        // Arrange
        let value = ModInt::new(5, 7);
        let one = ModInt::new(1, 7);
        let expected = value;

        // Act
        let actual = value * one;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn modint_times_all_integer_types_returns_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [6; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (value * operand_i8).value(),
            (value * operand_i16).value(),
            (value * operand_i32).value(),
            (value * operand_i64).value(),
            (value * operand_isize).value(),
            (value * operand_u8).value(),
            (value * operand_u16).value(),
            (value * operand_u32).value(),
            (value * operand_u64).value(),
            (value * operand_usize).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_integer_types_times_modint_return_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [6; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (operand_i8 * value).value(),
            (operand_i16 * value).value(),
            (operand_i32 * value).value(),
            (operand_i64 * value).value(),
            (operand_isize * value).value(),
            (operand_u8 * value).value(),
            (operand_u16 * value).value(),
            (operand_u32 * value).value(),
            (operand_u64 * value).value(),
            (operand_usize * value).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiplication_retains_modulus() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = ModInt::new(3, 7);
        let expected = left.modulus();

        // Act
        let actual = (left * right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn multiplication_near_u32_max_modulus_is_reduced() {
        // Arrange
        let modulus = u32::MAX;
        let value = ModInt::new(i64::from(modulus - 1), modulus);
        let expected = 1;

        // Act
        let actual = (value * value).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_multiplication_uses_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = 4;

        // Act
        let actual = (left * right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_multiplication_retains_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = left.modulus();

        // Act
        let actual = (left * right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn exhaustive_small_modulus_multiplication_matches_reference() {
        // Arrange
        let moduli = [1_u32, 2, 3, 5, 8, 11];
        let cases: Vec<_> = moduli
            .into_iter()
            .flat_map(|modulus| {
                (0..modulus).flat_map(move |left| {
                    (0..modulus).map(move |right| {
                        let expected =
                            ((u64::from(left) * u64::from(right)) % u64::from(modulus)) as u32;
                        (modulus, left, right, expected)
                    })
                })
            })
            .collect();
        let expected_values: Vec<_> = cases.iter().map(|(_, _, _, expected)| *expected).collect();

        // Act
        let actual_values: Vec<_> = cases
            .iter()
            .map(|&(modulus, left, right, _)| {
                (ModInt::new(i64::from(left), modulus) * ModInt::new(i64::from(right), modulus))
                    .value()
            })
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn multiplication_by_zero_returns_zero() {
        // Arrange
        let value = ModInt::new(5, 7);
        let zero = ModInt::new(0, 7);
        let expected = 0;

        // Act
        let actual = (value * zero).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore = "manual performance comparison"]
    fn benchmark_repeated_multiplication() {
        // Arrange
        let iterations = 10_000_usize;
        let modulus = 998_244_353;
        let multiplier = ModInt::new(3, modulus);
        let mut value = ModInt::new(1, modulus);
        let expected_upper_bound = modulus;

        // Act
        let start = Instant::now();
        for _ in 0..iterations {
            value = black_box(value * multiplier);
        }
        let elapsed = start.elapsed();
        let actual = value.value();

        // Assert
        eprintln!("repeated multiplication: {elapsed:?}");
        assert!(actual < expected_upper_bound);
    }

    #[test]
    fn mul_assign_modint_matches_multiplication() {
        // Arrange
        let mut actual = ModInt::new(5, 7);
        let right = ModInt::new(4, 7);
        let expected = actual * right;

        // Act
        actual *= right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn mul_assign_supports_all_integer_types() {
        // Arrange
        let mut values = [ModInt::new(3, 7); 10];
        let expected = [6; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        values[0] *= operand_i8;
        values[1] *= operand_i16;
        values[2] *= operand_i32;
        values[3] *= operand_i64;
        values[4] *= operand_isize;
        values[5] *= operand_u8;
        values[6] *= operand_u16;
        values[7] *= operand_u32;
        values[8] *= operand_u64;
        values[9] *= operand_usize;
        let actual = values.map(|value| value.value());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn reversed_cross_modulus_division_uses_new_left_modulus() {
        // Arrange
        let left = ModInt::new(4, 7);
        let right = ModInt::new(1, 5);
        let expected = 4;

        // Act
        let actual = (left / right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn division_matches_known_value() {
        // Arrange
        let numerator = ModInt::new(5, 7);
        let denominator = ModInt::new(3, 7);
        let expected = 4;

        // Act
        let actual = (numerator / denominator).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn division_by_non_invertible_value_panics() {
        // Arrange
        let numerator = ModInt::new(1, 4);
        let denominator = ModInt::new(2, 4);

        // Act
        let actual_result = panics(|| {
            let _ = numerator / denominator;
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn modint_divided_by_all_integer_types_returns_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [5; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (value / operand_i8).value(),
            (value / operand_i16).value(),
            (value / operand_i32).value(),
            (value / operand_i64).value(),
            (value / operand_isize).value(),
            (value / operand_u8).value(),
            (value / operand_u16).value(),
            (value / operand_u32).value(),
            (value / operand_u64).value(),
            (value / operand_usize).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn all_integer_types_divided_by_modint_return_expected_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = [3; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        let actual = [
            (operand_i8 / value).value(),
            (operand_i16 / value).value(),
            (operand_i32 / value).value(),
            (operand_i64 / value).value(),
            (operand_isize / value).value(),
            (operand_u8 / value).value(),
            (operand_u16 / value).value(),
            (operand_u32 / value).value(),
            (operand_u64 / value).value(),
            (operand_usize / value).value(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn division_retains_modulus() {
        // Arrange
        let left = ModInt::new(2, 7);
        let right = ModInt::new(3, 7);
        let expected = left.modulus();

        // Act
        let actual = (left / right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn division_near_u32_max_modulus_panics_when_inverse_is_calculated() {
        // Arrange
        let modulus = u32::MAX;
        let numerator = ModInt::new(1, modulus);
        let denominator = ModInt::new(2, modulus);

        // Act
        let actual_result = panics(|| {
            let _ = numerator / denominator;
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn cross_modulus_division_uses_right_inverse_and_left_result_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(2, 7);
        let expected = 4;

        // Act
        let actual = (left / right).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_division_retains_left_modulus() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(2, 7);
        let expected = left.modulus();

        // Act
        let actual = (left / right).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn division_by_zero_panics() {
        // Arrange
        let numerator = ModInt::new(1, 7);
        let denominator = ModInt::new(0, 7);

        // Act
        let actual_result = panics(|| {
            let _ = numerator / denominator;
        });

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn div_assign_modint_matches_division() {
        // Arrange
        let mut actual = ModInt::new(5, 7);
        let right = ModInt::new(3, 7);
        let expected = actual / right;

        // Act
        actual /= right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn div_assign_supports_all_integer_types() {
        // Arrange
        let mut values = [ModInt::new(3, 7); 10];
        let expected = [5; 10];

        let operand_i8 = 2_i8;
        let operand_i16 = 2_i16;
        let operand_i32 = 2_i32;
        let operand_i64 = 2_i64;
        let operand_isize = 2_isize;
        let operand_u8 = 2_u8;
        let operand_u16 = 2_u16;
        let operand_u32 = 2_u32;
        let operand_u64 = 2_u64;
        let operand_usize = 2_usize;
        // Act
        values[0] /= operand_i8;
        values[1] /= operand_i16;
        values[2] /= operand_i32;
        values[3] /= operand_i64;
        values[4] /= operand_isize;
        values[5] /= operand_u8;
        values[6] /= operand_u16;
        values[7] /= operand_u32;
        values[8] /= operand_u64;
        values[9] /= operand_usize;
        let actual = values.map(|value| value.value());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn double_negation_returns_original_value() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = value;

        // Act
        let actual = -(-value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negation_retains_modulus() {
        // Arrange
        let value = ModInt::new(3, 7);
        let expected = value.modulus();

        // Act
        let actual = (-value).modulus();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn negation_of_zero_returns_zero() {
        // Arrange
        let value = ModInt::new(0, 7);
        let expected = 0;

        // Act
        let actual = (-value).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn display_outputs_only_normalized_decimal_value() {
        // Arrange
        let value = ModInt::new(-1, 7);
        let expected = "6";

        // Act
        let actual = value.to_string();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn same_value_with_different_modulus_is_not_equal() {
        // Arrange
        let left = ModInt::new(1, 5);
        let right = ModInt::new(1, 7);
        let expected = false;

        // Act
        let actual = left == right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn zero_power_with_modulus_one_returns_normalized_identity() {
        // Arrange
        let value = ModInt::new(0, 1);
        let exponent = 0;
        let expected = 0;

        // Act
        let actual = value.pow(exponent).value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_add_assign_uses_left_modulus() {
        // Arrange
        let mut actual = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = ModInt::new(0, 5);

        // Act
        actual += right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_sub_assign_uses_left_modulus() {
        // Arrange
        let mut actual = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = ModInt::new(2, 5);

        // Act
        actual -= right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_mul_assign_uses_left_modulus() {
        // Arrange
        let mut actual = ModInt::new(1, 5);
        let right = ModInt::new(4, 7);
        let expected = ModInt::new(4, 5);

        // Act
        actual *= right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn cross_modulus_div_assign_uses_right_inverse_and_left_result_modulus() {
        // Arrange
        let mut actual = ModInt::new(1, 5);
        let right = ModInt::new(2, 7);
        let expected = ModInt::new(4, 5);

        // Act
        actual /= right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn congruent_inputs_with_same_modulus_are_equal() {
        // Arrange
        let left = ModInt::new(1, 7);
        let right = ModInt::new(8, 7);
        let expected = true;

        // Act
        let actual = left == right;

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    #[ignore = "manual performance comparison"]
    fn benchmark_repeated_construction() {
        // Arrange
        let iterations = 100_000_i64;
        let modulus = 998_244_353;
        let mut checksum = 0_u64;
        let expected_lower_bound = 0;

        // Act
        let start = Instant::now();
        for raw_value in 0..iterations {
            checksum = black_box(checksum + u64::from(ModInt::new(raw_value, modulus).value()));
        }
        let elapsed = start.elapsed();
        let actual = checksum;

        // Assert
        eprintln!("repeated construction: {elapsed:?}");
        assert!(expected_lower_bound < actual);
    }
}
