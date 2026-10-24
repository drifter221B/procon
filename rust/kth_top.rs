use std::collections::{BTreeMap, VecDeque};

pub struct FenwickTree {
    size: usize,
    tree: Vec<i64>,
    bit_ceil: u32,
}

impl FenwickTree {
    pub fn new(n: usize) -> Self {
        Self {
            size: n,
            tree: vec![0; n],
            bit_ceil: (n + 1).next_power_of_two().trailing_zeros(),
        }
    }

    #[inline]
    fn lsb(index: usize) -> usize {
        index & index.wrapping_neg()
    }

    pub fn from_slice(values: &[i64]) -> Self {
        let mut tree = Self::new(values.len());
        tree.tree.copy_from_slice(values);

        for i in 1..=tree.size {
            if i + (Self::lsb(i)) <= tree.size {
                let target = i + Self::lsb(i) - 1;
                let val = tree.tree[i - 1];
                tree.tree[target] += val;
            }
        }

        tree
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn add(&mut self, index: usize, value: i64) {
        if index < self.size {
            let mut idx = index + 1;

            while idx <= self.size {
                self.tree[idx - 1] += value;
                idx += Self::lsb(idx);
            }
        }
    }

    fn subtract(&mut self, index: usize, value: i64) {
        if index < self.size {
            let mut idx = index + 1;

            while idx <= self.size {
                self.tree[idx - 1] -= value;
                idx += Self::lsb(idx);
            }
        }
    }

    #[inline]
    pub fn update(&mut self, index: usize, value: i64) {
        if let Some(current_value) = self.select(index) {
            self.subtract(index, current_value);
            self.add(index, value);
        }
    }

    fn _sum(&self, tree: &[i64], r: usize) -> i64 {
        let mut s = 0;
        let mut r = r;

        while r > 0 {
            s += tree[r - 1];
            r -= Self::lsb(r);
        }

        s
    }

    pub fn sum(&self, left_index: usize, right_index: usize) -> Option<i64> {
        if left_index <= right_index && right_index <= self.size {
            Some(self._sum(&self.tree, right_index) - self._sum(&self.tree, left_index))
        } else {
            None
        }
    }

    #[inline]
    pub fn select(&self, index: usize) -> Option<i64> {
        self.sum(index, index + 1)
    }

    #[inline]
    pub fn select_all(&self) -> Option<i64> {
        self.sum(0, self.size)
    }

    pub fn lower_bound(&self, value: i64) -> usize {
        let mut value = value;
        if value <= 0 {
            return 0;
        }

        let mut power = 1 << self.bit_ceil;
        let mut right_index = 0;

        while 0 < power {
            if power + right_index - 1 < self.size && self.tree[power + right_index - 1] < value {
                right_index += power;
                value -= self.tree[right_index - 1];
            }

            power >>= 1;
        }

        right_index
    }

    pub fn upper_bound(&self, value: i64) -> usize {
        let mut value = value;
        let mut power = 1 << self.bit_ceil;
        let mut right_index = 0;

        while 0 < power {
            if power + right_index - 1 < self.size && self.tree[power + right_index - 1] <= value {
                right_index += power;
                value -= self.tree[right_index - 1];
            }

            power >>= 1;
        }

        right_index
    }
}

struct KthTop {
    k: usize,
    sums: FenwickTree,
    counters: FenwickTree,
    mp: BTreeMap<i64, usize>,
    que: VecDeque<(usize, i64)>,
}

impl KthTop {
    pub fn new(k: usize, nums: &[i64]) -> Self {
        let mut a = nums.to_vec();
        a.sort_unstable();
        a.dedup();

        let mut mp = BTreeMap::new();
        for (i, a) in a.into_iter().enumerate() {
            mp.insert(a, i);
        }

        let m = mp.len();

        Self {
            k,
            sums: FenwickTree::new(m),
            counters: FenwickTree::new(m),
            mp,
            que: VecDeque::with_capacity(k + 1),
        }
    }

    pub fn push(&mut self, x: i64) -> bool {
        let Some(&index) = self.mp.get(&x) else {
            return false;
        };

        if self.k == 0 {
            return true;
        }

        if self.que.len() == self.k {
            let (old_index, old_value) = self.que.pop_front().unwrap();
            self._remove(old_index, old_value);
        }

        self.que.push_back((index, x));
        self._push(index, x);
        true
    }

    fn _push(&mut self, index: usize, x: i64) {
        self.sums.add(index, x);
        self.counters.add(index, 1);
    }

    fn _remove(&mut self, index: usize, x: i64) {
        self.sums.subtract(index, x);
        self.counters.add(index, -1);
    }

