fn bubble_sort<T: Ord>(vec: &mut Vec<T>) {

    for i in 0..vec.len() {
        for j in 0..(vec.len() - 1 - i) {
            if vec[j + 1] < vec[j] {
                vec.swap(j, j + 1);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::bubble_sort;
    use std::cmp::Ordering;

    #[derive(Clone, Debug)]
    struct StableItem {
        key: i32,
        label: &'static str,
    }

    impl PartialEq for StableItem {
        fn eq(&self, other: &Self) -> bool {
            self.key == other.key
        }
    }

    impl Eq for StableItem {}

    impl PartialOrd for StableItem {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for StableItem {
        fn cmp(&self, other: &Self) -> Ordering {
            self.key.cmp(&other.key)
        }
    }

    fn create_stable_item(key: i32, label: &'static str) -> StableItem {
        StableItem { key, label }
    }

    #[test]
    fn bubble_sort_with_empty_vector_keeps_empty_vector_without_panic() {
        // Arrange
        let mut input_values: Vec<i32> = vec![];
        let expected_values: Vec<i32> = vec![];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_singleton_vector_keeps_single_value() {
        // Arrange
        let single_value = 42;
        let mut input_values = vec![single_value];
        let expected_values = vec![single_value];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_two_sorted_elements_keeps_order_unchanged() {
        // Arrange
        let mut input_values = vec![1, 2];
        let expected_values = vec![1, 2];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_two_reverse_elements_returns_ascending_order() {
        // Arrange
        let mut input_values = vec![2, 1];
        let expected_values = vec![1, 2];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_unsorted_multiple_elements_returns_ascending_order() {
        // Arrange
        let mut input_values = vec![3, 1, 4, 2];
        let expected_values = vec![1, 2, 3, 4];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_already_sorted_multiple_elements_keeps_order_unchanged() {
        // Arrange
        let mut input_values = vec![1, 2, 3, 4, 5];
        let expected_values = vec![1, 2, 3, 4, 5];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_descending_input_returns_ascending_order() {
        // Arrange
        let mut input_values = vec![5, 4, 3, 2, 1];
        let expected_values = vec![1, 2, 3, 4, 5];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_duplicate_values_preserves_multiplicity() {
        // Arrange
        let mut input_values = vec![3, 1, 3, 2, 1];
        let expected_values = vec![1, 1, 2, 3, 3];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_identical_values_keeps_value_and_count() {
        // Arrange
        let repeated_value = 7;
        let mut input_values = vec![
            repeated_value,
            repeated_value,
            repeated_value,
            repeated_value,
        ];
        let expected_values = vec![
            repeated_value,
            repeated_value,
            repeated_value,
            repeated_value,
        ];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_boundary_values_x_minus_1_x_x_plus_1_returns_ordered_result() {
        // Arrange
        let mut input_values = vec![1, 0, -1];
        let expected_values = vec![-1, 0, 1];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_lower_bound_near_i32_min_returns_ascending_order() {
        // Arrange
        let lower_value = i32::MIN;
        let middle_value = i32::MIN + 1;
        let upper_value = i32::MIN + 2;
        let mut input_values = vec![upper_value, lower_value, middle_value];
        let expected_values = vec![lower_value, middle_value, upper_value];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_upper_bound_near_i32_max_returns_ascending_order() {
        // Arrange
        let lower_value = i32::MAX - 2;
        let middle_value = i32::MAX - 1;
        let upper_value = i32::MAX;
        let mut input_values = vec![middle_value, upper_value, lower_value];
        let expected_values = vec![lower_value, middle_value, upper_value];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bubble_sort_with_equal_keys_keeps_relative_order_for_stability() {
        // Arrange
        let first_label = "first";
        let second_label = "second";
        let third_label = "third";
        let duplicate_key = 2;
        let mut input_values = vec![
            create_stable_item(duplicate_key, first_label),
            create_stable_item(1, "head"),
            create_stable_item(duplicate_key, second_label),
            create_stable_item(duplicate_key, third_label),
        ];
        let expected_sorted_keys = vec![1, 2, 2, 2];
        let expected_labels_for_equal_key = vec![first_label, second_label, third_label];

        // Act
        bubble_sort(&mut input_values);
        let actual_sorted_keys: Vec<i32> = input_values.iter().map(|item| item.key).collect();
        let actual_labels_for_equal_key: Vec<&str> = input_values
            .iter()
            .filter(|item| item.key == duplicate_key)
            .map(|item| item.label)
            .collect();

        // Assert
        assert_eq!(actual_sorted_keys, expected_sorted_keys);
        assert_eq!(actual_labels_for_equal_key, expected_labels_for_equal_key);
    }

    #[test]
    fn bubble_sort_with_string_values_returns_lexicographic_order() {
        // Arrange
        let mut input_values = vec![String::from("b"), String::from("aa"), String::from("a")];
        let expected_values = vec![String::from("a"), String::from("aa"), String::from("b")];

        // Act
        bubble_sort(&mut input_values);
        let actual_values = input_values;

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
