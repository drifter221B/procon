pub fn dominant<T>(values: &[T]) -> (T, usize)
where T: Clone + PartialEq {

    assert!(!values.is_empty());

    let majority_candidate = candidate(values);
    let occurrence_count = values
        .iter()
        .filter(|value| **value == majority_candidate)
        .count();

    assert!(values.len() < 2 * occurrence_count);

    (majority_candidate, occurrence_count)
}

fn candidate<T>(values: &[T]) -> T
where T: Clone + PartialEq {

    let mut candidate = values[0].clone();
    let mut counter = 0_i64;

    for value in values.iter() {

        counter += if *value == candidate { 1 } else { -1 };

        if counter == 0 {
            counter = 1;
            candidate = value.clone();
        }
    }

    candidate
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn dominant_panics_when_values_is_empty_set() {
        // Arrange
        let values: Vec<i32> = vec![];

        // Act
        let dominant_result = std::panic::catch_unwind(|| dominant(&values));

        // Assert
        assert!(dominant_result.is_err());
    }

    #[test]
    fn dominant_returns_single_value_and_count_for_singleton_set() {
        // Arrange
        let values = vec![7];
        let expected_value = 7;
        let expected_count = 1;

        // Act
        let (actual_value, actual_count) = dominant(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn dominant_returns_majority_and_count_for_multiple_elements_set() {
        // Arrange
        let values = vec![2, 1, 2];
        let expected_value = 2;
        let expected_count = 2;

        // Act
        let (actual_value, actual_count) = dominant(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn dominant_even_length_panics_at_or_below_half_and_returns_majority_above_half() {
        // Arrange
        let values_x_minus_1 = vec![1, 2, 3, 4, 5, 5];
        let values_x = vec![1, 2, 3, 4, 3, 3];
        let values_x_plus_1 = vec![1, 2, 3, 3, 3, 3];
        let expected_value_x_plus_1 = 3;
        let expected_count_x_plus_1 = 4;

        // Act
        let dominant_result_x_minus_1 = std::panic::catch_unwind(|| dominant(&values_x_minus_1));
        let dominant_result_x = std::panic::catch_unwind(|| dominant(&values_x));
        let dominant_result_x_plus_1 = dominant(&values_x_plus_1);

        // Assert
        assert!(dominant_result_x_minus_1.is_err());
        assert!(dominant_result_x.is_err());
        assert_eq!(dominant_result_x_plus_1.0, expected_value_x_plus_1);
        assert_eq!(dominant_result_x_plus_1.1, expected_count_x_plus_1);
    }

    #[test]
    fn dominant_odd_length_panics_below_threshold_and_returns_majority_at_or_above_threshold() {
        // Arrange
        let values_x_minus_1 = vec![1, 3, 4, 2, 2];
        let values_x = vec![1, 2, 3, 3, 3];
        let values_x_plus_1 = vec![4, 4, 1, 4, 4];
        let expected_value_x = 3;
        let expected_count_x = 3;
        let expected_value_x_plus_1 = 4;
        let expected_count_x_plus_1 = 4;

        // Act
        let dominant_result_x_minus_1 = std::panic::catch_unwind(|| dominant(&values_x_minus_1));
        let dominant_result_x = dominant(&values_x);
        let dominant_result_x_plus_1 = dominant(&values_x_plus_1);

        // Assert
        assert!(dominant_result_x_minus_1.is_err());
        assert_eq!(dominant_result_x.0, expected_value_x);
        assert_eq!(dominant_result_x.1, expected_count_x);
        assert_eq!(dominant_result_x_plus_1.0, expected_value_x_plus_1);
        assert_eq!(dominant_result_x_plus_1.1, expected_count_x_plus_1);
    }

    #[test]
    fn dominant_panics_when_occurrence_count_is_exactly_half() {
        // Arrange
        let values = vec![1, 1, 2, 2];

        // Act
        let dominant_result = std::panic::catch_unwind(|| dominant(&values));

        // Assert
        assert!(dominant_result.is_err());
    }

    #[test]
    fn dominant_returns_correct_result_when_majority_appears_late() {
        // Arrange
        let values = vec![3, 1, 3, 3, 2];
        let expected_value = 3;
        let expected_count = 3;

        // Act
        let (actual_value, actual_count) = dominant(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn dominant_returns_same_result_for_different_input_orders() {
        // Arrange
        let values_first_order = vec![2, 1, 2, 2, 3];
        let values_second_order = vec![3, 2, 1, 2, 2];
        let expected_value = 2;
        let expected_count = 3;

        // Act
        let dominant_result_first_order = dominant(&values_first_order);
        let dominant_result_second_order = dominant(&values_second_order);

        // Assert
        assert_eq!(dominant_result_first_order, dominant_result_second_order);
        assert_eq!(dominant_result_first_order.0, expected_value);
        assert_eq!(dominant_result_first_order.1, expected_count);
    }

    #[test]
    fn dominant_supports_non_copy_string_values() {
        // Arrange
        let values = vec![
            String::from("apple"),
            String::from("banana"),
            String::from("apple"),
        ];
        let expected_value = String::from("apple");
        let expected_count = 2;

        // Act
        let (actual_value, actual_count) = dominant(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn dominant_handles_i32_min_and_i32_max_boundaries() {
        // Arrange
        let values = vec![i32::MIN, i32::MAX, i32::MIN, i32::MIN];
        let expected_value = i32::MIN;
        let expected_count = 3;

        // Act
        let (actual_value, actual_count) = dominant(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn candidate_panics_when_values_is_empty_set() {
        // Arrange
        let values: Vec<i32> = vec![];

        // Act
        let candidate_result = std::panic::catch_unwind(|| candidate(&values));

        // Assert
        assert!(candidate_result.is_err());
    }

    #[test]
    fn candidate_returns_only_value_for_singleton_set() {
        // Arrange
        let values = vec![String::from("a")];
        let expected_value = String::from("a");

        // Act
        let actual_value = candidate(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn candidate_returns_expected_candidate_when_cancellation_repeats() {
        // Arrange
        let values = vec![1, 2, 1, 2, 1];
        let expected_value = 1;

        // Act
        let actual_value = candidate(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn candidate_without_majority_returns_last_value_for_this_input() {
        // Arrange
        let values = vec![1, 2, 3, 4];
        let expected_value = 4;

        // Act
        let actual_value = candidate(&values);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
