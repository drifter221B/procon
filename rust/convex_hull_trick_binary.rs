//! Binary-search convex hull trick.

use std::cmp::Ordering;

// Keep the shared module here so this implementation can be compiled and tested standalone.
#[path = "convex_hull_interface.rs"]
#[allow(dead_code)]
mod convex_hull_interface;

use crate::convex_hull_interface::{ConvexHull, HullMode, Line};

/// CHT for arbitrary-order queries (binary search on intersections).
/// Slopes must be added in non-increasing order for [`HullMode::Min`] and
/// non-decreasing order for [`HullMode::Max`].
pub struct ConvexHullTrickBinary {
    mode: HullMode,
    lines: Vec<Line>,
    crosses: Vec<(i128, i128)>,
}

impl ConvexHullTrickBinary {
    pub fn new(mode: HullMode) -> Self {
        Self {
            mode,
            lines: Vec::new(),
            crosses: Vec::new(),
        }
    }

    pub fn add_line(&mut self, m: i64, c: i64) {
        let line = Line::new(m, c);
        if self.lines.is_empty() {
            self.lines.push(line);
            return;
        }

        while let Some(&last_line) = self.lines.last() {
            if m != last_line.slope() {
                break;
            }

            let new_line_is_at_least_as_good = match self.mode {
                HullMode::Min => c <= last_line.intercept(),
                HullMode::Max => last_line.intercept() <= c,
            };

            if !new_line_is_at_least_as_good {
                return;
            }

            self.lines.pop();
            if !self.crosses.is_empty() {
                self.crosses.pop();
            }
        }

        if self.lines.is_empty() {
            self.lines.push(line);
            return;
        }

        while 2 <= self.lines.len() {
            let first_line = self.lines[self.lines.len() - 2];
            let second_line = self.lines[self.lines.len() - 1];

            if self.should_cull(first_line, second_line, line) {
                self.lines.pop();
                self.crosses.pop();
            } else {
                break;
            }
        }

        let cross = Self::intersection_fraction(*self.lines.last().unwrap(), line);
        self.crosses.push(cross);
        self.lines.push(line);
    }

    pub fn query(&mut self, x: i64) -> Option<i64> {
        if self.lines.is_empty() {
            return None;
        }

        let index = self
            .crosses
            .partition_point(|cross_point| cross_point.0 < (x as i128) * cross_point.1);

        Some(self.lines[index].eval(x))
    }

    fn should_cull(&self, first_line: Line, second_line: Line, third_line: Line) -> bool {
        let first_cross = Self::intersection_fraction(first_line, second_line);
        let second_cross = Self::intersection_fraction(second_line, third_line);

        compare_signed_products(first_cross.0, second_cross.1, second_cross.0, first_cross.1)
            != Ordering::Less
    }

    fn intersection_fraction(first_line: Line, second_line: Line) -> (i128, i128) {
        let numerator = i128::from(first_line.intercept()) - i128::from(second_line.intercept());
        let denominator = i128::from(second_line.slope()) - i128::from(first_line.slope());
        debug_assert_ne!(denominator, 0);

        if denominator < 0 {
            (-numerator, -denominator)
        } else {
            (numerator, denominator)
        }
    }
}

fn compare_signed_products(
    lhs_first_factor: i128,
    lhs_second_factor: i128,
    rhs_first_factor: i128,
    rhs_second_factor: i128,
) -> Ordering {
    let lhs_sign = lhs_first_factor.signum() * lhs_second_factor.signum();
    let rhs_sign = rhs_first_factor.signum() * rhs_second_factor.signum();

    let sign_order = lhs_sign.cmp(&rhs_sign);
    if sign_order != Ordering::Equal {
        return sign_order;
    }

    // Every factor is an i64 difference, so each magnitude is at most 2^64 - 1
    // and its product fits in u128 even when it does not fit in i128.
    let lhs_magnitude = lhs_first_factor.unsigned_abs() * lhs_second_factor.unsigned_abs();
    let rhs_magnitude = rhs_first_factor.unsigned_abs() * rhs_second_factor.unsigned_abs();

    if lhs_sign < 0 {
        rhs_magnitude.cmp(&lhs_magnitude)
    } else {
        lhs_magnitude.cmp(&rhs_magnitude)
    }
}

impl ConvexHull for ConvexHullTrickBinary {
    fn add_line(&mut self, m: i64, c: i64) {
        ConvexHullTrickBinary::add_line(self, m, c);
    }

