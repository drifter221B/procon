// Each implementation includes the shared interface so it can also be compiled standalone.
#![allow(clippy::duplicate_mod)]

//! Convex Hull Trick (CHT)
//! Provides two variants:
//! - Monotonic queries: Min slopes are non-increasing, Max slopes are non-decreasing,
//!   and queries come in non-decreasing x; amortized O(1).
//! - Binary search: slopes follow the same mode-specific order and queries can be arbitrary;
//!   O(log N).

#[path = "convex_hull_interface.rs"]
mod convex_hull_interface;
#[path = "convex_hull_trick_binary.rs"]
mod convex_hull_trick_binary;
#[path = "convex_hull_trick_monotonic.rs"]
mod convex_hull_trick_monotonic;

pub use crate::convex_hull_interface::{ConvexHull, HullMode, Line};
pub use crate::convex_hull_trick_binary::ConvexHullTrickBinary;
pub use crate::convex_hull_trick_monotonic::{ConvexHullTrickMonotonic, ConvexHullTrickMonotonous};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryOrder {
    Monotonic,
    Arbitrary,
}

pub enum ConvexHullTrick {
    Monotonic(ConvexHullTrickMonotonic),
    Binary(ConvexHullTrickBinary),
}

impl ConvexHullTrick {
    pub fn new_monotonic(mode: HullMode) -> Self {
        Self::Monotonic(ConvexHullTrickMonotonic::new(mode))
    }

    pub fn new_binary(mode: HullMode) -> Self {
        Self::Binary(ConvexHullTrickBinary::new(mode))
    }

    pub fn new(mode: HullMode, order: QueryOrder) -> Self {
        match order {
            QueryOrder::Monotonic => Self::new_monotonic(mode),
            QueryOrder::Arbitrary => Self::new_binary(mode),
        }
    }

    pub fn add_line(&mut self, m: i64, c: i64) {
        match self {
            Self::Monotonic(inner) => inner.add_line(m, c),
            Self::Binary(inner) => inner.add_line(m, c),
        }
    }

    pub fn query(&mut self, x: i64) -> Option<i64> {
        match self {
            Self::Monotonic(inner) => inner.query(x),
            Self::Binary(inner) => inner.query(x),
        }
    }
}

impl ConvexHull for ConvexHullTrick {
    fn add_line(&mut self, m: i64, c: i64) {
        ConvexHullTrick::add_line(self, m, c);
    }

