trait NumericInteger: Copy + Ord {
    fn unsigned_abs(self) -> u128;
    fn is_negative(self) -> bool;
    fn try_from_magnitude(magnitude: u128, negative: bool) -> Option<Self>;
}

macro_rules! impl_numeric_integer {
    ($($t:ty),* $(,)?) => {
        $(
            impl NumericInteger for $t {
                fn unsigned_abs(self) -> u128 {
                    <$t>::unsigned_abs(self) as u128
                }

                fn is_negative(self) -> bool {
                    <$t>::is_negative(self)
                }

                fn try_from_magnitude(magnitude: u128, negative: bool) -> Option<Self> {
                    let maximum_positive = <$t>::MAX as u128;
                    let maximum_negative = maximum_positive + 1;

                    if negative {
                        if magnitude == maximum_negative {
                            Some(<$t>::MIN)
                        } else if magnitude <= maximum_positive {
                            Some(-(magnitude as $t))
                        } else {
                            None
                        }
                    } else if magnitude <= maximum_positive {
                        Some(magnitude as $t)
                    } else {
                        None
                    }
                }
            }
        )*
    };
}

impl_numeric_integer!(i8, i16, i32, i64, i128, isize);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NumericError {
    InvalidBase(usize),
    InvalidDigit { digit: char, base: usize },
    InvalidReplacementDigit(usize),
    EmptyRepresentation,
    Overflow,
}

#[derive(Debug, Clone, Copy)]
struct Numeric<T>
where
    T: NumericInteger,
{
    number: T,
    magnitude: u128,
    len: Option<usize>,
}

