use std::ops::Add;

pub trait Monoid {
    type S: Clone;

    fn id() -> Self::S;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
}

pub struct SegmentTree<M>
where
    M: Monoid,
{
    size: usize,
    leaf_size: usize,
    data: Vec<M::S>,
}

impl<M> SegmentTree<M>
where
    M: Monoid,
{

    pub fn new(size: usize) -> Self {

        let leaf_size = Self::leaf_size(size);
        let data = vec![M::id(); leaf_size << 1];

        Self { size, leaf_size, data }
    }

    pub fn from_slice(values: &[M::S]) -> Self {

        let mut tree = Self::new(values.len());

        for (i, v) in values.iter().enumerate() {
            tree.data[tree.leaf_size + i] = v.clone();
        }

        tree.build_parents();

        tree
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
        if index < self.size {
            Some(self.data[self.leaf_size + index].clone())
        } else {
            None
        }
    }

    #[inline]
    pub fn select_all(&self) -> M::S {
        self.data[1].clone()
    }

    pub fn query(&self, query_left: usize, query_right: usize) -> Option<M::S> {

        if query_left > query_right || self.size < query_right {
            return None;
        }

        let mut left_result = M::id();
        let mut right_result = M::id();

        let mut l = query_left + self.leaf_size;
        let mut r = query_right + self.leaf_size;

        while l < r {

            if l & 1 == 1 {
                left_result = M::binary_operation(&left_result, &self.data[l]);
                l += 1;
            }

            l >>= 1;

            if r & 1 == 1 {
                r -= 1;
                right_result = M::binary_operation(&self.data[r], &right_result);
            }

            r >>= 1;
        }

        Some(M::binary_operation(&left_result, &right_result))
    }

    pub fn update(&mut self, index: usize, value: M::S) -> Option<()> {

        if self.size <= index {
            return None;
        }

        let mut idx = index + self.leaf_size;
        self.data[idx] = value;

        while 1 < idx {
            idx >>= 1;
            self.data[idx] = M::binary_operation(&self.data[idx << 1], &self.data[idx << 1 | 1]);
        }

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
        let mut index = query_left + self.leaf_size;

        loop {

            while index & 1 == 0 {
                index >>= 1;
            }

            let next = M::binary_operation(&result, &self.data[index]);
            if !func(&next) {

                while index < self.leaf_size {

                    index <<= 1;
                    let candidate = M::binary_operation(&result, &self.data[index]);

                    if func(&candidate) {
                        result = candidate;
                        index += 1;
                    }
                }

                return Some(index - self.leaf_size);
            }

            result = next;
            index += 1;

            if index & index.wrapping_neg() == index {
                break;
            }
        }

        Some(self.size)
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
        let mut index = query_right + self.leaf_size;

        loop {

            index -= 1;
            while 1 < index && index & 1 == 1 {
                index >>= 1;
            }

            let next = M::binary_operation(&self.data[index], &result);
            if !func(&next) {

                while index < self.leaf_size {

                    index = (index << 1) | 1;
                    let candidate = M::binary_operation(&self.data[index], &result);

                    if func(&candidate) {
                        result = candidate;
                        index -= 1;
                    }
                }

                return Some(index + 1 - self.leaf_size);
            }

            result = next;

            if index & index.wrapping_neg() == index {
                break;
            }
        }

        Some(0)
    }

    fn build_parents(&mut self) {
        for idx in (1..self.leaf_size).rev() {
            self.data[idx] = M::binary_operation(&self.data[idx << 1], &self.data[idx << 1 | 1]);
        }
    }

    fn leaf_size(size: usize) -> usize {
        if size == 0 {
            1
        } else {
            size.next_power_of_two()
        }
    }
}

#[derive(Clone, Copy)]
struct T;

impl Monoid for T {
    type S = i32;

    fn id() -> Self::S {
        i32::MIN
    }

    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
        (*lhs).max(*rhs)
    }
}


#[cfg(test)]
mod tests {
    use super::{Monoid, SegmentTree};

    #[derive(Clone, Copy)]
    struct SumMonoid;

    impl Monoid for SumMonoid {
        type S = i32;

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

    #[test]
    fn new_size_zero_returns_empty_state_and_identity_select_all() {
        // Arrange
        let tree_size = 0usize;
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_select_all = 0i32;
        let tree: SegmentTree<SumMonoid> = SegmentTree::new(tree_size);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_select_all = tree.select_all();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_select_all, expected_select_all);
    }

