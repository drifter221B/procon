use std::ops::Add;

pub trait Monoid {
    type S: Clone;

    fn id() -> Self::S;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
}

pub struct DynamicSegmentTree<M>
where
    M: Monoid,
{
    size: usize,
    root: Option<Box<Node<M>>>,
}

struct Node<M>
where
    M: Monoid,
{
    value: M::S,
    left: Option<Box<Node<M>>>,
    right: Option<Box<Node<M>>>,
}

impl<M> Node<M>
where
    M: Monoid,
{
    fn new() -> Self {
        Self {
            value: M::id(),
            left: None,
            right: None,
        }
    }

    fn value(node: &Option<Box<Self>>) -> M::S {
        node.as_ref()
            .map(|current| current.value.clone())
            .unwrap_or_else(M::id)
    }

    fn recalculate(&mut self) {
        self.value = M::binary_operation(&Self::value(&self.left), &Self::value(&self.right));
    }
}

impl<M> DynamicSegmentTree<M>
where
    M: Monoid,
{
    pub fn new(size: usize) -> Self {
        Self { size, root: None }
    }

    pub fn from_slice(values: &[M::S]) -> Self {
        Self {
            size: values.len(),
            root: Self::build(values, 0, values.len()),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    pub fn select(&self, index: usize) -> Option<M::S> {
        if self.size <= index {
            return None;
        }
        Some(Self::query_inner(
            self.root.as_deref(),
            0,
            self.size,
            index,
            index + 1,
        ))
    }

    #[inline]
    pub fn select_all(&self) -> M::S {
        self.root
            .as_ref()
            .map(|node| node.value.clone())
            .unwrap_or_else(M::id)
    }

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

    pub fn update(&mut self, index: usize, value: M::S) -> Option<()> {
        if self.size <= index {
            return None;
        }
        Self::update_inner(&mut self.root, 0, self.size, index, value);
        Some(())
    }

    pub fn add(&mut self, index: usize, value: M::S) -> Option<()>
    where
        M::S: Add<Output = M::S>,
    {
        let current = self.select(index)?;
        self.update(index, current + value)
    }

    // query_left is inclusive, returned index is not inclusive
    pub fn max_right<P>(&self, query_left: usize, func: P) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        if self.size < query_left {
            return None;
        }
        if query_left == self.size {
            return Some(self.size);
        }

        assert!(func(&M::id()));

        let mut result = M::id();
        Self::max_right_inner(
            self.root.as_deref(),
            0,
            self.size,
            query_left,
            &func,
            &mut result,
        )
        .or(Some(self.size))
    }

    // query_right is not inclusive, returned index is inclusive
    pub fn min_left<P>(&self, query_right: usize, func: P) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        if self.size < query_right {
            return None;
        }
        if query_right == 0 {
            return Some(0);
        }

        assert!(func(&M::id()));

        let mut result = M::id();
        Self::min_left_inner(
            self.root.as_deref(),
            0,
            self.size,
            query_right,
            &func,
            &mut result,
        )
        .or(Some(0))
    }

    fn build(values: &[M::S], segment_left: usize, segment_right: usize) -> Option<Box<Node<M>>> {
        if segment_left == segment_right {
            return None;
        }

        if segment_left + 1 == segment_right {
            return Some(Box::new(Node {
                value: values[segment_left].clone(),
                left: None,
                right: None,
            }));
        }

        let segment_mid = Self::midpoint(segment_left, segment_right);
        let left = Self::build(values, segment_left, segment_mid);
        let right = Self::build(values, segment_mid, segment_right);
        let mut node = Node {
            value: M::id(),
            left,
            right,
        };
        node.recalculate();
        Some(Box::new(node))
    }

    fn query_inner(
        node: Option<&Node<M>>,
        segment_left: usize,
        segment_right: usize,
        query_left: usize,
        query_right: usize,
    ) -> M::S {
        let Some(node) = node else {
            return M::id();
        };

        if query_right <= segment_left || segment_right <= query_left {
            return M::id();
        }
        if query_left <= segment_left && segment_right <= query_right {
            return node.value.clone();
        }

        let segment_mid = Self::midpoint(segment_left, segment_right);
        let left_value = Self::query_inner(
            node.left.as_deref(),
            segment_left,
            segment_mid,
            query_left,
            query_right,
        );
        let right_value = Self::query_inner(
            node.right.as_deref(),
            segment_mid,
            segment_right,
            query_left,
            query_right,
        );
        M::binary_operation(&left_value, &right_value)
    }

    fn update_inner(
        node: &mut Option<Box<Node<M>>>,
        segment_left: usize,
        segment_right: usize,
        index: usize,
        value: M::S,
    ) {
        let current = node.get_or_insert_with(|| Box::new(Node::new()));

        if segment_left + 1 == segment_right {
            current.value = value;
            return;
        }

        let segment_mid = Self::midpoint(segment_left, segment_right);
        if index < segment_mid {
            Self::update_inner(&mut current.left, segment_left, segment_mid, index, value);
        } else {
            Self::update_inner(&mut current.right, segment_mid, segment_right, index, value);
        }
        current.recalculate();
    }

    fn max_right_inner<P>(
        node: Option<&Node<M>>,
        segment_left: usize,
        segment_right: usize,
        query_left: usize,
        func: &P,
        result: &mut M::S,
    ) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        let Some(node) = node else {
            return None;
        };

        if segment_right <= query_left {
            return None;
        }
        if query_left <= segment_left {
            let next = M::binary_operation(result, &node.value);
            if func(&next) {
                *result = next;
                return None;
            }
            if segment_left + 1 == segment_right {
                return Some(segment_left);
            }
        }

        let segment_mid = Self::midpoint(segment_left, segment_right);
        Self::max_right_inner(
            node.left.as_deref(),
            segment_left,
            segment_mid,
            query_left,
            func,
            result,
        )
        .or_else(|| {
            Self::max_right_inner(
                node.right.as_deref(),
                segment_mid,
                segment_right,
                query_left,
                func,
                result,
            )
        })
    }

    fn min_left_inner<P>(
        node: Option<&Node<M>>,
        segment_left: usize,
        segment_right: usize,
        query_right: usize,
        func: &P,
        result: &mut M::S,
    ) -> Option<usize>
    where
        P: Fn(&M::S) -> bool,
    {
        let Some(node) = node else {
            return None;
        };

        if query_right <= segment_left {
            return None;
        }
        if segment_right <= query_right {
            let next = M::binary_operation(&node.value, result);
            if func(&next) {
                *result = next;
                return None;
            }
            if segment_left + 1 == segment_right {
                return Some(segment_right);
            }
        }

        let segment_mid = Self::midpoint(segment_left, segment_right);
        Self::min_left_inner(
            node.right.as_deref(),
            segment_mid,
            segment_right,
            query_right,
            func,
            result,
        )
        .or_else(|| {
            Self::min_left_inner(
                node.left.as_deref(),
                segment_left,
                segment_mid,
                query_right,
                func,
                result,
            )
        })
    }

    fn midpoint(left: usize, right: usize) -> usize {
        left + ((right - left) >> 1)
    }
}

