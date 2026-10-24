fn digit_index(value: usize, p: usize) -> usize {
    (value / p) % 10
}

fn counting_sort(vec: &mut Vec<usize>, p: usize) {
    let max_v = 10;
    let mut counter = vec![0; max_v];

    for v in vec.iter() {
        counter[digit_index(*v, p)] += 1;
    }

    for i in 0..counter.len() - 1 {
        counter[i + 1] += counter[i];
    }

    let mut result = vec![0; vec.len()];

    for v in vec.iter().rev() {
        let index = digit_index(*v, p);
        result[counter[index] - 1] = *v;
        counter[index] -= 1;
    }

    std::mem::swap(vec, &mut result);
}

fn raddix_sort(vec: &mut Vec<usize>) {
    let mut p = 1;
    let max_v = *vec.iter().max().unwrap();

    while p <= max_v {
        counting_sort(vec, p);
        p = match p.checked_mul(10) {
            Some(next_p) => next_p,
            None => break,
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::BTreeMap;

    fn build_frequency_map(values: &[usize]) -> BTreeMap<usize, usize> {
        let mut frequency_map = BTreeMap::new();

        for value in values.iter() {
            let entry = frequency_map.entry(*value).or_insert(0);
            *entry += 1;
        }

        frequency_map
    }

    #[test]
    fn counting_sort_with_empty_set_and_p_equals_1_keeps_empty_vector_without_panic() {
        // Arrange
        let mut input_values = vec![];
        let digit_place = 1;
        let expected_values: Vec<usize> = vec![];

        // Act
        let sort_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            counting_sort(&mut input_values, digit_place);
        }));

        // Assert
        assert!(sort_result.is_ok());
        assert_eq!(input_values, expected_values);
    }

    #[test]
    fn counting_sort_with_singleton_and_p_equals_1_keeps_same_value() {
        // Arrange
        let single_value = 7;
        let mut input_values = vec![single_value];
        let digit_place = 1;
        let expected_values = vec![single_value];

        // Act
        counting_sort(&mut input_values, digit_place);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn counting_sort_with_multiple_values_and_p_equals_1_sorts_by_ones_place() {
        // Arrange
        let mut input_values = vec![9, 0, 5, 3];
        let digit_place = 1;
        let expected_values = vec![0, 3, 5, 9];

        // Act
        counting_sort(&mut input_values, digit_place);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn counting_sort_with_same_ones_place_and_p_equals_1_keeps_relative_order() {
        // Arrange
        let mut input_values = vec![31, 21, 11];
        let digit_place = 1;
        let expected_values = vec![31, 21, 11];

        // Act
        counting_sort(&mut input_values, digit_place);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn counting_sort_with_multiple_digits_and_p_equals_10_sorts_by_tens_place_stably() {
        // Arrange
        let mut input_values = vec![29, 11, 25, 12];
        let digit_place = 10;
        let expected_values = vec![11, 12, 29, 25];

        // Act
        counting_sort(&mut input_values, digit_place);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn counting_sort_with_lower_boundary_values_0_and_1_distributes_buckets_correctly() {
        // Arrange
        let mut input_values = vec![1, 0, 1];
        let digit_place = 1;
        let expected_values = vec![0, 1, 1];

        // Act
        counting_sort(&mut input_values, digit_place);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn counting_sort_with_upper_boundary_values_8_and_9_distributes_buckets_correctly() {
        // Arrange
        let mut input_values = vec![9, 8, 9];
        let digit_place = 1;
        let expected_values = vec![8, 9, 9];

        // Act
        counting_sort(&mut input_values, digit_place);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_empty_set_panics_as_current_behavior() {
        // Arrange
        let mut input_values: Vec<usize> = vec![];

        // Act
        let sort_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            raddix_sort(&mut input_values);
        }));

        // Assert
        assert!(sort_result.is_err());
    }

    #[test]
    fn raddix_sort_with_singleton_zero_keeps_same_value() {
        // Arrange
        let mut input_values = vec![0];
        let expected_values = vec![0];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_max_value_1_sorts_values_in_ascending_order() {
        // Arrange
        let mut input_values = vec![1, 0];
        let expected_values = vec![0, 1];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_multiple_values_sorts_in_ascending_order() {
        // Arrange
        let mut input_values = vec![3, 1, 2, 1];
        let expected_values = vec![1, 1, 2, 3];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_already_sorted_values_keeps_same_order() {
        // Arrange
        let mut input_values = vec![0, 1, 2, 3];
        let expected_values = vec![0, 1, 2, 3];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_descending_values_returns_ascending_order() {
        // Arrange
        let mut input_values = vec![9, 8, 7, 6];
        let expected_values = vec![6, 7, 8, 9];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_duplicates_preserves_element_multiplicity_in_result() {
        // Arrange
        let mut input_values = vec![5, 1, 5, 2, 1];
        let expected_values = vec![1, 1, 2, 5, 5];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_boundary_max_9_sorts_values_correctly() {
        // Arrange
        let mut input_values = vec![9, 0, 8];
        let expected_values = vec![0, 8, 9];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_boundary_max_10_sorts_values_correctly() {
        // Arrange
        let mut input_values = vec![10, 2, 1];
        let expected_values = vec![1, 2, 10];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_boundary_max_11_sorts_values_correctly() {
        // Arrange
        let mut input_values = vec![11, 2, 10];
        let expected_values = vec![2, 10, 11];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_boundary_values_99_100_101_sorts_considering_hundreds_place() {
        // Arrange
        let mut input_values = vec![101, 99, 100, 3];
        let expected_values = vec![3, 99, 100, 101];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_with_zero_and_multi_digit_values_sorts_correctly() {
        // Arrange
        let mut input_values = vec![0, 100, 10, 1];
        let expected_values = vec![0, 1, 10, 100];

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;

        // Assert
        assert_eq!(sorted_values, expected_values);
    }

    #[test]
    fn raddix_sort_preserves_length_and_multiset_before_and_after_sort() {
        // Arrange
        let mut input_values = vec![20, 3, 20, 1, 0, 3];
        let expected_len = input_values.len();
        let expected_frequency_map = build_frequency_map(&input_values);

        // Act
        raddix_sort(&mut input_values);
        let sorted_values = input_values;
        let actual_len = sorted_values.len();
        let actual_frequency_map = build_frequency_map(&sorted_values);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_frequency_map, expected_frequency_map);
    }

    #[test]
    fn raddix_sort_matches_vec_sort_for_multiple_input_cases() {
        // Arrange
        let input_cases = vec![
            vec![42, 7, 100, 3, 42],
            vec![9, 90, 900, 0, 99],
            vec![15, 14, 13, 12, 11, 10],
            vec![2, 1],
            vec![12345, 54321, 22222, 11111],
        ];

        let expected_cases: Vec<Vec<usize>> = input_cases
            .iter()
            .map(|input_case| {
                let mut expected_sorted_values = input_case.clone();
                expected_sorted_values.sort();
                expected_sorted_values
            })
            .collect();

        // Act
        let actual_cases: Vec<Vec<usize>> = input_cases
            .into_iter()
            .map(|mut input_case| {
                raddix_sort(&mut input_case);
                input_case
            })
            .collect();

        // Assert
        assert_eq!(actual_cases, expected_cases);
    }

    #[test]
    fn raddix_sort_with_values_near_usize_max_finishes_without_panic_and_sorts() {
        // Arrange
        let upper_value = usize::MAX;
        let middle_value = usize::MAX - 1;
        let lower_value = usize::MAX - 2;
        let mut input_values = vec![upper_value, lower_value, middle_value, 0];
        let expected_values = vec![0, lower_value, middle_value, upper_value];

        // Act
        let sort_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            raddix_sort(&mut input_values);
        }));
        let sorted_values = input_values;

        // Assert
        assert!(sort_result.is_ok());
        assert_eq!(sorted_values, expected_values);
    }
}