    #[test]
    fn new_single_size_returns_identity_for_select_zero_and_query_full_range() {
        // Arrange
        let tree_size = 1usize;
        let select_index = 0usize;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_selected_value = Some(0i32);
        let expected_query_result = Some(0i32);
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_selected_value = tree.select(select_index);
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn new_non_power_of_two_size_query_full_range_returns_identity() {
        // Arrange
        let tree_size = 3usize;
        let query_left = 0usize;
        let query_right = 3usize;
        let expected_query_result = Some(0i32);
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn from_slice_empty_returns_identity_for_query_zero_zero() {
        // Arrange
        let values: Vec<i32> = vec![];
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_query_result = Some(0i32);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn from_slice_single_element_select_zero_matches_select_all() {
        // Arrange
        let element_value = 42i32;
        let values = vec![element_value];
        let select_index = 0usize;
        let expected_selected_value = Some(element_value);
        let expected_select_all_value = element_value;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_selected_value = tree.select(select_index);
        let actual_select_all_value = tree.select_all();

        // Assert
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_select_all_value, expected_select_all_value);
    }

    #[test]
    fn from_slice_empty_reports_empty_state_and_select_zero_none() {
        // Arrange
        let values: Vec<i32> = vec![];
        let select_index = 0usize;
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_selected_value: Option<i32> = None;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_selected_value = tree.select(select_index);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_selected_value, expected_selected_value);
    }

    #[test]
    fn from_slice_multiple_elements_query_full_range_matches_fold_result() {
        // Arrange
        let first_value = 1i32;
        let second_value = 2i32;
        let third_value = 3i32;
        let values = vec![first_value, second_value, third_value];
        let query_left = 0usize;
        let query_right = values.len();
        let expected_query_result = Some(first_value + second_value + third_value);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn select_boundary_returns_some_at_len_minus_one_and_none_at_len() {
        // Arrange
        let values = vec![10i32, 20i32, 30i32];
        let valid_index = values.len() - 1;
        let invalid_index = values.len();
        let expected_valid_select = Some(30i32);
        let expected_invalid_select: Option<i32> = None;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_valid_select = tree.select(valid_index);
        let actual_invalid_select = tree.select(invalid_index);

        // Assert
        assert_eq!(actual_valid_select, expected_valid_select);
        assert_eq!(actual_invalid_select, expected_invalid_select);
    }

    #[test]
    fn query_when_left_and_right_are_len_returns_identity_for_non_empty_tree() {
        // Arrange
        let values = vec![4i32, 6i32, 8i32];
        let query_point = values.len();
        let expected_query_result = Some(0i32);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_point, query_point);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_invalid_range_returns_none_when_right_exceeds_len() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let query_left = 0usize;
        let query_right = values.len() + 1usize;
        let expected_query_result: Option<i32> = None;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_single_element_range_matches_select_at_same_index() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let query_left = 1usize;
        let query_right = query_left + 1usize;
        let select_index = query_left;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);
        let actual_selected_value = tree.select(select_index);

        // Assert
        assert_eq!(actual_query_result, actual_selected_value);
    }

