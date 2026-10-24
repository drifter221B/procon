use std::sync::Arc;

pub trait Monoid {
    type S: Clone;

    fn id() -> Self::S;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
}

/// 永続セグメント木（一点更新・区間取得）。
/// 更新は元の木を破壊せず新しいバージョンを返す。
pub struct PersistentSegmentTree<M>
where
    M: Monoid,
{
    size: usize,
    root: Option<Arc<Node<M>>>,
}

struct Node<M>
where
    M: Monoid,
{
    value: M::S,
    left: Option<Arc<Node<M>>>,
    right: Option<Arc<Node<M>>>,
}

impl<M> Clone for PersistentSegmentTree<M>
where
    M: Monoid,
{
    fn clone(&self) -> Self {
        Self {
            size: self.size,
            root: self.root.clone(),
        }
    }
}

impl<M> Node<M>
where
    M: Monoid,
{
    fn new_leaf(value: M::S) -> Self {
        Self {
            value,
            left: None,
            right: None,
        }
    }

    fn new_internal(left: Option<Arc<Node<M>>>, right: Option<Arc<Node<M>>>) -> Self {
        let left_value = left
            .as_ref()
            .map(|child| child.value.clone())
            .unwrap_or_else(M::id);
        let right_value = right
            .as_ref()
            .map(|child| child.value.clone())
            .unwrap_or_else(M::id);
        let value = M::binary_operation(&left_value, &right_value);
        Self { value, left, right }
    }
}

