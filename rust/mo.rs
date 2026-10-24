#[derive(Clone, Copy)]
struct Query {
    left: usize,
    right: usize,
    index: usize,
}

pub struct Mo {
    length: usize,
    bucket_width: usize,
    bucket_size: usize,
    bucket_queries: Vec<Vec<Query>>,
    counters: Vec<usize>,
    current_distinct: usize,
    query_count: usize,
}

impl Mo {

    // queries: [(left, right)]
    pub fn new(length: usize, max_value: usize, queries: &[(usize, usize)]) -> Self {
        let query_count = queries.len();
        let bucket_width = Self::compute_bucket_width(length, query_count);
        let bucket_size = length / bucket_width + 1;
        let mut bucket_queries = vec![Vec::new(); bucket_size];

        for (index, &(left, right)) in queries.iter().enumerate() {
            assert!(left <= right, "left must be <= right");
            assert!(right < length, "query range out of bounds");
            bucket_queries[left / bucket_width].push(Query { left, right, index });
        }

        for (bucket_index, bucket) in bucket_queries.iter_mut().enumerate() {
            if bucket_index % 2 == 0 {
                bucket.sort_by_key(|lhs| std::cmp::Reverse(lhs.right));
            } else {
                bucket.sort_by_key(|lhs| lhs.right);
            }
        }

        Self {
            length,
            bucket_width,
            bucket_size,
            bucket_queries,
            counters: vec![0; max_value + 1],
            current_distinct: 0,
            query_count,
        }
    }

    pub fn run(&mut self, values: &[usize]) -> Vec<usize> {
        assert!(values.len() == self.length, "values length must match n");
        self.reset_state();

        if self.query_count == 0 {
            return Vec::new();
        }

        let mut answers = vec![0; self.query_count];
        let mut left = 0usize;
        let mut right = 0usize;

        for bucket_index in 0..self.bucket_size {

            let bucket_queries = self.bucket_queries[bucket_index].clone();

            for query in bucket_queries.iter() {

                while right <= query.right {
                    self.add_value(values[right]);
                    right += 1;
                }

                while query.left < left {
                    left -= 1;
                    self.add_value(values[left]);
                }

                while left < query.left {
                    self.remove_value(values[left]);
                    left += 1;
                }

                while query.right + 1 < right {
                    self.remove_value(values[right - 1]);
                    right -= 1;
                }

                answers[query.index] = self.answer();
            }
        }

        answers
    }

    fn compute_bucket_width(length: usize, query_count: usize) -> usize {
        if query_count == 0 {
            return length.max(1);
        }

        let denominator = (2.0 * query_count as f64 / 3.0).sqrt();
        let base = (length as f64 / denominator).floor().max(1.0) as usize;

        base + 1
    }

    fn reset_state(&mut self) {
        self.counters.fill(0);
        self.current_distinct = 0;
    }

    fn add_value(&mut self, value: usize) {
        assert!(value < self.counters.len(), "value exceeds max_value");

        if self.counters[value] == 0 {
            self.current_distinct += 1;
        }

        self.counters[value] += 1;
    }

    fn remove_value(&mut self, value: usize) {
        assert!(value < self.counters.len(), "value exceeds max_value");

        if self.counters[value] == 1 {
            self.current_distinct -= 1;
        }

        self.counters[value] -= 1;
    }

    fn answer(&self) -> usize {
        self.current_distinct
    }
}

#[cfg(test)]
mod tests {
    use super::Mo;
    use std::collections::HashSet;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn capture_panic<F>(test_target: F) -> std::thread::Result<()>
    where
        F: FnOnce(),
    {
        catch_unwind(AssertUnwindSafe(test_target))
    }

    fn naive_distinct_count(values: &[usize], left: usize, right: usize) -> usize {
        values[left..right + 1]
            .iter()
            .copied()
            .collect::<HashSet<_>>()
            .len()
    }

    fn naive_answers(values: &[usize], queries: &[(usize, usize)]) -> Vec<usize> {
        queries
            .iter()
            .map(|&(left, right)| naive_distinct_count(values, left, right))
            .collect()
    }