    pub fn medians(&self) -> (Option<i64>, Option<i64>) {
        (
            self.nth(self.k as i64 / 2),
            if self.k & 1 == 1 {
                None
            } else {
                self.nth(self.k as i64 / 2 - 1)
            },
        )
    }

    // 0 - indexed
    pub fn nth(&self, n: i64) -> Option<i64> {
        if n < 0 || self.counters.sum(0, self.counters.len()).unwrap() <= n {
            None
        } else {
            let index = self.counters.lower_bound(n + 1);

            Some(self.sums.select(index).unwrap() / self.counters.select(index).unwrap())
        }
    }

    pub fn less_than_x_sum(&self, x: i64) -> i64 {
        let (_, &index) = self
            .mp
            .range((std::ops::Bound::Included(x), std::ops::Bound::Unbounded))
            .next()
            .unwrap_or((&0, &self.counters.len()));

        self.sums.sum(0, index).unwrap()
    }

    pub fn greater_than_x_sum(&self, x: i64) -> i64 {
        let (_, &index) = self
            .mp
            .range((std::ops::Bound::Excluded(x), std::ops::Bound::Unbounded))
            .next()
            .unwrap_or((&0, &self.counters.len()));

        self.sums.sum(index, self.sums.len()).unwrap()
    }

    pub fn x_sum(&self, x: i64) -> i64 {
        if let Some(&index) = self.mp.get(&x) {
            self.sums.select(index).unwrap_or(0)
        } else {
            0
        }
    }

    pub fn total_sum(&self) -> i64 {
        self.sums.select_all().unwrap()
    }
}

#[cfg(test)]
mod fenwick_tree_checklist_tests {
    use super::FenwickTree;