#[cfg(test)]
mod tests {
    use super::{DynamicSegmentTree as GenericDynamicSegmentTree, Monoid};

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
            let mut value = lhs.clone();
            value.push_str(rhs);
            value
        }
    }

    type DynamicSegmentTree = GenericDynamicSegmentTree<SumMonoid>;

    fn tree_from_values(values: &[i64]) -> DynamicSegmentTree {
        let mut tree = DynamicSegmentTree::new(values.len());
        for (index, value) in values.iter().copied().enumerate() {
            let add_result = tree.add(index, value);
            assert_eq!(add_result, Some(()));
        }
        tree
    }

    fn naive_sum(values: &[i64], left: usize, right: usize) -> i64 {
        values[left..right].iter().copied().sum()
    }

    fn naive_max_right(values: &[i64], left: usize, upper_bound: i64) -> usize {
        let mut sum = 0;
        for (offset, value) in values[left..].iter().copied().enumerate() {
            if sum + value > upper_bound {
                return left + offset;
            }
            sum += value;
        }
        values.len()
    }

    fn naive_min_left(values: &[i64], right: usize, upper_bound: i64) -> usize {
        let mut sum = 0;
        for index in (0..right).rev() {
            if values[index] + sum > upper_bound {
                return index + 1;
            }
            sum += values[index];
        }
        0
    }

    fn next_random(seed: &mut u64) -> u64 {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *seed
    }

    #[test]
    fn new_with_size_zero_has_expected_properties_and_zero_sum() {
        // Arrange
        let size = 0;
        let expected_len = 0;
        let expected_is_empty = true;
        let query_left = 0;
        let query_right = 0;
        let expected_sum = Some(0);

        // Act
        let tree = DynamicSegmentTree::new(size);
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_size_one_has_expected_properties_and_zero_sum() {
        // Arrange
        let size = 1;
        let expected_len = 1;
        let expected_is_empty = false;
        let query_left = 0;
        let query_right = 1;
        let expected_sum = Some(0);

        // Act
        let tree = DynamicSegmentTree::new(size);
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn new_with_non_power_of_two_size_keeps_input_length() {
        // Arrange
        let size = 3;
        let expected_len = 3;

        // Act
        let tree = DynamicSegmentTree::new(size);
        let actual_len = tree.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn from_slice_supports_select_select_all_and_query() {
        // Arrange
        let values = [2, 7, -3, 5];
        let selected_index = 2;
        let query_left = 1;
        let query_right = 4;
        let expected_selected = Some(-3);
        let expected_all = 11;
        let expected_query = Some(9);

        // Act
        let tree = DynamicSegmentTree::from_slice(&values);
        let actual_selected = tree.select(selected_index);
        let actual_all = tree.select_all();
        let actual_query = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_selected, expected_selected);
        assert_eq!(actual_all, expected_all);
        assert_eq!(actual_query, expected_query);
    }

    #[test]
    fn from_slice_with_empty_slice_has_expected_empty_properties() {
        // Arrange
        let values = [];
        let expected_len = 0;
        let expected_is_empty = true;
        let expected_selected = None;
        let expected_all = 0;
        let expected_query = Some(0);
        let selected_index = 0;
        let query_range = (0, 0);

        // Act
        let tree = DynamicSegmentTree::from_slice(&values);
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_selected = tree.select(selected_index);
        let actual_all = tree.select_all();
        let actual_query = tree.query(query_range.0, query_range.1);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_selected, expected_selected);
        assert_eq!(actual_all, expected_all);
        assert_eq!(actual_query, expected_query);
    }

    #[test]
    fn from_slice_with_single_value_returns_value_from_all_read_apis() {
        // Arrange
        let values = [13];
        let expected_selected = Some(13);
        let expected_all = 13;
        let expected_query = Some(13);
        let selected_index = 0;
        let query_left = 0;
        let query_right = 1;

        // Act
        let tree = DynamicSegmentTree::from_slice(&values);
        let actual_selected = tree.select(selected_index);
        let actual_all = tree.select_all();
        let actual_query = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_selected, expected_selected);
        assert_eq!(actual_all, expected_all);
        assert_eq!(actual_query, expected_query);
    }

    #[test]
    fn select_returns_identity_for_unupdated_index_and_none_out_of_range() {
        // Arrange
        let size = 4;
        let updated_index = 1;
        let updated_value = 8;
        let unupdated_index = 2;
        let out_of_range_index = 4;
        let expected_unupdated = Some(0);
        let expected_out_of_range = None;

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.update(updated_index, updated_value);
        let actual_unupdated = tree.select(unupdated_index);
        let actual_out_of_range = tree.select(out_of_range_index);

        // Assert
        assert_eq!(actual_unupdated, expected_unupdated);
        assert_eq!(actual_out_of_range, expected_out_of_range);
    }

    #[test]
    fn select_distinguishes_last_valid_index_from_size() {
        // Arrange
        let size = 4;
        let last_index = size - 1;
        let expected_last = Some(0);
        let expected_out_of_range = None;

        // Act
        let tree = DynamicSegmentTree::new(size);
        let actual_last = tree.select(last_index);
        let actual_out_of_range = tree.select(size);

        // Assert
        assert_eq!(actual_last, expected_last);
        assert_eq!(actual_out_of_range, expected_out_of_range);
    }

    #[test]
    fn select_reflects_update_followed_by_add_on_same_index() {
        // Arrange
        let size = 3;
        let index = 1;
        let updated_value = 8;
        let added_value = -3;
        let expected_selected = Some(5);
        let expected_mutation_result = Some(());

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let update_result = tree.update(index, updated_value);
        let add_result = tree.add(index, added_value);
        let actual_selected = tree.select(index);

        // Assert
        assert_eq!(update_result, expected_mutation_result);
        assert_eq!(add_result, expected_mutation_result);
        assert_eq!(actual_selected, expected_selected);
    }

    #[test]
    fn select_all_on_unupdated_tree_returns_identity() {
        // Arrange
        let size = 5;
        let expected_all = 0;

        // Act
        let tree = DynamicSegmentTree::new(size);
        let actual_all = tree.select_all();

        // Assert
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn select_all_matches_full_query_after_multiple_updates() {
        // Arrange
        let size = 5;
        let operations = [(true, 0, 3), (false, 2, 7), (true, 4, -2), (false, 0, 4)];
        let query_left = 0;
        let query_right = size;

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        for (is_update, index, value) in operations {
            if is_update {
                let _ = tree.update(index, value);
            } else {
                let _ = tree.add(index, value);
            }
        }
        let actual_all = tree.select_all();
        let actual_full_query = tree.query(query_left, query_right);

        // Assert
        assert_eq!(Some(actual_all), actual_full_query);
    }

    #[test]
    fn query_on_empty_tree_returns_zero_for_valid_ranges() {
        // Arrange
        let size = 5;
        let expected_first = Some(0);
        let expected_second = Some(0);
        let expected_third = Some(0);
        let first_left = 0;
        let first_right = 0;
        let second_left = 0;
        let second_right = 3;
        let third_left = 5;
        let third_right = 5;

        // Act
        let tree = DynamicSegmentTree::new(size);
        let actual_first = tree.query(first_left, first_right);
        let actual_second = tree.query(second_left, second_right);
        let actual_third = tree.query(third_left, third_right);

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_second, expected_second);
        assert_eq!(actual_third, expected_third);
    }

    #[test]
    fn query_of_point_ranges_returns_each_current_value() {
        // Arrange
        let values = [4, -2, 7];
        let first_left = 0;
        let first_right = 1;
        let second_left = 1;
        let second_right = 2;
        let third_left = 2;
        let third_right = 3;
        let expected_first = Some(4);
        let expected_second = Some(-2);
        let expected_third = Some(7);

        // Act
        let tree = tree_from_values(&values);
        let actual_first = tree.query(first_left, first_right);
        let actual_second = tree.query(second_left, second_right);
        let actual_third = tree.query(third_left, third_right);

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_second, expected_second);
        assert_eq!(actual_third, expected_third);
    }

    #[test]
    fn query_of_multiple_elements_matches_naive_calculation() {
        // Arrange
        let values = [3, -1, 4, 1, 5, -9];
        let query_left = 1;
        let query_right = 5;
        let expected_sum = Some(naive_sum(&values, query_left, query_right));

        // Act
        let tree = tree_from_values(&values);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_x_minus_1_x_x_plus_1_boundaries_are_correct() {
        // Arrange
        let values = [1, 2, 3, 4, 5];
        let x = 2;
        let left_first = x - 1;
        let right_first = x;
        let left_second = x;
        let right_second = x + 1;
        let left_third = x - 1;
        let right_third = x + 1;
        let expected_first = Some(2);
        let expected_second = Some(3);
        let expected_third = Some(5);

        // Act
        let tree = tree_from_values(&values);
        let actual_first = tree.query(left_first, right_first);
        let actual_second = tree.query(left_second, right_second);
        let actual_third = tree.query(left_third, right_third);

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_second, expected_second);
        assert_eq!(actual_third, expected_third);
    }

    #[test]
    fn query_of_left_edge_and_right_edge_ranges_is_correct() {
        // Arrange
        let values = [2, 4, 6, 8];
        let left_edge_left = 0;
        let left_edge_right = 1;
        let right_edge_left = 3;
        let right_edge_right = 4;
        let expected_left_edge = Some(2);
        let expected_right_edge = Some(8);

        // Act
        let tree = tree_from_values(&values);
        let actual_left_edge = tree.query(left_edge_left, left_edge_right);
        let actual_right_edge = tree.query(right_edge_left, right_edge_right);

        // Assert
        assert_eq!(actual_left_edge, expected_left_edge);
        assert_eq!(actual_right_edge, expected_right_edge);
    }

    #[test]
    fn query_of_full_range_is_correct() {
        // Arrange
        let values = [2, 4, 6, 8];
        let query_left = 0;
        let query_right = 4;
        let expected_sum = Some(20);

        // Act
        let tree = tree_from_values(&values);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_of_empty_ranges_returns_zero_for_lower_middle_and_upper_indices() {
        // Arrange
        let values = [1, 3, 5, 7, 9];
        let lower_index = 0;
        let middle_index = 2;
        let upper_index = 5;
        let expected_lower = Some(0);
        let expected_middle = Some(0);
        let expected_upper = Some(0);

        // Act
        let tree = tree_from_values(&values);
        let actual_lower = tree.query(lower_index, lower_index);
        let actual_middle = tree.query(middle_index, middle_index);
        let actual_upper = tree.query(upper_index, upper_index);

        // Assert
        assert_eq!(actual_lower, expected_lower);
        assert_eq!(actual_middle, expected_middle);
        assert_eq!(actual_upper, expected_upper);
    }

    #[test]
    fn query_with_left_greater_than_right_returns_none() {
        // Arrange
        let values = [1, 2, 3, 4, 5];
        let query_left = 3;
        let query_right = 2;
        let expected_sum = None;

        // Act
        let tree = tree_from_values(&values);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn query_with_right_greater_than_size_returns_none() {
        // Arrange
        let values = [1, 2, 3, 4, 5];
        let query_left = 0;
        let query_right = 6;
        let expected_sum = None;

        // Act
        let tree = tree_from_values(&values);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_and_query_support_usize_max_tree_size() {
        // Arrange
        let size = usize::MAX;
        let index = usize::MAX - 1;
        let value = 42;
        let query_left = index;
        let query_right = usize::MAX;
        let expected_update = Some(());
        let expected_query = Some(42);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let actual_update = tree.update(index, value);
        let actual_query = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_update, expected_update);
        assert_eq!(actual_query, expected_query);
    }

    #[test]
    fn update_on_unupdated_index_sets_value() {
        // Arrange
        let size = 3;
        let index = 2;
        let value = 9;
        let query_left = 2;
        let query_right = 3;
        let expected_set_result = Some(());
        let expected_sum = Some(9);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let actual_set_result = tree.update(index, value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_set_result, expected_set_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_over_existing_value_overwrites_instead_of_accumulating() {
        // Arrange
        let size = 3;
        let index = 1;
        let initial_delta = 4;
        let overwritten_value = 7;
        let query_left = 1;
        let query_right = 2;
        let expected_sum = Some(7);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(index, initial_delta);
        let _ = tree.update(index, overwritten_value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_keeps_values_of_other_indices_unchanged() {
        // Arrange
        let values = [1, 2, 3, 4];
        let target_index = 1;
        let target_value = 10;
        let unaffected_left = 0;
        let unaffected_right = 1;
        let unaffected_second_left = 2;
        let unaffected_second_right = 4;
        let expected_first_unaffected_sum = Some(1);
        let expected_second_unaffected_sum = Some(7);

        // Act
        let mut tree = tree_from_values(&values);
        let _ = tree.update(target_index, target_value);
        let actual_first_unaffected_sum = tree.query(unaffected_left, unaffected_right);
        let actual_second_unaffected_sum =
            tree.query(unaffected_second_left, unaffected_second_right);

        // Assert
        assert_eq!(actual_first_unaffected_sum, expected_first_unaffected_sum);
        assert_eq!(actual_second_unaffected_sum, expected_second_unaffected_sum);
    }

    #[test]
    fn update_with_index_equal_to_size_returns_none_and_keeps_state() {
        // Arrange
        let size = 3;
        let initial_index = 0;
        let initial_delta = 5;
        let invalid_index = 3;
        let invalid_value = 1;
        let query_left = 0;
        let query_right = 3;
        let expected_set_result = None;
        let expected_sum = Some(5);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(initial_index, initial_delta);
        let actual_set_result = tree.update(invalid_index, invalid_value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_set_result, expected_set_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_on_zero_sized_tree_returns_none_without_panic() {
        // Arrange
        let size = 0;
        let index = 0;
        let value = 1;
        let query_left = 0;
        let query_right = 0;
        let expected_set_result = None;
        let expected_sum = Some(0);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let actual_set_result = tree.update(index, value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_set_result, expected_set_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_with_negative_value_updates_point_to_negative() {
        // Arrange
        let size = 2;
        let index = 1;
        let value = -8;
        let query_left = 1;
        let query_right = 2;
        let expected_sum = Some(-8);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.update(index, value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_with_usize_max_index_returns_none_without_panic() {
        // Arrange
        let size = 3;
        let valid_index = 0;
        let valid_delta = 3;
        let invalid_index = usize::MAX;
        let invalid_value = 10;
        let query_left = 0;
        let query_right = 3;
        let expected_set_result = None;
        let expected_sum = Some(3);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(valid_index, valid_delta);
        let actual_set_result = tree.update(invalid_index, invalid_value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_set_result, expected_set_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_and_select_support_last_index_of_usize_max_sized_tree() {
        // Arrange
        let size = usize::MAX;
        let index = usize::MAX - 1;
        let value = 42;
        let expected_add = Some(());
        let expected_selected = Some(42);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let actual_add = tree.add(index, value);
        let actual_selected = tree.select(index);

        // Assert
        assert_eq!(actual_add, expected_add);
        assert_eq!(actual_selected, expected_selected);
    }

    #[test]
    fn add_with_zero_delta_keeps_query_result_unchanged() {
        // Arrange
        let size = 3;
        let index = 1;
        let initial_delta = 9;
        let zero_delta = 0;
        let query_left = 0;
        let query_right = 3;
        let expected_before_sum = Some(9);
        let expected_after_sum = Some(9);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(index, initial_delta);
        let actual_before_sum = tree.query(query_left, query_right);
        let _ = tree.add(index, zero_delta);
        let actual_after_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_before_sum, expected_before_sum);
        assert_eq!(actual_after_sum, expected_after_sum);
    }

    #[test]
    fn add_on_empty_tree_returns_some_and_updates_point_sum() {
        // Arrange
        let size = 1;
        let index = 0;
        let delta = 5;
        let expected_add_result = Some(());
        let query_left = 0;
        let query_right = 1;
        let expected_sum = Some(5);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let actual_add_result = tree.add(index, delta);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_multiple_times_on_same_index_accumulates_values() {
        // Arrange
        let size = 2;
        let index = 1;
        let first_delta = 3;
        let second_delta = 4;
        let query_left = 1;
        let query_right = 2;
        let expected_sum = Some(7);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(index, first_delta);
        let _ = tree.add(index, second_delta);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_on_multiple_indices_matches_total_of_point_values() {
        // Arrange
        let size = 4;
        let updates = [(0, 2), (2, -1), (3, 5)];
        let query_left = 0;
        let query_right = 4;
        let expected_sum = Some(6);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        for (index, delta) in updates {
            let _ = tree.add(index, delta);
        }
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_index_equal_to_size_returns_none_and_keeps_state() {
        // Arrange
        let size = 3;
        let valid_index = 0;
        let valid_delta = 1;
        let invalid_index = 3;
        let invalid_delta = 7;
        let query_left = 0;
        let query_right = 3;
        let expected_add_result = None;
        let expected_sum = Some(1);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(valid_index, valid_delta);
        let actual_add_result = tree.add(invalid_index, invalid_delta);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_usize_max_index_returns_none_without_panic() {
        // Arrange
        let size = 3;
        let valid_index = 1;
        let valid_delta = 2;
        let invalid_index = usize::MAX;
        let invalid_delta = 5;
        let query_left = 0;
        let query_right = 3;
        let expected_add_result = None;
        let expected_sum = Some(2);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(valid_index, valid_delta);
        let actual_add_result = tree.add(invalid_index, invalid_delta);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_add_result, expected_add_result);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_negative_delta_decreases_range_sum() {
        // Arrange
        let size = 2;
        let index = 0;
        let first_delta = 10;
        let second_delta = -4;
        let query_left = 0;
        let query_right = 1;
        let expected_sum = Some(6);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.add(index, first_delta);
        let _ = tree.add(index, second_delta);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_on_zero_sized_tree_returns_none_and_keeps_identity() {
        // Arrange
        let size = 0;
        let index = 0;
        let delta = 5;
        let mut tree = DynamicSegmentTree::new(size);
        let expected_add = None;
        let expected_all = 0;

        // Act
        let actual_add = tree.add(index, delta);
        let actual_all = tree.select_all();

        // Assert
        assert_eq!(actual_add, expected_add);
        assert_eq!(actual_all, expected_all);
    }

    #[test]
    fn max_right_at_len_returns_len() {
        // Arrange
        let values = [1, 2, 3];
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = Some(values.len());
        let query_left = values.len();
        let accepts_identity = true;

        // Act
        let actual_boundary = tree.max_right(query_left, |_| accepts_identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_past_len_returns_none() {
        // Arrange
        let values = [1, 2, 3];
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = None;
        let query_left = values.len() + 1;
        let accepts_identity = true;

        // Act
        let actual_boundary = tree.max_right(query_left, |_| accepts_identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_with_always_true_predicate_from_middle_returns_len() {
        // Arrange
        let values = [1, 2, 3, 4];
        let query_left = 2;
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = Some(values.len());
        let accepts_identity = true;

        // Act
        let actual_boundary = tree.max_right(query_left, |_| accepts_identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_returns_query_left_when_first_value_breaks_predicate() {
        // Arrange
        let values = [1, 7, 3];
        let query_left = 1;
        let upper_bound = 6;
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = Some(query_left);

        // Act
        let actual_boundary = tree.max_right(query_left, |value| *value <= upper_bound);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_returns_first_boundary_that_breaks_predicate() {
        // Arrange
        let values = [1, 2, 3, 4, 5];
        let query_left = 1;
        let upper_bound = 6;
        let expected_boundary = Some(3);

        // Act
        let tree = DynamicSegmentTree::from_slice(&values);
        let actual_boundary = tree.max_right(query_left, |value| *value <= upper_bound);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_on_zero_sized_tree_returns_zero() {
        // Arrange
        let tree = DynamicSegmentTree::new(0);
        let expected_boundary = Some(0);
        let query_left = 0;
        let identity = 0;

        // Act
        let actual_boundary = tree.max_right(query_left, |value| *value == identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_panics_when_predicate_rejects_identity() {
        // Arrange
        let values = [1];
        let tree = DynamicSegmentTree::from_slice(&values);
        let query_left = 0;
        let accepts_identity = false;

        // Act
        let actual_result =
            std::panic::catch_unwind(|| tree.max_right(query_left, |_| accepts_identity));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn min_left_past_len_returns_none() {
        // Arrange
        let values = [1, 2, 3];
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = None;
        let query_right = values.len() + 1;
        let accepts_identity = true;

        // Act
        let actual_boundary = tree.min_left(query_right, |_| accepts_identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_returns_query_right_when_first_value_breaks_predicate() {
        // Arrange
        let values = [1, 2, 7, 4];
        let query_right = 3;
        let upper_bound = 6;
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = Some(query_right);

        // Act
        let actual_boundary = tree.min_left(query_right, |value| *value <= upper_bound);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_returns_first_boundary_that_breaks_predicate() {
        // Arrange
        let values = [1, 2, 3, 4, 5];
        let query_right = 4;
        let upper_bound = 7;
        let expected_boundary = Some(2);

        // Act
        let tree = DynamicSegmentTree::from_slice(&values);
        let actual_boundary = tree.min_left(query_right, |value| *value <= upper_bound);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_on_zero_sized_tree_returns_zero() {
        // Arrange
        let tree = DynamicSegmentTree::new(0);
        let expected_boundary = Some(0);
        let query_right = 0;
        let identity = 0;

        // Act
        let actual_boundary = tree.min_left(query_right, |value| *value == identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_at_zero_returns_zero() {
        // Arrange
        let values = [1, 2, 3];
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = Some(0);
        let query_right = 0;
        let accepts_identity = true;

        // Act
        let actual_boundary = tree.min_left(query_right, |_| accepts_identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_with_always_true_predicate_from_middle_returns_zero() {
        // Arrange
        let values = [1, 2, 3, 4];
        let query_right = 2;
        let tree = DynamicSegmentTree::from_slice(&values);
        let expected_boundary = Some(0);
        let accepts_identity = true;

        // Act
        let actual_boundary = tree.min_left(query_right, |_| accepts_identity);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_panics_when_predicate_rejects_identity() {
        // Arrange
        let values = [1];
        let tree = DynamicSegmentTree::from_slice(&values);
        let query_right = 1;
        let accepts_identity = false;

        // Act
        let actual_result =
            std::panic::catch_unwind(|| tree.min_left(query_right, |_| accepts_identity));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn partial_query_with_non_commutative_monoid_keeps_left_to_right_order() {
        // Arrange
        let values = [
            "a".to_string(),
            "b".to_string(),
            "c".to_string(),
            "d".to_string(),
            "e".to_string(),
        ];
        let query_left = 1;
        let query_right = 4;
        let expected_query = Some("bcd".to_string());

        // Act
        let tree = GenericDynamicSegmentTree::<StringConcatMonoid>::from_slice(&values);
        let actual_query = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query, expected_query);
    }

    #[test]
    fn alternating_add_and_update_matches_naive_simulation_results() {
        // Arrange
        let size = 5;
        let operations = [
            (false, 0, 3),
            (true, 2, 5),
            (false, 2, -1),
            (true, 0, 4),
            (false, 4, 7),
            (true, 2, 1),
        ];
        let queries = [(0, 5), (1, 4)];
        let mut values = vec![0_i64; size];
        for &(is_update, index, value) in &operations {
            if is_update {
                values[index] = value;
            } else {
                values[index] += value;
            }
        }
        let expected_sums = queries.map(|(left, right)| Some(naive_sum(&values, left, right)));

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        for (is_update, index, value) in operations {
            if is_update {
                let _ = tree.update(index, value);
            } else {
                let _ = tree.add(index, value);
            }
        }
        let actual_sums = queries.map(|(left, right)| tree.query(left, right));

        // Assert
        assert_eq!(actual_sums, expected_sums);
    }

    #[test]
    fn non_commutative_monoid_keeps_left_to_right_order_after_update() {
        // Arrange
        let values = ["a".to_string(), "b".to_string(), "c".to_string()];
        let updated_index = 1;
        let updated_value = "x".to_string();
        let query_left = 0;
        let query_right = 3;
        let expected_before = Some("abc".to_string());
        let expected_after = Some("axc".to_string());

        // Act
        let mut tree = GenericDynamicSegmentTree::<StringConcatMonoid>::from_slice(&values);
        let actual_before = tree.query(query_left, query_right);
        let _ = tree.update(updated_index, updated_value);
        let actual_after = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_before, expected_before);
        assert_eq!(actual_after, expected_after);
    }

    #[test]
    fn fixed_seed_mixed_operations_match_naive_array_for_multiple_sizes() {
        // Arrange
        #[derive(Clone, Copy)]
        enum Operation {
            Update(usize, i64),
            Add(usize, i64),
            Select(usize),
            Query(usize, usize),
        }
        #[derive(Debug, PartialEq)]
        enum Observation {
            Mutation(Option<()>),
            Value(Option<i64>),
        }
        let sizes = [0_usize, 1, 4, 5];
        let operation_count = 200;
        let mut seed = 0x5eed_u64;
        let mut cases = Vec::new();
        for size in sizes {
            let mut values = vec![0_i64; size];
            let mut operations = Vec::new();
            let mut expected = Vec::new();
            for _ in 0..operation_count {
                match next_random(&mut seed) % 4 {
                    kind @ (0 | 1) => {
                        let index = next_random(&mut seed) as usize % (size + 1);
                        let value = (next_random(&mut seed) % 21) as i64 - 10;
                        operations.push(if kind == 0 {
                            Operation::Update(index, value)
                        } else {
                            Operation::Add(index, value)
                        });
                        let result = values.get_mut(index).map(|entry| {
                            if kind == 0 {
                                *entry = value;
                            } else {
                                *entry += value;
                            }
                        });
                        expected.push(Observation::Mutation(result));
                    }
                    2 => {
                        let index = next_random(&mut seed) as usize % (size + 1);
                        operations.push(Operation::Select(index));
                        expected.push(Observation::Value(values.get(index).copied()));
                    }
                    _ => {
                        let first = next_random(&mut seed) as usize % (size + 1);
                        let second = next_random(&mut seed) as usize % (size + 1);
                        let (left, right) = (first.min(second), first.max(second));
                        operations.push(Operation::Query(left, right));
                        expected.push(Observation::Value(Some(naive_sum(&values, left, right))));
                    }
                }
            }
            cases.push((size, operations, expected));
        }

        // Act
        let actual: Vec<Vec<_>> = cases
            .iter()
            .map(|(size, operations, _)| {
                let mut tree = DynamicSegmentTree::new(*size);
                operations
                    .iter()
                    .map(|operation| match *operation {
                        Operation::Update(index, value) => {
                            Observation::Mutation(tree.update(index, value))
                        }
                        Operation::Add(index, value) => {
                            Observation::Mutation(tree.add(index, value))
                        }
                        Operation::Select(index) => Observation::Value(tree.select(index)),
                        Operation::Query(left, right) => {
                            Observation::Value(tree.query(left, right))
                        }
                    })
                    .collect()
            })
            .collect();

        // Assert
        for (actual, (_, _, expected)) in actual.iter().zip(&cases) {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn updating_i64_min_and_i64_max_on_different_indices_gives_total_minus_one() {
        // Arrange
        let size = 2;
        let first_index = 0;
        let first_value = i64::MIN;
        let second_index = 1;
        let second_value = i64::MAX;
        let query_left = 0;
        let query_right = 2;
        let expected_sum = Some(-1);

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.update(first_index, first_value);
        let _ = tree.update(second_index, second_value);
        let actual_sum = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn boundary_searches_keep_non_commutative_operation_order() {
        // Arrange
        let values = ["a".to_string(), "b".to_string(), "c".to_string()];
        let max_right_target = "abx";
        let min_left_target = "xbc";
        let expected_max_right = Some(2);
        let expected_min_left = Some(1);
        let query_left = 0;
        let query_right = 3;

        // Act
        let tree = GenericDynamicSegmentTree::<StringConcatMonoid>::from_slice(&values);
        let actual_max_right =
            tree.max_right(query_left, |value| max_right_target.starts_with(value));
        let actual_min_left = tree.min_left(query_right, |value| min_left_target.ends_with(value));

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn boundary_searches_on_sparse_tree_treat_missing_nodes_as_identity() {
        // Arrange
        let size = 8;
        let updated_index = 5;
        let updated_value = 4;
        let max_right_upper_bound = 3;
        let min_left_upper_bound = 3;
        let expected_max_right = Some(5);
        let expected_min_left = Some(6);
        let query_left = 0;

        // Act
        let mut tree = DynamicSegmentTree::new(size);
        let _ = tree.update(updated_index, updated_value);
        let actual_max_right = tree.max_right(query_left, |value| *value <= max_right_upper_bound);
        let actual_min_left = tree.min_left(size, |value| *value <= min_left_upper_bound);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn boundary_searches_match_naive_scan_for_dense_and_sparse_trees() {
        // Arrange
        let dense_values = [1, 0, 3, 2, 4];
        let sparse_values = [0, 0, 3, 0, 2, 0];
        let sparse_updates = [(2, 3), (4, 2)];
        let upper_bounds = [0, 1, 3, 6, 10];
        let inputs = [dense_values.as_slice(), sparse_values.as_slice()];
        let queries: Vec<Vec<_>> = inputs
            .iter()
            .map(|values| {
                upper_bounds
                    .iter()
                    .flat_map(|&bound| (0..=values.len()).map(move |index| (bound, index)))
                    .collect()
            })
            .collect();
        let expected: Vec<Vec<_>> = inputs
            .iter()
            .zip(&queries)
            .map(|(values, queries)| {
                queries
                    .iter()
                    .map(|&(bound, index)| {
                        (
                            Some(naive_max_right(values, index, bound)),
                            Some(naive_min_left(values, index, bound)),
                        )
                    })
                    .collect()
            })
            .collect();

        // Act
        let dense_tree = DynamicSegmentTree::from_slice(&dense_values);
        let mut sparse_tree = DynamicSegmentTree::new(sparse_values.len());
        for (index, value) in sparse_updates {
            let _ = sparse_tree.update(index, value);
        }
        let actual: Vec<Vec<_>> = [&dense_tree, &sparse_tree]
            .iter()
            .zip(&queries)
            .map(|(tree, queries)| {
                queries
                    .iter()
                    .map(|&(bound, index)| {
                        (
                            tree.max_right(index, |value| *value <= bound),
                            tree.min_left(index, |value| *value <= bound),
                        )
                    })
                    .collect()
            })
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }
}
