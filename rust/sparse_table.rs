struct SparseTable<T, F> where T: Clone, F: Fn(&T, &T) -> T {
    merge: F,
    n: usize,
    table: Vec<Vec<T>>,
    log: Vec<usize>,
    pow2: Vec<usize>,
}

impl<T, F> SparseTable<T, F> where T: Clone, F: Fn(&T, &T) -> T {

    pub fn new(data: &[T], merge: F) -> Self {

        let n = data.len();
        assert!(0 < n);

        let log = Self::build_log(n);
        let max_log = log[n];
        let pow2 = Self::build_pow2(max_log);

        let mut table: Vec<Vec<T>> = Vec::with_capacity(max_log + 1);
        table.push(data.to_owned());

        for k in 0..max_log {

            let step = pow2[k];
            let prev = &table[k];
            let mut row = Vec::with_capacity(prev.len().saturating_sub(step));

            for i in 0..prev.len() - step {
                row.push(merge(&prev[i], &prev[i + step]));
            }

            table.push(row);
        }

        Self { merge, n, table, log, pow2 }
    }

    pub fn query(&self, left: usize, right: usize) -> Option<T> {

        if right <= left || self.n < right {
            return None;
        }

        let length = right - left;
        let k = self.log[length];

        Some((self.merge)(&self.table[k][left], &self.table[k][right - self.pow2[k]]))
    }

    pub fn lower_bound<P>(&self, left: usize, predicate: P) -> Option<usize> where P: Fn(&T) -> bool {

        if self.n <= left || !predicate(self.query(left, self.n).as_ref()?) {
            return None;
        }

        let mut low = left;
        let mut high = self.n;
        while 1 < high - low {

            let mid = (low + high) >> 1;

            if predicate(self.query(left, mid).as_ref()?) {
                high = mid;
            } else {
                low = mid;
            }
        }

        Some(high)
    }

    fn build_log(n: usize) -> Vec<usize> {
        std::iter::once(0)
            .chain((1..=n)
                .map(|i| (usize::BITS - 1 - i.leading_zeros()) as usize))
            .collect()
    }

    fn build_pow2(max_log: usize) -> Vec<usize> {
        std::iter::successors(Some(1usize), |v| Some(v << 1))
            .take(max_log + 1)
            .collect()
    }

}

#[cfg(test)]
mod tests {
    use super::SparseTable;

    fn min_merge(left_value: &i32, right_value: &i32) -> i32 {
        std::cmp::min(*left_value, *right_value)
    }

    fn max_merge(left_value: &i32, right_value: &i32) -> i32 {
        std::cmp::max(*left_value, *right_value)
    }

    fn create_sparse_table_with_min(values: &[i32]) -> SparseTable<i32, fn(&i32, &i32) -> i32> {
        SparseTable::new(values, min_merge as fn(&i32, &i32) -> i32)
    }

    fn create_sparse_table_with_max(values: &[i32]) -> SparseTable<i32, fn(&i32, &i32) -> i32> {
        SparseTable::new(values, max_merge as fn(&i32, &i32) -> i32)
    }

    fn calculate_linear_min(values: &[i32], left: usize, right: usize) -> i32 {
        values[left..right].iter().copied().min().unwrap()
    }