impl<T> Numeric<T>
where
    T: NumericInteger,
{
    pub fn new(n: T) -> Self {
        Numeric {
            number: n,
            magnitude: n.unsigned_abs(),
            len: None,
        }
    }

    fn with_original_sign(&self, magnitude: u128) -> Option<T> {
        T::try_from_magnitude(magnitude, self.number.is_negative())
    }

    pub fn value(&self) -> T {
        self.number
    }

    pub fn abs_value(&self) -> u128 {
        self.magnitude
    }

    pub fn len(&mut self) -> usize {
        if let Some(len) = self.len {
            return len;
        }

        let mut len = 1;
        let mut now = self.magnitude;

        while 10 <= now {
            now /= 10;
            len += 1;
        }

        self.len = Some(len);
        len
    }

    // 0 - indexed
    pub fn digit(&mut self, right_pos: usize) -> usize {
        assert!(
            right_pos < self.len(),
            "right_pos must be less than the digit length"
        );

        let power = 10u128.pow(right_pos as u32);
        ((self.magnitude / power) % 10) as usize
    }

    pub fn digit_sum(&self) -> usize {
        let mut result = 0;
        let mut tmp = self.magnitude;

        while 0 < tmp {
            result += (tmp % 10) as usize;
            tmp /= 10;
        }

        result
    }

    pub fn reversed_number(&self) -> Option<T> {
        let mut result = 0u128;
        let mut tmp = self.magnitude;
        while 0 < tmp {
            result = result.checked_mul(10)?.checked_add(tmp % 10)?;
            tmp /= 10;
        }

        self.with_original_sign(result)
    }

    // 1 - indexed
    pub fn round(&self, place: usize) -> Option<T> {
        if place == 0 {
            return Some(self.number);
        }

        let Some(power) = Self::decimal_power(place) else {
            return self.with_original_sign(0);
        };
        let remainder = self.magnitude % power;
        let result = if power / 2 <= remainder {
            self.magnitude.checked_add(power - remainder)?
        } else {
            self.magnitude - remainder
        };

        self.with_original_sign(result)
    }

    // 1 - indexed
    pub fn continuous_round(&self, round_to_place: usize) -> Option<T> {
        if round_to_place == 0 {
            return Some(self.number);
        }

        let mut result = self.magnitude;
        let mut power = 1u128;

        for _ in 0..round_to_place {
            let Some(next_power) = power.checked_mul(10) else {
                result = 0;
                break;
            };
            power = next_power;
            let remainder = result % power;

            if power / 2 <= remainder {
                result = result.checked_add(power - remainder)?;
            } else {
                result -= remainder;
            }

            if result == 0 {
                break;
            }
        }

        self.with_original_sign(result)
    }

    pub fn convert_n_adic_number(&self, n: usize) -> Result<String, NumericError> {
        Self::validate_base(n)?;
        if self.magnitude == 0 {
            return Ok("0".to_owned());
        }

        const DIGITS: &[u8; 36] = b"0123456789ABCDEFGHIJKLMNOPQRSTUVWXYZ";
        let mut magnitude = self.magnitude;
        let mut digits = Vec::new();
        while 0 < magnitude {
            digits.push(DIGITS[(magnitude % n as u128) as usize] as char);
            magnitude /= n as u128;
        }
        if self.number.is_negative() {
            digits.push('-');
        }
        digits.reverse();

        Ok(digits.into_iter().collect())
    }

    pub fn revert_n_adic_number(value: &str, n: usize) -> Result<T, NumericError> {
        Self::validate_base(n)?;
        let (negative, digits) = match value.strip_prefix('-') {
            Some(digits) => (true, digits),
            None => (false, value),
        };
        if digits.is_empty() {
            return Err(NumericError::EmptyRepresentation);
        }

        let mut magnitude = 0u128;
        for character in digits.chars() {
            let Some(digit) = character.to_digit(36) else {
                return Err(NumericError::InvalidDigit {
                    digit: character,
                    base: n,
                });
            };
            if n <= digit as usize {
                return Err(NumericError::InvalidDigit {
                    digit: character,
                    base: n,
                });
            }
            magnitude = magnitude
                .checked_mul(n as u128)
                .and_then(|value| value.checked_add(digit as u128))
                .ok_or(NumericError::Overflow)?;
        }

        T::try_from_magnitude(magnitude, negative).ok_or(NumericError::Overflow)
    }

    pub fn replace_digit(&self, old_digit: usize, new_digit: usize) -> Result<T, NumericError> {
        for digit in [old_digit, new_digit] {
            if 9 < digit {
                return Err(NumericError::InvalidReplacementDigit(digit));
            }
        }

        if self.magnitude == 0 {
            let magnitude = if old_digit == 0 { new_digit as u128 } else { 0 };
            return self
                .with_original_sign(magnitude)
                .ok_or(NumericError::Overflow);
        }

        let mut tmp = self.magnitude;
        let mut place = 1u128;
        let mut result = 0u128;
        while 0 < tmp {
            let digit = (tmp % 10) as usize;
            let replacement = if digit == old_digit { new_digit } else { digit } as u128;
            result = result
                .checked_add(
                    replacement
                        .checked_mul(place)
                        .ok_or(NumericError::Overflow)?,
                )
                .ok_or(NumericError::Overflow)?;
            tmp /= 10;
            if 0 < tmp {
                place = place.checked_mul(10).ok_or(NumericError::Overflow)?;
            }
        }

        self.with_original_sign(result)
            .ok_or(NumericError::Overflow)
    }

    pub fn contains(&self, digit: usize) -> bool {
        if 9 < digit {
            return false;
        }

        let mut tmp = self.magnitude;
        if tmp == 0 && digit == 0 {
            return true;
        }

        while 0 < tmp {
            if tmp % 10 == digit as u128 {
                return true;
            }

            tmp /= 10;
        }

        false
    }

    fn decimal_power(place: usize) -> Option<u128> {
        u32::try_from(place)
            .ok()
            .and_then(|exponent| 10u128.checked_pow(exponent))
    }

    fn validate_base(base: usize) -> Result<(), NumericError> {
        if (2..=36).contains(&base) {
            Ok(())
        } else {
            Err(NumericError::InvalidBase(base))
        }
    }
}

impl<T> PartialEq for Numeric<T>
where
    T: NumericInteger,
{
    fn eq(&self, other: &Self) -> bool {
        self.number == other.number
    }
}

impl<T> Eq for Numeric<T> where T: NumericInteger {}

