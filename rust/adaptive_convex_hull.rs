//! Adaptive facade that selects the appropriate line container from input constraints.

#[path = "convex_hull_interface.rs"]
#[allow(dead_code)]
mod convex_hull_interface;
#[path = "convex_hull_trick_binary.rs"]
mod convex_hull_trick_binary;
#[path = "convex_hull_trick_monotonic.rs"]
mod convex_hull_trick_monotonic;
#[path = "li_chao_tree.rs"]
mod li_chao_tree;

pub use crate::convex_hull_interface::{ConvexHull, HullMode, Line};
pub use crate::convex_hull_trick_binary::ConvexHullTrickBinary;
pub use crate::convex_hull_trick_monotonic::{ConvexHullTrickMonotonic, ConvexHullTrickMonotonous};
pub use crate::li_chao_tree::LiChaoTree;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlopeOrder {
    Monotonic,
    Arbitrary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum QueryOrder {
    Monotonic,
    Arbitrary,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConvexHullStrategy {
    Monotonic,
    Binary,
    LiChaoTree,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConvexHullConditions {
    pub mode: HullMode,
    pub slope_order: SlopeOrder,
    pub query_order: QueryOrder,
}

impl ConvexHullConditions {
    pub fn new(mode: HullMode, slope_order: SlopeOrder, query_order: QueryOrder) -> Self {
        Self {
            mode,
            slope_order,
            query_order,
        }
    }

    pub fn strategy(&self) -> ConvexHullStrategy {
        match (self.slope_order, self.query_order) {
            (SlopeOrder::Monotonic, QueryOrder::Monotonic) => ConvexHullStrategy::Monotonic,
            (SlopeOrder::Monotonic, QueryOrder::Arbitrary) => ConvexHullStrategy::Binary,
            (SlopeOrder::Arbitrary, _) => ConvexHullStrategy::LiChaoTree,
        }
    }
}

pub enum AdaptiveConvexHull {
    Monotonic(ConvexHullTrickMonotonic),
    Binary(ConvexHullTrickBinary),
    LiChaoTree(LiChaoTree),
}

impl AdaptiveConvexHull {
    pub fn new(conditions: ConvexHullConditions) -> Self {
        match conditions.strategy() {
            ConvexHullStrategy::Monotonic => {
                Self::Monotonic(ConvexHullTrickMonotonic::new(conditions.mode))
            }
            ConvexHullStrategy::Binary => Self::Binary(ConvexHullTrickBinary::new(conditions.mode)),
            ConvexHullStrategy::LiChaoTree => Self::LiChaoTree(LiChaoTree::new(conditions.mode)),
        }
    }

    pub fn strategy(&self) -> ConvexHullStrategy {
        match self {
            Self::Monotonic(_) => ConvexHullStrategy::Monotonic,
            Self::Binary(_) => ConvexHullStrategy::Binary,
            Self::LiChaoTree(_) => ConvexHullStrategy::LiChaoTree,
        }
    }

    pub fn add_line(&mut self, m: i64, c: i64) {
        match self {
            Self::Monotonic(inner) => inner.add_line(m, c),
            Self::Binary(inner) => inner.add_line(m, c),
            Self::LiChaoTree(inner) => inner.add_line(m, c),
        }
    }

    pub fn query(&mut self, x: i64) -> Option<i64> {
        match self {
            Self::Monotonic(inner) => inner.query(x),
            Self::Binary(inner) => inner.query(x),
            Self::LiChaoTree(inner) => inner.query(x),
        }
    }
}

impl ConvexHull for AdaptiveConvexHull {
    fn add_line(&mut self, m: i64, c: i64) {
        AdaptiveConvexHull::add_line(self, m, c);
    }

    fn query(&mut self, x: i64) -> Option<i64> {
        AdaptiveConvexHull::query(self, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_uses_li_chao_tree_for_arbitrary_slope_and_arbitrary_query() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Arbitrary, QueryOrder::Arbitrary);
        let expected_strategy = ConvexHullStrategy::LiChaoTree;

        // Act
        let adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let actual_strategy = adaptive_convex_hull.strategy();

        // Assert
        assert_eq!(actual_strategy, expected_strategy);
    }

    #[test]
    fn conditions_select_monotonic_strategy_for_monotonic_slope_and_query() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Monotonic);
        let expected_strategy = ConvexHullStrategy::Monotonic;

        // Act
        let actual_strategy = conditions.strategy();

        // Assert
        assert_eq!(actual_strategy, expected_strategy);
    }

    #[test]
    fn conditions_select_binary_strategy_for_monotonic_slope_and_arbitrary_query() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Arbitrary);
        let expected_strategy = ConvexHullStrategy::Binary;

        // Act
        let actual_strategy = conditions.strategy();

        // Assert
        assert_eq!(actual_strategy, expected_strategy);
    }

    #[test]
    fn conditions_select_li_chao_tree_strategy_for_arbitrary_slope_and_monotonic_query() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Max, SlopeOrder::Arbitrary, QueryOrder::Monotonic);
        let expected_strategy = ConvexHullStrategy::LiChaoTree;

        // Act
        let actual_strategy = conditions.strategy();

        // Assert
        assert_eq!(actual_strategy, expected_strategy);
    }

    #[test]
    fn query_returns_none_without_lines_for_all_strategies() {
        // Arrange
        let conditions_list = vec![
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Monotonic),
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Arbitrary),
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Arbitrary, QueryOrder::Arbitrary),
        ];
        let expected_values = vec![None, None, None];

        let query_x = 0;

        // Act
        let actual_values = conditions_list
            .into_iter()
            .map(AdaptiveConvexHull::new)
            .map(|mut convex_hull| convex_hull.query(query_x))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn adaptive_strategy_matches_conditions_strategy_for_all_strategy_cases() {
        // Arrange
        let conditions_list = vec![
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Monotonic),
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Arbitrary),
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Arbitrary, QueryOrder::Monotonic),
        ];
        let expected_strategies = conditions_list
            .iter()
            .map(ConvexHullConditions::strategy)
            .collect::<Vec<_>>();

        // Act
        let actual_strategies = conditions_list
            .into_iter()
            .map(AdaptiveConvexHull::new)
            .map(|convex_hull| convex_hull.strategy())
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_strategies, expected_strategies);
    }

    #[test]
    fn common_trait_uses_monotonic_variant_when_constraints_are_monotonic() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Monotonic);
        let mut adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let x_value = 3i64;
        let expected_value = Some(7);

        let lines = [(3, 0), (2, 1)];

        // Act
        for (slope, intercept) in lines {
            ConvexHull::add_line(&mut adaptive_convex_hull, slope, intercept);
        }
        let actual_value = ConvexHull::query(&mut adaptive_convex_hull, x_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn common_trait_uses_binary_variant_when_query_order_is_arbitrary() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Monotonic, QueryOrder::Arbitrary);
        let mut adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let x_value = -3i64;
        let expected_value = Some(-11);

        let lines = [(3, -2), (2, 0), (1, 5)];

        // Act
        for (slope, intercept) in lines {
            ConvexHull::add_line(&mut adaptive_convex_hull, slope, intercept);
        }
        let actual_value = ConvexHull::query(&mut adaptive_convex_hull, x_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn common_trait_uses_li_chao_tree_variant_when_slope_order_is_arbitrary() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Min, SlopeOrder::Arbitrary, QueryOrder::Arbitrary);
        let mut adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let x_value = 2i64;
        let expected_value = Some(-1);

        let lines = [(3, 5), (-1, 1)];

        // Act
        for (slope, intercept) in lines {
            ConvexHull::add_line(&mut adaptive_convex_hull, slope, intercept);
        }
        let actual_value = ConvexHull::query(&mut adaptive_convex_hull, x_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_mode_monotonic_strategy_returns_maximum_value() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Max, SlopeOrder::Monotonic, QueryOrder::Monotonic);
        let mut adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let expected_values = vec![Some(0), Some(4)];

        let add_line_slope = 1;
        let add_line_intercept = 1;
        let add_line_slope_2 = 2;
        let add_line_intercept_2 = 0;
        let query_x = -1;
        let query_x_2 = 2;

        // Act
        adaptive_convex_hull.add_line(add_line_slope, add_line_intercept);
        adaptive_convex_hull.add_line(add_line_slope_2, add_line_intercept_2);
        let actual_values = vec![
            adaptive_convex_hull.query(query_x),
            adaptive_convex_hull.query(query_x_2),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_mode_binary_strategy_returns_maximum_value_for_arbitrary_queries() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Max, SlopeOrder::Monotonic, QueryOrder::Arbitrary);
        let mut adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let expected_values = vec![Some(5), Some(8)];

        let add_line_slope = 1;
        let add_line_intercept = 5;
        let add_line_slope_2 = 3;
        let add_line_intercept_2 = -2;
        let query_x = 0;
        let query_x_2 = 3;

        // Act
        adaptive_convex_hull.add_line(add_line_slope, add_line_intercept);
        adaptive_convex_hull.add_line(add_line_slope_2, add_line_intercept_2);
        let actual_values = vec![
            adaptive_convex_hull.query(query_x),
            adaptive_convex_hull.query(query_x_2),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn max_mode_li_chao_tree_strategy_returns_maximum_value_for_arbitrary_slopes() {
        // Arrange
        let conditions =
            ConvexHullConditions::new(HullMode::Max, SlopeOrder::Arbitrary, QueryOrder::Arbitrary);
        let mut adaptive_convex_hull = AdaptiveConvexHull::new(conditions);
        let expected_values = vec![Some(5), Some(4)];

        let add_line_slope = 2;
        let add_line_intercept = 1;
        let add_line_slope_2 = -1;
        let add_line_intercept_2 = 5;
        let query_x = 0;
        let query_x_2 = 1;

        // Act
        adaptive_convex_hull.add_line(add_line_slope, add_line_intercept);
        adaptive_convex_hull.add_line(add_line_slope_2, add_line_intercept_2);
        let actual_values = vec![
            adaptive_convex_hull.query(query_x),
            adaptive_convex_hull.query(query_x_2),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