    #[test]
    fn new_non_empty_tree_is_zero_initialized() {
        // Arrange
        let size = 3;
        let indices = [0, 1, 2];
        let expected_len = 3;
        let expected_is_empty = false;
        let expected_values = vec![Some(0), Some(0), Some(0)];
        let expected_total = Some(0);
        let tree = FenwickTree::new(size);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_values = indices
            .iter()
            .map(|&index| tree.select(index))
            .collect::<Vec<_>>();
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn new_zero_length_tree_has_empty_query_results() {
        // Arrange
        let size = 0;
        let index = 0;
        let expected_len = 0;
        let expected_is_empty = true;
        let expected_sum = Some(0);
        let expected_value = None;
        let expected_total = Some(0);
        let tree = FenwickTree::new(size);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_sum = tree.sum(index, index);
        let actual_value = tree.select(index);
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn from_empty_slice_builds_empty_tree() {
        // Arrange
        let values = Vec::new();
        let expected_is_empty = true;
        let expected_total = Some(0);

        // Act
        let tree = FenwickTree::from_slice(&values);
        let actual_is_empty = tree.is_empty();
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn from_mixed_slice_matches_point_range_and_total_sums() {
        // Arrange
        let values = vec![-2, 3, 0, -1];
        let indices = [0, 1, 2, 3];
        let range_start = 1;
        let range_end = 4;
        let expected_values = vec![Some(-2), Some(3), Some(0), Some(-1)];
        let expected_range_sum = Some(2);
        let expected_total = Some(0);

        // Act
        let tree = FenwickTree::from_slice(&values);
        let actual_values = indices
            .iter()
            .map(|&index| tree.select(index))
            .collect::<Vec<_>>();
        let actual_range_sum = tree.sum(range_start, range_end);
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_range_sum, expected_range_sum);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn add_accumulates_at_first_last_and_repeated_indices() {
        // Arrange
        let size = 3;
        let operations = [(0, 2), (2, 5), (2, -1)];
        let indices = [0, 1, 2];
        let expected_values = vec![Some(2), Some(0), Some(4)];
        let expected_total = Some(6);
        let mut tree = FenwickTree::new(size);

        // Act
        operations
            .iter()
            .for_each(|&(index, value)| tree.add(index, value));
        let actual_values = indices
            .iter()
            .map(|&index| tree.select(index))
            .collect::<Vec<_>>();
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn add_outside_non_empty_and_empty_trees_is_ignored() {
        // Arrange
        let values = vec![1, 2, 3];
        let out_of_range_index = values.len();
        let added_value = 10;
        let empty_size = 0;
        let empty_index = 0;
        let expected_non_empty_total = Some(6);
        let expected_empty_total = Some(0);
        let mut non_empty_tree = FenwickTree::from_slice(&values);
        let mut empty_tree = FenwickTree::new(empty_size);

        // Act
        non_empty_tree.add(out_of_range_index, added_value);
        empty_tree.add(empty_index, added_value);
        let actual_non_empty_total = non_empty_tree.select_all();
        let actual_empty_total = empty_tree.select_all();

        // Assert
        assert_eq!(actual_non_empty_total, expected_non_empty_total);
        assert_eq!(actual_empty_total, expected_empty_total);
    }

    #[test]
    fn update_replaces_existing_value_instead_of_adding() {
        // Arrange
        let values = vec![1, 2, 3];
        let updated_index = 1;
        let updated_value = 10;
        let indices = [0, 1, 2];
        let expected_values = vec![Some(1), Some(10), Some(3)];
        let expected_total = Some(14);
        let mut tree = FenwickTree::from_slice(&values);

        // Act
        tree.update(updated_index, updated_value);
        let actual_values = indices
            .iter()
            .map(|&index| tree.select(index))
            .collect::<Vec<_>>();
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn update_at_len_is_ignored() {
        // Arrange
        let values = vec![1, 2, 3];
        let out_of_range_index = values.len();
        let updated_value = 10;
        let first_index = 0;
        let expected_values = vec![Some(1), Some(2), Some(3)];
        let expected_total = Some(6);
        let mut tree = FenwickTree::from_slice(&values);

        // Act
        tree.update(out_of_range_index, updated_value);
        let actual_values = (first_index..values.len())
            .map(|index| tree.select(index))
            .collect::<Vec<_>>();
        let actual_total = tree.select_all();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn sum_returns_middle_half_open_range() {
        // Arrange
        let values = vec![1, 2, 3, 4];
        let left_index = 1;
        let right_index = 3;
        let expected_sum = Some(5);
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_returns_zero_for_empty_ranges_at_all_boundaries() {
        // Arrange
        let values = vec![1, 2, 3, 4];
        let boundaries = [0, 2, values.len()];
        let expected_sums = vec![Some(0), Some(0), Some(0)];
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_sums = boundaries
            .iter()
            .map(|&boundary| tree.sum(boundary, boundary))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_sums, expected_sums);
    }

    #[test]
    fn sum_returns_none_when_left_is_greater_than_right() {
        // Arrange
        let values = vec![1, 2, 3, 4];
        let left_index = 3;
        let right_index = 2;
        let expected_sum = None;
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_returns_none_when_right_exceeds_len() {
        // Arrange
        let values = vec![1, 2, 3, 4];
        let left_index = 0;
        let right_index = values.len() + 1;
        let expected_sum = None;
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn select_returns_boundary_values_and_none_at_len() {
        // Arrange
        let values = vec![1, 2, 3, 4];
        let first_index = 0;
        let last_index = values.len() - 1;
        let out_of_range_index = values.len();
        let expected_values = (Some(1), Some(4), None);
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_values = (
            tree.select(first_index),
            tree.select(last_index),
            tree.select(out_of_range_index),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn lower_bound_handles_prefix_boundaries_and_total_overflow() {
        // Arrange
        let values = vec![2, 5, 3];
        let queries = [1, 2, 3, 10, 11];
        let expected_indices = vec![0, 0, 1, 2, 3];
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_indices = queries
            .iter()
            .map(|&value| tree.lower_bound(value))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    #[test]
    fn lower_bound_skips_zero_prefix_plateaus() {
        // Arrange
        let values = vec![0, 2, 0, 1];
        let queries = [1, 2, 3];
        let expected_indices = vec![1, 1, 3];
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_indices = queries
            .iter()
            .map(|&value| tree.lower_bound(value))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    #[test]
    fn lower_bound_on_empty_tree_returns_zero_for_positive_value() {
        // Arrange
        let size = 0;
        let query = 1;
        let expected_index = 0;
        let tree = FenwickTree::new(size);

        // Act
        let actual_index = tree.lower_bound(query);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn upper_bound_handles_prefix_boundaries_and_total_or_larger_values() {
        // Arrange
        let values = vec![2, 5, 3];
        let queries = [1, 2, 3, 10, 11];
        let expected_indices = vec![0, 1, 1, 3, 3];
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_indices = queries
            .iter()
            .map(|&value| tree.upper_bound(value))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    #[test]
    fn upper_bound_skips_zero_prefix_plateaus() {
        // Arrange
        let values = vec![0, 2, 0, 1];
        let queries = [0, 2, 3];
        let expected_indices = vec![1, 3, 4];
        let tree = FenwickTree::from_slice(&values);

        // Act
        let actual_indices = queries
            .iter()
            .map(|&value| tree.upper_bound(value))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    #[test]
    fn upper_bound_on_empty_tree_returns_zero() {
        // Arrange
        let size = 0;
        let queries = [0, 1];
        let expected_indices = vec![0, 0];
        let tree = FenwickTree::new(size);

        // Act
        let actual_indices = queries
            .iter()
            .map(|&value| tree.upper_bound(value))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }
}
#[cfg(test)]
mod tests {
    use super::KthTop;

    #[test]
    fn deterministic_operation_sequence_matches_reference_implementation() {
        // Arrange
        let k = 3;
        let candidates = vec![-2, 0, 3];
        let operations = vec![-2, 3, 99, -2, 0, 3, -7, 0, -2];
        let ranks = vec![-1, 0, 1, 2, 3];
        let thresholds = vec![-3, -2, -1, 0, 1, 3, 4, 99];
        let exact_values = vec![-7, -2, 0, 3, 99];
        let mut actual_kth_top = KthTop::new(k, &candidates);
        let mut reference_kth_top = ReferenceKthTop::new(k, &candidates);

        let expected_snapshots = operations
            .iter()
            .map(|&value| {
                let push_result = reference_kth_top.push(value);
                reference_snapshot(
                    &reference_kth_top,
                    push_result,
                    &ranks,
                    &thresholds,
                    &exact_values,
                )
            })
            .collect::<Vec<_>>();

        // Act
        let actual_snapshots = operations
            .iter()
            .map(|&value| {
                let push_result = actual_kth_top.push(value);
                actual_snapshot(
                    &actual_kth_top,
                    push_result,
                    &ranks,
                    &thresholds,
                    &exact_values,
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_snapshots, expected_snapshots);
    }

    #[test]
    fn duplicates_are_counted_in_nth_and_medians() {
        // Arrange
        let values = vec![5];
        let mut kth_top = KthTop::new(3, &values);
        let expected_nth = Some(5);
        let expected_medians = (Some(5), None);

        let push_value = 5;
        let nth_rank = 1;

        // Act
        let actual_push_result_1 = kth_top.push(push_value);
        let actual_push_result_2 = kth_top.push(push_value);
        let actual_push_result_3 = kth_top.push(push_value);
        let actual_nth = kth_top.nth(nth_rank);
        let actual_medians = kth_top.medians();

        // Assert
        assert!(actual_push_result_3);
        assert!(actual_push_result_2);
        assert!(actual_push_result_1);
        assert_eq!(actual_nth, expected_nth);
        assert_eq!(actual_medians, expected_medians);
    }

    #[test]
    fn unordered_duplicate_candidates_support_rank_and_odd_median_queries() {
        // Arrange
        let k = 3;
        let candidates = vec![3, 1, 3, 2];
        let pushed_values = [3, 1, 2];
        let ranks = [0, 1, 2];
        let expected_push_results = vec![true, true, true];
        let expected_nth_values = vec![Some(1), Some(2), Some(3)];
        let expected_medians = (Some(2), None);
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_results = pushed_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_nth_values = ranks
            .iter()
            .map(|&rank| kth_top.nth(rank))
            .collect::<Vec<_>>();
        let actual_medians = kth_top.medians();

        // Assert
        assert_eq!(actual_push_results, expected_push_results);
        assert_eq!(actual_nth_values, expected_nth_values);
        assert_eq!(actual_medians, expected_medians);
    }

    #[test]
    fn empty_candidate_set_returns_empty_medians_and_zero_exact_sum() {
        // Arrange
        let k = 2;
        let candidates = Vec::new();
        let queried_value = 7;
        let expected_medians = (None, None);
        let expected_exact_sum = 0;
        let kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_medians = kth_top.medians();
        let actual_exact_sum = kth_top.x_sum(queried_value);

        // Assert
        assert_eq!(actual_medians, expected_medians);
        assert_eq!(actual_exact_sum, expected_exact_sum);
    }

    #[test]
    fn partial_even_window_returns_only_the_reachable_median() {
        // Arrange
        let k = 4;
        let candidates = vec![1, 3];
        let pushed_values = [1, 3];
        let expected_push_results = vec![true, true];
        let expected_medians = (None, Some(3));
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_results = pushed_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_medians = kth_top.medians();

        // Assert
        assert_eq!(actual_push_results, expected_push_results);
        assert_eq!(actual_medians, expected_medians);
    }

    #[test]
    fn nth_returns_none_when_current_length_is_too_small() {
        // Arrange
        let values = vec![1, 2, 3];
        let mut kth_top = KthTop::new(4, &values);
        let expected_value = None;

        let push_value = 1;
        let push_value_2 = 3;
        let nth_rank = 2;

        // Act
        let actual_push_result_1 = kth_top.push(push_value);
        let actual_push_result_2 = kth_top.push(push_value_2);
        let actual_value = kth_top.nth(nth_rank);

        // Assert
        assert!(actual_push_result_2);
        assert!(actual_push_result_1);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn single_value_window_returns_its_median_and_strict_threshold_sums() {
        // Arrange
        let value = 10;
        let candidates = vec![value];
        let window_size = 1;
        let thresholds = [9, 10, 11];
        let expected_medians = (Some(value), None);
        let expected_sums = [0, 0, value];
        let mut kth_top = KthTop::new(window_size, &candidates);

        // Act
        let actual_pushed = kth_top.push(value);
        let actual_medians = kth_top.medians();
        let actual_sums = thresholds.map(|threshold| kth_top.less_than_x_sum(threshold));

        // Assert
        assert!(actual_pushed);
        assert_eq!(actual_medians, expected_medians);
        assert_eq!(actual_sums, expected_sums);
    }

    #[test]
    fn push_unknown_value_returns_false_and_preserves_empty_window() {
        // Arrange
        let candidates = vec![5, 6];
        let window_size = 1;
        let unknown_value = 7;
        let ranks = [0];
        let thresholds = [4, 6];
        let exact_values = [5, 6];
        let mut kth_top = KthTop::new(window_size, &candidates);
        let expected_state = actual_snapshot(&kth_top, false, &ranks, &thresholds, &exact_values);

        // Act
        let actual_pushed = kth_top.push(unknown_value);
        let actual_state =
            actual_snapshot(&kth_top, actual_pushed, &ranks, &thresholds, &exact_values);

        // Assert
        assert!(!actual_pushed);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn empty_candidate_set_rejects_push_and_keeps_queries_empty() {
        // Arrange
        let candidates = Vec::new();
        let window_size = 2;
        let value = 1;
        let rank = 0;
        let expected_state = (false, 0, None, 0, 0);
        let mut kth_top = KthTop::new(window_size, &candidates);

        // Act
        let actual_pushed = kth_top.push(value);
        let actual_state = (
            actual_pushed,
            kth_top.total_sum(),
            kth_top.nth(rank),
            kth_top.less_than_x_sum(value),
            kth_top.greater_than_x_sum(value),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn threshold_queries_cover_x_minus_one_x_and_x_plus_one() {
        // Arrange
        let candidates = vec![2, 4, 6];
        let window_size = 3;
        let thresholds = [3, 4, 5];
        let expected_sums = [(2, 10), (2, 6), (6, 6)];
        let mut kth_top = KthTop::new(window_size, &candidates);
        for &value in &candidates {
            kth_top.push(value);
        }

        // Act
        let actual_sums = thresholds.map(|threshold| {
            (
                kth_top.less_than_x_sum(threshold),
                kth_top.greater_than_x_sum(threshold),
            )
        });

        // Assert
        assert_eq!(actual_sums, expected_sums);
    }

    use std::collections::VecDeque;

    #[derive(Debug, PartialEq)]
    struct Snapshot {
        push_result: bool,
        nth_values: Vec<Option<i64>>,
        medians: (Option<i64>, Option<i64>),
        less_than_sums: Vec<i64>,
        greater_than_sums: Vec<i64>,
        exact_sums: Vec<i64>,
        total_sum: i64,
    }

    struct ReferenceKthTop {
        k: usize,
        candidates: Vec<i64>,
        values: VecDeque<i64>,
    }

    impl ReferenceKthTop {
        fn new(k: usize, candidates: &[i64]) -> Self {
            Self {
                k,
                candidates: candidates.to_vec(),
                values: VecDeque::new(),
            }
        }

        fn push(&mut self, value: i64) -> bool {
            if !self.candidates.contains(&value) {
                return false;
            }

            self.values.push_back(value);
            if self.values.len() > self.k {
                self.values.pop_front();
            }
            true
        }

        fn sorted_values(&self) -> Vec<i64> {
            let mut values = self.values.iter().copied().collect::<Vec<_>>();
            values.sort_unstable();
            values
        }

        fn nth(&self, n: i64) -> Option<i64> {
            if n < 0 {
                return None;
            }
            self.sorted_values().get(n as usize).copied()
        }

        fn medians(&self) -> (Option<i64>, Option<i64>) {
            (
                self.nth(self.k as i64 / 2),
                if self.k & 1 == 1 {
                    None
                } else {
                    self.nth(self.k as i64 / 2 - 1)
                },
            )
        }

        fn less_than_x_sum(&self, x: i64) -> i64 {
            self.values.iter().filter(|&&value| value < x).sum()
        }

        fn greater_than_x_sum(&self, x: i64) -> i64 {
            self.values.iter().filter(|&&value| value > x).sum()
        }

        fn x_sum(&self, x: i64) -> i64 {
            self.values.iter().filter(|&&value| value == x).sum()
        }

        fn total_sum(&self) -> i64 {
            self.values.iter().sum()
        }
    }

    fn actual_snapshot(
        kth_top: &KthTop,
        push_result: bool,
        ranks: &[i64],
        thresholds: &[i64],
        exact_values: &[i64],
    ) -> Snapshot {
        Snapshot {
            push_result,
            nth_values: ranks.iter().map(|&rank| kth_top.nth(rank)).collect(),
            medians: kth_top.medians(),
            less_than_sums: thresholds
                .iter()
                .map(|&threshold| kth_top.less_than_x_sum(threshold))
                .collect(),
            greater_than_sums: thresholds
                .iter()
                .map(|&threshold| kth_top.greater_than_x_sum(threshold))
                .collect(),
            exact_sums: exact_values
                .iter()
                .map(|&value| kth_top.x_sum(value))
                .collect(),
            total_sum: kth_top.total_sum(),
        }
    }

    fn reference_snapshot(
        kth_top: &ReferenceKthTop,
        push_result: bool,
        ranks: &[i64],
        thresholds: &[i64],
        exact_values: &[i64],
    ) -> Snapshot {
        Snapshot {
            push_result,
            nth_values: ranks.iter().map(|&rank| kth_top.nth(rank)).collect(),
            medians: kth_top.medians(),
            less_than_sums: thresholds
                .iter()
                .map(|&threshold| kth_top.less_than_x_sum(threshold))
                .collect(),
            greater_than_sums: thresholds
                .iter()
                .map(|&threshold| kth_top.greater_than_x_sum(threshold))
                .collect(),
            exact_sums: exact_values
                .iter()
                .map(|&value| kth_top.x_sum(value))
                .collect(),
            total_sum: kth_top.total_sum(),
        }
    }

    #[test]
    fn repeated_window_evictions_keep_only_the_latest_k_values() {
        // Arrange
        let k = 2;
        let candidates = vec![1, 2, 3, 4];
        let pushed_values = [1, 4, 2, 3];
        let ranks = [0, 1];
        let less_than_threshold = 3;
        let greater_than_threshold = 2;
        let expected_push_results = vec![true, true, true, true];
        let expected_nth_values = vec![Some(2), Some(3)];
        let expected_medians = (Some(3), Some(2));
        let expected_total_sum = 5;
        let expected_less_than_sum = 2;
        let expected_greater_than_sum = 3;
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_results = pushed_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_nth_values = ranks
            .iter()
            .map(|&rank| kth_top.nth(rank))
            .collect::<Vec<_>>();
        let actual_medians = kth_top.medians();
        let actual_total_sum = kth_top.total_sum();
        let actual_less_than_sum = kth_top.less_than_x_sum(less_than_threshold);
        let actual_greater_than_sum = kth_top.greater_than_x_sum(greater_than_threshold);

        // Assert
        assert_eq!(actual_push_results, expected_push_results);
        assert_eq!(actual_nth_values, expected_nth_values);
        assert_eq!(actual_medians, expected_medians);
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_less_than_sum, expected_less_than_sum);
        assert_eq!(actual_greater_than_sum, expected_greater_than_sum);
    }

    #[test]
    fn zero_capacity_keeps_all_queries_empty_after_successful_push() {
        // Arrange
        let k = 0;
        let candidates = vec![1];
        let pushed_value = 1;
        let rank = 0;
        let less_than_threshold = 2;
        let greater_than_threshold = 0;
        let expected_push_result = true;
        let expected_state = (None, (None, None), 0, 0, 0, 0);
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_result = kth_top.push(pushed_value);
        let actual_state = (
            kth_top.nth(rank),
            kth_top.medians(),
            kth_top.less_than_x_sum(less_than_threshold),
            kth_top.greater_than_x_sum(greater_than_threshold),
            kth_top.x_sum(pushed_value),
            kth_top.total_sum(),
        );

        // Assert
        assert_eq!(actual_push_result, expected_push_result);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn mixed_signed_duplicate_values_are_supported_by_all_queries() {
        // Arrange
        let k = 5;
        let candidates = vec![-5, -2, 0, 4];
        let pushed_values = [-5, -5, -2, 0, 4];
        let rank = 2;
        let threshold = -2;
        let exact_value = -5;
        let expected_push_results = vec![true, true, true, true, true];
        let expected_state = (Some(-2), (Some(-2), None), -10, 4, -10, -8);
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_results = pushed_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_state = (
            kth_top.nth(rank),
            kth_top.medians(),
            kth_top.less_than_x_sum(threshold),
            kth_top.greater_than_x_sum(threshold),
            kth_top.x_sum(exact_value),
            kth_top.total_sum(),
        );

        // Assert
        assert_eq!(actual_push_results, expected_push_results);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn threshold_queries_respect_minimum_and_maximum_candidate_boundaries() {
        // Arrange
        let k = 3;
        let candidates = vec![2, 4, 6];
        let pushed_values = [2, 4, 6];
        let thresholds = [1, 2, 6, 7];
        let expected_push_results = vec![true, true, true];
        let expected_less_than_sums = vec![0, 0, 6, 12];
        let expected_greater_than_sums = vec![12, 10, 0, 0];
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_results = pushed_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_less_than_sums = thresholds
            .iter()
            .map(|&threshold| kth_top.less_than_x_sum(threshold))
            .collect::<Vec<_>>();
        let actual_greater_than_sums = thresholds
            .iter()
            .map(|&threshold| kth_top.greater_than_x_sum(threshold))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_push_results, expected_push_results);
        assert_eq!(actual_less_than_sums, expected_less_than_sums);
        assert_eq!(actual_greater_than_sums, expected_greater_than_sums);
    }

    #[test]
    fn x_sum_returns_zero_for_values_not_in_index_map() {
        // Arrange
        let candidates = vec![1, 3, 5];
        let window_size = 2;
        let initial_values = [1, 3];
        let absent_values = [2, 4];
        let expected_sums = [0, 0];
        let mut kth_top = KthTop::new(window_size, &candidates);
        for value in initial_values {
            kth_top.push(value);
        }

        // Act
        let actual_sums = absent_values.map(|value| kth_top.x_sum(value));

        // Assert
        assert_eq!(actual_sums, expected_sums);
    }

    #[test]
    fn duplicate_exact_sum_decreases_by_one_value_after_eviction() {
        // Arrange
        let k = 3;
        let candidates = vec![2, 5];
        let initial_values = [2, 2, 5];
        let next_value = 5;
        let queried_value = 2;
        let expected_initial_push_results = vec![true, true, true];
        let expected_next_push_result = true;
        let expected_sum_before = 4;
        let expected_sum_after = 2;
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_initial_push_results = initial_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_sum_before = kth_top.x_sum(queried_value);
        let actual_next_push_result = kth_top.push(next_value);
        let actual_sum_after = kth_top.x_sum(queried_value);

        // Assert
        assert_eq!(actual_initial_push_results, expected_initial_push_results);
        assert_eq!(actual_sum_before, expected_sum_before);
        assert_eq!(actual_next_push_result, expected_next_push_result);
        assert_eq!(actual_sum_after, expected_sum_after);
    }

    #[test]
    fn full_even_window_returns_upper_and_lower_medians_and_total_sum() {
        // Arrange
        let candidates = vec![1, 2, 3];
        let window_size = 2;
        let values = [1, 3];
        let expected_pushed = [true, true];
        let expected_medians = (Some(3), Some(1));
        let expected_sum = 4;
        let mut kth_top = KthTop::new(window_size, &candidates);

        // Act
        let actual_pushed = values.map(|value| kth_top.push(value));
        let actual_medians = kth_top.medians();
        let actual_sum = kth_top.total_sum();

        // Assert
        assert_eq!(actual_pushed, expected_pushed);
        assert_eq!(actual_medians, expected_medians);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn oldest_value_is_removed_when_window_exceeds_k() {
        // Arrange
        let values = vec![1, 2, 3];
        let mut kth_top = KthTop::new(2, &values);
        let expected_total_sum = 5;
        let expected_first = 0;

        let push_value = 1;
        let push_value_2 = 2;
        let push_value_3 = 3;
        let x_sum_value = 1;

        // Act
        let actual_push_result_1 = kth_top.push(push_value);
        let actual_push_result_2 = kth_top.push(push_value_2);
        let actual_push_result_3 = kth_top.push(push_value_3);
        let actual_total_sum = kth_top.total_sum();
        let actual_first = kth_top.x_sum(x_sum_value);

        // Assert
        assert!(actual_push_result_3);
        assert!(actual_push_result_2);
        assert!(actual_push_result_1);
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_first, expected_first);
    }

    #[test]
    fn zero_window_keeps_total_sum_zero() {
        // Arrange
        let values = vec![1, 2];
        let mut kth_top = KthTop::new(0, &values);
        let expected_total_sum = 0;

        let push_value = 1;

        // Act
        let actual_push_result_1 = kth_top.push(push_value);
        let actual_total_sum = kth_top.total_sum();

        // Assert
        assert!(actual_push_result_1);
        assert_eq!(actual_total_sum, expected_total_sum);
    }

    #[test]
    fn rejected_value_does_not_change_full_window_or_eviction_order() {
        // Arrange
        let k = 2;
        let candidates = vec![1, 2, 3];
        let initial_values = [1, 2];
        let unknown_value = 99;
        let next_value = 3;
        let first_rank = 0;
        let second_rank = 1;
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_initial_push_results = vec![true, true];
        let expected_unchanged_state = (Some(1), Some(2), (Some(2), Some(1)), 3, 1, 2);
        let expected_unknown_push_result = false;
        let expected_next_push_result = true;
        let expected_final_state = (Some(2), Some(3), (Some(3), Some(2)), 5, 0, 2, 3);
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_initial_push_results = initial_values
            .iter()
            .map(|&value| kth_top.push(value))
            .collect::<Vec<_>>();
        let actual_state_before_unknown = (
            kth_top.nth(first_rank),
            kth_top.nth(second_rank),
            kth_top.medians(),
            kth_top.total_sum(),
            kth_top.x_sum(first_value),
            kth_top.x_sum(second_value),
        );
        let actual_unknown_push_result = kth_top.push(unknown_value);
        let actual_state_after_unknown = (
            kth_top.nth(first_rank),
            kth_top.nth(second_rank),
            kth_top.medians(),
            kth_top.total_sum(),
            kth_top.x_sum(first_value),
            kth_top.x_sum(second_value),
        );
        let actual_next_push_result = kth_top.push(next_value);
        let actual_final_state = (
            kth_top.nth(first_rank),
            kth_top.nth(second_rank),
            kth_top.medians(),
            kth_top.total_sum(),
            kth_top.x_sum(first_value),
            kth_top.x_sum(second_value),
            kth_top.x_sum(third_value),
        );

        // Assert
        assert_eq!(actual_initial_push_results, expected_initial_push_results);
        assert_eq!(actual_state_before_unknown, expected_unchanged_state);
        assert_eq!(actual_unknown_push_result, expected_unknown_push_result);
        assert_eq!(actual_state_after_unknown, expected_unchanged_state);
        assert_eq!(actual_next_push_result, expected_next_push_result);
        assert_eq!(actual_final_state, expected_final_state);
    }

    #[test]
    fn negative_rank_returns_none_without_changing_state() {
        // Arrange
        let k = 2;
        let candidates = vec![1, 2];
        let pushed_value = 2;
        let negative_rank = -1;
        let retained_rank = 0;
        let expected_push_result = true;
        let expected_rank_value = None;
        let expected_state = (Some(2), 2);
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_push_result = kth_top.push(pushed_value);
        let actual_state_before = (kth_top.nth(retained_rank), kth_top.total_sum());
        let actual_rank_value = kth_top.nth(negative_rank);
        let actual_state_after = (kth_top.nth(retained_rank), kth_top.total_sum());

        // Assert
        assert_eq!(actual_push_result, expected_push_result);
        assert_eq!(actual_state_before, expected_state);
        assert_eq!(actual_rank_value, expected_rank_value);
        assert_eq!(actual_state_after, expected_state);
    }

    #[test]
    fn evicting_i64_min_does_not_overflow() {
        // Arrange
        let k = 1;
        let candidates = vec![i64::MIN, -1];
        let first_value = i64::MIN;
        let next_value = -1;
        let rank = 0;
        let expected_first_push_result = true;
        let expected_next_push_result = true;
        let expected_state = (Some(-1), 0, -1);
        let mut kth_top = KthTop::new(k, &candidates);

        // Act
        let actual_first_push_result = kth_top.push(first_value);
        let actual_next_push_result = kth_top.push(next_value);
        let actual_state = (
            kth_top.nth(rank),
            kth_top.x_sum(first_value),
            kth_top.total_sum(),
        );

        // Assert
        assert_eq!(actual_first_push_result, expected_first_push_result);
        assert_eq!(actual_next_push_result, expected_next_push_result);
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn valid_single_value_window_returns_rank_and_exact_and_threshold_sums() {
        // Arrange
        let candidates = vec![5, 6];
        let window_size = 1;
        let value = 5;
        let rank = 0;
        let upper_threshold = 6;
        let lower_threshold = 4;
        let absent_value = 6;
        let expected_state = (Some(value), value, value, 0);
        let mut kth_top = KthTop::new(window_size, &candidates);
        kth_top.push(value);

        // Act
        let actual_state = (
            kth_top.nth(rank),
            kth_top.less_than_x_sum(upper_threshold),
            kth_top.greater_than_x_sum(lower_threshold),
            kth_top.x_sum(absent_value),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

}