impl<M> PersistentSegmentTree<M>
where
    M: Monoid,
{
    /// 要素数 `size`（初期値 `Monoid::id()`）のセグメント木を作る。
    pub fn new(size: usize) -> Self {
        let root = Self::build_identity(size, 0, size);
        Self { size, root }
    }

    /// スライスからセグメント木を構築する。
    pub fn from_slice(values: &[M::S]) -> Self {
        let root = Self::build_from_slice(values, 0, values.len());
        Self {
            size: values.len(),
            root,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    /// 一点を指定値に置き換えた新しい木を返す。範囲外なら `None`。
    ///
    /// 更新経路だけをパスコピーし、未更新部分木は元の木と共有するため、
    /// 時間計算量と追加メモリ使用量はいずれも `O(log n)`。
    pub fn set(&self, index: usize, value: M::S) -> Option<Self> {
        if self.size <= index {
            return None;
        }
        let root = self
            .root
            .as_ref()
            .map(|root| Self::set_inner(root, 0, self.size, index, value));
        Some(Self {
            size: self.size,
            root,
        })
    }

    /// 半開区間 `[query_left, query_right)` の集約値を返す。範囲不正なら `None`。
    pub fn query(&self, query_left: usize, query_right: usize) -> Option<M::S> {
        if query_left > query_right || self.size < query_right {
            return None;
        }
        Some(Self::query_inner(
            self.root.as_deref(),
            0,
            self.size,
            query_left,
            query_right,
        ))
    }

    /// `query` の互換エイリアス。
    pub fn sum(&self, query_left: usize, query_right: usize) -> Option<M::S> {
        self.query(query_left, query_right)
    }

    fn build_identity(
        size: usize,
        segment_left: usize,
        segment_right: usize,
    ) -> Option<Arc<Node<M>>> {
        if size == 0 {
            return None;
        }
        if segment_left + 1 == segment_right {
            return Some(Arc::new(Node::new_leaf(M::id())));
        }
        let segment_mid = (segment_left + segment_right) >> 1;
        let left_child = Self::build_identity(size, segment_left, segment_mid);
        let right_child = Self::build_identity(size, segment_mid, segment_right);
        Some(Arc::new(Node::new_internal(left_child, right_child)))
    }

    fn build_from_slice(
        values: &[M::S],
        segment_left: usize,
        segment_right: usize,
    ) -> Option<Arc<Node<M>>> {
        if values.is_empty() {
            return None;
        }
        if segment_left + 1 == segment_right {
            return Some(Arc::new(Node::new_leaf(values[segment_left].clone())));
        }
        let segment_mid = (segment_left + segment_right) >> 1;
        let left_child = Self::build_from_slice(values, segment_left, segment_mid);
        let right_child = Self::build_from_slice(values, segment_mid, segment_right);
        Some(Arc::new(Node::new_internal(left_child, right_child)))
    }

    fn set_inner(
        node: &Arc<Node<M>>,
        segment_left: usize,
        segment_right: usize,
        target_index: usize,
        value: M::S,
    ) -> Arc<Node<M>> {
        if segment_left + 1 == segment_right {
            return Arc::new(Node::new_leaf(value));
        }
        let segment_mid = (segment_left + segment_right) >> 1;
        let (left, right) = if target_index < segment_mid {
            let left = node
                .left
                .as_ref()
                .map(|left| Self::set_inner(left, segment_left, segment_mid, target_index, value));
            (left, node.right.clone())
        } else {
            let right = node.right.as_ref().map(|right| {
                Self::set_inner(right, segment_mid, segment_right, target_index, value)
            });
            (node.left.clone(), right)
        };

        Arc::new(Node::new_internal(left, right))
    }

    fn query_inner(
        node: Option<&Node<M>>,
        segment_left: usize,
        segment_right: usize,
        query_left: usize,
        query_right: usize,
    ) -> M::S {
        if query_right <= segment_left || segment_right <= query_left {
            return M::id();
        }
        if query_left <= segment_left && segment_right <= query_right {
            return node
                .map(|current_node| current_node.value.clone())
                .unwrap_or_else(M::id);
        }
        let segment_mid = (segment_left + segment_right) >> 1;
        let left_value = Self::query_inner(
            node.and_then(|current| current.left.as_deref()),
            segment_left,
            segment_mid,
            query_left,
            query_right,
        );
        let right_value = Self::query_inner(
            node.and_then(|current| current.right.as_deref()),
            segment_mid,
            segment_right,
            query_left,
            query_right,
        );
        M::binary_operation(&left_value, &right_value)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;
    use std::sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    };

    use super::{Monoid, Node, PersistentSegmentTree as GenericPersistentSegmentTree};

    #[derive(Clone, Copy)]
    struct SumMonoid;

    impl Monoid for SumMonoid {
        type S = i64;

        fn id() -> Self::S {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            *lhs + *rhs
        }
    }

    #[derive(Clone, Copy)]
    struct StringConcatMonoid;

    impl Monoid for StringConcatMonoid {
        type S = String;

        fn id() -> Self::S {
            String::new()
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            let mut merged_text = lhs.clone();
            merged_text.push_str(rhs);
            merged_text
        }
    }

    type PersistentSegmentTree = GenericPersistentSegmentTree<SumMonoid>;

    #[derive(Clone, Copy)]
    struct ProductMonoid;

    impl Monoid for ProductMonoid {
        type S = i64;

        fn id() -> Self::S {
            1
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            *lhs * *rhs
        }
    }

    static VALUE_CLONE_COUNT: AtomicUsize = AtomicUsize::new(0);

    #[derive(Debug, PartialEq, Eq)]
    struct CloneCountedValue(i64);

    impl Clone for CloneCountedValue {
        fn clone(&self) -> Self {
            VALUE_CLONE_COUNT.fetch_add(1, Ordering::SeqCst);
            Self(self.0)
        }
    }

    struct CloneCountedSumMonoid;

    impl Monoid for CloneCountedSumMonoid {
        type S = CloneCountedValue;

        fn id() -> Self::S {
            CloneCountedValue(0)
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            CloneCountedValue(lhs.0 + rhs.0)
        }
    }

    fn interval_cases(values: &[i64]) -> Vec<((usize, usize), Option<i64>)> {
        (0..=values.len())
            .flat_map(|left| {
                (left..=values.len())
                    .map(move |right| ((left, right), Some(values[left..right].iter().sum())))
            })
            .collect()
    }

    fn observe_intervals(
        tree: &PersistentSegmentTree,
        intervals: &[(usize, usize)],
    ) -> Vec<Option<i64>> {
        intervals
            .iter()
            .map(|&(left, right)| tree.sum(left, right))
            .collect()
    }

    fn inspect_sum_node_invariants(node: &Arc<Node<SumMonoid>>) -> (usize, Vec<String>) {
        match (&node.left, &node.right) {
            (None, None) => (1, Vec::new()),
            (Some(left), Some(right)) => {
                let (left_leaves, mut diagnostics) = inspect_sum_node_invariants(left);
                let (right_leaves, right_diagnostics) = inspect_sum_node_invariants(right);
                diagnostics.extend(right_diagnostics);
                let expected = left.value + right.value;
                if node.value != expected {
                    diagnostics.push(format!(
                        "node value {} differs from child sum {expected}",
                        node.value
                    ));
                }
                (left_leaves + right_leaves, diagnostics)
            }
            _ => (
                0,
                vec!["an internal node must have both children".to_string()],
            ),
        }
    }

    fn collect_node_addresses<M>(
        node: &Option<Arc<Node<M>>>,
        addresses: &mut HashSet<*const Node<M>>,
    ) where
        M: Monoid,
    {
        if let Some(node) = node {
            addresses.insert(Arc::as_ptr(node));
            collect_node_addresses(&node.left, addresses);
            collect_node_addresses(&node.right, addresses);
        }
    }

    #[test]
    fn new_size_one_returns_zero_for_full_range_sum() {
        // Arrange
        let tree_size = 1usize;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_len = 1usize;
        let expected_is_empty = false;
        let expected_sum = Some(0i64);
        let tree = PersistentSegmentTree::new(tree_size);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_sum = tree.sum(query_left, query_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_size_three_returns_zero_for_full_and_partial_sums() {
        // Arrange
        let tree_size = 3usize;
        let full_left = 0usize;
        let full_right = 3usize;
        let partial_left = 1usize;
        let partial_right = 3usize;
        let expected_full_sum = Some(0i64);
        let expected_partial_sum = Some(0i64);
        let tree = PersistentSegmentTree::new(tree_size);

        // Act
        let actual_full_sum = tree.sum(full_left, full_right);
        let actual_partial_sum = tree.sum(partial_left, partial_right);

        // Assert
        assert_eq!(actual_full_sum, expected_full_sum);
        assert_eq!(actual_partial_sum, expected_partial_sum);
    }

    #[test]
    fn new_with_nonzero_identity_returns_identity_for_full_interval() {
        // Arrange
        let size = 5;
        let (left, right) = (0, 5);
        let expected = Some(1);

        // Act
        let tree = GenericPersistentSegmentTree::<ProductMonoid>::new(size);
        let actual = tree.query(left, right);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn new_with_nonzero_identity_returns_identity_for_empty_interval() {
        // Arrange
        let size = 5;
        let (left, right) = (3, 3);
        let expected = Some(1);

        // Act
        let tree = GenericPersistentSegmentTree::<ProductMonoid>::new(size);
        let actual = tree.query(left, right);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn new_with_nonzero_identity_returns_identity_for_nonempty_partial_interval() {
        // Arrange
        let size = 5;
        let (left, right) = (1, 4);
        let expected = Some(1);

        // Act
        let tree = GenericPersistentSegmentTree::<ProductMonoid>::new(size);
        let actual = tree.query(left, right);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn from_slice_multiple_values_returns_length_and_full_and_partial_sums() {
        // Arrange
        let values = [1, 2, 3, 4];
        let intervals = [(0, 4), (1, 3)];
        let expected_len = values.len();
        let expected_sums = vec![Some(10), Some(5)];

        // Act
        let tree = PersistentSegmentTree::from_slice(&values);
        let actual_len = tree.len();
        let actual_sums = observe_intervals(&tree, &intervals);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_sums, expected_sums);
    }

    #[test]
    fn from_slice_empty_returns_empty_tree_and_zero_sum() {
        // Arrange
        let values: [i64; 0] = [];
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_sum = Some(0i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_sum = tree.sum(query_left, query_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn from_slice_singleton_returns_value_for_single_point_and_zero_for_empty_interval() {
        // Arrange
        let only_value = 7i64;
        let values = [only_value];
        let point_left = 0usize;
        let point_right = 1usize;
        let empty_left = 1usize;
        let empty_right = 1usize;
        let expected_point_sum = Some(only_value);
        let expected_empty_sum = Some(0i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_point_sum = tree.sum(point_left, point_right);
        let actual_empty_sum = tree.sum(empty_left, empty_right);

        // Assert
        assert_eq!(actual_point_sum, expected_point_sum);
        assert_eq!(actual_empty_sum, expected_empty_sum);
    }

    #[test]
    fn from_slice_boundary_triplet_matches_full_and_point_sums() {
        // Arrange
        let center_value = 0i64;
        let values = [center_value - 1, center_value, center_value + 1];
        let full_left = 0usize;
        let full_right = 3usize;
        let left_point_left = 0usize;
        let left_point_right = 1usize;
        let center_point_left = 1usize;
        let center_point_right = 2usize;
        let right_point_left = 2usize;
        let right_point_right = 3usize;
        let expected_full_sum = Some(0i64);
        let expected_left_point_sum = Some(-1i64);
        let expected_center_point_sum = Some(0i64);
        let expected_right_point_sum = Some(1i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_full_sum = tree.sum(full_left, full_right);
        let actual_left_point_sum = tree.sum(left_point_left, left_point_right);
        let actual_center_point_sum = tree.sum(center_point_left, center_point_right);
        let actual_right_point_sum = tree.sum(right_point_left, right_point_right);

        // Assert
        assert_eq!(actual_full_sum, expected_full_sum);
        assert_eq!(actual_left_point_sum, expected_left_point_sum);
        assert_eq!(actual_center_point_sum, expected_center_point_sum);
        assert_eq!(actual_right_point_sum, expected_right_point_sum);
    }

    #[test]
    fn non_power_of_two_tree_satisfies_node_invariants_after_construction() {
        // Arrange
        let values = [2, 7, 1, 8, 2];
        let expected_leaf_count = values.len();
        let expected_diagnostics = Vec::<String>::new();

        // Act
        let observed_tree = PersistentSegmentTree::from_slice(&values);
        let actual_len = observed_tree.len();
        let (actual_leaf_count, actual_diagnostics) = inspect_sum_node_invariants(
            observed_tree
                .root
                .as_ref()
                .expect("nonempty tree should have a root"),
        );

        // Assert
        assert_eq!(actual_leaf_count, expected_leaf_count);
        assert_eq!(actual_len, expected_leaf_count);
        assert_eq!(actual_diagnostics, expected_diagnostics);
    }

    #[test]
    fn new_empty_tree_returns_identity_and_rejects_point_updates() {
        // Arrange
        let size = 0;
        let interval = (0, 0);
        let update = (0, 1);
        let expected_empty = true;
        let expected_sum = Some(0);
        let expected_update_missing = true;

        // Act
        let tree = PersistentSegmentTree::new(size);
        let actual_empty = tree.is_empty();
        let actual_sum = tree.sum(interval.0, interval.1);
        let actual_update = tree.set(update.0, update.1);
        let actual_update_missing = actual_update.is_none();

        // Assert
        assert_eq!(actual_empty, expected_empty);
        assert_eq!(actual_sum, expected_sum);
        assert_eq!(actual_update_missing, expected_update_missing);
    }

    #[test]
    fn non_power_of_two_tree_satisfies_node_invariants_after_update() {
        // Arrange
        let values = [2, 7, 1, 8, 2];
        let expected_leaf_count = values.len();
        let expected_diagnostics = Vec::<String>::new();
        let update = (3, -9);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let observed_tree = tree
            .set(update.0, update.1)
            .expect("in-range update should succeed");
        let actual_len = observed_tree.len();
        let (actual_leaf_count, actual_diagnostics) = inspect_sum_node_invariants(
            observed_tree
                .root
                .as_ref()
                .expect("nonempty tree should have a root"),
        );

        // Assert
        assert_eq!(actual_leaf_count, expected_leaf_count);
        assert_eq!(actual_len, expected_leaf_count);
        assert_eq!(actual_diagnostics, expected_diagnostics);
    }

    #[test]
    fn set_first_index_updates_point_and_full_sum() {
        // Arrange
        let values = [10i64, 20i64, 30i64];
        let update_index = 0usize;
        let update_value = 7i64;
        let point_left = 0usize;
        let point_right = 1usize;
        let full_left = 0usize;
        let full_right = 3usize;
        let expected_point_sum = Some(7i64);
        let expected_full_sum = Some(57i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let updated_tree = tree
            .set(update_index, update_value)
            .expect("set(0, value) should return updated tree");
        let actual_point_sum = updated_tree.sum(point_left, point_right);
        let actual_full_sum = updated_tree.sum(full_left, full_right);

        // Assert
        assert_eq!(actual_point_sum, expected_point_sum);
        assert_eq!(actual_full_sum, expected_full_sum);
    }

    #[test]
    fn set_last_index_updates_point_and_full_sum() {
        // Arrange
        let values = [10i64, 20i64, 30i64];
        let update_index = 2usize;
        let update_value = 5i64;
        let point_left = 2usize;
        let point_right = 3usize;
        let full_left = 0usize;
        let full_right = 3usize;
        let expected_point_sum = Some(5i64);
        let expected_full_sum = Some(35i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let updated_tree = tree
            .set(update_index, update_value)
            .expect("set(len - 1, value) should return updated tree");
        let actual_point_sum = updated_tree.sum(point_left, point_right);
        let actual_full_sum = updated_tree.sum(full_left, full_right);

        // Assert
        assert_eq!(actual_point_sum, expected_point_sum);
        assert_eq!(actual_full_sum, expected_full_sum);
    }

    #[test]
    fn set_len_index_returns_none_and_keeps_original_version() {
        // Arrange
        let values = [1i64, 2i64, 3i64];
        let update_index = 3usize;
        let update_value = 10i64;
        let full_left = 0usize;
        let full_right = 3usize;
        let expected_set_result_is_none = true;
        let expected_original_sum_before = Some(6i64);
        let expected_original_sum_after = Some(6i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_original_sum_before = tree.sum(full_left, full_right);
        let actual_set_result = tree.set(update_index, update_value);
        let actual_set_result_is_none = actual_set_result.is_none();
        let actual_original_sum_after = tree.sum(full_left, full_right);

        // Assert
        assert_eq!(actual_set_result_is_none, expected_set_result_is_none);
        assert_eq!(actual_original_sum_before, expected_original_sum_before);
        assert_eq!(actual_original_sum_after, expected_original_sum_after);
    }

    #[test]
    fn set_usize_max_returns_none_without_panic() {
        // Arrange
        let values = [1i64, 2i64, 3i64];
        let update_index = usize::MAX;
        let update_value = 5i64;
        let expected_set_result_is_none = true;
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_set_result = tree.set(update_index, update_value);
        let actual_set_result_is_none = actual_set_result.is_none();

        // Assert
        assert_eq!(actual_set_result_is_none, expected_set_result_is_none);
    }

    #[test]
    fn set_from_same_base_creates_independent_branches() {
        // Arrange
        let values = [1i64, 2i64, 3i64];
        let first_branch_index = 0usize;
        let first_branch_value = 10i64;
        let second_branch_index = 2usize;
        let second_branch_value = 20i64;
        let first_branch_unique_left = 2usize;
        let first_branch_unique_right = 3usize;
        let second_branch_unique_left = 0usize;
        let second_branch_unique_right = 1usize;
        let expected_first_branch_unique_sum = Some(3i64);
        let expected_second_branch_unique_sum = Some(1i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let first_branch_tree = tree
            .set(first_branch_index, first_branch_value)
            .expect("first branch update should succeed");
        let second_branch_tree = tree
            .set(second_branch_index, second_branch_value)
            .expect("second branch update should succeed");
        let actual_first_branch_unique_sum =
            first_branch_tree.sum(first_branch_unique_left, first_branch_unique_right);
        let actual_second_branch_unique_sum =
            second_branch_tree.sum(second_branch_unique_left, second_branch_unique_right);

        // Assert
        assert_eq!(
            actual_first_branch_unique_sum,
            expected_first_branch_unique_sum
        );
        assert_eq!(
            actual_second_branch_unique_sum,
            expected_second_branch_unique_sum
        );
    }

    #[test]
    fn set_same_index_twice_keeps_each_version_value() {
        // Arrange
        let values = [5i64, 6i64];
        let update_index = 1usize;
        let first_value = 10i64;
        let second_value = -4i64;
        let point_left = 1usize;
        let point_right = 2usize;
        let expected_first_version_sum = Some(10i64);
        let expected_second_version_sum = Some(-4i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let first_updated_tree = tree
            .set(update_index, first_value)
            .expect("first update should succeed");
        let second_updated_tree = first_updated_tree
            .set(update_index, second_value)
            .expect("second update should succeed");
        let actual_first_version_sum = first_updated_tree.sum(point_left, point_right);
        let actual_second_version_sum = second_updated_tree.sum(point_left, point_right);

        // Assert
        assert_eq!(actual_first_version_sum, expected_first_version_sum);
        assert_eq!(actual_second_version_sum, expected_second_version_sum);
    }

    #[test]
    fn set_does_not_change_non_overlapping_interval_sum() {
        // Arrange
        let values = [1i64, 2i64, 3i64, 4i64, 5i64];
        let update_index = 3usize;
        let update_value = 100i64;
        let non_overlapping_left = 0usize;
        let non_overlapping_right = 2usize;
        let expected_sum_before = Some(3i64);
        let expected_sum_after = Some(3i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_sum_before = tree.sum(non_overlapping_left, non_overlapping_right);
        let updated_tree = tree
            .set(update_index, update_value)
            .expect("update in range should succeed");
        let actual_sum_after = updated_tree.sum(non_overlapping_left, non_overlapping_right);

        // Assert
        assert_eq!(actual_sum_before, expected_sum_before);
        assert_eq!(actual_sum_after, expected_sum_after);
    }

    #[test]
    fn set_multiple_times_matches_naive_array_total_sum() {
        // Arrange
        let initial_values = [1i64, 2i64, 3i64, 4i64];
        let first_index = 1usize;
        let first_value = 10i64;
        let second_index = 3usize;
        let second_value = -2i64;
        let third_index = 0usize;
        let third_value = 5i64;
        let full_left = 0usize;
        let full_right = 4usize;
        let expected_total_sum = Some(16i64);
        let tree = PersistentSegmentTree::from_slice(&initial_values);

        // Act
        let updated_tree_first = tree
            .set(first_index, first_value)
            .expect("first update should succeed");
        let updated_tree_second = updated_tree_first
            .set(second_index, second_value)
            .expect("second update should succeed");
        let updated_tree_third = updated_tree_second
            .set(third_index, third_value)
            .expect("third update should succeed");
        let actual_total_sum = updated_tree_third.sum(full_left, full_right);

        // Assert
        assert_eq!(actual_total_sum, expected_total_sum);
    }

    #[test]
    fn set_element_back_to_identity_reaggregates_without_changing_old_version() {
        // Arrange
        let values = [2, 3, 5, 7];
        let update_index = 1;
        let update_value = ProductMonoid::id();
        let updated_intervals = [(1, 2), (0, 3), (0, 4)];
        let original_intervals = [(1, 2), (0, 4)];
        let expected_updated = vec![Some(1), Some(10), Some(70)];
        let expected_original = vec![Some(3), Some(210)];
        let tree = GenericPersistentSegmentTree::<ProductMonoid>::from_slice(&values);

        // Act
        let updated_tree = tree
            .set(update_index, update_value)
            .expect("in-range update should succeed");
        let actual_updated: Vec<_> = updated_intervals
            .into_iter()
            .map(|(left, right)| updated_tree.query(left, right))
            .collect();
        let actual_original: Vec<_> = original_intervals
            .into_iter()
            .map(|(left, right)| tree.query(left, right))
            .collect();

        // Assert
        assert_eq!(actual_updated, expected_updated);
        assert_eq!(actual_original, expected_original);
    }

    #[test]
    fn every_interval_of_each_non_power_of_two_version_matches_naive_array() {
        // Arrange
        let initial_values = [3, -1, 4, 1, 5];
        let updates = [(1, 8), (4, -2), (0, 0)];
        let reference_versions = [
            initial_values,
            [3, 8, 4, 1, 5],
            [3, 8, 4, 1, -2],
            [0, 8, 4, 1, -2],
        ];
        let cases: Vec<_> = reference_versions
            .iter()
            .map(|values| interval_cases(values))
            .collect();
        let intervals: Vec<_> = cases[0].iter().map(|(interval, _)| *interval).collect();
        let expected: Vec<Vec<_>> = cases
            .iter()
            .map(|version| version.iter().map(|(_, expected)| *expected).collect())
            .collect();

        // Act
        let mut versions = vec![PersistentSegmentTree::from_slice(&initial_values)];
        let mut actual_at_creation = vec![observe_intervals(&versions[0], &intervals)];
        for (index, value) in updates {
            let updated = versions
                .last()
                .unwrap()
                .set(index, value)
                .expect("in-range update should succeed");
            actual_at_creation.push(observe_intervals(&updated, &intervals));
            versions.push(updated);
        }
        let actual_after_updates: Vec<_> = versions
            .iter()
            .map(|tree| observe_intervals(tree, &intervals))
            .collect();

        // Assert
        assert_eq!(actual_at_creation, expected);
        assert_eq!(actual_after_updates, expected);
    }

    #[test]
    fn branches_from_middle_version_keep_their_updated_and_untouched_points() {
        // Arrange
        let base_values = [1, 2, 3, 4, 5];
        let middle_update = (2, 30);
        let left_update = (0, 10);
        let right_update = (4, 50);
        let reference_versions = [
            base_values,
            [1, 2, 30, 4, 5],
            [10, 2, 30, 4, 5],
            [1, 2, 30, 4, 50],
        ];
        let point_intervals: Vec<_> = (0..base_values.len())
            .map(|index| (index, index + 1))
            .collect();
        let expected: Vec<Vec<_>> = reference_versions
            .iter()
            .map(|values| values.iter().copied().map(Some).collect())
            .collect();
        let base_tree = PersistentSegmentTree::from_slice(&base_values);

        // Act
        let middle_tree = base_tree
            .set(middle_update.0, middle_update.1)
            .expect("middle update should succeed");
        let left_branch = middle_tree
            .set(left_update.0, left_update.1)
            .expect("left branch update should succeed");
        let right_branch = middle_tree
            .set(right_update.0, right_update.1)
            .expect("right branch update should succeed");
        let actual: Vec<_> = [&base_tree, &middle_tree, &left_branch, &right_branch]
            .into_iter()
            .map(|tree| observe_intervals(tree, &point_intervals))
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn set_on_singleton_keeps_old_and_new_values_and_metadata() {
        // Arrange
        let values = [11];
        let update = (0, -4);
        let interval = (0, 1);
        let expected_updated = (Some(-4), 1, false);
        let expected_original = (Some(11), 1, false);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let updated = tree
            .set(update.0, update.1)
            .expect("singleton update should succeed");
        let actual_updated = (
            updated.query(interval.0, interval.1),
            updated.len(),
            updated.is_empty(),
        );
        let actual_original = (
            tree.query(interval.0, interval.1),
            tree.len(),
            tree.is_empty(),
        );

        // Assert
        assert_eq!(actual_updated, expected_updated);
        assert_eq!(actual_original, expected_original);
    }

    #[test]
    fn set_clone_and_allocation_counts_follow_height_and_share_untouched_nodes() {
        // Arrange
        let sizes = [2usize, 4, 8, 16, 32, 64];
        let cases: Vec<_> = sizes
            .into_iter()
            .map(|size| {
                let values = (0..size)
                    .map(|value| CloneCountedValue(value as i64))
                    .collect::<Vec<_>>();
                let tree =
                    GenericPersistentSegmentTree::<CloneCountedSumMonoid>::from_slice(&values);
                let height = size.trailing_zeros() as usize;
                let expected = (
                    2 * height,
                    height + 1,
                    2 * size - 1,
                    2 * size - 1,
                    2 * size - 1 - (height + 1),
                );
                (tree, size / 2, CloneCountedValue(-1), expected)
            })
            .collect();
        let expected: Vec<_> = cases.iter().map(|case| case.3).collect();

        // Act
        let actual: Vec<_> = cases
            .into_iter()
            .map(|(tree, index, value, _)| {
                VALUE_CLONE_COUNT.store(0, Ordering::SeqCst);
                let updated = tree
                    .set(index, value)
                    .expect("in-range update should succeed");
                let clone_count = VALUE_CLONE_COUNT.load(Ordering::SeqCst);
                let mut old_nodes = HashSet::new();
                collect_node_addresses(&tree.root, &mut old_nodes);
                let mut new_nodes = HashSet::new();
                collect_node_addresses(&updated.root, &mut new_nodes);
                (
                    clone_count,
                    new_nodes.difference(&old_nodes).count(),
                    old_nodes.len(),
                    new_nodes.len(),
                    new_nodes.intersection(&old_nodes).count(),
                )
            })
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn successive_updates_preserve_prior_version_interval_sums() {
        // Arrange
        let values = [1, 2, 3];
        let first_update = (1, 5);
        let second_update = (0, -1);
        let full_interval = (0, 3);
        let partial_interval = (0, 2);
        let expected_full = (Some(6), Some(9));
        let expected_partial = (Some(4), Some(6));
        let initial = PersistentSegmentTree::from_slice(&values);

        // Act
        let first = initial.set(first_update.0, first_update.1).unwrap();
        let actual_full = (
            initial.sum(full_interval.0, full_interval.1),
            first.sum(full_interval.0, full_interval.1),
        );
        let second = first.set(second_update.0, second_update.1).unwrap();
        let actual_partial = (
            second.sum(partial_interval.0, partial_interval.1),
            first.sum(partial_interval.0, partial_interval.1),
        );

        // Assert
        assert_eq!(actual_full, expected_full);
        assert_eq!(actual_partial, expected_partial);
    }

    #[test]
    fn query_with_string_concatenation_preserves_order_and_original_version_after_update() {
        // Arrange
        let values = vec!["ab".to_string(), "cd".to_string(), "ef".to_string()];
        let update_index = 1usize;
        let update_value = "XX".to_string();
        let query_left = 0usize;
        let query_right = 3usize;
        let expected_before_update = Some("abcdef".to_string());
        let expected_after_update = Some("abXXef".to_string());
        let tree = GenericPersistentSegmentTree::<StringConcatMonoid>::from_slice(&values);

        // Act
        let actual_before_update = tree.query(query_left, query_right);
        let updated_tree = tree
            .set(update_index, update_value)
            .expect("update in range should succeed");
        let actual_after_update = updated_tree.query(query_left, query_right);
        let actual_original_after_update = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_before_update, expected_before_update);
        assert_eq!(actual_after_update, expected_after_update);
        assert_eq!(actual_original_after_update, expected_before_update);
    }

    #[test]
    fn query_non_commutative_interval_crossing_split_keeps_left_to_right_order() {
        // Arrange
        let values = ["a", "b", "c", "d", "e"].map(|value| value.to_string());
        let interval = (1, 4);
        let expected = Some("bcd".to_string());
        let tree = GenericPersistentSegmentTree::<StringConcatMonoid>::from_slice(&values);

        // Act
        let actual = tree.query(interval.0, interval.1);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_with_usize_max_right_returns_none_without_panic() {
        // Arrange
        let values = [1, 2, 3];
        let interval = (0, usize::MAX);
        let expected: Option<i64> = None;
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual = tree.query(interval.0, interval.1);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_with_both_bounds_usize_max_returns_none_without_panic() {
        // Arrange
        let values = [1, 2, 3];
        let interval = (usize::MAX, usize::MAX);
        let expected: Option<i64> = None;
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual = tree.query(interval.0, interval.1);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn query_and_sum_match_for_valid_nonempty_interval() {
        // Arrange
        let values = [3, 1, 4, 1, 5];
        let interval = (1, 4);
        let expected: Option<i64> = Some(6);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let query_result = tree.query(interval.0, interval.1);
        let sum_result = tree.sum(interval.0, interval.1);

        // Assert
        assert_eq!(query_result, expected);
        assert_eq!(sum_result, expected);
    }

    #[test]
    fn query_and_sum_match_for_valid_empty_interval() {
        // Arrange
        let values = [3, 1, 4, 1, 5];
        let interval = (3, 3);
        let expected: Option<i64> = Some(0);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let query_result = tree.query(interval.0, interval.1);
        let sum_result = tree.sum(interval.0, interval.1);

        // Assert
        assert_eq!(query_result, expected);
        assert_eq!(sum_result, expected);
    }

    #[test]
    fn query_and_sum_both_return_none_for_invalid_interval() {
        // Arrange
        let values = [3, 1, 4, 1, 5];
        let interval = (4, 2);
        let expected: Option<i64> = None;
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let query_result = tree.query(interval.0, interval.1);
        let sum_result = tree.sum(interval.0, interval.1);

        // Assert
        assert_eq!(query_result, expected);
        assert_eq!(sum_result, expected);
    }

    #[test]
    fn sum_out_of_range_returns_none() {
        // Arrange
        let size = 2;
        let intervals = [(2, 1), (0, 3)];
        let expected = vec![None, None];
        let tree = PersistentSegmentTree::new(size);

        // Act
        let actual = observe_intervals(&tree, &intervals);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn sum_empty_interval_returns_zero_for_start_middle_and_end() {
        // Arrange
        let values = [3i64, 1i64, 4i64, 1i64, 5i64, 9i64];
        let first_left = 0usize;
        let first_right = 0usize;
        let second_left = 3usize;
        let second_right = 3usize;
        let third_left = 6usize;
        let third_right = 6usize;
        let expected_first_sum = Some(0i64);
        let expected_second_sum = Some(0i64);
        let expected_third_sum = Some(0i64);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let actual_first_sum = tree.sum(first_left, first_right);
        let actual_second_sum = tree.sum(second_left, second_right);
        let actual_third_sum = tree.sum(third_left, third_right);

        // Assert
        assert_eq!(actual_first_sum, expected_first_sum);
        assert_eq!(actual_second_sum, expected_second_sum);
        assert_eq!(actual_third_sum, expected_third_sum);
    }

    #[test]
    fn sum_boundary_triplet_around_updated_index_matches_expected() {
        // Arrange
        let tree_size = 5usize;
        let update_index = 2usize;
        let update_value = 7i64;
        let left_interval_left = 1usize;
        let left_interval_right = 2usize;
        let center_interval_left = 2usize;
        let center_interval_right = 3usize;
        let around_interval_left = 1usize;
        let around_interval_right = 3usize;
        let expected_left_sum = Some(0i64);
        let expected_center_sum = Some(7i64);
        let expected_around_sum = Some(7i64);
        let tree = PersistentSegmentTree::new(tree_size);

        // Act
        let updated_tree = tree
            .set(update_index, update_value)
            .expect("update in range should succeed");
        let actual_left_sum = updated_tree.sum(left_interval_left, left_interval_right);
        let actual_center_sum = updated_tree.sum(center_interval_left, center_interval_right);
        let actual_around_sum = updated_tree.sum(around_interval_left, around_interval_right);

        // Assert
        assert_eq!(actual_left_sum, expected_left_sum);
        assert_eq!(actual_center_sum, expected_center_sum);
        assert_eq!(actual_around_sum, expected_around_sum);
    }

    #[test]
    fn sum_accepts_right_equal_len_and_rejects_right_greater_than_len() {
        // Arrange
        let tree_size = 4usize;
        let valid_left = 0usize;
        let valid_right = 4usize;
        let invalid_left = 0usize;
        let invalid_right = 5usize;
        let expected_valid_sum = Some(0i64);
        let expected_invalid_sum = None;
        let tree = PersistentSegmentTree::new(tree_size);

        // Act
        let actual_valid_sum = tree.sum(valid_left, valid_right);
        let actual_invalid_sum = tree.sum(invalid_left, invalid_right);

        // Assert
        assert_eq!(actual_valid_sum, expected_valid_sum);
        assert_eq!(actual_invalid_sum, expected_invalid_sum);
    }

    #[test]
    fn sum_returns_none_for_left_greater_than_right_and_len_plus_one_interval() {
        // Arrange
        let tree_size = 3usize;
        let reversed_left = 1usize;
        let reversed_right = 0usize;
        let out_of_upper_left = 4usize;
        let out_of_upper_right = 4usize;
        let expected_reversed_sum = None;
        let expected_out_of_upper_sum = None;
        let tree = PersistentSegmentTree::new(tree_size);

        // Act
        let actual_reversed_sum = tree.sum(reversed_left, reversed_right);
        let actual_out_of_upper_sum = tree.sum(out_of_upper_left, out_of_upper_right);

        // Assert
        assert_eq!(actual_reversed_sum, expected_reversed_sum);
        assert_eq!(actual_out_of_upper_sum, expected_out_of_upper_sum);
    }

    #[test]
    fn updating_a_cloned_tree_keeps_original_intervals_and_metadata_unchanged() {
        // Arrange
        let values = [2, 7, 1, 8, 2];
        let update = (2, 100);
        let point_interval = (2, 3);
        let cases = interval_cases(&values);
        let intervals: Vec<_> = cases.iter().map(|(interval, _)| *interval).collect();
        let expected_intervals: Vec<_> = cases.iter().map(|(_, expected)| *expected).collect();
        let expected_metadata = (values.len(), false);
        let expected_shared_root = true;
        let expected_updated = Some(100);
        let tree = PersistentSegmentTree::from_slice(&values);

        // Act
        let cloned = tree.clone();
        let actual_shared_root =
            Arc::ptr_eq(tree.root.as_ref().unwrap(), cloned.root.as_ref().unwrap());
        let updated = cloned
            .set(update.0, update.1)
            .expect("update through clone should succeed");
        let actual_updated = updated.query(point_interval.0, point_interval.1);
        let actual_original = observe_intervals(&tree, &intervals);
        let actual_cloned = observe_intervals(&cloned, &intervals);
        let actual_metadata = (tree.len(), tree.is_empty());

        // Assert
        assert_eq!(actual_shared_root, expected_shared_root);
        assert_eq!(actual_updated, expected_updated);
        assert_eq!(actual_original, expected_intervals);
        assert_eq!(actual_cloned, expected_intervals);
        assert_eq!(actual_metadata, expected_metadata);
    }

    #[test]
    fn numeric_tree_preserves_send_and_sync_auto_traits() {
        fn assert_send_sync<T: Send + Sync>() {}

        assert_send_sync::<PersistentSegmentTree>();
    }
}