impl<T> PartialOrd for Numeric<T>
where
    T: NumericInteger,
{
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T> Ord for Numeric<T>
where
    T: NumericInteger,
{
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.number.cmp(&other.number)
    }
}

impl<T> PartialEq<T> for Numeric<T>
where
    T: NumericInteger,
{
    fn eq(&self, other: &T) -> bool {
        self.number == *other
    }
}

impl<T> PartialOrd<T> for Numeric<T>
where
    T: NumericInteger,
{
    fn partial_cmp(&self, other: &T) -> Option<std::cmp::Ordering> {
        self.number.partial_cmp(other)
    }
}

#[cfg(test)]
mod api_tests {
    use super::*;
    use std::cmp::Ordering;

    #[test]
    fn new_preserves_unsigned_magnitude_for_each_signed_minimum() {
        // Arrange
        let input_value_1 = i8::MIN;
        let input_value_2 = i16::MIN;
        let input_value_3 = i32::MIN;
        let input_value_4 = i64::MIN;
        let input_value_5 = i128::MIN;
        let expected_i8 = 1u128 << 7;
        let expected_i16 = 1u128 << 15;
        let expected_i32 = 1u128 << 31;
        let expected_i64 = 1u128 << 63;
        let expected_i128 = 1u128 << 127;
        let minimum_isize = isize::MIN;
        let expected_isize = 1u128 << (isize::BITS - 1);
        let expected_check_1 = [
            expected_i8,
            expected_i16,
            expected_i32,
            expected_i64,
            expected_i128,
            expected_isize,
        ];

        // Act
        let actual = [
            Numeric::new(input_value_1).abs_value(),
            Numeric::new(input_value_2).abs_value(),
            Numeric::new(input_value_3).abs_value(),
            Numeric::new(input_value_4).abs_value(),
            Numeric::new(input_value_5).abs_value(),
            Numeric::new(minimum_isize).abs_value(),
        ];

        // Assert
        assert_eq!(actual, expected_check_1);
    }

