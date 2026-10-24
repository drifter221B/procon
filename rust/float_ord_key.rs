#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct KeyError;

macro_rules! define_float_ord_key {
    ($key_name:ident, $value_type:ty $(,)?) => {
        define_float_ord_key!(
            $key_name,
            $value_type,
            |value: &$value_type| value.is_nan(),
        );
    };

    ($key_name:ident, $value_type:ty, $is_invalid:expr $(,)?) => {
        define_float_ord_key!(
            #[derive(Debug, Clone, Copy, PartialEq)]
            $key_name,
            $value_type,
            $is_invalid,
        );
    };

    (
        $(#[$key_meta:meta])*
        $key_name:ident,
        $value_type:ty,
        $is_invalid:expr $(,)?
    ) => {
        $(#[$key_meta])*
        pub struct $key_name {
            value: $value_type,
        }

        impl $key_name {
            pub fn new(value: $value_type) -> Result<Self, KeyError> {
                if ($is_invalid)(&value) {
                    Err(KeyError)
                } else {
                    Ok(Self { value })
                }
            }

            pub fn get(self) -> $value_type {
                self.value
            }
        }

        impl Eq for $key_name {}

        impl PartialOrd for $key_name {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }

        impl Ord for $key_name {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                self.value
                    .partial_cmp(&other.value)
                    .expect(concat!(stringify!($key_name), " must not contain invalid value"))
            }
        }

        impl std::convert::TryFrom<$value_type> for $key_name {
            type Error = KeyError;

            fn try_from(value: $value_type) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl From<$key_name> for $value_type {
            fn from(key: $key_name) -> Self {
                key.value
            }
        }
    };
}

define_float_ord_key!(F64Key, f64);
define_float_ord_key!(F32Key, f32);

#[cfg(test)]
mod tests {
    use super::{F32Key, F64Key, KeyError};
    use std::cmp::Ordering;
    use std::convert::TryFrom;

    #[derive(Debug, Clone, Copy, PartialEq)]
    pub struct TestDecimal {
        value: i32,
        is_invalid: bool,
    }

    impl PartialOrd for TestDecimal {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            self.value.partial_cmp(&other.value)
        }
    }

    define_float_ord_key!(
        #[derive(Debug, Clone, Copy, PartialEq)]
        TestDecimalKey,
        TestDecimal,
        |value: &TestDecimal| value.is_invalid,
    );

    #[test]
    fn new_with_finite_value_returns_key_with_same_value() {
        // Arrange
        let value = 1.25f64;
        let expected_value = value;

        // Act
        let key = F64Key::new(value).expect("finite value should create F64Key");
        let actual_value = key.get();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn new_with_nan_returns_error() {
        // Arrange
        let value = f64::NAN;
        let expected_error = KeyError;

        // Act
        let actual_error = F64Key::new(value).expect_err("NaN should be rejected");

        // Assert
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn f32_new_with_finite_value_returns_key_with_same_value() {
        // Arrange
        let value = 1.25f32;
        let expected_value = value;

        // Act
        let key = F32Key::new(value).expect("finite value should create F32Key");
        let actual_value = key.get();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn f32_new_with_nan_returns_error() {
        // Arrange
        let value = f32::NAN;
        let expected_error = KeyError;

        // Act
        let actual_error = F32Key::new(value).expect_err("NaN should be rejected");

        // Assert
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn custom_key_new_with_invalid_value_returns_error() {
        // Arrange
        let value = TestDecimal {
            value: 10,
            is_invalid: true,
        };
        let expected_error = KeyError;

        // Act
        let actual_error =
            TestDecimalKey::new(value).expect_err("invalid value should be rejected");

        // Assert
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn get_returns_wrapped_value() {
        // Arrange
        let value = -3.5f64;
        let expected_value = value;
        let key = F64Key::new(value).expect("finite value should create F64Key");

        // Act
        let actual_value = key.get();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn custom_key_get_returns_wrapped_value() {
        // Arrange
        let value = TestDecimal {
            value: 10,
            is_invalid: false,
        };
        let expected_value = value;
        let key = TestDecimalKey::new(value).expect("valid value should create TestDecimalKey");

        // Act
        let actual_value = key.get();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn partial_cmp_with_smaller_and_larger_values_returns_some_less() {
        // Arrange
        let smaller_value = 1.0f64;
        let larger_value = 2.0f64;
        let expected_ordering = Some(Ordering::Less);
        let smaller_key = F64Key::new(smaller_value).expect("finite value should create F64Key");
        let larger_key = F64Key::new(larger_value).expect("finite value should create F64Key");

        // Act
        let actual_ordering = smaller_key.partial_cmp(&larger_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn cmp_with_smaller_and_larger_values_returns_less() {
        // Arrange
        let smaller_value = -2.0f64;
        let larger_value = 4.0f64;
        let expected_ordering = Ordering::Less;
        let smaller_key = F64Key::new(smaller_value).expect("finite value should create F64Key");
        let larger_key = F64Key::new(larger_value).expect("finite value should create F64Key");

        // Act
        let actual_ordering = smaller_key.cmp(&larger_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn cmp_with_negative_infinity_and_finite_value_returns_less() {
        // Arrange
        let smaller_value = f64::NEG_INFINITY;
        let larger_value = 0.0f64;
        let expected_ordering = Ordering::Less;
        let smaller_key = F64Key::new(smaller_value).expect("infinity should create F64Key");
        let larger_key = F64Key::new(larger_value).expect("finite value should create F64Key");

        // Act
        let actual_ordering = smaller_key.cmp(&larger_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn cmp_with_finite_value_and_positive_infinity_returns_less() {
        // Arrange
        let smaller_value = 0.0f64;
        let larger_value = f64::INFINITY;
        let expected_ordering = Ordering::Less;
        let smaller_key = F64Key::new(smaller_value).expect("finite value should create F64Key");
        let larger_key = F64Key::new(larger_value).expect("infinity should create F64Key");

        // Act
        let actual_ordering = smaller_key.cmp(&larger_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn cmp_with_negative_zero_and_positive_zero_returns_equal() {
        // Arrange
        let negative_zero = -0.0f64;
        let positive_zero = 0.0f64;
        let expected_ordering = Ordering::Equal;
        let negative_zero_key =
            F64Key::new(negative_zero).expect("negative zero should create F64Key");
        let positive_zero_key =
            F64Key::new(positive_zero).expect("positive zero should create F64Key");

        // Act
        let actual_ordering = negative_zero_key.cmp(&positive_zero_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn f32_cmp_with_smaller_and_larger_values_returns_less() {
        // Arrange
        let smaller_value = -2.0f32;
        let larger_value = 4.0f32;
        let expected_ordering = Ordering::Less;
        let smaller_key = F32Key::new(smaller_value).expect("finite value should create F32Key");
        let larger_key = F32Key::new(larger_value).expect("finite value should create F32Key");

        // Act
        let actual_ordering = smaller_key.cmp(&larger_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn custom_key_cmp_with_smaller_and_larger_values_returns_less() {
        // Arrange
        let smaller_value = TestDecimal {
            value: 10,
            is_invalid: false,
        };
        let larger_value = TestDecimal {
            value: 20,
            is_invalid: false,
        };
        let expected_ordering = Ordering::Less;
        let smaller_key =
            TestDecimalKey::new(smaller_value).expect("valid value should create TestDecimalKey");
        let larger_key =
            TestDecimalKey::new(larger_value).expect("valid value should create TestDecimalKey");

        // Act
        let actual_ordering = smaller_key.cmp(&larger_key);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn try_from_with_finite_value_returns_key_with_same_value() {
        // Arrange
        let value = 8.75f64;
        let expected_value = value;

        // Act
        let key = F64Key::try_from(value).expect("finite value should create F64Key");
        let actual_value = key.get();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn try_from_with_nan_returns_error() {
        // Arrange
        let value = f64::NAN;
        let expected_error = KeyError;

        // Act
        let actual_error = F64Key::try_from(value).expect_err("NaN should be rejected");

        // Assert
        assert_eq!(actual_error, expected_error);
    }

    #[test]
    fn f32_try_from_with_finite_value_returns_key_with_same_value() {
        // Arrange
        let value = 8.75f32;
        let expected_value = value;

        // Act
        let key = F32Key::try_from(value).expect("finite value should create F32Key");
        let actual_value = key.get();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn from_f64_key_into_f64_returns_wrapped_value() {
        // Arrange
        let value = 12.5f64;
        let expected_value = value;
        let key = F64Key::new(value).expect("finite value should create F64Key");

        // Act
        let actual_value = f64::from(key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