    fn query(&mut self, x: i64) -> Option<i64> {
        ConvexHullTrick::query(self, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_monotonic_wrapper_returns_none_before_adding_lines() {
        // Arrange
        let mode = HullMode::Min;
        let query_x = 0;
        let expected_value = None;
        let mut convex_hull = ConvexHullTrick::new_monotonic(mode);

        // Act
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn new_monotonic_with_min_mode_selects_monotonic_variant() {
        // Arrange
        let mode = HullMode::Min;
        let expected_is_monotonic = true;

        // Act
        let convex_hull = ConvexHullTrick::new_monotonic(mode);
        let actual_is_monotonic = matches!(convex_hull, ConvexHullTrick::Monotonic(_));

        // Assert
        assert_eq!(actual_is_monotonic, expected_is_monotonic);
    }

    #[test]
    fn new_monotonic_with_min_mode_returns_minima_for_increasing_queries() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_values = [0, 3];
        let expected_values = [Some(0), Some(7)];
        let mut convex_hull = ConvexHullTrick::new_monotonic(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_monotonic_with_max_mode_returns_maxima_for_increasing_queries() {
        // Arrange
        let mode = HullMode::Max;
        let first_slope = 1;
        let first_intercept = 4;
        let second_slope = 3;
        let second_intercept = 0;
        let query_values = [0, 3];
        let expected_values = [Some(4), Some(9)];
        let mut convex_hull = ConvexHullTrick::new_monotonic(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_binary_wrapper_returns_none_before_adding_lines() {
        // Arrange
        let mode = HullMode::Max;
        let query_x = 0;
        let expected_value = None;
        let mut convex_hull = ConvexHullTrick::new_binary(mode);

        // Act
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn new_binary_with_min_mode_selects_binary_variant() {
        // Arrange
        let mode = HullMode::Min;
        let expected_is_binary = true;

        // Act
        let convex_hull = ConvexHullTrick::new_binary(mode);
        let actual_is_binary = matches!(convex_hull, ConvexHullTrick::Binary(_));

        // Assert
        assert_eq!(actual_is_binary, expected_is_binary);
    }

    #[test]
    fn new_binary_with_min_mode_returns_minima_for_arbitrary_queries() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_values = [3, 0];
        let expected_values = [Some(7), Some(0)];
        let mut convex_hull = ConvexHullTrick::new_binary(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_binary_with_max_mode_returns_maxima_for_arbitrary_queries() {
        // Arrange
        let mode = HullMode::Max;
        let first_slope = 1;
        let first_intercept = 4;
        let second_slope = 3;
        let second_intercept = 0;
        let query_values = [3, 0];
        let expected_values = [Some(9), Some(4)];
        let mut convex_hull = ConvexHullTrick::new_binary(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_with_monotonic_order_selects_monotonic_variant() {
        // Arrange
        let mode = HullMode::Min;
        let order = QueryOrder::Monotonic;
        let expected_is_monotonic = true;

        // Act
        let convex_hull = ConvexHullTrick::new(mode, order);
        let actual_is_monotonic = matches!(convex_hull, ConvexHullTrick::Monotonic(_));

        // Assert
        assert_eq!(actual_is_monotonic, expected_is_monotonic);
    }

    #[test]
    fn new_with_arbitrary_order_selects_binary_variant() {
        // Arrange
        let mode = HullMode::Max;
        let order = QueryOrder::Arbitrary;
        let expected_is_binary = true;

        // Act
        let convex_hull = ConvexHullTrick::new(mode, order);
        let actual_is_binary = matches!(convex_hull, ConvexHullTrick::Binary(_));

        // Assert
        assert_eq!(actual_is_binary, expected_is_binary);
    }

    #[test]
    fn new_propagates_min_mode_to_monotonic_variant() {
        // Arrange
        let mode = HullMode::Min;
        let order = QueryOrder::Monotonic;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_values = [0, 3];
        let expected_values = [Some(0), Some(7)];
        let mut convex_hull = ConvexHullTrick::new(mode, order);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_propagates_max_mode_to_monotonic_variant() {
        // Arrange
        let mode = HullMode::Max;
        let order = QueryOrder::Monotonic;
        let first_slope = 1;
        let first_intercept = 4;
        let second_slope = 3;
        let second_intercept = 0;
        let query_values = [0, 3];
        let expected_values = [Some(4), Some(9)];
        let mut convex_hull = ConvexHullTrick::new(mode, order);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_propagates_min_mode_to_binary_variant() {
        // Arrange
        let mode = HullMode::Min;
        let order = QueryOrder::Arbitrary;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_values = [3, 0];
        let expected_values = [Some(7), Some(0)];
        let mut convex_hull = ConvexHullTrick::new(mode, order);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn new_propagates_max_mode_to_binary_variant() {
        // Arrange
        let mode = HullMode::Max;
        let order = QueryOrder::Arbitrary;
        let first_slope = 1;
        let first_intercept = 4;
        let second_slope = 3;
        let second_intercept = 0;
        let query_values = [3, 0];
        let expected_values = [Some(9), Some(4)];
        let mut convex_hull = ConvexHullTrick::new(mode, order);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn common_trait_add_and_query_returns_minimum_value() {
        // Arrange
        let mode = HullMode::Min;
        let order = QueryOrder::Arbitrary;
        let first_line = (2, 1);
        let second_line = (1, 3);
        let query_x = 2;
        let mut convex_hull = ConvexHullTrick::new(mode, order);
        let expected_value = Some(5);

        // Act
        ConvexHull::add_line(&mut convex_hull, first_line.0, first_line.1);
        ConvexHull::add_line(&mut convex_hull, second_line.0, second_line.1);
        let actual_value = ConvexHull::query(&mut convex_hull, query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn wrapper_delegates_add_and_query_to_monotonic_variant() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = -2;
        let second_slope = 2;
        let second_intercept = 0;
        let third_slope = 1;
        let third_intercept = 5;
        let query_values = [-3, 3, 10];
        let expected_values = [Some(-11), Some(6), Some(15)];
        let mut convex_hull = ConvexHullTrick::new_monotonic(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        convex_hull.add_line(third_slope, third_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
            convex_hull.query(query_values[2]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn wrapper_delegates_add_and_query_to_binary_variant() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = -2;
        let second_slope = 2;
        let second_intercept = 0;
        let third_slope = 1;
        let third_intercept = 5;
        let query_values = [10, 3, -3];
        let expected_values = [Some(15), Some(6), Some(-11)];
        let mut convex_hull = ConvexHullTrick::new_binary(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        convex_hull.add_line(third_slope, third_intercept);
        let actual_values = [
            convex_hull.query(query_values[0]),
            convex_hull.query(query_values[1]),
            convex_hull.query(query_values[2]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn trait_delegates_to_monotonic_variant_away_from_intersection() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_values = [0, 3];
        let expected_values = [Some(0), Some(7)];
        let mut convex_hull = ConvexHullTrick::new_monotonic(mode);

        // Act
        ConvexHull::add_line(&mut convex_hull, first_slope, first_intercept);
        ConvexHull::add_line(&mut convex_hull, second_slope, second_intercept);
        let actual_values = [
            ConvexHull::query(&mut convex_hull, query_values[0]),
            ConvexHull::query(&mut convex_hull, query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn trait_delegates_to_binary_variant_for_arbitrary_queries() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_values = [3, 0];
        let expected_values = [Some(7), Some(0)];
        let mut convex_hull = ConvexHullTrick::new_binary(mode);

        // Act
        ConvexHull::add_line(&mut convex_hull, first_slope, first_intercept);
        ConvexHull::add_line(&mut convex_hull, second_slope, second_intercept);
        let actual_values = [
            ConvexHull::query(&mut convex_hull, query_values[0]),
            ConvexHull::query(&mut convex_hull, query_values[1]),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reexported_monotonous_alias_supports_add_and_query() {
        // Arrange
        let mode = HullMode::Min;
        let first_slope = 3;
        let first_intercept = 0;
        let second_slope = 1;
        let second_intercept = 4;
        let query_x = 3;
        let expected_value = Some(7);
        let mut convex_hull = ConvexHullTrickMonotonous::new(mode);

        // Act
        convex_hull.add_line(first_slope, first_intercept);
        convex_hull.add_line(second_slope, second_intercept);
        let actual_value = convex_hull.query(query_x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