    #[test]
    fn new_with_empty_slice_panics() {
        // Arrange
        let values: Vec<i32> = vec![];

        // Act
        let actual_result = std::panic::catch_unwind(|| create_sparse_table_with_min(&values));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn query_for_single_element_slice_returns_the_element_value() {
        // Arrange
        let values = vec![42];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_value = 42i32;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_two_element_slice_returns_merged_value() {
        // Arrange
        let values = vec![8, 3];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = 2usize;
        let expected_value = min_merge(&values[0], &values[1]);

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_non_power_of_two_length_slice_returns_expected_value() {
        // Arrange
        let values = vec![4, 2, 5, 1, 3];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = values.len();
        let expected_value = 1i32;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_with_right_less_than_left_returns_none() {
        // Arrange
        let values = vec![9, 7, 5, 3];
        let table = create_sparse_table_with_min(&values);
        let query_left = 3usize;
        let query_right = 2usize;
        let expected_value = None;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_with_right_equal_to_left_returns_none() {
        // Arrange
        let values = vec![9, 7, 5, 3];
        let table = create_sparse_table_with_min(&values);
        let query_left = 2usize;
        let query_right = 2usize;
        let expected_value = None;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_with_right_greater_than_length_returns_none() {
        // Arrange
        let values = vec![9, 7, 5];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = 4usize;
        let expected_value = None;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_for_prefix_singleton_returns_first_element_value() {
        // Arrange
        let values = vec![11, 5, 9];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_value = values[0];

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_suffix_singleton_returns_last_element_value() {
        // Arrange
        let values = vec![11, 5, 9];
        let table = create_sparse_table_with_min(&values);
        let query_left = values.len() - 1usize;
        let query_right = values.len();
        let expected_value = values[values.len() - 1usize];

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_full_range_returns_expected_value() {
        // Arrange
        let values = vec![5, 1, 4, 2];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = values.len();
        let expected_value = 1i32;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_range_length_power_of_two_minus_one_returns_expected_value() {
        // Arrange
        let values = vec![9, 4, 7, 3, 8, 1, 6, 2, 5, 0];
        let table = create_sparse_table_with_min(&values);
        let query_left = 1usize;
        let query_right = 8usize;
        let expected_value = 1i32;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_range_length_power_of_two_returns_expected_value() {
        // Arrange
        let values = vec![9, 4, 7, 3, 8, 1, 6, 2, 5, 0];
        let table = create_sparse_table_with_min(&values);
        let query_left = 1usize;
        let query_right = 9usize;
        let expected_value = 1i32;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_range_length_power_of_two_plus_one_returns_expected_value() {
        // Arrange
        let values = vec![9, 4, 7, 3, 8, 1, 6, 2, 5, 0];
        let table = create_sparse_table_with_min(&values);
        let query_left = 0usize;
        let query_right = 9usize;
        let expected_value = 1i32;

        // Act
        let actual_value = table.query(query_left, query_right);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn query_for_all_valid_ranges_matches_linear_scan_result() {
        // Arrange
        let values = vec![7, 1, 5, 3, 9, 2];
        let table = create_sparse_table_with_min(&values);
        let range_length = values.len();
        let values_ref = &values;
        let expected_values = (0..range_length)
            .flat_map(|left_index| {
                (left_index + 1..=range_length).map(move |right_index| {
                    (
                        left_index,
                        right_index,
                        calculate_linear_min(values_ref, left_index, right_index),
                    )
                })
            })
            .collect::<Vec<_>>();

        // Act
        let actual_values = expected_values
            .iter()
            .map(|&(left_index, right_index, _)| (left_index, right_index, table.query(left_index, right_index)))
            .collect::<Vec<_>>();

        // Assert
        for ((left_index, right_index, expected_value), (_, _, actual_value)) in
            expected_values.into_iter().zip(actual_values)
        {
            assert_eq!(
                actual_value,
                Some(expected_value),
                "left_index: {left_index}, right_index: {right_index}"
            );
        }
    }

    #[test]
    fn lower_bound_with_left_equal_to_length_returns_none() {
        // Arrange
        let values = vec![1, 4, 2];
        let table = create_sparse_table_with_max(&values);
        let query_left = values.len();
        let threshold = 3i32;
        let expected_position = None;

        // Act
        let actual_position = table.lower_bound(query_left, |value| threshold <= *value);

        // Assert
        assert_eq!(actual_position, expected_position);
    }

    #[test]
    fn lower_bound_with_left_greater_than_length_returns_none() {
        // Arrange
        let values = vec![1, 4, 2];
        let table = create_sparse_table_with_max(&values);
        let query_left = values.len() + 1usize;
        let threshold = 3i32;
        let expected_position = None;

        // Act
        let actual_position = table.lower_bound(query_left, |value| threshold <= *value);

        // Assert
        assert_eq!(actual_position, expected_position);
    }

    #[test]
    fn lower_bound_returns_none_when_predicate_is_false_for_full_range() {
        // Arrange
        let values = vec![1, 2, 3];
        let table = create_sparse_table_with_max(&values);
        let query_left = 0usize;
        let threshold = 4i32;
        let expected_position = None;

        // Act
        let actual_position = table.lower_bound(query_left, |value| threshold <= *value);

        // Assert
        assert_eq!(actual_position, expected_position);
    }

    #[test]
    fn lower_bound_returns_left_plus_one_when_first_satisfied_position_is_left_plus_one() {
        // Arrange
        let values = vec![1, 5, 2];
        let table = create_sparse_table_with_max(&values);
        let query_left = 0usize;
        let threshold = 1i32;
        let expected_position = Some(query_left + 1usize);

        // Act
        let actual_position = table.lower_bound(query_left, |value| threshold <= *value);

        // Assert
        assert_eq!(actual_position, expected_position);
    }

    #[test]
    fn lower_bound_returns_length_when_first_satisfied_position_is_length() {
        // Arrange
        let values = vec![1, 2, 10];
        let table = create_sparse_table_with_max(&values);
        let query_left = 0usize;
        let threshold = 10i32;
        let expected_position = Some(values.len());

        // Act
        let actual_position = table.lower_bound(query_left, |value| threshold <= *value);

        // Assert
        assert_eq!(actual_position, expected_position);
    }

    #[test]
    fn lower_bound_returns_expected_positions_for_different_left_indices() {
        // Arrange
        let values = vec![1, 3, 2, 8, 5];
        let table = create_sparse_table_with_max(&values);
        let threshold = 8i32;
        let query_left_a = 0usize;
        let query_left_b = 2usize;
        let query_left_c = 3usize;
        let expected_position_a = Some(4usize);
        let expected_position_b = Some(4usize);
        let expected_position_c = Some(4usize);

        // Act
        let actual_position_a = table.lower_bound(query_left_a, |value| threshold <= *value);
        let actual_position_b = table.lower_bound(query_left_b, |value| threshold <= *value);
        let actual_position_c = table.lower_bound(query_left_c, |value| threshold <= *value);

        // Assert
        assert_eq!(actual_position_a, expected_position_a);
        assert_eq!(actual_position_b, expected_position_b);
        assert_eq!(actual_position_c, expected_position_c);
    }

}