    #[test]
    fn run_right_boundary_x_minus_one_x_x_plus_one_with_new_x_value_increases_distinct_count() {
        // Arrange
        let length = 4;
        let queries = vec![(0, 1), (0, 2), (0, 3)];
        let max_value = 3;
        let values = vec![1, 2, 3, 3];
        let expected_answers = vec![2, 3, 3];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn new_with_query_right_equal_to_length_panics() {
        // Arrange
        let length = 3;
        let queries = vec![(0, 3)];
        let max_value = 3;

        // Act
        let panic_result = capture_panic(move || {
            let _ = Mo::new(length, max_value, &queries);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_query_left_equal_to_length_panics() {
        // Arrange
        let length = 3;
        let queries = vec![(3, 3)];
        let max_value = 3;

        // Act
        let panic_result = capture_panic(move || {
            let _ = Mo::new(length, max_value, &queries);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_query_left_greater_than_right_panics() {
        // Arrange
        let length = 4;
        let queries = vec![(3, 2)];
        let max_value = 4;

        // Act
        let panic_result = capture_panic(move || {
            let _ = Mo::new(length, max_value, &queries);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_length_zero_and_non_empty_queries_panics() {
        // Arrange
        let length = 0;
        let queries = vec![(0, 0)];
        let max_value = 0;

        // Act
        let panic_result = capture_panic(move || {
            let _ = Mo::new(length, max_value, &queries);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn run_with_empty_queries_returns_empty_vector() {
        // Arrange
        let length = 5;
        let queries: Vec<(usize, usize)> = Vec::new();
        let max_value = 3;
        let values = vec![0, 1, 2, 1, 0];
        let expected_answers: Vec<usize> = Vec::new();

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_with_values_length_equal_to_length_returns_answers_for_all_queries() {
        // Arrange
        let length = 5;
        let queries = vec![(0, 2), (1, 4), (2, 2)];
        let max_value = 3;
        let values = vec![0, 1, 0, 2, 1];
        let expected_answers = naive_answers(&values, &queries);

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_with_values_length_length_minus_one_and_length_plus_one_panics() {
        // Arrange
        let length = 3;
        let queries = vec![(0, 2)];
        let max_value = 2;
        let values_with_length_minus_one = vec![0, 1];
        let values_with_length_plus_one = vec![0, 1, 2, 1];

        // Act
        let mut mo_for_short_values = Mo::new(length, max_value, &queries);
        let short_values_panic_result = capture_panic(move || {
            let _ = mo_for_short_values.run(&values_with_length_minus_one);
        });
        let mut mo_for_long_values = Mo::new(length, max_value, &queries);
        let long_values_panic_result = capture_panic(move || {
            let _ = mo_for_long_values.run(&values_with_length_plus_one);
        });

        // Assert
        assert!(short_values_panic_result.is_err());
        assert!(long_values_panic_result.is_err());
    }

    #[test]
    fn run_single_element_query_returns_one_distinct_value() {
        // Arrange
        let length = 3;
        let queries = vec![(0, 0)];
        let max_value = 5;
        let values = vec![5, 5, 5];
        let expected_answers = vec![1];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_range_with_all_equal_values_returns_one_distinct_value() {
        // Arrange
        let length = 5;
        let queries = vec![(1, 3)];
        let max_value = 2;
        let values = vec![2, 2, 2, 2, 2];
        let expected_answers = vec![1];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_range_with_all_distinct_values_returns_range_length() {
        // Arrange
        let length = 5;
        let queries = vec![(1, 4)];
        let max_value = 4;
        let values = vec![0, 1, 2, 3, 4];
        let expected_answers = vec![4];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_boundary_ranges_return_expected_distinct_counts() {
        // Arrange
        let length = 4;
        let queries = vec![(0, 0), (0, 3), (3, 3)];
        let max_value = 3;
        let values = vec![1, 2, 1, 3];
        let expected_answers = vec![1, 3, 1];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_right_boundary_x_minus_one_x_x_plus_one_with_existing_x_value_keeps_distinct_count() {
        // Arrange
        let length = 4;
        let queries = vec![(0, 1), (0, 2), (0, 3)];
        let max_value = 2;
        let values = vec![1, 2, 1, 2];
        let expected_answers = vec![2, 2, 2];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_with_unsorted_queries_preserves_original_query_order_in_answers() {
        // Arrange
        let length = 5;
        let queries = vec![(2, 4), (0, 1), (1, 3)];
        let max_value = 3;
        let values = vec![0, 1, 0, 2, 1];
        let expected_answers = naive_answers(&values, &queries);

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_queries_crossing_bucket_boundaries_match_naive_distinct_counts() {
        // Arrange
        let length = 20;
        let queries = vec![
            (0, 5),
            (6, 11),
            (5, 14),
            (10, 19),
            (2, 17),
            (8, 9),
            (15, 19),
            (4, 4),
        ];
        let max_value = 4;
        let values = vec![
            0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4, 0, 1, 2, 3, 4,
        ];
        let expected_answers = naive_answers(&values, &queries);

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_with_values_containing_max_value_computes_distinct_count_without_panic() {
        // Arrange
        let length = 4;
        let queries = vec![(0, 3)];
        let max_value = 5;
        let values = vec![1, 5, 5, 2];
        let expected_answers = vec![3];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_with_value_exceeding_max_value_panics() {
        // Arrange
        let length = 2;
        let queries = vec![(0, 1)];
        let max_value = 5;
        let values = vec![0, 6];

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let panic_result = capture_panic(move || {
            let _ = mo.run(&values);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn run_with_length_zero_and_empty_queries_returns_empty_vector() {
        // Arrange
        let length = 0;
        let queries: Vec<(usize, usize)> = Vec::new();
        let max_value = 0;
        let values: Vec<usize> = Vec::new();
        let expected_answers: Vec<usize> = Vec::new();

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let actual_answers = mo.run(&values);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn run_twice_with_same_values_returns_identical_answers() {
        // Arrange
        let length = 4;
        let queries = vec![(0, 3), (1, 2)];
        let max_value = 3;
        let values = vec![0, 1, 0, 2];
        let expected_answers = naive_answers(&values, &queries);

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let first_answers = mo.run(&values);
        let second_answers = mo.run(&values);

        // Assert
        assert_eq!(first_answers, expected_answers);
        assert_eq!(second_answers, expected_answers);
    }

    #[test]
    fn run_twice_with_different_values_uses_only_second_values_on_second_run() {
        // Arrange
        let length = 4;
        let queries = vec![(0, 3), (1, 2), (2, 3)];
        let max_value = 2;
        let first_values = vec![0, 1, 0, 2];
        let second_values = vec![2, 2, 2, 2];
        let expected_second_answers = naive_answers(&second_values, &queries);

        // Act
        let mut mo = Mo::new(length, max_value, &queries);
        let _ = mo.run(&first_values);
        let actual_second_answers = mo.run(&second_values);

        // Assert
        assert_eq!(actual_second_answers, expected_second_answers);
    }
}
