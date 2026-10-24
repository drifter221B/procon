use std::ops::{Add, AddAssign, Sub, SubAssign};
struct PrefixSum2D<T> {
    row: usize,
    col: usize,
    diff: Vec<Vec<T>>,
    pre_sum: Vec<Vec<T>>,
}

impl<T> PrefixSum2D<T>
where
    T: Add<Output = T> + AddAssign + Sub<Output = T> + SubAssign + Clone + Copy + Default,
{
    pub fn new(row: usize, col: usize) -> Self {
        let buffer = vec![vec![T::default(); col + 1]; row + 1];
        Self {
            row,
            col,
            diff: buffer.clone(),
            pre_sum: buffer,
        }
    }

    pub fn add(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, value: T) {
        self.diff[x1][y1] += value;
        self.diff[x1][y2 + 1] -= value;
        self.diff[x2 + 1][y1] -= value;
        self.diff[x2 + 1][y2 + 1] += value;
    }

    pub fn add1(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, value: T) {
        self.add(x1 - 1, y1 - 1, x2 - 1, y2 - 1, value);
    }

    pub fn build(&mut self) {
        let mut cell_values = self.diff.clone();

        for x in 0..=self.row {
            for y in 0..self.col {
                let current_value = cell_values[x][y];
                cell_values[x][y + 1] += current_value;
            }
        }

        for x in 0..self.row {
            for y in 0..=self.col {
                let current_value = cell_values[x][y];
                cell_values[x + 1][y] += current_value;
            }
        }

        self.pre_sum = vec![vec![T::default(); self.col + 1]; self.row + 1];
        for x in 0..self.row {
            for y in 0..self.col {
                let accumulated_value = self.pre_sum[x + 1][y] + self.pre_sum[x][y + 1]
                    - self.pre_sum[x][y]
                    + cell_values[x][y];
                self.pre_sum[x + 1][y + 1] = accumulated_value;
            }
        }
    }

    pub fn query(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> T {
        if x2 < x1 || y2 < y1 {
            return T::default();
        }

        self.pre_sum[x2 + 1][y2 + 1] - self.pre_sum[x2 + 1][y1] - self.pre_sum[x1][y2 + 1]
            + self.pre_sum[x1][y1]
    }

    pub fn query1(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> T {
        self.query(x1 - 1, y1 - 1, x2 - 1, y2 - 1)
    }
}

#[cfg(test)]
mod tests {
    use super::PrefixSum2D;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn build_with_add_updates(
        row: usize,
        col: usize,
        updates: &[(usize, usize, usize, usize, i64)],
    ) -> PrefixSum2D<i64> {
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        for &(x1, y1, x2, y2, value) in updates {
            prefix_sum.add(x1, y1, x2, y2, value);
        }
        prefix_sum.build();
        prefix_sum
    }

    fn build_with_add1_updates(
        row: usize,
        col: usize,
        updates: &[(usize, usize, usize, usize, i64)],
    ) -> PrefixSum2D<i64> {
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        for &(x1, y1, x2, y2, value) in updates {
            prefix_sum.add1(x1, y1, x2, y2, value);
        }
        prefix_sum.build();
        prefix_sum
    }

    fn has_panicked<F>(f: F) -> bool
    where
        F: FnOnce(),
    {
        catch_unwind(AssertUnwindSafe(f)).is_err()
    }

    #[test]
    fn new_with_zero_row_and_zero_col_creates_1_by_1_buffer() {
        // Arrange
        let row = 0usize;
        let col = 0usize;
        let expected_row_len = 1usize;
        let expected_col_len = 1usize;

        // Act
        let prefix_sum = PrefixSum2D::<i64>::new(row, col);
        let actual_row_len = prefix_sum.pre_sum.len();
        let actual_col_len = prefix_sum.pre_sum[0].len();

        // Assert
        assert_eq!(actual_row_len, expected_row_len);
        assert_eq!(actual_col_len, expected_col_len);
    }

    #[test]
    fn new_with_one_row_and_one_col_creates_2_by_2_buffer() {
        // Arrange
        let row = 1usize;
        let col = 1usize;
        let expected_row_len = 2usize;
        let expected_col_len = 2usize;

        // Act
        let prefix_sum = PrefixSum2D::<i64>::new(row, col);
        let actual_row_len = prefix_sum.pre_sum.len();
        let actual_col_len = prefix_sum.pre_sum[0].len();

        // Assert
        assert_eq!(actual_row_len, expected_row_len);
        assert_eq!(actual_col_len, expected_col_len);
    }

    #[test]
    fn new_with_row_boundary_0_1_2_does_not_panic() {
        // Arrange
        let row_candidates = vec![0usize, 1usize, 2usize];
        let col = 1usize;
        let expected_panicked = vec![false, false, false];

        // Act
        let actual_panicked: Vec<bool> = row_candidates
            .iter()
            .map(|&row| {
                has_panicked(|| {
                    let _prefix_sum = PrefixSum2D::<i64>::new(row, col);
                })
            })
            .collect();

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn new_with_col_boundary_0_1_2_does_not_panic() {
        // Arrange
        let col_candidates = vec![0usize, 1usize, 2usize];
        let row = 1usize;
        let expected_panicked = vec![false, false, false];

        // Act
        let actual_panicked: Vec<bool> = col_candidates
            .iter()
            .map(|&col| {
                has_panicked(|| {
                    let _prefix_sum = PrefixSum2D::<i64>::new(row, col);
                })
            })
            .collect();

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn add_with_negative_update_then_query_returns_signed_sum() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let updates = vec![
            (0usize, 0usize, 1usize, 1usize, 10i64),
            (0usize, 0usize, 0usize, 1usize, -4i64),
        ];
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 0usize;
        let y2 = 1usize;
        let expected_sum = 12i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_with_lower_bound_origin_returns_correct_sum() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![(0usize, 0usize, 1usize, 1usize, 2i64)];
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_sum = 8i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_with_upper_bound_bottom_right_returns_correct_sum() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![(1usize, 1usize, 2usize, 2usize, 3i64)];
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 2usize;
        let y2 = 2usize;
        let expected_sum = 12i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_with_inclusive_side_lengths_one_two_three_returns_expected_sums() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![(0usize, 0usize, 2usize, 2usize, 1i64)];
        let expected_size_0_sum = 1i64;
        let expected_size_1_sum = 4i64;
        let expected_size_2_sum = 9i64;
        let query_ranges = [(0, 0, 0, 0), (0, 0, 1, 1), (0, 0, 2, 2)];

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_size_0_sum = prefix_sum.query(
            query_ranges[0].0,
            query_ranges[0].1,
            query_ranges[0].2,
            query_ranges[0].3,
        );
        let actual_size_1_sum = prefix_sum.query(
            query_ranges[1].0,
            query_ranges[1].1,
            query_ranges[1].2,
            query_ranges[1].3,
        );
        let actual_size_2_sum = prefix_sum.query(
            query_ranges[2].0,
            query_ranges[2].1,
            query_ranges[2].2,
            query_ranges[2].3,
        );

        // Assert
        assert_eq!(actual_size_0_sum, expected_size_0_sum);
        assert_eq!(actual_size_1_sum, expected_size_1_sum);
        assert_eq!(actual_size_2_sum, expected_size_2_sum);
    }

    #[test]
    fn add_single_row_range_then_query_returns_row_direction_sum() {
        // Arrange
        let row = 3usize;
        let col = 4usize;
        let updates = vec![(1usize, 0usize, 1usize, 3usize, 2i64)];
        let x1 = 1usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 3usize;
        let expected_sum = 8i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_single_col_range_then_query_returns_col_direction_sum() {
        // Arrange
        let row = 4usize;
        let col = 4usize;
        let updates = vec![(0usize, 2usize, 3usize, 2usize, 3i64)];
        let x1 = 0usize;
        let y1 = 2usize;
        let x2 = 3usize;
        let y2 = 2usize;
        let expected_sum = 12i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_full_range_then_query_returns_total_sum() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![(0usize, 0usize, 2usize, 2usize, 5i64)];
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 2usize;
        let y2 = 2usize;
        let expected_sum = 45i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_of_partial_queries_equals_whole_query() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![
            (0usize, 0usize, 2usize, 2usize, 1i64),
            (1usize, 1usize, 2usize, 2usize, 2i64),
        ];

        let coordinate_zero = 0usize;
        let coordinate_two = 2usize;
        let coordinate_one = 1usize;
        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_top_sum = prefix_sum.query(
            coordinate_zero,
            coordinate_zero,
            coordinate_zero,
            coordinate_two,
        );
        let actual_bottom_sum = prefix_sum.query(
            coordinate_one,
            coordinate_zero,
            coordinate_two,
            coordinate_two,
        );
        let actual_whole_sum = prefix_sum.query(
            coordinate_zero,
            coordinate_zero,
            coordinate_two,
            coordinate_two,
        );
        let actual_partition_sum = actual_top_sum + actual_bottom_sum;

        // Assert
        assert_eq!(actual_partition_sum, actual_whole_sum);
    }

    #[test]
    fn add1_with_upper_bound_then_query1_returns_total_sum() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let updates = vec![(1usize, 1usize, 2usize, 2usize, 4i64)];
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 2usize;
        let y2 = 2usize;
        let expected_sum = 16i64;

        // Act
        let prefix_sum = build_with_add1_updates(row, col, &updates);
        let actual_sum = prefix_sum.query1(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_after_build_without_updates_returns_zero() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_sum = 0i64;

        // Act
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        prefix_sum.build();
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_single_cell_then_query_same_cell_returns_added_value() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![(1usize, 1usize, 1usize, 1usize, 7i64)];
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_sum = 7i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_overlapping_rectangles_then_overlap_query_returns_accumulated_value() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![
            (0usize, 0usize, 1usize, 1usize, 2i64),
            (1usize, 1usize, 2usize, 2usize, 3i64),
        ];
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_sum = 5i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_with_x2_equal_row_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 2usize;
        let y2 = 1usize;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.build();
            let _query_result = prefix_sum.query(x1, y1, x2, y2);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn query_with_y2_equal_col_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 2usize;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.build();
            let _query_result = prefix_sum.query(x1, y1, x2, y2);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn query_with_x1_greater_than_x2_returns_zero_without_panic() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 1usize;
        let y1 = 0usize;
        let x2 = 0usize;
        let y2 = 1usize;
        let expected_sum = 0i64;

        // Act
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        prefix_sum.build();
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_with_y1_greater_than_y2_returns_zero_without_panic() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 1usize;
        let x2 = 1usize;
        let y2 = 0usize;
        let expected_sum = 0i64;

        // Act
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        prefix_sum.build();
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add1_with_lower_bound_then_query1_returns_value() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let updates = vec![(1usize, 1usize, 1usize, 1usize, 7i64)];
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_sum = 7i64;

        // Act
        let prefix_sum = build_with_add1_updates(row, col, &updates);
        let actual_sum = prefix_sum.query1(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query1_with_x2_equal_row_plus_1_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 3usize;
        let y2 = 2usize;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.build();
            let _query_result = prefix_sum.query1(x1, y1, x2, y2);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn query1_with_y2_equal_col_plus_1_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 2usize;
        let y2 = 3usize;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.build();
            let _query_result = prefix_sum.query1(x1, y1, x2, y2);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn build_once_returns_expected_query_value() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let updates = vec![(0usize, 0usize, 1usize, 1usize, 1i64)];
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_sum = 4i64;

        // Act
        let prefix_sum = build_with_add_updates(row, col, &updates);
        let actual_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn build_twice_keeps_query_result_unchanged() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 1usize;
        let expected_first_sum = 4i64;
        let expected_second_sum = 4i64;
        let update = (0usize, 0usize, 1usize, 1usize, 1i64);

        // Act
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        prefix_sum.add(update.0, update.1, update.2, update.3, update.4);
        prefix_sum.build();
        let actual_first_sum = prefix_sum.query(x1, y1, x2, y2);
        prefix_sum.build();
        let actual_second_sum = prefix_sum.query(x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_first_sum, expected_first_sum);
        assert_eq!(actual_second_sum, expected_second_sum);
    }

    #[test]
    fn add_after_build_then_rebuild_accumulates_all_updates() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let first_update_value = 1i64;
        let second_update_value = 2i64;
        let expected_sum = 3i64;
        let first_update = (0usize, 0usize, 0usize, 0usize);
        let second_update = (1usize, 1usize, 1usize, 1usize);
        let query_range = (0usize, 0usize, 1usize, 1usize);

        // Act
        let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
        prefix_sum.add(
            first_update.0,
            first_update.1,
            first_update.2,
            first_update.3,
            first_update_value,
        );
        prefix_sum.build();
        prefix_sum.add(
            second_update.0,
            second_update.1,
            second_update.2,
            second_update.3,
            second_update_value,
        );
        prefix_sum.build();
        let actual_sum =
            prefix_sum.query(query_range.0, query_range.1, query_range.2, query_range.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_x2_equal_row_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 2usize;
        let y2 = 1usize;
        let value = 1i64;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.add(x1, y1, x2, y2, value);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn add_with_y2_equal_col_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 0usize;
        let y1 = 0usize;
        let x2 = 1usize;
        let y2 = 2usize;
        let value = 1i64;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.add(x1, y1, x2, y2, value);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn add1_with_x1_boundary_0_1_2_matches_current_behavior() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let y1 = 1usize;
        let y2 = 1usize;
        let value = 5i64;
        let expected_zero_panicked = true;
        let expected_one_sum = 5i64;
        let expected_two_sum = 5i64;

        let coordinate_zero = 0usize;
        let coordinate_one = 1usize;
        let coordinate_two = 2usize;
        // Act
        let actual_zero_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.add1(coordinate_zero, y1, coordinate_one, y2, value);
        });
        let mut one_prefix_sum = PrefixSum2D::<i64>::new(row, col);
        one_prefix_sum.add1(coordinate_one, y1, coordinate_one, y2, value);
        one_prefix_sum.build();
        let actual_one_sum = one_prefix_sum.query1(coordinate_one, y1, coordinate_one, y2);
        let mut two_prefix_sum = PrefixSum2D::<i64>::new(row, col);
        two_prefix_sum.add1(coordinate_two, y1, coordinate_two, y2, value);
        two_prefix_sum.build();
        let actual_two_sum = two_prefix_sum.query1(coordinate_two, y1, coordinate_two, y2);

        // Assert
        assert_eq!(actual_zero_panicked, expected_zero_panicked);
        assert_eq!(actual_one_sum, expected_one_sum);
        assert_eq!(actual_two_sum, expected_two_sum);
    }

    #[test]
    fn add1_with_y1_boundary_0_1_2_matches_current_behavior() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let x1 = 1usize;
        let x2 = 1usize;
        let value = 6i64;
        let expected_zero_panicked = true;
        let expected_one_sum = 6i64;
        let expected_two_sum = 6i64;

        let coordinate_zero = 0usize;
        let coordinate_one = 1usize;
        let coordinate_two = 2usize;
        // Act
        let actual_zero_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.add1(x1, coordinate_zero, x2, coordinate_one, value);
        });
        let mut one_prefix_sum = PrefixSum2D::<i64>::new(row, col);
        one_prefix_sum.add1(x1, coordinate_one, x2, coordinate_one, value);
        one_prefix_sum.build();
        let actual_one_sum = one_prefix_sum.query1(x1, coordinate_one, x2, coordinate_one);
        let mut two_prefix_sum = PrefixSum2D::<i64>::new(row, col);
        two_prefix_sum.add1(x1, coordinate_two, x2, coordinate_two, value);
        two_prefix_sum.build();
        let actual_two_sum = two_prefix_sum.query1(x1, coordinate_two, x2, coordinate_two);

        // Assert
        assert_eq!(actual_zero_panicked, expected_zero_panicked);
        assert_eq!(actual_one_sum, expected_one_sum);
        assert_eq!(actual_two_sum, expected_two_sum);
    }

    #[test]
    fn add1_with_x2_equal_row_plus_1_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 3usize;
        let y2 = 2usize;
        let value = 1i64;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.add1(x1, y1, x2, y2, value);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn add1_with_y2_equal_col_plus_1_panics() {
        // Arrange
        let row = 2usize;
        let col = 2usize;
        let x1 = 1usize;
        let y1 = 1usize;
        let x2 = 2usize;
        let y2 = 3usize;
        let value = 1i64;
        let expected_panicked = true;

        // Act
        let actual_panicked = has_panicked(|| {
            let mut prefix_sum = PrefixSum2D::<i64>::new(row, col);
            prefix_sum.add1(x1, y1, x2, y2, value);
        });

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
    }

    #[test]
    fn zero_index_and_one_index_api_return_same_result_for_same_region() {
        // Arrange
        let row = 3usize;
        let col = 3usize;
        let zero_index_updates = vec![(0usize, 0usize, 1usize, 2usize, 2i64)];
        let one_index_updates = vec![(1usize, 1usize, 2usize, 3usize, 2i64)];
        let zero_x1 = 0usize;
        let zero_y1 = 1usize;
        let zero_x2 = 1usize;
        let zero_y2 = 2usize;
        let one_x1 = 1usize;
        let one_y1 = 2usize;
        let one_x2 = 2usize;
        let one_y2 = 3usize;

        // Act
        let zero_index_prefix_sum = build_with_add_updates(row, col, &zero_index_updates);
        let one_index_prefix_sum = build_with_add1_updates(row, col, &one_index_updates);
        let actual_zero_index_sum = zero_index_prefix_sum.query(zero_x1, zero_y1, zero_x2, zero_y2);
        let actual_one_index_sum = one_index_prefix_sum.query1(one_x1, one_y1, one_x2, one_y2);

        // Assert
        assert_eq!(actual_zero_index_sum, actual_one_index_sum);
    }
}
