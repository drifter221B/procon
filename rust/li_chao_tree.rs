//! Li Chao Tree
//! Maintains a dynamic set of lines `y = m x + c` and answers
//! queries in `O(log C)` where `C` is the coordinate range of `x`; `[-INF, INF]`

#[path = "convex_hull_interface.rs"]
#[allow(dead_code)]
mod convex_hull_interface;

use crate::convex_hull_interface::{clamp_i128_to_i64, ConvexHull, HullMode, Line};

#[derive(Debug)]
struct Node {
    line: Line,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(line: Line) -> Self {
        Node {
            line,
            left: None,
            right: None,
        }
    }
}

/// Li Chao Tree.
/// x; `[-INF, INF]`
pub struct LiChaoTree {
    mode: HullMode,
    root: Option<Box<Node>>,
    inf: i64,
}

impl LiChaoTree {
    pub fn new(mode: HullMode) -> Self {
        LiChaoTree {
            mode,
            root: None,
            inf: 1_i64 << 60,
        }
    }

    pub fn add_line(&mut self, m: i64, c: i64) {
        let line = Line::new(m, c);

        let mode = self.mode;
        let inf = self.inf;

        Self::insert_internal(mode, &mut self.root, -inf, inf + 1, line);
    }

    /// Returns `None` for an empty tree or a coordinate outside `[-INF, INF]`.
    pub fn query(&self, x: i64) -> Option<i64> {
        if x < -self.inf || self.inf < x || self.root.is_none() {
            return None;
        }

        let inf = self.inf;

        self.query_internal(&self.root, -inf, inf + 1, x)
            .map(clamp_i128_to_i64)
    }

    fn insert_internal(
        mode: HullMode,
        node: &mut Option<Box<Node>>,
        left: i64,
        right: i64,
        mut line: Line,
    ) {
        if node.is_none() {
            *node = Some(Box::new(Node::new(line)));
            return;
        }

        let n = node.as_mut().unwrap();
        let mid = left + (right - left) / 2;

        let left_better = Self::is_better(mode, line.eval128(left), n.line.eval128(left));
        let mid_better = Self::is_better(mode, line.eval128(mid), n.line.eval128(mid));

        if mid_better {
            std::mem::swap(&mut n.line, &mut line);
        }

        if right - left == 1 {
            return;
        }

        if left_better != mid_better {
            Self::insert_internal(mode, &mut n.left, left, mid, line);
        } else {
            Self::insert_internal(mode, &mut n.right, mid, right, line);
        }
    }

    fn query_internal(
        &self,
        node: &Option<Box<Node>>,
        left: i64,
        right: i64,
        x: i64,
    ) -> Option<i128> {
        if node.is_none() {
            return None;
        }

        let n = node.as_ref().unwrap();
        let result = n.line.eval128(x);

        if right - left == 1 {
            return Some(result);
        }

        let mid = left + (right - left) / 2;
        let child_result = if x < mid {
            self.query_internal(&n.left, left, mid, x)
        } else {
            self.query_internal(&n.right, mid, right, x)
        };

        let candidate = match child_result {
            Some(child_value) => match self.mode {
                HullMode::Min => std::cmp::min(result, child_value),
                HullMode::Max => std::cmp::max(result, child_value),
            },
            None => result,
        };

        Some(candidate)
    }

    #[inline]
    fn is_better(mode: HullMode, a: i128, b: i128) -> bool {
        match mode {
            HullMode::Min => a < b,
            HullMode::Max => a > b,
        }
    }
}

impl ConvexHull for LiChaoTree {
    fn add_line(&mut self, m: i64, c: i64) {
        LiChaoTree::add_line(self, m, c);
    }

