struct Palindrome;

impl Palindrome {

    // 1-indexed
    fn kth_palindrome(k: usize) -> String {

        assert!(0 < k);

        if k == 1 {
            return String::from("0");
        }

        let mut k = k - 1;

        for length in 1.. {

            let base = 10usize.pow((length as u32 - 1) / 2);
            let limit = 9 * base;

            let result = Self::d_digit_kth_palindrome(k, length);

            match result {
                Some(result) => return result,
                None => k -= limit,
            }
        }

        unreachable!()
    }

    // 1-indexed
    fn d_digit_kth_palindrome(k: usize, length: usize) -> Option<String> {

        assert!(0 < k);
        assert!((length - 1) / 2 <= 18);

        let base = 10usize.pow((length as u32 - 1) / 2);
        let limit = 9 * base;

        if limit < k {
            return None;
        }

        let first = (k - 1 + base).to_string().chars().collect::<Vec<char>>();
        let mut result = first.clone();
        if (length & 1) == 1 {
            result.pop();
        }

        result.extend(first.iter().rev());
        let result = result.into_iter().collect::<String>();

        Some(result)
    }
}

#[cfg(test)]
mod todo_tests {
    use super::Palindrome;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn kth_palindrome_returns_expected_values_at_enumeration_start_boundary() {
        // Arrange
        let k_minus_one = 1;
        let k = 2;
        let k_plus_one = 3;
        let expected_values = ("0".to_string(), "1".to_string(), "2".to_string());

        // Act
        let actual_values = (
            Palindrome::kth_palindrome(k_minus_one),
            Palindrome::kth_palindrome(k),
            Palindrome::kth_palindrome(k_plus_one),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn kth_palindrome_returns_expected_values_at_two_digit_transition_boundary() {
        // Arrange
        let k_minus_one = 10;
        let k = 11;
        let k_plus_one = 12;
        let expected_values = ("9".to_string(), "11".to_string(), "22".to_string());

        // Act
        let actual_values = (
            Palindrome::kth_palindrome(k_minus_one),
            Palindrome::kth_palindrome(k),
            Palindrome::kth_palindrome(k_plus_one),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn kth_palindrome_returns_expected_values_at_base_transition_from_one_to_ten() {
        // Arrange
        let k_minus_one = 19;
        let k = 20;
        let k_plus_one = 21;
        let expected_values = ("99".to_string(), "101".to_string(), "111".to_string());

        // Act
        let actual_values = (
            Palindrome::kth_palindrome(k_minus_one),
            Palindrome::kth_palindrome(k),
            Palindrome::kth_palindrome(k_plus_one),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn kth_palindrome_returns_expected_values_at_base_transition_from_ten_to_hundred() {
        // Arrange
        let k_minus_one = 199;
        let k = 200;
        let k_plus_one = 201;
        let expected_values = (
            "9999".to_string(),
            "10001".to_string(),
            "10101".to_string(),
        );

        // Act
        let actual_values = (
            Palindrome::kth_palindrome(k_minus_one),
            Palindrome::kth_palindrome(k),
            Palindrome::kth_palindrome(k_plus_one),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn kth_palindrome_panics_when_k_is_zero() {
        // Arrange
        let k = 0;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| Palindrome::kth_palindrome(k)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn d_digit_kth_palindrome_returns_expected_values_for_one_digit_boundary() {
        // Arrange
        let k_minus_one = 8;
        let k = 9;
        let k_plus_one = 10;
        let length = 1;
        let expected_values = (
            Some("8".to_string()),
            Some("9".to_string()),
            None,
        );

        // Act
        let actual_values = (
            Palindrome::d_digit_kth_palindrome(k_minus_one, length),
            Palindrome::d_digit_kth_palindrome(k, length),
            Palindrome::d_digit_kth_palindrome(k_plus_one, length),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn d_digit_kth_palindrome_returns_expected_values_for_two_digit_boundary() {
        // Arrange
        let k_minus_one = 8;
        let k = 9;
        let k_plus_one = 10;
        let length = 2;
        let expected_values = (
            Some("88".to_string()),
            Some("99".to_string()),
            None,
        );

        // Act
        let actual_values = (
            Palindrome::d_digit_kth_palindrome(k_minus_one, length),
            Palindrome::d_digit_kth_palindrome(k, length),
            Palindrome::d_digit_kth_palindrome(k_plus_one, length),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn d_digit_kth_palindrome_returns_expected_values_for_three_digit_upper_boundary() {
        // Arrange
        let k_minus_one = 89;
        let k = 90;
        let k_plus_one = 91;
        let length = 3;
        let expected_values = (
            Some("989".to_string()),
            Some("999".to_string()),
            None,
        );

        // Act
        let actual_values = (
            Palindrome::d_digit_kth_palindrome(k_minus_one, length),
            Palindrome::d_digit_kth_palindrome(k, length),
            Palindrome::d_digit_kth_palindrome(k_plus_one, length),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn d_digit_kth_palindrome_returns_expected_value_for_odd_length_example() {
        // Arrange
        let k = 24;
        let length = 5;
        let expected_value = Some("12321".to_string());

        // Act
        let actual_value = Palindrome::d_digit_kth_palindrome(k, length);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn d_digit_kth_palindrome_returns_expected_value_for_even_length_example() {
        // Arrange
        let k = 24;
        let length = 6;
        let expected_value = Some("123321".to_string());

        // Act
        let actual_value = Palindrome::d_digit_kth_palindrome(k, length);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn d_digit_kth_palindrome_panics_when_k_is_zero() {
        // Arrange
        let k = 0;
        let length = 1;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| {
            Palindrome::d_digit_kth_palindrome(k, length)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn d_digit_kth_palindrome_panics_when_length_is_zero() {
        // Arrange
        let k = 1;
        let length = 0;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| {
            Palindrome::d_digit_kth_palindrome(k, length)
        }));

        // Assert
        assert!(actual_result.is_err());
    }
}
