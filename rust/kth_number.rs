/// 与えられた文字集合で長さ 1 からの並び順に並べたとき、`k` 番目の文字列を返す（1-indexed）。
pub fn kth_number(k: u64, digits: &[char]) -> Option<String> {
    if digits.is_empty() || k == 0 {
        return None;
    }

    let base = digits.len() as u64;
    let mut remaining = k;
    let mut digit_length = 1u64;
    let mut count_per_length = base;

    while remaining > count_per_length {
        remaining -= count_per_length;
        digit_length += 1;

        let Some(next_count) = count_per_length.checked_mul(base) else {
            break;
        };
        count_per_length = next_count;
    }

    let mut x = remaining - 1;
    let mut result = String::new();

    for _ in 0..digit_length {
        let idx = (x % base) as usize;
        result.push(digits[idx]);
        x /= base;
    }

    Some(result.chars().rev().collect())
}

#[cfg(test)]
mod tests {
    use super::kth_number;

    #[test]
    fn empty_digits_return_none() {
        // Arrange
        let digits = vec![];
        let k = 1;
        let expected_value = None;

        // Act
        let actual_value = kth_number(k, &digits);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn zero_k_returns_none() {
        // Arrange
        let digits = vec!['0'];
        let k = 0;
        let expected_value = None;

        // Act
        let actual_value = kth_number(k, &digits);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn generates_numbers_with_single_digit() {
        // Arrange
        let digits = vec!['a'];
        let first_k = 1;
        let third_k = 3;
        let expected_values = (Some("a".to_string()), Some("aaa".to_string()));

        // Act
        let actual_values = (kth_number(first_k, &digits), kth_number(third_k, &digits));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn maximum_k_with_binary_digits_returns_sixty_four_zeroes() {
        // Arrange
        let digits = vec!['0', '1'];
        let k = u64::MAX;
        let expected_value = Some("0".repeat(64));

        // Act
        let actual_value = kth_number(k, &digits);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn boundary_between_lengths_changes_string_length() {
        // Arrange
        let digits = vec!['0', '1'];
        let ks = (2, 3);
        let expected_values = (Some("1".to_string()), Some("00".to_string()));

        // Act
        let actual_values = (kth_number(ks.0, &digits), kth_number(ks.1, &digits));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn digit_order_follows_given_input_order() {
        // Arrange
        let digits = vec!['b', 'a'];
        let ks = (1, 2, 3);
        let expected_values = (
            Some("b".to_string()),
            Some("a".to_string()),
            Some("bb".to_string()),
        );

        // Act
        let actual_values = (
            kth_number(ks.0, &digits),
            kth_number(ks.1, &digits),
            kth_number(ks.2, &digits),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn duplicate_digits_produce_expected_repeated_strings() {
        // Arrange
        let digits = vec!['a', 'a'];
        let ks = (1, 2, 3);
        let expected_values = (
            Some("a".to_string()),
            Some("a".to_string()),
            Some("aa".to_string()),
        );

        // Act
        let actual_values = (
            kth_number(ks.0, &digits),
            kth_number(ks.1, &digits),
            kth_number(ks.2, &digits),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