    fn query(&mut self, x: i64) -> Option<i64> {
        LiChaoTree::query(self, x)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build_tree(mode: HullMode, lines: &[(i64, i64)]) -> LiChaoTree {
        let mut tree = LiChaoTree::new(mode);
        for &(m, c) in lines {
            tree.add_line(m, c);
        }
        tree
    }

    fn naive_query(mode: HullMode, lines: &[(i64, i64)], x: i64) -> i64 {
        let mut best = match mode {
            HullMode::Min => i128::MAX,
            HullMode::Max => i128::MIN,
        };

        for &(m, c) in lines {
            let value = Line::new(m, c).eval128(x);
            best = match mode {
                HullMode::Min => best.min(value),
                HullMode::Max => best.max(value),
            };
        }

        best as i64
    }

    fn next_lcg(seed: u64) -> u64 {
        seed.wrapping_mul(6364136223846793005).wrapping_add(1)
    }

    #[test]
    fn line_new_returns_slope_value() {
        // Arrange
        let slope_value = -7;
        let intercept_value = 11;
        let line = Line::new(slope_value, intercept_value);
        let expected = slope_value;

        // Act
        let actual = line.slope();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn line_new_returns_intercept_value() {
        // Arrange
        let slope_value = 3;
        let intercept_value = -9;
        let line = Line::new(slope_value, intercept_value);
        let expected = intercept_value;

        // Act
        let actual = line.intercept();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn eval_returns_mx_plus_c_in_normal_range() {
        // Arrange
        let line = Line::new(7, -5);
        let x_value = 4;
        let expected = 23;

        // Act
        let actual = line.eval(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn eval_saturates_to_i64_max_on_positive_overflow() {
        // Arrange
        let line = Line::new(i64::MAX, 1);
        let x_value = 2;
        let expected = i64::MAX;

        // Act
        let actual = line.eval(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn eval_saturates_to_i64_min_on_negative_overflow() {
        // Arrange
        let line = Line::new(i64::MIN, -1);
        let x_value = 2;
        let expected = i64::MIN;

        // Act
        let actual = line.eval(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn eval128_returns_exact_value_for_large_input() {
        // Arrange
        let line = Line::new(i64::MAX, i64::MAX);
        let x_value = 2;
        let expected = (i64::MAX as i128) * 2 + i64::MAX as i128;

        // Act
        let actual = line.eval128(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn min_mode_is_unchanged_when_adding_identical_lines() {
        // Arrange
        let single_line_tree = build_tree(HullMode::Min, &[(3, 5)]);
        let duplicate_line_tree = build_tree(HullMode::Min, &[(3, 5), (3, 5), (3, 5)]);
        let x_value = 7;

        // Act
        let actual_single = single_line_tree.query(x_value);
        let actual_duplicate = duplicate_line_tree.query(x_value);

        // Assert
        assert_eq!(actual_single, actual_duplicate);
    }

    #[test]
    fn min_mode_prefers_smaller_intercept_when_slopes_are_equal() {
        // Arrange
        let tree = build_tree(HullMode::Min, &[(4, 9), (4, -2)]);
        let x_values = [-1, 0, 1];
        let expected0 = 4 * x_values[0] - 2;
        let expected1 = 4 * x_values[1] - 2;
        let expected2 = 4 * x_values[2] - 2;

        // Act
        let actual0 = tree.query(x_values[0]);
        let actual1 = tree.query(x_values[1]);
        let actual2 = tree.query(x_values[2]);

        // Assert
        assert_eq!(actual0, Some(expected0));
        assert_eq!(actual1, Some(expected1));
        assert_eq!(actual2, Some(expected2));
    }

    #[test]
    fn min_mode_is_order_independent_for_insertions() {
        // Arrange
        let lines_forward = [(2, 3), (-1, 8), (0, -4)];
        let lines_reverse = [(0, -4), (-1, 8), (2, 3)];
        let tree_forward = build_tree(HullMode::Min, &lines_forward);
        let tree_reverse = build_tree(HullMode::Min, &lines_reverse);
        let x_value = 5;

        // Act
        let actual_forward = tree_forward.query(x_value);
        let actual_reverse = tree_reverse.query(x_value);

        // Assert
        assert_eq!(actual_forward, actual_reverse);
    }

    #[test]
    fn max_mode_is_unchanged_when_adding_identical_lines() {
        // Arrange
        let single_line_tree = build_tree(HullMode::Max, &[(3, 5)]);
        let duplicate_line_tree = build_tree(HullMode::Max, &[(3, 5), (3, 5), (3, 5)]);
        let x_value = -4;

        // Act
        let actual_single = single_line_tree.query(x_value);
        let actual_duplicate = duplicate_line_tree.query(x_value);

        // Assert
        assert_eq!(actual_single, actual_duplicate);
    }

    #[test]
    fn max_mode_prefers_larger_intercept_when_slopes_are_equal() {
        // Arrange
        let tree = build_tree(HullMode::Max, &[(4, 9), (4, -2)]);
        let x_values = [-1, 0, 1];
        let expected0 = 4 * x_values[0] + 9;
        let expected1 = 4 * x_values[1] + 9;
        let expected2 = 4 * x_values[2] + 9;

        // Act
        let actual0 = tree.query(x_values[0]);
        let actual1 = tree.query(x_values[1]);
        let actual2 = tree.query(x_values[2]);

        // Assert
        assert_eq!(actual0, Some(expected0));
        assert_eq!(actual1, Some(expected1));
        assert_eq!(actual2, Some(expected2));
    }

    #[test]
    fn max_mode_is_order_independent_for_insertions() {
        // Arrange
        let lines_forward = [(2, 3), (-1, 8), (0, -4)];
        let lines_reverse = [(0, -4), (-1, 8), (2, 3)];
        let tree_forward = build_tree(HullMode::Max, &lines_forward);
        let tree_reverse = build_tree(HullMode::Max, &lines_reverse);
        let x_value = 5;

        // Act
        let actual_forward = tree_forward.query(x_value);
        let actual_reverse = tree_reverse.query(x_value);

        // Assert
        assert_eq!(actual_forward, actual_reverse);
    }

    #[test]
    fn query_returns_none_for_empty_tree() {
        // Arrange
        let tree = LiChaoTree::new(HullMode::Min);
        let x_value = 0;
        let expected: Option<i64> = None;

        // Act
        let actual = tree.query(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_on_single_line_tree_returns_line_value() {
        // Arrange
        let line_m = -2;
        let line_c = 7;
        let tree = build_tree(HullMode::Min, &[(line_m, line_c)]);
        let x_values = [-3, 0, 4];
        let expected0 = line_m * x_values[0] + line_c;
        let expected1 = line_m * x_values[1] + line_c;
        let expected2 = line_m * x_values[2] + line_c;

        // Act
        let actual0 = tree.query(x_values[0]);
        let actual1 = tree.query(x_values[1]);
        let actual2 = tree.query(x_values[2]);

        // Assert
        assert_eq!(actual0, Some(expected0));
        assert_eq!(actual1, Some(expected1));
        assert_eq!(actual2, Some(expected2));
    }

    #[test]
    fn query_handles_negative_inf_boundary() {
        // Arrange
        let inf = 1_i64 << 60;
        let lines = [(2, 3), (-1, 10)];
        let tree = build_tree(HullMode::Min, &lines);
        let x_value = -inf;
        let expected = naive_query(HullMode::Min, &lines, x_value);

        // Act
        let actual = tree.query(x_value);

        // Assert
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn query_handles_inf_minus_one_boundary() {
        // Arrange
        let inf = 1_i64 << 60;
        let lines = [(2, 3), (-1, 10)];
        let tree = build_tree(HullMode::Max, &lines);
        let x_value = inf - 1;
        let expected = naive_query(HullMode::Max, &lines, x_value);

        // Act
        let actual = tree.query(x_value);

        // Assert
        assert_eq!(actual, Some(expected));
    }

    #[test]
    fn query_is_correct_at_minus_one_zero_one_boundary() {
        // Arrange
        let lines = [(2, 1), (-3, 4), (0, -2)];
        let tree = build_tree(HullMode::Min, &lines);
        let x_minus = -1;
        let x_zero = 0;
        let x_plus = 1;
        let expected_minus = naive_query(HullMode::Min, &lines, x_minus);
        let expected_zero = naive_query(HullMode::Min, &lines, x_zero);
        let expected_plus = naive_query(HullMode::Min, &lines, x_plus);

        // Act
        let actual_minus = tree.query(x_minus);
        let actual_zero = tree.query(x_zero);
        let actual_plus = tree.query(x_plus);

        // Assert
        assert_eq!(actual_minus, Some(expected_minus));
        assert_eq!(actual_zero, Some(expected_zero));
        assert_eq!(actual_plus, Some(expected_plus));
    }

    #[test]
    fn min_mode_switches_best_line_around_intersection() {
        // Arrange
        let lines = [(2, 0), (-2, 8)];
        let tree = build_tree(HullMode::Min, &lines);
        let cross_x = 2;
        let x_left = cross_x - 1;
        let x_cross = cross_x;
        let x_right = cross_x + 1;
        let expected_left = naive_query(HullMode::Min, &lines, x_left);
        let expected_cross = naive_query(HullMode::Min, &lines, x_cross);
        let expected_right = naive_query(HullMode::Min, &lines, x_right);

        // Act
        let actual_left = tree.query(x_left);
        let actual_cross = tree.query(x_cross);
        let actual_right = tree.query(x_right);

        // Assert
        assert_eq!(actual_left, Some(expected_left));
        assert_eq!(actual_cross, Some(expected_cross));
        assert_eq!(actual_right, Some(expected_right));
    }

    #[test]
    fn min_mode_matches_naive_with_multiple_lines() {
        // Arrange
        let lines = [(3, 1), (-2, 4), (1, -5), (0, 7)];
        let tree = build_tree(HullMode::Min, &lines);
        let x_values = [-4, 0, 6];
        let expected0 = naive_query(HullMode::Min, &lines, x_values[0]);
        let expected1 = naive_query(HullMode::Min, &lines, x_values[1]);
        let expected2 = naive_query(HullMode::Min, &lines, x_values[2]);

        // Act
        let actual0 = tree.query(x_values[0]);
        let actual1 = tree.query(x_values[1]);
        let actual2 = tree.query(x_values[2]);

        // Assert
        assert_eq!(actual0, Some(expected0));
        assert_eq!(actual1, Some(expected1));
        assert_eq!(actual2, Some(expected2));
    }

    #[test]
    fn max_mode_switches_best_line_around_intersection() {
        // Arrange
        let lines = [(2, 0), (-2, 8)];
        let tree = build_tree(HullMode::Max, &lines);
        let cross_x = 2;
        let x_left = cross_x - 1;
        let x_cross = cross_x;
        let x_right = cross_x + 1;
        let expected_left = naive_query(HullMode::Max, &lines, x_left);
        let expected_cross = naive_query(HullMode::Max, &lines, x_cross);
        let expected_right = naive_query(HullMode::Max, &lines, x_right);

        // Act
        let actual_left = tree.query(x_left);
        let actual_cross = tree.query(x_cross);
        let actual_right = tree.query(x_right);

        // Assert
        assert_eq!(actual_left, Some(expected_left));
        assert_eq!(actual_cross, Some(expected_cross));
        assert_eq!(actual_right, Some(expected_right));
    }

    #[test]
    fn max_mode_matches_naive_with_multiple_lines() {
        // Arrange
        let lines = [(3, 1), (-2, 4), (1, -5), (0, 7)];
        let tree = build_tree(HullMode::Max, &lines);
        let x_values = [-4, 0, 6];
        let expected0 = naive_query(HullMode::Max, &lines, x_values[0]);
        let expected1 = naive_query(HullMode::Max, &lines, x_values[1]);
        let expected2 = naive_query(HullMode::Max, &lines, x_values[2]);

        // Act
        let actual0 = tree.query(x_values[0]);
        let actual1 = tree.query(x_values[1]);
        let actual2 = tree.query(x_values[2]);

        // Assert
        assert_eq!(actual0, Some(expected0));
        assert_eq!(actual1, Some(expected1));
        assert_eq!(actual2, Some(expected2));
    }

    #[test]
    fn large_values_remain_correct_with_eval128_comparison() {
        // Arrange
        let lines = [
            (1_000_000_000, 1_000_000_000),
            (-1_000_000_000, 2_000_000_000),
            (500_000_000, -3_000_000_000),
        ];
        let tree = build_tree(HullMode::Min, &lines);
        let x_values = [-1_000_000_000, 0, 1_000_000_000];
        let expected0 = naive_query(HullMode::Min, &lines, x_values[0]);
        let expected1 = naive_query(HullMode::Min, &lines, x_values[1]);
        let expected2 = naive_query(HullMode::Min, &lines, x_values[2]);

        // Act
        let actual0 = tree.query(x_values[0]);
        let actual1 = tree.query(x_values[1]);
        let actual2 = tree.query(x_values[2]);

        // Assert
        assert_eq!(actual0, Some(expected0));
        assert_eq!(actual1, Some(expected1));
        assert_eq!(actual2, Some(expected2));
    }

    #[test]
    fn query_terminates_with_frequently_switching_lines_near_boundaries() {
        // Arrange
        let inf = 1_i64 << 60;
        let lines = [
            (1, 0),
            (-1, 0),
            (2, 100),
            (-2, -100),
            (3, 7),
            (-3, -7),
            (4, 1),
            (-4, -1),
        ];
        let tree = build_tree(HullMode::Max, &lines);
        let x_left = -inf;
        let x_mid = 0;
        let x_right = inf - 1;

        // Act
        let actual_left = tree.query(x_left);
        let actual_mid = tree.query(x_mid);
        let actual_right = tree.query(x_right);

        // Assert
        assert!(actual_left.is_some());
        assert!(actual_mid.is_some());
        assert!(actual_right.is_some());
    }

    #[test]
    fn min_query_returns_large_line_value_even_when_empty_child_exists() {
        // Arrange
        let mut tree = LiChaoTree::new(HullMode::Min);
        let slope_value = 0i64;
        let intercept_value = i64::MAX;
        let x_value = 0i64;
        let expected = Some(i64::MAX);

        // Act
        tree.add_line(slope_value, intercept_value);
        let actual = tree.query(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn max_query_returns_small_line_value_even_when_empty_child_exists() {
        // Arrange
        let mut tree = LiChaoTree::new(HullMode::Max);
        let slope_value = 0i64;
        let intercept_value = i64::MIN;
        let x_value = 0i64;
        let expected = Some(i64::MIN);

        // Act
        tree.add_line(slope_value, intercept_value);
        let actual = tree.query(x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn random_cases_match_naive_implementation() {
        // Arrange
        let mut seed = 1_u64;
        let line_count = 40;
        let query_count = 60;
        let mut lines: Vec<(i64, i64)> = Vec::with_capacity(line_count);
        for _ in 0..line_count {
            seed = next_lcg(seed);
            let slope = (seed % 2001) as i64 - 1000;
            seed = next_lcg(seed);
            let intercept = (seed % 2001) as i64 - 1000;
            lines.push((slope, intercept));
        }
        let mut query_cases = Vec::with_capacity(query_count);
        for _ in 0..query_count {
            seed = next_lcg(seed);
            let x_value = (seed % 4001) as i64 - 2000;
            let expected = naive_query(HullMode::Min, &lines, x_value);
            query_cases.push((x_value, Some(expected)));
        }
        let tree = build_tree(HullMode::Min, &lines);
        let expected_values: Vec<Option<i64>> =
            query_cases.iter().map(|(_, expected)| *expected).collect();

        // Act
        let actual_values: Vec<Option<i64>> = query_cases
            .iter()
            .map(|(x_value, _)| tree.query(*x_value))
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn common_trait_accepts_li_chao_tree_impl() {
        // Arrange
        let mode = HullMode::Min;
        let lines = [(1, 0), (-1, 3)];
        let x_value = 2i64;
        let expected = Some(1);
        let mut tree = LiChaoTree::new(mode);

        // Act
        for (slope, intercept) in lines {
            ConvexHull::add_line(&mut tree, slope, intercept);
        }
        let actual = ConvexHull::query(&mut tree, x_value);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_at_inclusive_domain_boundaries_matches_naive() {
        // Arrange
        let inf = 1_i64 << 60;
        let modes = [HullMode::Min, HullMode::Max];
        // Fill the rightmost path so a line that first wins at INF reaches a leaf.
        let duplicate_line_count = 62;
        let mut lines = vec![(0, 0); duplicate_line_count];
        lines.extend([(-1, inf - 1), (1, inf - 1)]);
        let coordinates = [-inf, -inf + 1, inf - 1, inf];
        let cases: Vec<_> = modes
            .into_iter()
            .map(|mode| {
                let expected: Vec<_> = coordinates
                    .iter()
                    .map(|&x| Some(naive_query(mode, &lines, x)))
                    .collect();
                (build_tree(mode, &lines), expected)
            })
            .collect();
        let expected: Vec<_> = cases.iter().map(|(_, expected)| expected.clone()).collect();

        // Act
        let actual: Vec<Vec<_>> = cases
            .iter()
            .map(|(tree, _)| coordinates.iter().map(|&x| tree.query(x)).collect())
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_just_outside_domain_returns_none() {
        // Arrange
        let inf = 1_i64 << 60;
        let coordinates = [-inf - 1, inf + 1];
        let lines = [(1, 0), (-1, 1)];
        let trees = [
            build_tree(HullMode::Min, &lines),
            build_tree(HullMode::Max, &lines),
        ];
        let expected = vec![vec![None; coordinates.len()]; trees.len()];

        // Act
        let actual: Vec<Vec<_>> = trees
            .iter()
            .map(|tree| coordinates.iter().map(|&x| tree.query(x)).collect())
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }
}
