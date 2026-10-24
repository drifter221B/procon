/// `min_value` から `max_value` までの整数を要素に持つ配列を計数ソートする。
pub fn counting_sort(nums: &[i64], min_value: i64, max_value: i64) -> Vec<i64> {

    if nums.is_empty() {
        return Vec::new();
    }

    assert!(min_value <= max_value);

    let size = (max_value - min_value + 1) as usize;
    let offset = 0 - min_value;
    let mut counter = vec![0usize; size + 1];

    for &num in nums {
        let index = (num + offset) as usize;
        counter[index] += 1;
    }

    for i in 0..size {
        counter[i + 1] += counter[i];
    }

    let mut results = vec![0i64; nums.len()];
    for &num in nums {
        let idx = (num + offset) as usize;
        let position = counter[idx] - 1;
        counter[idx] -= 1;
        results[position] = num;
    }

    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counting_sort_with_empty_input_and_valid_range_returns_empty_vector() {
        // Arrange
        let nums = vec![];
        let min_value = 0;
        let max_value = 3;
        let expected_sorted_nums = vec![];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_empty_input_and_invalid_range_returns_empty_vector_without_panic() {
        // Arrange
        let nums = vec![];
        let min_value = 3;
        let max_value = 2;
        let expected_sorted_nums = vec![];

        // Act
        let sort_result = std::panic::catch_unwind(|| counting_sort(&nums, min_value, max_value));

        // Assert
        assert!(sort_result.is_ok());
        let sorted_nums = sort_result.unwrap();
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_singleton_and_fixed_range_returns_same_single_value() {
        // Arrange
        let single_value = 7;
        let nums = vec![single_value];
        let min_value = single_value;
        let max_value = single_value;
        let expected_sorted_nums = vec![single_value];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_multiple_identical_values_keeps_multiplicity() {
        // Arrange
        let repeated_value = 5;
        let nums = vec![repeated_value, repeated_value, repeated_value];
        let min_value = repeated_value;
        let max_value = repeated_value;
        let expected_sorted_nums = vec![repeated_value, repeated_value, repeated_value];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_unsorted_values_and_duplicates_returns_sorted_result() {
        // Arrange
        let nums = vec![3, 1, 2, 1, 0];
        let min_value = 0;
        let max_value = 3;
        let expected_sorted_nums = vec![0, 1, 1, 2, 3];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_already_sorted_values_keeps_ordered_result() {
        // Arrange
        let nums = vec![0, 1, 1, 2, 3];
        let min_value = 0;
        let max_value = 3;
        let expected_sorted_nums = vec![0, 1, 1, 2, 3];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_descending_values_returns_ascending_result() {
        // Arrange
        let nums = vec![3, 2, 1, 1, 0];
        let min_value = 0;
        let max_value = 3;
        let expected_sorted_nums = vec![0, 1, 1, 2, 3];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_lower_boundary_x_minus_1_panics_for_value_below_minimum() {
        // Arrange
        let nums = vec![-3, -2, -1];
        let min_value = -2;
        let max_value = -1;

        // Act
        let sort_result = std::panic::catch_unwind(|| counting_sort(&nums, min_value, max_value));

        // Assert
        assert!(sort_result.is_err());
    }

    #[test]
    fn counting_sort_with_upper_boundary_x_plus_1_accepts_value_equal_to_max_plus_one() {
        // Arrange
        let nums = vec![2, 3, 4];
        let min_value = 2;
        let max_value = 3;
        let expected_sorted_nums = vec![2, 3, 4];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_non_empty_input_and_invalid_range_panics() {
        // Arrange
        let nums = vec![1];
        let min_value = 2;
        let max_value = 1;

        // Act
        let sort_result = std::panic::catch_unwind(|| counting_sort(&nums, min_value, max_value));

        // Assert
        assert!(sort_result.is_err());
    }

    #[test]
    fn counting_sort_with_negative_and_positive_range_returns_sorted_result() {
        // Arrange
        let nums = vec![2, -3, 0, -1, 1, -2];
        let min_value = -3;
        let max_value = 2;
        let expected_sorted_nums = vec![-3, -2, -1, 0, 1, 2];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_values_near_i64_max_returns_sorted_result() {
        // Arrange
        let upper_value = i64::MAX;
        let middle_value = i64::MAX - 1;
        let lower_value = i64::MAX - 2;
        let nums = vec![upper_value, lower_value, middle_value];
        let min_value = lower_value;
        let max_value = upper_value;
        let expected_sorted_nums = vec![lower_value, middle_value, upper_value];

        // Act
        let sorted_nums = counting_sort(&nums, min_value, max_value);

        // Assert
        assert_eq!(sorted_nums, expected_sorted_nums);
    }

    #[test]
    fn counting_sort_with_min_value_i64_min_panics_on_offset_overflow() {
        // Arrange
        let min_value = i64::MIN;
        let max_value = i64::MIN;
        let nums = vec![i64::MIN];

        // Act
        let sort_result = std::panic::catch_unwind(|| counting_sort(&nums, min_value, max_value));

        // Assert
        assert!(sort_result.is_err());
    }
}