    fn query(&mut self, x: i64) -> Option<i64> {
        ConvexHullTrickBinary::query(self, x)
    }
}

#[cfg(test)]
mod tests {
    use crate::convex_hull_interface::clamp_i128_to_i64;

    use super::*;

    fn collect_queries<T: ConvexHull>(
        convex_hull: &mut T,
        query_values: &[i64],
    ) -> Vec<Option<i64>> {
        query_values
            .iter()
            .map(|&query_value| convex_hull.query(query_value))
            .collect()
    }

    #[test]
    fn add_line_discards_dominated_same_slope() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Min);
        let expected_value = Some(13);

        let lines = [(1, 5), (1, 3)];
        let query_x = 10;
        // Act
        for (slope, intercept) in lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn common_trait_accepts_binary_impl() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Min);
        let expected_value = Some(-1);

        let lines = [(3, -4), (1, 0)];
        let query_x = 1;
        // Act
        for (slope, intercept) in lines {
            ConvexHull::add_line(&mut convex_hull, slope, intercept);
        }
        let actual_value = ConvexHull::query(&mut convex_hull, query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn same_slope_in_max_mode_keeps_larger_intercept() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Max);
        let expected_value = Some(5);

        let lines = [(1, 1), (1, 5)];
        let query_x = 0;
        // Act
        for (slope, intercept) in lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_positive_integer_intersection_handles_neighboring_x_values() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 0), (0, 2)];
        let query_values = [1, 2, 3];
        let expected_values = vec![Some(1), Some(2), Some(2)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_positive_integer_intersection_handles_neighboring_x_values() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, 2), (1, 0)];
        let query_values = [1, 2, 3];
        let expected_values = vec![Some(2), Some(2), Some(3)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    fn naive_queries(
        lines: &[(i64, i64)],
        mode: HullMode,
        query_values: &[i64],
    ) -> Vec<Option<i64>> {
        query_values
            .iter()
            .map(|&query_value| {
                let values = lines.iter().map(|&(slope, intercept)| {
                    i128::from(slope) * i128::from(query_value) + i128::from(intercept)
                });
                let optimum = match mode {
                    HullMode::Min => values.min(),
                    HullMode::Max => values.max(),
                };
                optimum.map(clamp_i128_to_i64)
            })
            .collect()
    }

    fn next_pseudorandom(state: &mut u64) -> u64 {
        *state = state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *state
    }

    fn generate_reference_case(
        seed: u64,
        mode: HullMode,
        line_count: usize,
        query_count: usize,
    ) -> (Vec<(i64, i64)>, Vec<i64>) {
        let mut state = seed;
        let mut slope = match mode {
            HullMode::Min => 10_000,
            HullMode::Max => -10_000,
        };
        let mut lines = Vec::with_capacity(line_count);

        for _ in 0..line_count {
            let slope_step = (next_pseudorandom(&mut state) % 17) as i64;
            slope = match mode {
                HullMode::Min => slope - slope_step,
                HullMode::Max => slope + slope_step,
            };
            let intercept = (next_pseudorandom(&mut state) % 20_001) as i64 - 10_000;
            lines.push((slope, intercept));
        }

        let query_values = (0..query_count)
            .map(|_| (next_pseudorandom(&mut state) % 20_001) as i64 - 10_000)
            .collect();

        (lines, query_values)
    }

    #[test]
    fn min_fractional_intersection_selects_each_neighboring_line() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, 0), (0, 1)];
        let query_values = [0, 1];
        let expected_values = vec![Some(0), Some(1)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_fractional_intersection_selects_each_neighboring_line() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, 1), (2, 0)];
        let query_values = [0, 1];
        let expected_values = vec![Some(1), Some(2)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_negative_integer_intersection_handles_neighboring_x_values() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 0), (0, -2)];
        let query_values = [-3, -2, -1];
        let expected_values = vec![Some(-3), Some(-2), Some(-2)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_negative_integer_intersection_handles_neighboring_x_values() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, 0), (1, 2)];
        let query_values = [-3, -2, -1];
        let expected_values = vec![Some(0), Some(0), Some(1)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_zero_intersection_handles_neighboring_x_values() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 0), (0, 0)];
        let query_values = [-1, 0, 1];
        let expected_values = vec![Some(-1), Some(0), Some(0)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_zero_intersection_handles_neighboring_x_values() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, 0), (1, 0)];
        let query_values = [-1, 0, 1];
        let expected_values = vec![Some(0), Some(0), Some(1)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_culls_middle_line_that_is_always_dominated() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, 10), (1, 4)];
        let query_values = [0, 2, 3];
        let expected_values = vec![Some(0), Some(6), Some(7)];
        let expected_line_count = 2;
        let expected_cross_count = 1;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn max_culls_middle_line_that_is_always_dominated() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 4), (2, -10), (3, 0)];
        let query_values = [0, 2, 3];
        let expected_values = vec![Some(4), Some(6), Some(9)];
        let expected_line_count = 2;
        let expected_cross_count = 1;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn min_last_addition_culls_multiple_trailing_lines() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(4, 0), (3, 1), (2, 4), (1, 2)];
        let query_values = [0, 1];
        let expected_values = vec![Some(0), Some(3)];
        let expected_line_count = 2;
        let expected_cross_count = 1;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn max_last_addition_culls_multiple_trailing_lines() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(-4, 0), (-3, -1), (-2, -4), (-1, -2)];
        let query_values = [0, 1];
        let expected_values = vec![Some(0), Some(-3)];
        let expected_line_count = 2;
        let expected_cross_count = 1;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn min_culls_middle_line_when_three_lines_share_an_intersection() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, 1), (1, 2)];
        let query_values = [0, 1, 2];
        let expected_values = vec![Some(0), Some(3), Some(4)];
        let expected_line_count = 2;
        let expected_cross_count = 1;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn min_dominated_same_slope_line_leaves_hull_unchanged() {
        // Arrange
        let mode = HullMode::Min;
        let preferred_line = (1, 3);
        let dominated_line = (1, 5);
        let query_value = 4;
        let expected_values = (Some(7), Some(7));
        let expected_line_count = 1;
        let expected_cross_count = 0;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        convex_hull.add_line(preferred_line.0, preferred_line.1);
        let actual_value_before_add = convex_hull.query(query_value);
        convex_hull.add_line(dominated_line.0, dominated_line.1);
        let actual_value_after_add = convex_hull.query(query_value);
        let actual_values = (actual_value_before_add, actual_value_after_add);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn max_dominated_same_slope_line_does_not_corrupt_later_intersection() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 5), (1, 3), (2, 0)];
        let query_value = 4;
        let expected_value = Some(9);
        let expected_line_count = 2;
        let expected_cross_count = 1;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn min_better_same_slope_line_rebuilds_existing_intersection() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, 0), (1, 5), (1, 3)];
        let query_values = [2, 3, 4];
        let expected_values = vec![Some(4), Some(6), Some(7)];
        let expected_crosses = vec![(3, 1)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_crosses = convex_hull.crosses.clone();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_crosses, expected_crosses);
    }

    #[test]
    fn max_better_same_slope_line_rebuilds_existing_intersection() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 0), (2, -5), (2, -3)];
        let query_values = [2, 3, 4];
        let expected_values = vec![Some(2), Some(3), Some(5)];
        let expected_crosses = vec![(3, 1)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);
        let actual_crosses = convex_hull.crosses.clone();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_crosses, expected_crosses);
    }

    #[test]
    fn identical_line_added_three_times_keeps_single_line_without_crosses() {
        // Arrange
        let mode = HullMode::Min;
        let line = (2, -3);
        let repetition_count = 3;
        let query_value = 4;
        let expected_values = vec![Some(5), Some(5), Some(5)];
        let expected_line_count = 1;
        let expected_cross_count = 0;
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        let mut actual_values = Vec::with_capacity(repetition_count);
        for _ in 0..repetition_count {
            convex_hull.add_line(line.0, line.1);
            actual_values.push(convex_hull.query(query_value));
        }
        let actual_line_count = convex_hull.lines.len();
        let actual_cross_count = convex_hull.crosses.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_cross_count, expected_cross_count);
    }

    #[test]
    fn single_line_clamps_positive_evaluation_overflow() {
        // Arrange
        let mode = HullMode::Min;
        let line = (i64::MAX, 0);
        let query_value = 2;
        let expected_value = Some(i64::MAX);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        convex_hull.add_line(line.0, line.1);
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_intersection_handles_full_i64_intercept_difference() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, i64::MAX), (1, i64::MIN)];
        let query_value = 0;
        let expected_value = Some(i64::MAX);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_intersection_handles_full_i64_slope_difference() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(i64::MIN, 0), (i64::MAX, 0)];
        let query_value = 0;
        let expected_value = Some(0);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_cull_handles_full_i64_intercept_difference() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, i64::MAX), (1, i64::MAX - 1), (0, i64::MIN)];
        let query_value = 0;
        let expected_value = Some(i64::MIN);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_cull_handles_full_i64_intercept_difference() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, i64::MAX), (2, 0), (3, i64::MIN)];
        let query_value = 0;
        let expected_value = Some(i64::MAX);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_cull_handles_full_i64_slope_difference() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(i64::MAX, 0), (0, 1), (i64::MIN, 2)];
        let query_value = 0;
        let expected_value = Some(0);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_cull_handles_full_i64_slope_difference() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(i64::MIN, 0), (-1, -1), (i64::MAX, 0)];
        let query_value = 0;
        let expected_value = Some(0);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_cull_handles_cross_product_beyond_i128_range() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [
            (i64::MAX, i64::MAX),
            (i64::MAX - 1, i64::MIN + 1),
            (i64::MIN, i64::MIN),
        ];
        let query_values = [-1, 0, 1, 2];
        let expected_values = naive_queries(&lines, mode, &query_values);
        let expected_shape = (3, 2);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_shape = (convex_hull.lines.len(), convex_hull.crosses.len());
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_shape, expected_shape);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_cull_handles_cross_product_beyond_i128_range() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [
            (i64::MIN, i64::MAX),
            (i64::MIN + 1, i64::MIN + 1),
            (i64::MAX, i64::MIN),
        ];
        let query_values = [-1, 0, 1, 2];
        let expected_values = naive_queries(&lines, mode, &query_values);
        let expected_shape = (2, 1);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_shape = (convex_hull.lines.len(), convex_hull.crosses.len());
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_shape, expected_shape);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_min_handles_random_order_x() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, -2), (2, 0), (1, 5)];
        let query_values = [10, 0, -3];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);
        let expected_values = vec![Some(15), Some(-2), Some(-11)];

        // Act
        for (slope, intercept) in lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_returns_none_when_no_lines_added() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Min);
        let expected_value = None;

        let query_x = 5;
        // Act
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_max_handles_large_values_without_overflow() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Max);
        let big_value = 1_000_000_000i64;
        let expected_value = Some(big_value * big_value);

        let lines = [(big_value, 0), (big_value + 1, -big_value)];
        // Act
        for (slope, intercept) in lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(big_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_on_intersection_returns_expected_value() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Min);
        let lines = [(1, 0), (0, 2)];
        let query_x = 2;
        let expected_value = Some(2);

        // Act
        for (slope, intercept) in lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_three_lines_support_arbitrary_query_order() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, -2), (2, 0), (1, 5)];
        let query_values = [10, 3, -3];
        let expected_values = vec![Some(15), Some(6), Some(-11)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_three_lines_support_arbitrary_query_order() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 5), (2, 3), (3, -2)];
        let query_values = [6, 3, 0];
        let expected_values = vec![Some(16), Some(9), Some(5)];
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn large_negative_x_is_clamped_without_overflow() {
        // Arrange
        let mut convex_hull = ConvexHullTrickBinary::new(HullMode::Max);
        let expected_value = Some(i64::MIN);

        let lines = [(i64::MAX, 0)];
        let query_x = -2;
        // Act
        for (slope, intercept) in lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_fixed_seed_case_matches_naive_scan_for_arbitrary_queries() {
        // Arrange
        let seed = 0x006d_696e_5f63_6874;
        let mode = HullMode::Min;
        let line_count = 256;
        let query_count = 512;
        let (lines, query_values) = generate_reference_case(seed, mode, line_count, query_count);
        let expected_values = naive_queries(&lines, mode, &query_values);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_fixed_seed_case_matches_naive_scan_for_arbitrary_queries() {
        // Arrange
        let seed = 0x006d_6178_5f63_6874;
        let mode = HullMode::Max;
        let line_count = 256;
        let query_count = 512;
        let (lines, query_values) = generate_reference_case(seed, mode, line_count, query_count);
        let expected_values = naive_queries(&lines, mode, &query_values);
        let mut convex_hull = ConvexHullTrickBinary::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