    #[test]
    fn query_full_range_matches_select_all() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let query_left = 0usize;
        let query_right = values.len();
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);
        let actual_select_all_result = Some(tree.select_all());

        // Assert
        assert_eq!(actual_query_result, actual_select_all_result);
    }

    #[test]
    fn query_when_left_equals_right_returns_identity() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let query_point = 1usize;
        let expected_query_result = Some(0i32);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_point, query_point);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_invalid_range_returns_none_when_left_is_greater_than_right() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let query_left = 2usize;
        let query_right = 1usize;
        let expected_query_result: Option<i32> = None;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_with_non_commutative_monoid_keeps_left_to_right_order() {
        // Arrange
        let first_text = String::from("a");
        let second_text = String::from("b");
        let third_text = String::from("c");
        let values = vec![first_text, second_text, third_text];
        let query_left = 0usize;
        let query_right = 3usize;
        let expected_query_result = Some(String::from("abc"));
        let tree = SegmentTree::<StringConcatMonoid>::from_slice(&values);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_on_empty_tree_returns_none_when_right_is_one() {
        // Arrange
        let tree_size = 0usize;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_query_result: Option<i32> = None;
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn update_boundary_succeeds_at_len_minus_one_and_fails_at_len() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let valid_index = values.len() - 1usize;
        let valid_update_value = 10i32;
        let invalid_index = values.len();
        let invalid_update_value = 99i32;
        let expected_valid_update_result = Some(());
        let expected_invalid_update_result: Option<()> = None;
        let expected_sum_after_valid_update = Some(13i32);
        let expected_sum_after_invalid_update = Some(13i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_valid_update_result = tree.update(valid_index, valid_update_value);
        let actual_sum_after_valid_update = tree.query(0usize, values.len());
        let actual_invalid_update_result = tree.update(invalid_index, invalid_update_value);
        let actual_sum_after_invalid_update = tree.query(0usize, values.len());

        // Assert
        assert_eq!(actual_valid_update_result, expected_valid_update_result);
        assert_eq!(
            actual_sum_after_valid_update,
            expected_sum_after_valid_update
        );
        assert_eq!(actual_invalid_update_result, expected_invalid_update_result);
        assert_eq!(
            actual_sum_after_invalid_update,
            expected_sum_after_invalid_update
        );
    }

    #[test]
    fn update_on_empty_tree_returns_none_and_select_all_remains_identity() {
        // Arrange
        let tree_size = 0usize;
        let update_index = 0usize;
        let update_value = 10i32;
        let expected_update_result: Option<()> = None;
        let expected_select_all_before = 0i32;
        let expected_select_all_after = 0i32;
        let mut tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_select_all_before = tree.select_all();
        let actual_update_result = tree.update(update_index, update_value);
        let actual_select_all_after = tree.select_all();

        // Assert
        assert_eq!(actual_select_all_before, expected_select_all_before);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_select_all_after, expected_select_all_after);
    }

    #[test]
    fn update_multiple_times_reflects_stepwise_in_full_range_query() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let first_update_index = 0usize;
        let first_update_value = 10i32;
        let second_update_index = 2usize;
        let second_update_value = 4i32;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_first_update_result = Some(());
        let expected_second_update_result = Some(());
        let expected_sum_after_first_update = Some(15i32);
        let expected_sum_after_second_update = Some(16i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_first_update_result = tree.update(first_update_index, first_update_value);
        let actual_sum_after_first_update = tree.query(query_left, query_right);
        let actual_second_update_result = tree.update(second_update_index, second_update_value);
        let actual_sum_after_second_update = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_first_update_result, expected_first_update_result);
        assert_eq!(
            actual_sum_after_first_update,
            expected_sum_after_first_update
        );
        assert_eq!(actual_second_update_result, expected_second_update_result);
        assert_eq!(
            actual_sum_after_second_update,
            expected_sum_after_second_update
        );
    }

    #[test]
    fn update_keeps_query_of_non_updated_range_unchanged() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32, 4i32];
        let update_index = 1usize;
        let update_value = 20i32;
        let query_left = 2usize;
        let query_right = 4usize;
        let expected_update_result = Some(());
        let expected_query_before = Some(7i32);
        let expected_query_after = Some(7i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_before = tree.query(query_left, query_right);
        let actual_update_result = tree.update(update_index, update_value);
        let actual_query_after = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_before, expected_query_before);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_query_after, expected_query_after);
    }

    #[test]
    fn update_with_non_commutative_monoid_rebuilds_parent_in_left_to_right_order() {
        // Arrange
        let values = vec![
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("d"),
        ];
        let update_index = 1usize;
        let updated_value = String::from("x");
        let query_left = 0usize;
        let query_right = values.len();
        let expected_update_result = Some(());
        let expected_query_result = Some(String::from("axcd"));
        let expected_select_all_result = String::from("axcd");
        let mut tree = SegmentTree::<StringConcatMonoid>::from_slice(&values);

        // Act
        let actual_update_result = tree.update(update_index, updated_value);
        let actual_query_result = tree.query(query_left, query_right);
        let actual_select_all_result = tree.select_all();

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_query_result, expected_query_result);
        assert_eq!(actual_select_all_result, expected_select_all_result);
    }

    #[test]
    fn add_on_empty_tree_returns_none_and_select_all_remains_identity() {
        // Arrange
        let tree_size = 0usize;
        let update_index = 0usize;
        let add_value = 10i32;
        let expected_update_result: Option<()> = None;
        let expected_select_all_before = 0i32;
        let expected_select_all_after = 0i32;
        let mut tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_select_all_before = tree.select_all();
        let actual_update_result = tree.add(update_index, add_value);
        let actual_select_all_after = tree.select_all();

        // Assert
        assert_eq!(actual_select_all_before, expected_select_all_before);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_select_all_after, expected_select_all_after);
    }

    #[test]
    fn add_updates_selected_value_and_full_query_result() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let update_index = 1usize;
        let add_value = 5i32;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_update_result = Some(());
        let expected_selected_value = Some(7i32);
        let expected_query_result = Some(11i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_update_result = tree.add(update_index, add_value);
        let actual_selected_value = tree.select(update_index);
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn add_with_negative_value_decreases_selected_and_full_query_result() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let update_index = 1usize;
        let add_value = -3i32;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_update_result = Some(());
        let expected_selected_value = Some(4i32);
        let expected_query_result = Some(18i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_update_result = tree.add(update_index, add_value);
        let actual_selected_value = tree.select(update_index);
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn add_with_zero_keeps_selected_and_full_query_result_unchanged() {
        // Arrange
        let values = vec![5i32, 7i32, 9i32];
        let update_index = 1usize;
        let add_value = 0i32;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_update_result = Some(());
        let expected_selected_value = Some(7i32);
        let expected_query_result = Some(21i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_update_result = tree.add(update_index, add_value);
        let actual_selected_value = tree.select(update_index);
        let actual_query_result = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn add_out_of_range_returns_none_and_keeps_tree_unchanged() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let update_index = values.len();
        let add_value = 5i32;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_update_result: Option<()> = None;
        let expected_query_before = Some(6i32);
        let expected_query_after = Some(6i32);
        let mut tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_query_before = tree.query(query_left, query_right);
        let actual_update_result = tree.add(update_index, add_value);
        let actual_query_after = tree.query(query_left, query_right);

        // Assert
        assert_eq!(actual_query_before, expected_query_before);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_query_after, expected_query_after);
    }

    #[test]
    fn max_right_returns_none_when_query_left_exceeds_len_plus_one_boundary() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let query_left = values.len() + 1usize;
        let expected_result: Option<usize> = None;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.max_right(query_left, |sum_value| *sum_value <= 100i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn max_right_returns_len_when_query_left_equals_len() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let query_left = values.len();
        let expected_result = Some(values.len());
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.max_right(query_left, |sum_value| *sum_value <= 100i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn max_right_returns_len_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let query_left = 1usize;
        let expected_result = Some(values.len());
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.max_right(query_left, |_| true);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn max_right_from_zero_returns_len_minus_one_boundary() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32, 10i32];
        let query_left = 0usize;
        let expected_boundary = Some(values.len() - 1usize);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.max_right(query_left, |sum_value| *sum_value <= 6i32);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn max_right_returns_query_left_when_first_extension_breaks_predicate() {
        // Arrange
        let values = vec![1i32, 5i32, 3i32];
        let query_left = 1usize;
        let expected_result = Some(query_left);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.max_right(query_left, |sum_value| *sum_value <= 4i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn max_right_on_empty_tree_returns_zero_when_query_left_is_zero() {
        // Arrange
        let tree_size = 0usize;
        let query_left = 0usize;
        let expected_result = Some(0usize);
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_result = tree.max_right(query_left, |sum_value| *sum_value <= 0i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn max_right_on_empty_tree_returns_none_when_query_left_is_one() {
        // Arrange
        let tree_size = 0usize;
        let query_left = 1usize;
        let expected_result: Option<usize> = None;
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_result = tree.max_right(query_left, |sum_value| *sum_value <= 0i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn max_right_returns_intermediate_boundary_when_predicate_breaks_in_middle() {
        // Arrange
        let values = vec![2i32, 3i32, 4i32, 5i32];
        let query_left = 1usize;
        let expected_boundary = Some(2usize);
        let expected_sum_until_boundary = Some(3i32);
        let expected_sum_until_next = Some(7i32);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.max_right(query_left, |sum_value| *sum_value <= 6i32);
        let actual_sum_until_boundary =
            actual_boundary.and_then(|boundary_index| tree.query(query_left, boundary_index));
        let actual_sum_until_next = actual_boundary
            .and_then(|boundary_index| tree.query(query_left, boundary_index + 1usize));

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_sum_until_boundary, expected_sum_until_boundary);
        assert_eq!(actual_sum_until_next, expected_sum_until_next);
    }

    #[test]
    fn max_right_panics_when_predicate_is_false_for_identity() {
        // Arrange
        let values = vec![1i32];
        let query_left = 0usize;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = std::panic::catch_unwind(|| tree.max_right(query_left, |_| false));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn max_right_with_non_commutative_monoid_keeps_order_when_finding_boundary() {
        // Arrange
        let values = vec![
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("d"),
        ];
        let query_left = 1usize;
        let expected_prefix = "bc";
        let expected_boundary = Some(3usize);
        let tree = SegmentTree::<StringConcatMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.max_right(query_left, |aggregated_value| {
            expected_prefix.starts_with(aggregated_value.as_str())
        });
        let actual_selected_text =
            actual_boundary.and_then(|boundary_index| tree.query(query_left, boundary_index));

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_selected_text, Some(String::from("bc")));
    }

    #[test]
    fn max_right_from_zero_returns_zero_when_first_element_breaks_predicate() {
        // Arrange
        let values = vec![5i32, 1i32, 1i32];
        let query_left = 0usize;
        let expected_boundary = Some(0usize);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.max_right(query_left, |sum_value| *sum_value <= 4i32);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_returns_none_when_query_right_exceeds_len_plus_one_boundary() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let query_right = values.len() + 1usize;
        let expected_result: Option<usize> = None;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.min_left(query_right, |sum_value| *sum_value <= 100i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_left_from_len_returns_len_when_last_element_breaks_predicate() {
        // Arrange
        let values = vec![1i32, 2i32, 5i32];
        let query_right = values.len();
        let expected_boundary = Some(values.len());
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.min_left(query_right, |sum_value| *sum_value <= 4i32);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_from_len_returns_one_boundary() {
        // Arrange
        let values = vec![7i32, 1i32, 2i32];
        let query_right = values.len();
        let expected_boundary = Some(1usize);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.min_left(query_right, |sum_value| *sum_value <= 4i32);

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
    }

    #[test]
    fn min_left_returns_zero_when_query_right_is_zero() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let query_right = 0usize;
        let expected_result = Some(0usize);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.min_left(query_right, |sum_value| *sum_value <= 100i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_left_returns_query_right_when_first_extension_breaks_predicate() {
        // Arrange
        let values = vec![1i32, 5i32, 3i32];
        let query_right = 2usize;
        let expected_result = Some(query_right);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.min_left(query_right, |sum_value| *sum_value <= 1i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_left_on_empty_tree_returns_zero_when_query_right_is_zero() {
        // Arrange
        let tree_size = 0usize;
        let query_right = 0usize;
        let expected_result = Some(0usize);
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_result = tree.min_left(query_right, |sum_value| *sum_value <= 0i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_left_on_empty_tree_returns_none_when_query_right_is_one() {
        // Arrange
        let tree_size = 0usize;
        let query_right = 1usize;
        let expected_result: Option<usize> = None;
        let tree = SegmentTree::<SumMonoid>::new(tree_size);

        // Act
        let actual_result = tree.min_left(query_right, |sum_value| *sum_value <= 0i32);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_left_returns_zero_when_predicate_is_always_true() {
        // Arrange
        let values = vec![1i32, 2i32, 3i32];
        let query_right = 2usize;
        let expected_result = Some(0usize);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = tree.min_left(query_right, |_| true);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_left_returns_intermediate_boundary_when_predicate_breaks_in_middle() {
        // Arrange
        let values = vec![2i32, 3i32, 4i32, 5i32];
        let query_right = 3usize;
        let expected_boundary = Some(2usize);
        let expected_sum_from_boundary = Some(4i32);
        let expected_sum_from_previous = Some(7i32);
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.min_left(query_right, |sum_value| *sum_value <= 6i32);
        let actual_sum_from_boundary =
            actual_boundary.and_then(|boundary_index| tree.query(boundary_index, query_right));
        let actual_sum_from_previous = actual_boundary.and_then(|boundary_index| {
            if boundary_index == 0usize {
                None
            } else {
                tree.query(boundary_index - 1usize, query_right)
            }
        });

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_sum_from_boundary, expected_sum_from_boundary);
        assert_eq!(actual_sum_from_previous, expected_sum_from_previous);
    }

    #[test]
    fn min_left_panics_when_predicate_is_false_for_identity() {
        // Arrange
        let values = vec![1i32];
        let query_right = 1usize;
        let tree = SegmentTree::<SumMonoid>::from_slice(&values);

        // Act
        let actual_result = std::panic::catch_unwind(|| tree.min_left(query_right, |_| false));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn min_left_with_non_commutative_monoid_keeps_order_when_finding_boundary() {
        // Arrange
        let values = vec![
            String::from("a"),
            String::from("b"),
            String::from("c"),
            String::from("d"),
        ];
        let query_right = 3usize;
        let expected_suffix = "bc";
        let expected_boundary = Some(1usize);
        let tree = SegmentTree::<StringConcatMonoid>::from_slice(&values);

        // Act
        let actual_boundary = tree.min_left(query_right, |aggregated_value| {
            expected_suffix.ends_with(aggregated_value.as_str())
        });
        let actual_selected_text =
            actual_boundary.and_then(|boundary_index| tree.query(boundary_index, query_right));

        // Assert
        assert_eq!(actual_boundary, expected_boundary);
        assert_eq!(actual_selected_text, Some(String::from("bc")));
    }
}
