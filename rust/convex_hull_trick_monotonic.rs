//! Monotonic-query convex hull trick.

// Keep the shared module here so this implementation can be compiled and tested standalone.
#[path = "convex_hull_interface.rs"]
#[allow(dead_code)]
mod convex_hull_interface;

use crate::convex_hull_interface::{ConvexHull, HullMode, Line};
use std::cmp::Ordering;
use std::collections::VecDeque;

/// Monotone-query CHT.
/// - Slopes are added in non-increasing order for `HullMode::Min`.
/// - Slopes are added in non-decreasing order for `HullMode::Max`.
/// - `query` must be called with non-decreasing x.
pub struct ConvexHullTrickMonotonic {
    mode: HullMode,
    lines: VecDeque<Line>,
}

pub type ConvexHullTrickMonotonous = ConvexHullTrickMonotonic;

impl ConvexHullTrickMonotonic {
    pub fn new(mode: HullMode) -> Self {
        Self {
            mode,
            lines: VecDeque::new(),
        }
    }

    pub fn add_line(&mut self, m: i64, c: i64) {
        let line = Line::new(m, c);

        if let Some(last_line) = self.lines.back().copied() {
            if last_line.slope() == line.slope() {
                let new_line_is_better = match self.mode {
                    HullMode::Min => line.intercept() < last_line.intercept(),
                    HullMode::Max => last_line.intercept() < line.intercept(),
                };

                if new_line_is_better {
                    self.lines.pop_back();
                } else {
                    return;
                }
            }
        }

        while 2 <= self.lines.len() {
            let first_line = self.lines[self.lines.len() - 2];
            let second_line = self.lines[self.lines.len() - 1];

            if self.should_cull(first_line, second_line, line) {
                self.lines.pop_back();
            } else {
                break;
            }
        }

        self.lines.push_back(line);
    }

    pub fn query(&mut self, x: i64) -> Option<i64> {
        if self.lines.is_empty() {
            return None;
        }

        while 2 <= self.lines.len() {
            let first_line = self.lines[0];
            let second_line = self.lines[1];
            if self.should_cull_front(first_line, second_line, x) {
                self.lines.pop_front();
            } else {
                break;
            }
        }

        Some(self.lines[0].eval(x))
    }

    fn should_cull(&self, first_line: Line, second_line: Line, third_line: Line) -> bool {
        let first_intercept_difference =
            i128::from(second_line.intercept()) - i128::from(first_line.intercept());
        let second_slope_difference =
            i128::from(second_line.slope()) - i128::from(third_line.slope());
        let second_intercept_difference =
            i128::from(third_line.intercept()) - i128::from(second_line.intercept());
        let first_slope_difference =
            i128::from(first_line.slope()) - i128::from(second_line.slope());

        compare_signed_products(
            first_intercept_difference,
            second_slope_difference,
            second_intercept_difference,
            first_slope_difference,
        ) != Ordering::Less
    }

    fn is_smaller(&self, first_line: Line, second_line: Line, x: i128) -> bool {
        first_line.eval128_with_i128(x) < second_line.eval128_with_i128(x)
    }

    fn is_larger(&self, first_line: Line, second_line: Line, x: i128) -> bool {
        second_line.eval128_with_i128(x) < first_line.eval128_with_i128(x)
    }

