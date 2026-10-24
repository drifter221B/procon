use std::fmt::{self, Display, Formatter};

#[derive(Clone)]
pub struct FenwickTreeInterface<T, F, InvF, E>
where
    T: Copy + PartialOrd,
    F: Fn(T, T) -> T,
    InvF: Fn(T, T) -> T,
    E: Fn() -> T,
{
    size: usize,
    tree: Vec<T>,
    msb_power: usize,
    func: F,
    inv_func: InvF,
    identity: E,
}

impl<T, F, InvF, E> FenwickTreeInterface<T, F, InvF, E>
where
    T: Copy + PartialOrd,
    F: Fn(T, T) -> T,
    InvF: Fn(T, T) -> T,
    E: Fn() -> T,
{
    pub fn new(size: usize, func: F, inv_func: InvF, identity: E) -> Self {
        let initial = (identity)();

        Self {
            size,
            tree: vec![initial; size],
            msb_power: Self::highest_power_of_two(size),
            func,
            inv_func,
            identity,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.size
    }

    pub fn build_tree(&mut self, default_list: &[T]) {
        if self.size < default_list.len() {
            self.size = default_list.len();
        }
        self.tree = default_list.to_vec();

        let identity_value = (self.identity)();
        if self.tree.len() < self.size {
            self.tree.resize(self.size, identity_value);
        }
        self.msb_power = Self::highest_power_of_two(self.size);

        for i in 1..=self.size {
            let next = i + Self::lsb(i);
            if next <= self.size {
                let target = next - 1;
                let val = self.tree[i - 1];
                self.tree[target] = (self.func)(self.tree[target], val);
            }
        }
    }

    pub fn add(&mut self, index: usize, value: T) {
        if index < self.size {
            let mut idx = index + 1;

            while idx <= self.size {
                self.tree[idx - 1] = (self.func)(self.tree[idx - 1], value);
                idx += Self::lsb(idx);
            }
        }
    }

    pub fn update(&mut self, index: usize, value: T) {
        if index < self.size {
            let current = self.select(index).unwrap_or_else(|| (self.identity)());
            let delta = (self.func)((self.inv_func)((self.identity)(), current), value);
            self.add(index, delta);
        }
    }

    pub fn sum_(&self, left_index: usize, right_index: usize) -> Option<T> {
        if left_index <= right_index && right_index <= self.size {
            let left_prefix = self.prefix_sum(left_index);
            let right_prefix = self.prefix_sum(right_index);
            Some((self.inv_func)(right_prefix, left_prefix))
        } else {
            None
        }
    }

    #[inline]
    pub fn select(&self, index: usize) -> Option<T> {
        self.sum_(index, index + 1)
    }

    pub fn lower_bound(&self, value: T) -> usize {
        if value <= (self.identity)() {
            return 0;
        }

        let mut value = value;
        let mut power = self.msb_power;
        let mut right_index = 0usize;

        while 0 < power {
            let next = right_index + power;
            if next <= self.size && self.tree[next - 1] < value {
                right_index = next;
                value = (self.inv_func)(value, self.tree[right_index - 1]);
            }

            power >>= 1;
        }

        right_index
    }

    pub fn upper_bound(&self, value: T) -> usize {
        if value <= (self.identity)() {
            return 1;
        }

        let mut value = value;
        let mut power = self.msb_power;
        let mut right_index = 0usize;

        while 0 < power {
            let next = right_index + power;
            if next <= self.size && self.tree[next - 1] <= value {
                right_index = next;
                value = (self.inv_func)(value, self.tree[right_index - 1]);
            }

            power >>= 1;
        }

        right_index
    }

    #[inline]
    fn prefix_sum(&self, mut r: usize) -> T {
        let mut sum = (self.identity)();

        while 0 < r {
            sum = (self.func)(sum, self.tree[r - 1]);
            r -= Self::lsb(r);
        }

        sum
    }

    #[inline]
    fn lsb(index: usize) -> usize {
        index & index.wrapping_neg()
    }

    #[inline]
    fn highest_power_of_two(n: usize) -> usize {
        if n == 0 {
            0
        } else {
            let shift = usize::BITS - n.leading_zeros() - 1;
            1usize << shift
        }
    }
}

impl<T, F, InvF, E> Display for FenwickTreeInterface<T, F, InvF, E>
where
    T: Copy + PartialOrd + Display,
    F: Fn(T, T) -> T,
    InvF: Fn(T, T) -> T,
    E: Fn() -> T,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        write!(f, "Fenwick Tree: size({}), tree:[", self.size)?;

        for idx in 0..self.size {
            let value = self.select(idx).unwrap_or_else(|| (self.identity)());
            if idx + 1 == self.size {
                write!(f, "{}", value)?;
            } else {
                write!(f, "{} ", value)?;
            }
        }

        write!(f, "]")
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    type BinaryOperation = fn(i64, i64) -> i64;

    type IdentityOperation = fn() -> i64;

    type FenwickI64Tree =
        FenwickTreeInterface<i64, BinaryOperation, BinaryOperation, IdentityOperation>;

    fn sum_operation(left_value: i64, right_value: i64) -> i64 {
        left_value + right_value
    }

    fn sub_operation(left_value: i64, right_value: i64) -> i64 {
        left_value - right_value
    }

    fn zero_identity() -> i64 {
        0
    }

    fn ten_identity() -> i64 {
        10
    }

    fn keep_right_operation(_left_value: i64, right_value: i64) -> i64 {
        right_value
    }

    fn keep_left_operation(left_value: i64, _right_value: i64) -> i64 {
        left_value
    }

    fn create_sum_tree(size: usize) -> FenwickI64Tree {
        FenwickTreeInterface::new(size, sum_operation, sub_operation, zero_identity)
    }

    fn create_replace_tree_with_ten_identity(size: usize) -> FenwickI64Tree {
        FenwickTreeInterface::new(
            size,
            keep_right_operation,
            keep_left_operation,
            ten_identity,
        )
    }

    fn create_sum_tree_and_build(size: usize, values: &[i64]) -> FenwickI64Tree {
        let mut tree = create_sum_tree(size);
        tree.build_tree(values);
        tree
    }

    fn collect_selected_values(tree: &FenwickI64Tree) -> Vec<i64> {
        let length = tree.len();
        let mut selected_values = Vec::with_capacity(length);

        for index in 0..length {
            let selected_value = tree.select(index).unwrap_or_else(zero_identity);
            selected_values.push(selected_value);
        }

        selected_values
    }

    fn create_display_text_from_select(tree: &FenwickI64Tree) -> String {
        let tree_size = tree.len();
        let mut values = Vec::with_capacity(tree_size);

        for index in 0..tree_size {
            let value = tree.select(index).unwrap_or_else(zero_identity);
            values.push(value.to_string());
        }

        format!("Fenwick Tree: size({}), tree:[{}]", tree_size, values.join(" "))
    }

    #[test]
    fn new_and_len_returns_size_for_boundary_values_around_one() {
        // Arrange
        let size_x_minus_1 = 0usize;
        let size_x = 1usize;
        let size_x_plus_1 = 2usize;
        let tree_x_minus_1 = create_sum_tree(size_x_minus_1);
        let tree_x = create_sum_tree(size_x);
        let tree_x_plus_1 = create_sum_tree(size_x_plus_1);

        // Act
        let actual_len_x_minus_1 = tree_x_minus_1.len();
        let actual_len_x = tree_x.len();
        let actual_len_x_plus_1 = tree_x_plus_1.len();

        // Assert
        assert_eq!(actual_len_x_minus_1, size_x_minus_1);
        assert_eq!(actual_len_x, size_x);
        assert_eq!(actual_len_x_plus_1, size_x_plus_1);
    }

    #[test]
    fn new_with_identity_ten_returns_identity_for_each_select() {
        // Arrange
        let size = 3usize;
        let target_index_0 = 0usize;
        let target_index_1 = 1usize;
        let target_index_2 = 2usize;
        let expected_value = 10i64;
        let tree = create_replace_tree_with_ten_identity(size);

        // Act
        let actual_value_0 = tree.select(target_index_0);
        let actual_value_1 = tree.select(target_index_1);
        let actual_value_2 = tree.select(target_index_2);

        // Assert
        assert_eq!(actual_value_0, Some(expected_value));
        assert_eq!(actual_value_1, Some(expected_value));
        assert_eq!(actual_value_2, Some(expected_value));
    }

    #[test]
    fn build_tree_and_new_plus_add_return_same_select_and_sum_results() {
        // Arrange
        let values = vec![3i64, 1, 4, 1, 5];
        let expected_left_index = 1usize;
        let expected_right_index = 4usize;
        let tree_built = create_sum_tree_and_build(values.len(), &values);
        let mut tree_added = create_sum_tree(values.len());

        // Act
        for (index, value) in values.iter().enumerate() {
            tree_added.add(index, *value);
        }
        let actual_built_values = collect_selected_values(&tree_built);
        let actual_added_values = collect_selected_values(&tree_added);
        let actual_built_sum = tree_built.sum_(expected_left_index, expected_right_index);
        let actual_added_sum = tree_added.sum_(expected_left_index, expected_right_index);

        // Assert
        assert_eq!(actual_built_values, actual_added_values);
        assert_eq!(actual_built_sum, actual_added_sum);
    }

    #[test]
    fn add_on_same_index_multiple_times_accumulates_value() {
        // Arrange
        let size = 4usize;
        let target_index = 2usize;
        let first_add_value = 3i64;
        let second_add_value = 5i64;
        let expected_value = Some(8i64);
        let mut tree = create_sum_tree(size);

        // Act
        tree.add(target_index, first_add_value);
        tree.add(target_index, second_add_value);
        let actual_value = tree.select(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn add_updates_values_for_first_and_last_indices() {
        // Arrange
        let size = 4usize;
        let first_index = 0usize;
        let last_index = size - 1;
        let first_value = 11i64;
        let last_value = 13i64;
        let expected_first_value = Some(11i64);
        let expected_last_value = Some(13i64);
        let mut tree = create_sum_tree(size);

        // Act
        tree.add(first_index, first_value);
        tree.add(last_index, last_value);
        let actual_first_value = tree.select(first_index);
        let actual_last_value = tree.select(last_index);

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_last_value, expected_last_value);
    }

    #[test]
    fn add_with_index_equal_len_keeps_state_unchanged() {
        // Arrange
        let values = vec![2i64, 7, 1];
        let add_index = values.len();
        let add_value = 9i64;
        let mut tree = create_sum_tree_and_build(values.len(), &values);
        let expected_values = collect_selected_values(&tree);

        // Act
        tree.add(add_index, add_value);
        let actual_values = collect_selected_values(&tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn add_on_empty_tree_does_not_panic_and_keeps_state() {
        // Arrange
        let mut tree = create_sum_tree(0);
        let add_index = 0usize;
        let add_value = 5i64;
        let expected_len = 0usize;
        let expected_sum = Some(0i64);
        let sum_left = 0usize;
        let sum_right = 0usize;

        // Act
        tree.add(add_index, add_value);
        let actual_len = tree.len();
        let actual_sum = tree.sum_(sum_left, sum_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn add_with_identity_value_keeps_state_unchanged() {
        // Arrange
        let values = vec![4i64, 2, 6];
        let add_index = 1usize;
        let identity_value = 0i64;
        let mut tree = create_sum_tree_and_build(values.len(), &values);
        let expected_values = collect_selected_values(&tree);

        // Act
        tree.add(add_index, identity_value);
        let actual_values = collect_selected_values(&tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn display_after_build_add_update_matches_select_enumeration() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let add_index = 1usize;
        let add_value = 5i64;
        let update_index = 2usize;
        let update_value = 10i64;
        let expected_text = "Fenwick Tree: size(3), tree:[1 7 10]";
        let mut tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        tree.add(add_index, add_value);
        tree.update(update_index, update_value);
        let actual_text = format!("{}", tree);

        // Assert
        assert_eq!(actual_text, expected_text);
    }

    #[test]
    fn build_tree_with_negative_values_returns_expected_select_and_sum_results() {
        // Arrange
        let values = vec![-2i64, 3, -1];
        let expected_select_0 = Some(-2i64);
        let expected_select_1 = Some(3i64);
        let expected_select_2 = Some(-1i64);
        let expected_sum = Some(0i64);
        let select_indices = [0usize, 1, 2];
        let sum_left = 0usize;
        let sum_right = values.len();
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_select_0 = tree.select(select_indices[0]);
        let actual_select_1 = tree.select(select_indices[1]);
        let actual_select_2 = tree.select(select_indices[2]);
        let actual_sum = tree.sum_(sum_left, sum_right);

        // Assert
        assert_eq!(actual_select_0, expected_select_0);
        assert_eq!(actual_select_1, expected_select_1);
        assert_eq!(actual_select_2, expected_select_2);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_with_full_range_returns_total_value() {
        // Arrange
        let values = vec![2i64, 4, 6, 8];
        let left_index = 0usize;
        let right_index = values.len();
        let expected_sum = Some(20i64);
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_sum = tree.sum_(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_with_middle_range_returns_range_value() {
        // Arrange
        let values = vec![2i64, 4, 6, 8];
        let left_index = 1usize;
        let right_index = values.len() - 1;
        let expected_sum = Some(10i64);
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_sum = tree.sum_(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_with_equal_left_and_right_returns_identity_for_boundary_positions() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let index_x_minus_1 = 0usize;
        let index_x = values.len() / 2;
        let index_x_plus_1 = values.len();
        let expected_value = Some(0i64);
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_value_x_minus_1 = tree.sum_(index_x_minus_1, index_x_minus_1);
        let actual_value_x = tree.sum_(index_x, index_x);
        let actual_value_x_plus_1 = tree.sum_(index_x_plus_1, index_x_plus_1);

        // Assert
        assert_eq!(actual_value_x_minus_1, expected_value);
        assert_eq!(actual_value_x, expected_value);
        assert_eq!(actual_value_x_plus_1, expected_value);
    }

    #[test]
    fn sum_with_left_greater_than_right_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let left_index = 2usize;
        let right_index = 1usize;
        let expected_sum = None;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_sum = tree.sum_(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_with_right_over_len_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let left_index = 0usize;
        let right_index = values.len() + 1;
        let expected_sum = None;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_sum = tree.sum_(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_on_empty_tree_with_zero_range_returns_identity() {
        // Arrange
        let tree = create_sum_tree(0);
        let left_index = 0usize;
        let right_index = 0usize;
        let expected_sum = Some(0i64);

        // Act
        let actual_sum = tree.sum_(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_on_empty_tree_with_non_zero_range_returns_none() {
        // Arrange
        let tree = create_sum_tree(0);
        let left_index = 0usize;
        let right_index = 1usize;
        let expected_sum = None;

        // Act
        let actual_sum = tree.sum_(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn lower_bound_with_value_less_or_equal_identity_returns_zero() {
        // Arrange
        let values = vec![2i64, 3, 5];
        let value_x_minus_1 = -1i64;
        let value_x = 0i64;
        let expected_index = 0usize;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_index_x_minus_1 = tree.lower_bound(value_x_minus_1);
        let actual_index_x = tree.lower_bound(value_x);

        // Assert
        assert_eq!(actual_index_x_minus_1, expected_index);
        assert_eq!(actual_index_x, expected_index);
    }

    #[test]
    fn lower_bound_around_first_prefix_value_changes_index_as_expected() {
        // Arrange
        let values = vec![2i64, 3, 5];
        let value_x_minus_1 = 1i64;
        let value_x = 2i64;
        let value_x_plus_1 = 3i64;
        let expected_index_x_minus_1 = 0usize;
        let expected_index_x = 0usize;
        let expected_index_x_plus_1 = 1usize;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_index_x_minus_1 = tree.lower_bound(value_x_minus_1);
        let actual_index_x = tree.lower_bound(value_x);
        let actual_index_x_plus_1 = tree.lower_bound(value_x_plus_1);

        // Assert
        assert_eq!(actual_index_x_minus_1, expected_index_x_minus_1);
        assert_eq!(actual_index_x, expected_index_x);
        assert_eq!(actual_index_x_plus_1, expected_index_x_plus_1);
    }

    #[test]
    fn lower_bound_with_value_more_than_total_returns_len() {
        // Arrange
        let values = vec![2i64, 3, 5];
        let value = 11i64;
        let expected_index = values.len();
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_index = tree.lower_bound(value);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn lower_bound_on_empty_tree_with_value_more_than_identity_returns_zero() {
        // Arrange
        let tree = create_sum_tree(0);
        let value = 1i64;
        let expected_index = 0usize;

        // Act
        let actual_index = tree.lower_bound(value);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn upper_bound_with_value_less_or_equal_identity_returns_one() {
        // Arrange
        let values = vec![2i64, 3, 5];
        let value_x_minus_1 = -1i64;
        let value_x = 0i64;
        let expected_index = 1usize;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_index_x_minus_1 = tree.upper_bound(value_x_minus_1);
        let actual_index_x = tree.upper_bound(value_x);

        // Assert
        assert_eq!(actual_index_x_minus_1, expected_index);
        assert_eq!(actual_index_x, expected_index);
    }

    #[test]
    fn upper_bound_around_first_prefix_value_changes_index_as_expected() {
        // Arrange
        let values = vec![2i64, 3, 5];
        let value_x_minus_1 = 1i64;
        let value_x = 2i64;
        let value_x_plus_1 = 3i64;
        let expected_index_x_minus_1 = 0usize;
        let expected_index_x = 1usize;
        let expected_index_x_plus_1 = 1usize;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_index_x_minus_1 = tree.upper_bound(value_x_minus_1);
        let actual_index_x = tree.upper_bound(value_x);
        let actual_index_x_plus_1 = tree.upper_bound(value_x_plus_1);

        // Assert
        assert_eq!(actual_index_x_minus_1, expected_index_x_minus_1);
        assert_eq!(actual_index_x, expected_index_x);
        assert_eq!(actual_index_x_plus_1, expected_index_x_plus_1);
    }

    #[test]
    fn upper_bound_with_value_more_than_total_returns_len() {
        // Arrange
        let values = vec![2i64, 3, 5];
        let value = 11i64;
        let expected_index = values.len();
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_index = tree.upper_bound(value);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn upper_bound_on_empty_tree_with_value_more_than_identity_returns_zero() {
        // Arrange
        let tree = create_sum_tree(0);
        let value = 1i64;
        let expected_index = 0usize;

        // Act
        let actual_index = tree.upper_bound(value);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn upper_bound_on_empty_tree_with_value_less_or_equal_identity_returns_one() {
        // Arrange
        let tree = create_sum_tree(0);
        let value_x_minus_1 = -1i64;
        let value_x = 0i64;
        let expected_index = 1usize;

        // Act
        let actual_index_x_minus_1 = tree.upper_bound(value_x_minus_1);
        let actual_index_x = tree.upper_bound(value_x);

        // Assert
        assert_eq!(actual_index_x_minus_1, expected_index);
        assert_eq!(actual_index_x, expected_index);
    }

    #[test]
    fn build_tree_with_empty_slice_keeps_length_and_identity_values() {
        // Arrange
        let initial_size = 5usize;
        let values: [i64; 0] = [];
        let expected_values = vec![0i64; initial_size];
        let mut tree = create_sum_tree(initial_size);

        // Act
        tree.build_tree(&values);
        let actual_length = tree.len();
        let actual_values = collect_selected_values(&tree);

        // Assert
        assert_eq!(actual_length, initial_size);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn build_tree_with_single_element_sets_first_and_fills_rest_with_identity() {
        // Arrange
        let initial_size = 5usize;
        let values = vec![7i64];
        let expected_values = vec![7i64, 0, 0, 0, 0];
        let mut tree = create_sum_tree(initial_size);

        // Act
        tree.build_tree(&values);
        let actual_values = collect_selected_values(&tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn build_tree_with_larger_input_expands_length() {
        // Arrange
        let initial_size = 2usize;
        let values = vec![1i64, 2, 3, 4];
        let expected_length = values.len();
        let mut tree = create_sum_tree(initial_size);

        // Act
        tree.build_tree(&values);
        let actual_length = tree.len();

        // Assert
        assert_eq!(actual_length, expected_length);
    }

    #[test]
    fn select_returns_first_and_last_values_for_boundary_indices() {
        // Arrange
        let values = vec![9i64, 8, 7, 6];
        let first_index = 0usize;
        let last_index = values.len() - 1;
        let expected_first_value = Some(9i64);
        let expected_last_value = Some(6i64);
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_first_value = tree.select(first_index);
        let actual_last_value = tree.select(last_index);

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_last_value, expected_last_value);
    }

    #[test]
    fn select_with_index_equal_len_returns_none() {
        // Arrange
        let values = vec![9i64, 8, 7];
        let index = values.len();
        let expected_value = None;
        let tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        let actual_value = tree.select(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn select_on_empty_tree_returns_none() {
        // Arrange
        let tree = create_sum_tree(0);
        let index = 0usize;
        let expected_value = None;

        // Act
        let actual_value = tree.select(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn update_overwrites_existing_value_not_accumulate() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let update_index = 1usize;
        let update_value = 7i64;
        let expected_value = Some(7i64);
        let mut tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        tree.update(update_index, update_value);
        let actual_value = tree.select(update_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn update_on_first_and_last_indices_updates_expected_values() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let first_index = 0usize;
        let last_index = values.len() - 1;
        let update_first_value = 10i64;
        let update_last_value = 20i64;
        let expected_first_value = Some(10i64);
        let expected_last_value = Some(20i64);
        let mut tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        tree.update(first_index, update_first_value);
        tree.update(last_index, update_last_value);
        let actual_first_value = tree.select(first_index);
        let actual_last_value = tree.select(last_index);

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_last_value, expected_last_value);
    }

    #[test]
    fn update_with_index_equal_len_keeps_state_unchanged() {
        // Arrange
        let values = vec![6i64, 2, 8];
        let update_index = values.len();
        let update_value = 15i64;
        let mut tree = create_sum_tree_and_build(values.len(), &values);
        let expected_values = collect_selected_values(&tree);

        // Act
        tree.update(update_index, update_value);
        let actual_values = collect_selected_values(&tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn update_on_empty_tree_does_not_panic_and_keeps_state() {
        // Arrange
        let mut tree = create_sum_tree(0);
        let update_index = 0usize;
        let update_value = 10i64;
        let expected_len = 0usize;
        let expected_sum = Some(0i64);
        let sum_left = 0usize;
        let sum_right = 0usize;

        // Act
        tree.update(update_index, update_value);
        let actual_len = tree.len();
        let actual_sum = tree.sum_(sum_left, sum_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn update_changes_only_target_index_value() {
        // Arrange
        let values = vec![3i64, 1, 4, 1];
        let update_index = 2usize;
        let update_value = 9i64;
        let expected_values = vec![3i64, 1, 9, 1];
        let mut tree = create_sum_tree_and_build(values.len(), &values);

        // Act
        tree.update(update_index, update_value);
        let actual_values = collect_selected_values(&tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn display_on_empty_tree_matches_expected_format() {
        // Arrange
        let tree = create_sum_tree(0);
        let expected_text = "Fenwick Tree: size(0), tree:[]".to_string();

        // Act
        let actual_text = format!("{}", tree);

        // Assert
        assert_eq!(actual_text, expected_text);
    }

    #[test]
    fn display_on_single_element_tree_has_no_extra_space() {
        // Arrange
        let values = vec![7i64];
        let tree = create_sum_tree_and_build(values.len(), &values);
        let expected_text = "Fenwick Tree: size(1), tree:[7]".to_string();

        // Act
        let actual_text = format!("{}", tree);

        // Assert
        assert_eq!(actual_text, expected_text);
    }
}