    #[test]
    fn value_returns_original_number() {
        // Arrange
        let number = -12345i64;
        let numeric = Numeric::new(number);
        let expected = number;

        // Act
        let actual = numeric.value();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn len_handles_zero_decimal_boundaries_and_i64_max() {
        // Arrange
        let inputs = [0i64, 9, 10, 11, i64::MAX];
        let expected = [1usize, 1, 2, 2, 19];

        // Act
        let actual = inputs.map(|number| Numeric::new(number).len());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn digit_returns_each_digit_including_zeroes() {
        // Arrange
        let input_value_1 = 1200i64;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 3;
        let mut numeric = Numeric::new(input_value_1);
        let expected = [0usize, 0, 2, 1];

        // Act
        let actual = [
            numeric.digit(input_value_2),
            numeric.digit(input_value_3),
            numeric.digit(input_value_4),
            numeric.digit(input_value_5),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn digit_panics_when_position_equals_length() {
        // Arrange
        let number = 123i64;
        let mut numeric = Numeric::new(number);
        let position = numeric.len();
        let expected_message = "right_pos must be less than the digit length";
        let expected_check_2 = Some(expected_message);

        let expected_boolean_1 = true;

        // Act
        let panic_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| numeric.digit(position)));
        let actual_message = panic_result.as_ref().err().and_then(|payload| {
            payload
                .downcast_ref::<&str>()
                .copied()
                .or_else(|| payload.downcast_ref::<String>().map(String::as_str))
        });

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_message, expected_check_2);
    }

    #[test]
    fn digit_sum_handles_negative_number_and_internal_zeroes() {
        // Arrange
        let input_value_1 = -12i8;
        let input_value_2 = 10020i64;
        let negative = Numeric::new(input_value_1);
        let with_zeroes = Numeric::new(input_value_2);
        let expected = [3usize, 3];

        // Act
        let actual = [negative.digit_sum(), with_zeroes.digit_sum()];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn reversed_number_preserves_sign_and_drops_leading_zeroes() {
        // Arrange
        let input_value_1 = -12345i64;
        let input_value_2 = 1200i64;
        let negative = Numeric::new(input_value_1);
        let trailing_zeroes = Numeric::new(input_value_2);
        let expected = [Some(-54321i64), Some(21i64)];

        // Act
        let actual = [
            negative.reversed_number(),
            trailing_zeroes.reversed_number(),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn reversed_number_returns_none_when_result_exceeds_type() {
        // Arrange
        let input_value_1 = 127i8;
        let numeric = Numeric::new(input_value_1);
        let expected = None;

        // Act
        let actual = numeric.reversed_number();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn round_handles_boundaries_and_negative_values() {
        // Arrange
        let inputs = [9994i64, 9995, 9996, -12345, 12300];
        let places = [1usize, 1, 1, 2, 2];
        let expected = [
            Some(9990i64),
            Some(10000),
            Some(10000),
            Some(-12300),
            Some(12300),
        ];

        // Act
        let actual = std::array::from_fn(|index| Numeric::new(inputs[index]).round(places[index]));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn round_avoids_intermediate_overflow_when_result_fits() {
        // Arrange
        let input_value_1 = 118i8;
        let numeric = Numeric::new(input_value_1);
        let place = 1usize;
        let expected = Some(120i8);

        // Act
        let actual = numeric.round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn round_returns_none_when_final_result_exceeds_type() {
        // Arrange
        let input_value_1 = 127i8;
        let numeric = Numeric::new(input_value_1);
        let place = 1usize;
        let expected = None;

        // Act
        let actual = numeric.round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn round_returns_zero_when_power_exceeds_type() {
        // Arrange
        let input_value_1 = 1i8;
        let numeric = Numeric::new(input_value_1);
        let place = 3usize;
        let expected = Some(0i8);

        // Act
        let actual = numeric.round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[cfg(target_pointer_width = "64")]
    #[test]
    fn round_returns_zero_without_truncating_huge_place() {
        // Arrange
        let input_value_1 = 123i64;
        let numeric = Numeric::new(input_value_1);
        let place = u32::MAX as usize + 1;
        let expected = Some(0i64);

        // Act
        let actual = numeric.round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn continuous_round_handles_boundaries() {
        // Arrange
        let inputs = [4444i64, 4445, 4446];
        let place = 4usize;
        let expected = [Some(0i64), Some(10000), Some(10000)];

        // Act
        let actual = inputs.map(|number| Numeric::new(number).continuous_round(place));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn continuous_round_avoids_intermediate_overflow_when_result_fits() {
        // Arrange
        let input_value_1 = 118i8;
        let numeric = Numeric::new(input_value_1);
        let place = 1usize;
        let expected = Some(120i8);

        // Act
        let actual = numeric.continuous_round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn continuous_round_returns_none_when_final_result_exceeds_type() {
        // Arrange
        let input_value_1 = 127i8;
        let numeric = Numeric::new(input_value_1);
        let place = 1usize;
        let expected = None;

        // Act
        let actual = numeric.continuous_round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn continuous_round_returns_zero_when_power_exceeds_type() {
        // Arrange
        let input_value_1 = 1i8;
        let numeric = Numeric::new(input_value_1);
        let place = 3usize;
        let expected = Some(0i8);

        // Act
        let actual = numeric.continuous_round(place);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn base_conversion_uses_unambiguous_alphanumeric_representation() {
        // Arrange
        let input_value_1 = 71i8;
        let numeric = Numeric::new(input_value_1);
        let base = 36usize;
        let representation = "1Z";
        let expected_representation = Ok("1Z".to_owned());
        let expected_restored = Ok(71i8);

        // Act
        let actual_representation = numeric.convert_n_adic_number(base);
        let actual_restored = Numeric::<i8>::revert_n_adic_number(representation, base);

        // Assert
        assert_eq!(actual_representation, expected_representation);
        assert_eq!(actual_restored, expected_restored);
    }

    #[test]
    fn base_conversion_is_not_limited_by_internal_integer_digits() {
        // Arrange
        let input_value_1 = i8::MAX;
        let numeric = Numeric::new(input_value_1);
        let base = 2usize;
        let expected = Ok("1111111".to_owned());

        // Act
        let actual = numeric.convert_n_adic_number(base);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn base_conversion_rejects_out_of_range_bases() {
        // Arrange
        let input_value_1 = 10i64;
        let numeric = Numeric::new(input_value_1);
        let lower_invalid_base = 1usize;
        let upper_invalid_base = 37usize;
        let expected = [
            Err(NumericError::InvalidBase(lower_invalid_base)),
            Err(NumericError::InvalidBase(upper_invalid_base)),
        ];

        // Act
        let actual = [
            numeric.convert_n_adic_number(lower_invalid_base),
            numeric.convert_n_adic_number(upper_invalid_base),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn base_conversion_does_not_update_place_after_final_digit() {
        // Arrange
        let input_value_1 = 64i8;
        let numeric = Numeric::new(input_value_1);
        let base = 8usize;
        let representation = "100";
        let expected_converted = Ok("100".to_owned());
        let expected_restored = Ok(64i8);

        // Act
        let actual_converted = numeric.convert_n_adic_number(base);
        let actual_restored = Numeric::<i8>::revert_n_adic_number(representation, base);

        // Assert
        assert_eq!(actual_converted, expected_converted);
        assert_eq!(actual_restored, expected_restored);
    }

    #[test]
    fn base_conversion_round_trips_binary_octal_and_base_thirty_six() {
        // Arrange
        let cases = [(13i64, 2usize), (83, 8), (71, 36)];
        let expected = [Ok(13i64), Ok(83), Ok(71)];

        // Act
        let actual = cases.map(|(number, base)| {
            let representation = Numeric::new(number).convert_n_adic_number(base).unwrap();
            Numeric::<i64>::revert_n_adic_number(&representation, base)
        });

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn base_reversion_rejects_digit_outside_base() {
        // Arrange
        let representation = "128";
        let base = 8usize;
        let expected = Err(NumericError::InvalidDigit { digit: '8', base });

        // Act
        let actual = Numeric::<i64>::revert_n_adic_number(representation, base);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn base_reversion_returns_overflow_when_result_exceeds_type() {
        // Arrange
        let representation = "77";
        let base = 36usize;
        let expected = Err(NumericError::Overflow);

        // Act
        let actual = Numeric::<i8>::revert_n_adic_number(representation, base);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn replace_digit_treats_zero_as_one_zero_digit() {
        // Arrange
        let input_value_1 = 0i64;
        let input_value_2 = 0;
        let input_value_3 = 9;
        let numeric = Numeric::new(input_value_1);
        let expected = Ok(9i64);

        // Act
        let actual = numeric.replace_digit(input_value_2, input_value_3);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn replace_digit_rejects_old_and_new_digits_outside_decimal_range() {
        // Arrange
        let input_value_1 = 111i8;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let numeric = Numeric::new(input_value_1);
        let invalid_old_digit = 10usize;
        let invalid_new_digit = 128usize;
        let expected = [
            Err(NumericError::InvalidReplacementDigit(invalid_old_digit)),
            Err(NumericError::InvalidReplacementDigit(invalid_new_digit)),
        ];

        // Act
        let actual = [
            numeric.replace_digit(invalid_old_digit, input_value_2),
            numeric.replace_digit(input_value_3, invalid_new_digit),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn replace_digit_traverses_maximum_value_without_place_overflow() {
        // Arrange
        let input_value_1 = i8::MAX;
        let input_value_2 = 0;
        let numeric = Numeric::new(input_value_1);
        let absent_digit = 9usize;
        let expected = Ok(i8::MAX);

        // Act
        let actual = numeric.replace_digit(absent_digit, input_value_2);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn replace_digit_returns_overflow_when_result_exceeds_type() {
        // Arrange
        let input_value_1 = 127i8;
        let input_value_2 = 1;
        let input_value_3 = 9;
        let numeric = Numeric::new(input_value_1);
        let expected = Err(NumericError::Overflow);

        // Act
        let actual = numeric.replace_digit(input_value_2, input_value_3);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn replace_digit_replaces_all_matching_digits_and_preserves_sign() {
        // Arrange
        let input_value_1 = 12121i64;
        let input_value_2 = -1002i64;
        let input_value_3 = 1;
        let input_value_4 = 9;
        let input_value_5 = 0;
        let input_value_6 = 9;
        let positive = Numeric::new(input_value_1);
        let negative = Numeric::new(input_value_2);
        let expected = [Ok(92929i64), Ok(-1992i64)];

        // Act
        let actual = [
            positive.replace_digit(input_value_3, input_value_4),
            negative.replace_digit(input_value_5, input_value_6),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn contains_returns_false_for_every_non_decimal_digit() {
        // Arrange
        let input_value_1 = 123i8;
        let numeric = Numeric::new(input_value_1);
        let invalid_digits = [10usize, 128, usize::MAX];
        let expected = [false, false, false];

        // Act
        let actual = invalid_digits.map(|digit| numeric.contains(digit));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn contains_handles_zero_present_and_absent_digits() {
        // Arrange
        let input_value_1 = 0i64;
        let input_value_2 = 1203i64;
        let input_value_3 = 0;
        let input_value_4 = 1;
        let input_value_5 = 0;
        let input_value_6 = 9;
        let zero = Numeric::new(input_value_1);
        let with_zero = Numeric::new(input_value_2);
        let expected = [true, false, true, false];

        // Act
        let actual = [
            zero.contains(input_value_3),
            zero.contains(input_value_4),
            with_zero.contains(input_value_5),
            with_zero.contains(input_value_6),
        ];

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn ordering_and_sort_use_original_number() {
        // Arrange
        let input_value_1 = -1i64;
        let input_value_2 = 1i64;
        let input_value_3 = 0;
        let smaller = Numeric::new(input_value_1);
        let larger = Numeric::new(input_value_2);
        let mut values = [larger, Numeric::new(input_value_3), smaller];
        let expected_values = [-1i64, 0, 1];
        let expected_check_1 = Ordering::Less;
        let expected_check_2 = Some(Ordering::Less);

        // Act
        let actual_cmp = smaller.cmp(&larger);
        let actual_partial_cmp = smaller.partial_cmp(&larger);
        values.sort();
        let actual_values = values.map(|numeric| numeric.value());

        // Assert
        assert_eq!(actual_cmp, expected_check_1);
        assert_eq!(actual_partial_cmp, expected_check_2);
        assert_eq!(actual_values, expected_values);
    }
    #[test]
    fn digit_at_zero_on_zero_returns_zero() {
        // Arrange
        let number = 0i64;
        let mut numeric = Numeric::new(number);
        let position = 0;
        let expected = 0usize;

        // Act
        let actual = numeric.digit(position);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn digit_sum_of_zero_returns_zero() {
        // Arrange
        let number = 0i64;
        let numeric = Numeric::new(number);

        let expected = 0usize;

        // Act
        let actual = numeric.digit_sum();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn reversed_number_of_zero_returns_some_zero() {
        // Arrange
        let number = 0i64;
        let numeric = Numeric::new(number);

        let expected = Some(0i64);

        // Act
        let actual = numeric.reversed_number();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn round_at_place_zero_preserves_positive_negative_and_zero_values() {
        // Arrange
        let inputs = [123i64, -123, 0];
        let place = 0;
        let expected = [Some(123i64), Some(-123), Some(0)];

        // Act
        let actual = inputs.map(|number| Numeric::new(number).round(place));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn continuous_round_at_place_zero_preserves_positive_negative_and_zero_values() {
        // Arrange
        let inputs = [123i64, -123, 0];
        let place = 0;
        let expected = [Some(123i64), Some(-123), Some(0)];

        // Act
        let actual = inputs.map(|number| Numeric::new(number).continuous_round(place));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn continuous_round_on_negative_boundary_values_preserves_sign() {
        // Arrange
        let numbers = [-4444i64, -4445, -4446];
        let place = 4;
        let expected = [Some(0i64), Some(-10000), Some(-10000)];

        // Act
        let actual = numbers.map(|number| Numeric::new(number).continuous_round(place));

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn base_conversion_of_negative_and_zero_values_returns_signed_or_zero_representation() {
        // Arrange
        let numbers = [-13i64, 0];
        let base = 2;
        let expected = [Ok("-1101".to_owned()), Ok("0".to_owned())];

        // Act
        let actual = numbers.map(|number| Numeric::new(number).convert_n_adic_number(base));

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn base_reversion_of_negative_and_zero_representations_returns_original_values() {
        // Arrange
        let representations = ["-1101", "0", "-0"];
        let base = 2;
        let expected = [Ok(-13i64), Ok(0), Ok(0)];

        // Act
        let actual = representations
            .map(|representation| Numeric::<i64>::revert_n_adic_number(representation, base));

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn base_reversion_rejects_empty_and_sign_only_representations() {
        // Arrange
        let representations = ["", "-"];
        let base = 10;
        let expected = [Err(NumericError::EmptyRepresentation); 2];

        // Act
        let actual = representations
            .map(|representation| Numeric::<i64>::revert_n_adic_number(representation, base));

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn base_reversion_rejects_non_alphanumeric_digits() {
        // Arrange
        let representations = ["!", "1_"];
        let base = 10;
        let expected = [
            Err(NumericError::InvalidDigit { digit: '!', base }),
            Err(NumericError::InvalidDigit { digit: '_', base }),
        ];

        // Act
        let actual = representations
            .map(|representation| Numeric::<i64>::revert_n_adic_number(representation, base));

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn base_reversion_accepts_only_bases_inside_two_through_thirty_six_at_boundaries() {
        // Arrange
        let bases = [1, 2, 3, 35, 36, 37];
        let representation = "10";
        let expected = [
            Err(NumericError::InvalidBase(1)),
            Ok(2i64),
            Ok(3),
            Ok(35),
            Ok(36),
            Err(NumericError::InvalidBase(37)),
        ];

        // Act
        let actual = bases.map(|base| Numeric::<i64>::revert_n_adic_number(representation, base));

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn base_conversion_accepts_only_bases_inside_two_through_thirty_six_at_boundaries() {
        // Arrange
        let input_value_1 = 1i64;
        let bases = [1, 2, 3, 35, 36, 37];
        let numeric = Numeric::new(input_value_1);
        let expected = [
            Err(NumericError::InvalidBase(1)),
            Ok("1".to_owned()),
            Ok("1".to_owned()),
            Ok("1".to_owned()),
            Ok("1".to_owned()),
            Err(NumericError::InvalidBase(37)),
        ];

        // Act
        let actual = bases.map(|base| numeric.convert_n_adic_number(base));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn digit_sum_of_negative_twelve_returns_three_for_every_signed_integer_type() {
        // Arrange
        let i8_value = -12i8;
        let i16_value = -12i16;
        let i32_value = -12i32;
        let i64_value = -12i64;
        let i128_value = -12i128;
        let isize_value = -12isize;
        let expected = [3usize; 6];

        // Act
        let actual = [
            Numeric::new(i8_value).digit_sum(),
            Numeric::new(i16_value).digit_sum(),
            Numeric::new(i32_value).digit_sum(),
            Numeric::new(i64_value).digit_sum(),
            Numeric::new(i128_value).digit_sum(),
            Numeric::new(isize_value).digit_sum(),
        ];
        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn continuous_round_of_zero_at_positive_place_returns_some_zero() {
        // Arrange
        let value = 0i64;
        let place = 4;
        let expected = Some(0i64);
        let numeric = Numeric::new(value);

        // Act
        let actual = numeric.continuous_round(place);
        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn ordering_equal_and_greater_values_returns_expected_total_and_partial_ordering() {
        // Arrange
        let pairs = [(2i64, 2i64), (3, 2)];
        let expected = [
            (Ordering::Equal, Some(Ordering::Equal)),
            (Ordering::Greater, Some(Ordering::Greater)),
        ];

        // Act
        let actual = pairs.map(|(left, right)| {
            let left = Numeric::new(left);
            let right = Numeric::new(right);
            (left.cmp(&right), left.partial_cmp(&right))
        });
        // Assert
        assert_eq!(actual, expected);
    }
}