    fn should_cull_front(&self, first_line: Line, second_line: Line, x: i64) -> bool {
        match self.mode {
            HullMode::Min => !self.is_smaller(first_line, second_line, x as i128),
            HullMode::Max => !self.is_larger(first_line, second_line, x as i128),
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

    // Every factor is a difference of two i64 values, so each magnitude is at
    // most 2^64 - 1 and the product is representable exactly by u128.
    let lhs_magnitude = lhs_first_factor.unsigned_abs() * lhs_second_factor.unsigned_abs();
    let rhs_magnitude = rhs_first_factor.unsigned_abs() * rhs_second_factor.unsigned_abs();

    if lhs_sign < 0 {
        rhs_magnitude.cmp(&lhs_magnitude)
    } else {
        lhs_magnitude.cmp(&rhs_magnitude)
    }
}

impl ConvexHull for ConvexHullTrickMonotonic {
    fn add_line(&mut self, m: i64, c: i64) {
        ConvexHullTrickMonotonic::add_line(self, m, c);
    }

    fn query(&mut self, x: i64) -> Option<i64> {
        ConvexHullTrickMonotonic::query(self, x)
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
    fn common_trait_accepts_monotonic_impl() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, 0), (1, 3)];
        let query_value = 4;
        let expected_value = Some(7);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            ConvexHull::add_line(&mut convex_hull, slope, intercept);
        }
        let actual_value = ConvexHull::query(&mut convex_hull, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_add_line_culls_multiple_trailing_lines() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(4, 0), (3, 0), (2, 2), (1, 0)];
        let query_value = 1;
        let expected_values = (2, Some(1));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_add_line_culls_multiple_trailing_lines() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(-4, 0), (-3, 0), (-2, -2), (-1, 0)];
        let query_value = 1;
        let expected_values = (2, Some(-1));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_first_query_culls_multiple_leading_lines() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, -10), (1, 0)];
        let query_value = 20;
        let expected_values = (Some(20), 1);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let query_result = convex_hull.query(query_value);
        let line_count = convex_hull.lines.len();
        let actual_values = (query_result, line_count);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_supports_adding_next_slope_after_a_query() {
        // Arrange
        let mode = HullMode::Min;
        let initial_lines = [(3, 0), (2, 0)];
        let added_line = (1, 0);
        let first_query_value = 1;
        let second_query_value = 2;
        let expected_values = (Some(2), Some(2));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &initial_lines {
            convex_hull.add_line(slope, intercept);
        }
        let first_result = convex_hull.query(first_query_value);
        convex_hull.add_line(added_line.0, added_line.1);
        let second_result = convex_hull.query(second_query_value);
        let actual_values = (first_result, second_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    fn naive_query(lines: &[(i64, i64)], query_value: i64, mode: HullMode) -> Option<i64> {
        let evaluated_values = lines.iter().map(|&(slope, intercept)| {
            i128::from(slope) * i128::from(query_value) + i128::from(intercept)
        });
        let best_value = match mode {
            HullMode::Min => evaluated_values.min(),
            HullMode::Max => evaluated_values.max(),
        };

        best_value.map(clamp_i128_to_i64)
    }

    fn naive_queries(
        lines: &[(i64, i64)],
        query_values: &[i64],
        mode: HullMode,
    ) -> Vec<Option<i64>> {
        query_values
            .iter()
            .map(|&query_value| naive_query(lines, query_value, mode))
            .collect()
    }

    fn next_random(state: &mut u64) -> u64 {
        const MULTIPLIER: u64 = 6_364_136_223_846_793_005;
        const INCREMENT: u64 = 1_442_695_040_888_963_407;

        *state = state.wrapping_mul(MULTIPLIER).wrapping_add(INCREMENT);
        *state
    }

    #[test]
    fn min_better_equal_slope_line_replaces_existing_line() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 5), (1, 3)];
        let query_value = 0;
        let expected_values = (1, Some(3));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_returns_expected_values_around_integer_intersection() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 2), (2, 0)];
        let query_values = [1, 2, 3];
        let expected_values = vec![Some(3), Some(4), Some(6)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_returns_expected_values_around_non_integer_intersection() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, 0), (0, 1)];
        let query_values = [0, 1];
        let expected_values = vec![Some(0), Some(1)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_returns_expected_values_around_non_integer_intersection() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, 1), (2, 0)];
        let query_values = [0, 1];
        let expected_values = vec![Some(1), Some(2)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_returns_expected_values_around_negative_integer_intersection() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 0), (0, -2)];
        let query_values = [-3, -2, -1];
        let expected_values = vec![Some(-3), Some(-2), Some(-2)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_returns_expected_values_around_negative_integer_intersection() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(0, 0), (1, 2)];
        let query_values = [-3, -2, -1];
        let expected_values = vec![Some(0), Some(0), Some(1)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_culls_middle_line_that_is_never_optimal() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, 10), (1, 0)];
        let query_values = [-10, 1];
        let expected_values = (2, vec![Some(-30), Some(1)]);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_results = collect_queries(&mut convex_hull, &query_values);
        let actual_values = (line_count, query_results);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_keeps_middle_line_that_has_an_optimal_interval() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, -10), (1, 0)];
        let query_values = [-20, 0, 20];
        let expected_values = vec![Some(-60), Some(-10), Some(20)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_culls_middle_line_that_is_never_optimal() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 0), (2, -10), (3, 0)];
        let query_values = [-20, 1];
        let expected_values = (2, vec![Some(-20), Some(3)]);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_results = collect_queries(&mut convex_hull, &query_values);
        let actual_values = (line_count, query_results);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_keeps_middle_line_that_has_an_optimal_interval() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 0), (2, 10), (3, 0)];
        let query_values = [-20, 0, 20];
        let expected_values = vec![Some(-20), Some(10), Some(60)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_culls_middle_line_when_three_lines_meet_at_one_point() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, 1), (1, 2)];
        let query_values = [0, 1, 2];
        let expected_values = (2, vec![Some(0), Some(3), Some(4)]);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_results = collect_queries(&mut convex_hull, &query_values);
        let actual_values = (line_count, query_results);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_keeps_best_of_three_equal_slope_lines() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 5), (1, 1), (1, 3)];
        let query_value = 0;
        let expected_values = (1, Some(1));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_keeps_best_of_three_equal_slope_lines() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 1), (1, 5), (1, 3)];
        let query_value = 0;
        let expected_values = (1, Some(5));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_ignores_inferior_equal_slope_line_added_after_best_line() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 1), (1, 5)];
        let query_value = 0;
        let expected_values = (1, Some(1));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_replaces_inferior_equal_slope_line_with_better_line() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 1), (1, 5)];
        let query_value = 0;
        let expected_values = (1, Some(5));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_ignores_inferior_equal_slope_line_added_after_best_line() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 5), (1, 1)];
        let query_value = 0;
        let expected_values = (1, Some(5));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let line_count = convex_hull.lines.len();
        let query_result = convex_hull.query(query_value);
        let actual_values = (line_count, query_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn min_cull_handles_full_i64_intercept_difference() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, i64::MIN), (2, 0), (1, i64::MAX)];
        let query_value = 0;
        let expected_value = Some(i64::MIN);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

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
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

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
        let lines = [(i64::MAX, 0), (0, 1), (i64::MIN, 0)];
        let query_value = 0;
        let expected_value = Some(0);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

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
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn min_cull_compares_products_beyond_i128_range_exactly() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [
            (i64::MAX, i64::MAX),
            (i64::MAX - 1, i64::MIN + 1),
            (i64::MIN, i64::MIN),
        ];
        let query_values = [-1, 0, 1, 2];
        let expected_values = naive_queries(&lines, &query_values, mode);
        let expected_line_count = 3;
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_line_count = convex_hull.lines.len();
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_cull_compares_products_beyond_i128_range_exactly() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [
            (i64::MIN, i64::MAX),
            (i64::MIN + 1, i64::MIN + 1),
            (i64::MAX, i64::MIN),
        ];
        let query_values = [-1, 0, 1, 2];
        let expected_values = naive_queries(&lines, &query_values, mode);
        let expected_line_count = 2;
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_line_count = convex_hull.lines.len();
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_line_count, expected_line_count);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_min_returns_expected_values_for_increasing_x() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(3, 0), (2, 1)];
        let query_values = [0, 2, 3];
        let expected_values = vec![Some(0), Some(5), Some(7)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for (slope, intercept) in lines { convex_hull.add_line(slope, intercept); }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_max_returns_expected_values_for_increasing_x() {
        // Arrange
        let mode = HullMode::Max;
        let lines = [(1, 1), (2, 0)];
        let query_values = [-1, 2];
        let expected_values = vec![Some(0), Some(4)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for (slope, intercept) in lines { convex_hull.add_line(slope, intercept); }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_returns_none_when_no_lines_added() {
        // Arrange
        let mode = HullMode::Min;
        let query_value = 0;
        let expected_value = None;
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_min_returns_expected_values_around_integer_intersection() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, 0), (1, 2)];
        let query_values = [1, 2, 3];
        let expected_values = vec![Some(2), Some(4), Some(5)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_after_empty_query_uses_subsequently_added_line() {
        // Arrange
        let mode = HullMode::Min;
        let empty_query_value = 0;
        let line = (-2, 4);
        let next_query_value = 3;
        let expected_values = (None, Some(-2));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        let empty_result = convex_hull.query(empty_query_value);
        convex_hull.add_line(line.0, line.1);
        let next_result = convex_hull.query(next_query_value);
        let actual_values = (empty_result, next_result);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn query_clamps_value_below_i64_min() {
        // Arrange
        let mode = HullMode::Min;
        let line = (i64::MAX, 0);
        let query_value = -2;
        let expected_value = Some(i64::MIN);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        convex_hull.add_line(line.0, line.1);
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn query_clamps_value_above_i64_max() {
        // Arrange
        let mode = HullMode::Min;
        let line = (i64::MAX, 0);
        let query_value = 2;
        let expected_value = Some(i64::MAX);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        convex_hull.add_line(line.0, line.1);
        let actual_value = convex_hull.query(query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn repeated_same_x_queries_keep_same_answer() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 0), (0, 2)];
        let query_value = 3;
        let expected_values = (Some(2), Some(2));
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = (
            convex_hull.query(query_value),
            convex_hull.query(query_value),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn single_line_returns_expected_values_for_increasing_queries() {
        // Arrange
        let mode = HullMode::Min;
        let line = (-2, 3);
        let query_values = [-1, 0, 2];
        let expected_values = vec![Some(5), Some(3), Some(-1)];
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        convex_hull.add_line(line.0, line.1);
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn monotonous_alias_supports_add_and_increasing_queries() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(2, 0), (1, 2)];
        let query_values = [0, 3];
        let expected_values = vec![Some(0), Some(5)];
        let mut convex_hull: ConvexHullTrickMonotonous = ConvexHullTrickMonotonous::new(mode);

        // Act
        for &(slope, intercept) in &lines {
            convex_hull.add_line(slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn fixed_seed_min_queries_match_naive_scan() {
        // Arrange
        let mode = HullMode::Min;
        let seed = 0xA5A5_0123_89AB_CDEF_u64;
        let line_count = 128;
        let query_count = 256;
        let initial_slope = 500_i64;
        let initial_query_value = -300_i64;
        let slope_step_modulus = 4_u64;
        let query_step_modulus = 6_u64;
        let intercept_modulus = 20_001_u64;
        let intercept_offset = 10_000_i64;
        let mut state = seed;
        let mut slope = initial_slope;
        let mut lines = Vec::with_capacity(line_count);
        for _ in 0..line_count {
            slope -= (next_random(&mut state) % slope_step_modulus) as i64;
            let intercept = (next_random(&mut state) % intercept_modulus) as i64 - intercept_offset;
            lines.push((slope, intercept));
        }
        let mut query_value = initial_query_value;
        let mut query_values = Vec::with_capacity(query_count);
        for _ in 0..query_count {
            query_value += (next_random(&mut state) % query_step_modulus) as i64;
            query_values.push(query_value);
        }
        let expected_values = naive_queries(&lines, &query_values, mode);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(line_slope, intercept) in &lines {
            convex_hull.add_line(line_slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn fixed_seed_max_queries_match_naive_scan() {
        // Arrange
        let mode = HullMode::Max;
        let seed = 0x5A5A_FEDC_7654_3210_u64;
        let line_count = 128;
        let query_count = 256;
        let initial_slope = -500_i64;
        let initial_query_value = -300_i64;
        let slope_step_modulus = 4_u64;
        let query_step_modulus = 6_u64;
        let intercept_modulus = 20_001_u64;
        let intercept_offset = 10_000_i64;
        let mut state = seed;
        let mut slope = initial_slope;
        let mut lines = Vec::with_capacity(line_count);
        for _ in 0..line_count {
            slope += (next_random(&mut state) % slope_step_modulus) as i64;
            let intercept = (next_random(&mut state) % intercept_modulus) as i64 - intercept_offset;
            lines.push((slope, intercept));
        }
        let mut query_value = initial_query_value;
        let mut query_values = Vec::with_capacity(query_count);
        for _ in 0..query_count {
            query_value += (next_random(&mut state) % query_step_modulus) as i64;
            query_values.push(query_value);
        }
        let expected_values = naive_queries(&lines, &query_values, mode);
        let mut convex_hull = ConvexHullTrickMonotonic::new(mode);

        // Act
        for &(line_slope, intercept) in &lines {
            convex_hull.add_line(line_slope, intercept);
        }
        let actual_values = collect_queries(&mut convex_hull, &query_values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
