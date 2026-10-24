use std::ops::{Add, AddAssign, Sub, SubAssign};

#[derive(Clone)]
pub struct FenwickTree<T>
where
    T: Copy + Default + AddAssign + SubAssign + Add<Output = T> + Sub<Output = T>,
{
    size: usize,
    tree: Vec<T>,
    bit_ceil: u32
}

impl<T> FenwickTree<T>
where
    T: Copy + Default + Eq + PartialEq + Ord + PartialOrd + AddAssign + SubAssign + Add<Output = T> + Sub<Output = T>,
{

    pub fn new(n: usize) -> Self {

        Self {
            size: n,
            tree: vec![T::default(); n],
            bit_ceil: n.next_power_of_two().trailing_zeros(),
        }
    }

    #[inline]
    fn lsb(index: usize) -> usize {
        index & index.wrapping_neg()
    }

    pub fn from_slice(values: &[T]) -> Self {

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

    pub fn add(&mut self, index: usize, value: T) {

        if index < self.size {

            let mut idx = index + 1;

            while idx <= self.size {
                self.tree[idx - 1] += value;
                idx += Self::lsb(idx);
            }
        }
    }

    #[inline]
    pub fn update(&mut self, index: usize, value: T) {

        match self.select(index) {
            None => {},
            Some(x) => self.add(index, value - x),
        }

    }

    fn prefix_sum(&self, tree: &[T], r: usize) -> T {

        let mut s = T::default();
        let mut r = r;

        while r > 0 {
            s += tree[r - 1];
            r -= Self::lsb(r);
        }

        s
    }

    pub fn sum(&self, left_index: usize, right_index: usize) -> Option<T> {

        if left_index <= right_index && right_index <= self.size {
            Some(self.prefix_sum(&self.tree, right_index) - self.prefix_sum(&self.tree, left_index))
        } else {
            None
        }
    }

    #[inline]
    pub fn select(&self, index: usize) -> Option<T> {
        self.sum(index, index + 1)
    }

    #[inline]
    pub fn select_all(&self) -> Option<T> {
        self.sum(0, self.size)
    }

    // only T is non-negative
    pub fn lower_bound(&self, value: T) -> usize {

        let mut value = value;
        let mut power = 1 << self.bit_ceil;
        let mut right_index = 0;

        while 0 < power {

            if power + right_index <= self.size && self.tree[power + right_index - 1] < value {
                right_index += power;
                value -= self.tree[right_index - 1];
            }

            power >>= 1;
        }

        right_index
    }

    // only T is non-negative
    pub fn upper_bound(&self, value: T) -> usize {

        let mut value = value;
        let mut power = 1 << self.bit_ceil;
        let mut right_index = 0;

        while 0 < power {

            if power + right_index <= self.size && self.tree[power + right_index - 1] <= value {
                right_index += power;
                value -= self.tree[right_index - 1];
            }

            power >>= 1;
        }

        right_index
    }

}


#[cfg(test)]
mod tests {

    use self::FenwickTree;

    use super::*;

    #[test]
    fn from_slice_with_increasing_values_preserves_length_and_each_value() {
        // Arrange
        let n = 5;
        let values = (0..n).map(|x| x as i64).collect::<Vec<i64>>();
        let indices = 0..n;
        let expected_values = values.clone();
        let expected_len = n;

        // Act
        let fw = FenwickTree::from_slice(&values);
        let actual_len = fw.len();
        let actual_values: Vec<_> = indices.map(|index| fw.select(index).unwrap()).collect();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn from_slice_with_single_positive_value_returns_length_one_and_first_lower_bound() {
        // Arrange
        let defaults = vec![5i64];
        let fw = FenwickTree::from_slice(&defaults);
        let threshold_1 = 1;
        let expected_len_1 = defaults.len();
        let expected_lower_bound_2 = 0;

        // Act
        let actual_len_1 = fw.len();
        let actual_lower_bound_2 = fw.lower_bound(threshold_1);

        // Assert
        assert_eq!(actual_len_1, expected_len_1);
        assert_eq!(actual_lower_bound_2, expected_lower_bound_2);
    }

    #[test]
    fn from_slice_with_positive_values_preserves_points_and_prefix_boundaries() {
        // Arrange
        let values = vec![1i32, 2, 3, 4];
        let indices = 0..values.len();
        let threshold = 3;
        let expected_len = values.len();
        let expected_values = values.clone();
        let expected_bounds = (1, 2);

        // Act
        let fw = FenwickTree::from_slice(&values);
        let actual_len = fw.len();
        let actual_values: Vec<_> = indices.map(|index| fw.select(index).unwrap()).collect();
        let actual_bounds = (fw.lower_bound(threshold), fw.upper_bound(threshold));

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_bounds, expected_bounds);
    }

    #[test]
    fn from_slice_with_empty_input_returns_empty_tree_and_zero_sum() {
        // Arrange
        let values: Vec<i32> = Vec::new();
        let fw = FenwickTree::from_slice(&values);
        let left_1 = 0;
        let right_2 = 0;
        let expected_is_empty_1 = true;
        let expected_sum_2 = 0;

        // Act
        let actual_is_empty_1 = fw.is_empty();
        let actual_sum_2 = fw.sum(left_1, right_2).unwrap();

        // Assert
        assert_eq!(actual_is_empty_1, expected_is_empty_1);
        assert_eq!(actual_sum_2, expected_sum_2);
    }

    #[test]
    fn from_slice_with_negative_values_returns_expected_select_and_sum() {
        // Arrange
        let values = vec![-2i32, 3, -1];
        let expected_first_value = -2i32;
        let expected_second_value = 3i32;
        let expected_third_value = -1i32;
        let expected_total_sum = 0i32;
        let expected_tail_sum = 2i32;
        let fw = FenwickTree::from_slice(&values);
        let index_1 = 0;
        let index_2 = 1;
        let index_3 = 2;
        let left_4 = 0;
        let right_5 = 3;
        let left_6 = 1;
        let expected_first_value_result = Some(expected_first_value);
        let expected_second_value_result = Some(expected_second_value);
        let expected_third_value_result = Some(expected_third_value);
        let expected_total_sum_result = Some(expected_total_sum);
        let expected_tail_sum_result = Some(expected_tail_sum);

        // Act
        let first_value = fw.select(index_1);
        let second_value = fw.select(index_2);
        let third_value = fw.select(index_3);
        let total_sum = fw.sum(left_4, right_5);
        let tail_sum = fw.sum(left_6, right_5);

        // Assert
        assert_eq!(first_value, expected_first_value_result);
        assert_eq!(second_value, expected_second_value_result);
        assert_eq!(third_value, expected_third_value_result);
        assert_eq!(total_sum, expected_total_sum_result);
        assert_eq!(tail_sum, expected_tail_sum_result);
    }

    #[test]
    fn from_slice_and_add_builds_return_same_select_and_sum_results() {
        // Arrange
        let values = vec![4i32, -1, 0, 7, 2];
        let len = values.len();
        let from_slice_tree = FenwickTree::from_slice(&values);
        let mut add_tree: FenwickTree<i32> = FenwickTree::new(len);
        for (index, value) in values.iter().enumerate() {
            add_tree.add(index, *value);
        }

        // Act
        let selected_values_from_slice = (0..len)
            .map(|index| from_slice_tree.select(index))
            .collect::<Vec<Option<i32>>>();
        let selected_values_from_add = (0..len)
            .map(|index| add_tree.select(index))
            .collect::<Vec<Option<i32>>>();
        let range_sums_from_slice = (0..=len)
            .flat_map(|left_index| {
                (left_index..=len)
                    .map(|right_index| from_slice_tree.sum(left_index, right_index))
                    .collect::<Vec<Option<i32>>>()
            })
            .collect::<Vec<Option<i32>>>();
        let range_sums_from_add = (0..=len)
            .flat_map(|left_index| {
                (left_index..=len)
                    .map(|right_index| add_tree.sum(left_index, right_index))
                    .collect::<Vec<Option<i32>>>()
            })
            .collect::<Vec<Option<i32>>>();

        // Assert
        assert_eq!(selected_values_from_slice, selected_values_from_add);
        assert_eq!(range_sums_from_slice, range_sums_from_add);
    }

    #[test]
    fn is_empty_with_zero_length_returns_true() {
        // Arrange
        let n = 0;
        let fw: FenwickTree<i32> = FenwickTree::new(n);
        let expected_is_empty_1 = true;

        // Act
        let actual_is_empty_1 = fw.is_empty();

        // Assert
        assert_eq!(actual_is_empty_1, expected_is_empty_1);
    }

    #[test]
    fn is_empty_with_one_element_returns_false() {
        // Arrange
        let n = 1;
        let fw: FenwickTree<i32> = FenwickTree::new(n);
        let expected_is_empty_1 = false;

        // Act
        let actual_is_empty_1 = fw.is_empty();

        // Assert
        assert_eq!(actual_is_empty_1, expected_is_empty_1);
    }

    #[test]
    fn len_with_multiple_elements_returns_requested_size() {
        // Arrange
        let n = 5;
        let fw: FenwickTree<i32> = FenwickTree::new(n);
        let v = n;

        // Act
        let actual_len_1 = fw.len();

        // Assert
        assert_eq!(actual_len_1, v);
    }

    #[test]
    fn len_with_zero_size_returns_zero() {
        // Arrange
        let n = 0;
        let fw: FenwickTree<i32> = FenwickTree::new(n);
        let v = n;

        // Act
        let actual_len_1 = fw.len();

        // Assert
        assert_eq!(actual_len_1, v);
    }

    #[test]
    fn add_to_unmodified_point_stores_added_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 3;
        let v = v1;

        // Act
        fw.add(index1, v1);
        let actual_select_1 = fw.select(index1).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn add_zero_to_unmodified_point_preserves_zero() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 0;
        let v = v1;

        // Act
        fw.add(index1, v1);
        let actual_select_1 = fw.select(index1).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn add_at_first_and_last_indices_updates_values_and_ignores_index_at_len() {
        // Arrange
        let n = 3;
        let mut fw = FenwickTree::new(n);
        let index_1 = 0;
        let value_2 = 2;
        let index_3 = n - 1;
        let value_4 = 3;
        let value_5 = 5;
        let left_6 = 0;
        let expected_select_1 = 2;
        let expected_select_2 = 3;
        let expected_sum_3 = 5;

        // Act
        fw.add(index_1, value_2);
        fw.add(index_3, value_4);
        fw.add(n, value_5);
        let actual_select_1 = fw.select(index_1).unwrap();
        let actual_select_2 = fw.select(index_3).unwrap();
        let actual_sum_3 = fw.sum(left_6, n).unwrap();

        // Assert
        assert_eq!(actual_select_1, expected_select_1);
        assert_eq!(actual_select_2, expected_select_2);
        assert_eq!(actual_sum_3, expected_sum_3);
    }

    #[test]
    fn add_accumulates_values_on_same_index() {
        // Arrange
        let len = 5usize;
        let index = 2usize;
        let first_value = 3i32;
        let second_value = -1i32;
        let third_value = 5i32;
        let expected_value = first_value + second_value + third_value;
        let mut fw: FenwickTree<i32> = FenwickTree::new(len);
        let expected_selected_value = Some(expected_value);

        // Act
        fw.add(index, first_value);
        fw.add(index, second_value);
        fw.add(index, third_value);
        let selected_value = fw.select(index);

        // Assert
        assert_eq!(selected_value, expected_selected_value);
    }

    #[test]
    fn add_on_empty_tree_does_not_panic_and_keeps_state() {
        // Arrange
        let mut fw: FenwickTree<i32> = FenwickTree::new(0);
        let index = 0usize;
        let value = 7i32;
        let expected_total = Some(0i32);
        let expected_is_ok_1 = true;

        // Act
        let add_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fw.add(index, value)));
        let total_sum = fw.select_all();
        let actual_is_ok_1 = add_result.is_ok();

        // Assert
        assert_eq!(actual_is_ok_1, expected_is_ok_1);
        assert_eq!(total_sum, expected_total);
    }

    #[test]
    fn update_unmodified_point_stores_assigned_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 5;
        let v = v1;

        // Act
        fw.update(index1, v1);
        let actual_select_1 = fw.select(index1).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn update_unmodified_point_to_zero_preserves_zero() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 0;
        let v = v1;

        // Act
        fw.update(index1, v1);
        let actual_select_1 = fw.select(index1).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn update_out_of_range_does_not_panic_and_retains_state() {
        // Arrange
        let mut fw: FenwickTree<i32> = FenwickTree::new(3);
        let index_1 = 0;
        let value_2 = 2;
        let index_3 = 3;
        let value_4 = 5;
        let expected_is_ok_1 = true;
        let expected_select_all_2 = 2;

        // Act
        fw.add(index_1, value_2);
        let result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| fw.update(index_3, value_4)));
        let actual_is_ok_1 = result.is_ok();
        let actual_select_all_2 = fw.select_all().unwrap();

        // Assert
        assert_eq!(actual_is_ok_1, expected_is_ok_1);
        assert_eq!(actual_select_all_2, expected_select_all_2);
    }

    #[test]
    fn update_overwrites_existing_value_without_accumulation() {
        // Arrange
        let len = 4usize;
        let index = 1usize;
        let initial_value = 3i32;
        let updated_value = 10i32;
        let mut fw: FenwickTree<i32> = FenwickTree::new(len);
        fw.add(index, initial_value);
        let expected_selected_value = Some(updated_value);

        // Act
        fw.update(index, updated_value);
        let selected_value = fw.select(index);

        // Assert
        assert_eq!(selected_value, expected_selected_value);
    }

    #[test]
    fn update_overwrites_values_at_lower_and_upper_indices() {
        // Arrange
        let len = 4usize;
        let lower_index = 0usize;
        let upper_index = len - 1;
        let lower_value = 6i32;
        let upper_value = 8i32;
        let mut fw: FenwickTree<i32> = FenwickTree::new(len);
        let expected_selected_lower_value = Some(lower_value);
        let expected_selected_upper_value = Some(upper_value);

        // Act
        fw.update(lower_index, lower_value);
        fw.update(upper_index, upper_value);
        let selected_lower_value = fw.select(lower_index);
        let selected_upper_value = fw.select(upper_index);

        // Assert
        assert_eq!(selected_lower_value, expected_selected_lower_value);
        assert_eq!(selected_upper_value, expected_selected_upper_value);
    }

    #[test]
    fn sum_of_unmodified_nonempty_tree_returns_zero() {
        // Arrange
        let n = 5;
        let fw: FenwickTree<i32> = FenwickTree::new(n);
        let v = 0;
        let left_1 = 0;

        // Act
        let actual_sum_1 = fw.sum(left_1, n).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_prefix_before_only_updated_point_returns_zero() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = 0;
        let left_1 = 0;

        // Act
        fw.add(index1, v1);
        let actual_sum_1 = fw.sum(left_1, index1).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_prefix_ending_after_only_updated_point_returns_its_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1;
        let left_1 = 0;
        let right_2 = index1 + 1;

        // Act
        fw.add(index1, v1);
        let actual_sum_1 = fw.sum(left_1, right_2).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_full_range_with_one_updated_point_returns_its_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1;
        let left_1 = 0;

        // Act
        fw.add(index1, v1);
        let actual_sum_1 = fw.sum(left_1, n).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_prefix_before_all_updated_points_returns_zero() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 4;
        let v1 = 4;
        let v2 = 2;
        let v = 0;
        let left_1 = 0;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_sum_1 = fw.sum(left_1, index1).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_prefix_ending_after_first_updated_point_returns_first_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 4;
        let v1 = 4;
        let v2 = 2;
        let v = v1;
        let left_1 = 0;
        let right_2 = index1 + 1;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_sum_1 = fw.sum(left_1, right_2).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_prefix_excluding_second_updated_point_returns_first_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 4;
        let v1 = 4;
        let v2 = 2;
        let v = v1;
        let left_1 = 0;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_sum_1 = fw.sum(left_1, index2).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_prefix_ending_after_last_updated_point_returns_total() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 4;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2;
        let left_1 = 0;
        let right_2 = index2 + 1;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_sum_1 = fw.sum(left_1, right_2).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_of_full_range_with_multiple_updated_points_returns_total() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2;
        let left_1 = 0;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_sum_1 = fw.sum(left_1, n).unwrap();

        // Assert
        assert_eq!(actual_sum_1, v);
    }

    #[test]
    fn sum_with_left_greater_than_right_returns_none() {
        // Arrange
        let fw: FenwickTree<i32> = FenwickTree::new(3);
        let left_1 = 2;
        let right_2 = 1;
        let expected_sum_1 = true;

        // Act
        let actual_sum_1 = fw.sum(left_1, right_2).is_none();

        // Assert
        assert_eq!(actual_sum_1, expected_sum_1);
    }

    #[test]
    fn sum_of_prefixes_before_at_and_after_middle_boundary_returns_expected_totals() {
        // Arrange
        let values = vec![1i32, 2, 3];
        let fw = FenwickTree::from_slice(&values);
        let left_1 = 0;
        let right_2 = 1;
        let right_3 = 2;
        let right_4 = 3;
        let expected_sum_1 = 1;
        let expected_sum_2 = 3;
        let expected_sum_3 = 6;

        // Act
        let actual_sum_1 = fw.sum(left_1, right_2).unwrap();
        let actual_sum_2 = fw.sum(left_1, right_3).unwrap();
        let actual_sum_3 = fw.sum(left_1, right_4).unwrap();

        // Assert
        assert_eq!(actual_sum_1, expected_sum_1);
        assert_eq!(actual_sum_2, expected_sum_2);
        assert_eq!(actual_sum_3, expected_sum_3);
    }

    #[test]
    fn sum_returns_correct_value_for_middle_range() {
        // Arrange
        let values = vec![1i32, 2, 3, 4, 5];
        let left_index = 1usize;
        let right_index = 4usize;
        let expected_sum = 2 + 3 + 4;
        let fw = FenwickTree::from_slice(&values);
        let expected_sum_result = Some(expected_sum);

        // Act
        let actual_sum = fw.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum_result);
    }

    #[test]
    fn sum_returns_zero_for_equal_left_and_right_on_boundaries() {
        // Arrange
        let values = vec![3i32, 1, 4, 1, 5, 9];
        let start_index = 0usize;
        let middle_index = values.len() / 2;
        let end_index = values.len();
        let expected_sum = 0i32;
        let fw = FenwickTree::from_slice(&values);
        let expected_start_sum = Some(expected_sum);
        let expected_middle_sum = Some(expected_sum);
        let expected_end_sum = Some(expected_sum);

        // Act
        let start_sum = fw.sum(start_index, start_index);
        let middle_sum = fw.sum(middle_index, middle_index);
        let end_sum = fw.sum(end_index, end_index);

        // Assert
        assert_eq!(start_sum, expected_start_sum);
        assert_eq!(middle_sum, expected_middle_sum);
        assert_eq!(end_sum, expected_end_sum);
    }

    #[test]
    fn sum_returns_none_when_right_index_exceeds_len_by_one() {
        // Arrange
        let len = 3usize;
        let fw: FenwickTree<i32> = FenwickTree::new(len);
        let left_index = 0usize;
        let right_index = len + 1;
        let expected_sum = None;

        // Act
        let actual_sum = fw.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn select_unmodified_last_point_returns_zero() {
        // Arrange
        let n = 5;
        let fw: FenwickTree<i32> = FenwickTree::new(n);
        let v = 0;
        let index_1 = n - 1;

        // Act
        let actual_select_1 = fw.select(index_1).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn select_only_updated_point_returns_stored_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1;

        // Act
        fw.add(index1, v1);
        let actual_select_1 = fw.select(index1).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn select_second_updated_point_returns_its_negative_value() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 2;
        let v1 = 4;
        let v2 = -2;
        let v = v2;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_select_1 = fw.select(index2).unwrap();

        // Assert
        assert_eq!(actual_select_1, v);
    }

    #[test]
    fn select_returns_none_when_index_is_len() {
        // Arrange
        let len = 4usize;
        let fw: FenwickTree<i32> = FenwickTree::new(len);
        let out_of_range_index = len;
        let expected_selected_value = None;

        // Act
        let selected_value = fw.select(out_of_range_index);

        // Assert
        assert_eq!(selected_value, expected_selected_value);
    }

    #[test]
    fn select_returns_none_on_empty_tree() {
        // Arrange
        let fw: FenwickTree<i32> = FenwickTree::new(0);
        let index = 0usize;
        let expected_selected_value = None;

        // Act
        let selected_value = fw.select(index);

        // Assert
        assert_eq!(selected_value, expected_selected_value);
    }

    #[test]
    fn select_all_returns_zero_on_empty_tree() {
        // Arrange
        let fw: FenwickTree<i32> = FenwickTree::new(0);
        let expected_sum = 0i32;
        let expected_total_sum = Some(expected_sum);

        // Act
        let total_sum = fw.select_all();

        // Assert
        assert_eq!(total_sum, expected_total_sum);
    }

    #[test]
    fn lower_bound_with_zero_threshold_returns_zero() {
        // Arrange
        let n = 5;
        let fw = FenwickTree::new(n);
        let v = 0;
        let expected_lower_bound_1 = 0;

        // Act
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, expected_lower_bound_1);
    }

    #[test]
    fn lower_bound_below_only_positive_value_returns_its_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1 - 1;

        // Act
        fw.add(index1, v1);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, index1);
    }

    #[test]
    fn lower_bound_equal_to_only_positive_value_returns_its_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1;

        // Act
        fw.add(index1, v1);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, index1);
    }

    #[test]
    fn lower_bound_above_only_positive_value_returns_len() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1 + 1;

        // Act
        fw.add(index1, v1);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, n);
    }

    #[test]
    fn lower_bound_below_total_above_first_prefix_returns_last_updated_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2 - 1;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, index2);
    }

    #[test]
    fn lower_bound_equal_to_total_returns_last_updated_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, index2);
    }

    #[test]
    fn lower_bound_above_total_returns_len() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2 + 1;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, n);
    }

    #[test]
    fn lower_bound_below_first_value_returns_zero() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 0;
        let v1 = 4;
        let v = v1 - 1;
        let expected_lower_bound_1 = 0;

        // Act
        fw.add(index1, v1);
        let actual_lower_bound_1 = fw.lower_bound(v);

        // Assert
        assert_eq!(actual_lower_bound_1, expected_lower_bound_1);
    }

    #[test]
    fn empty_tree_returns_zero_for_both_bounds_and_empty_sum() {
        // Arrange
        let fw: FenwickTree<i32> = FenwickTree::new(0);
        let threshold_1 = 0;
        let left_2 = 0;
        let right_3 = 0;
        let expected_lower_bound_1 = 0;
        let expected_upper_bound_2 = 0;
        let expected_sum_3 = 0;

        // Act
        let actual_lower_bound_1 = fw.lower_bound(threshold_1);
        let actual_upper_bound_2 = fw.upper_bound(threshold_1);
        let actual_sum_3 = fw.sum(left_2, right_3).unwrap();

        // Assert
        assert_eq!(actual_lower_bound_1, expected_lower_bound_1);
        assert_eq!(actual_upper_bound_2, expected_upper_bound_2);
        assert_eq!(actual_sum_3, expected_sum_3);
    }

    #[test]
    fn lower_bound_changes_result_on_x_minus_one_x_and_x_plus_one() {
        // Arrange
        let values = vec![2i32, 5, 3];
        let threshold = 2i32;
        let below_threshold = threshold - 1;
        let equal_threshold = threshold;
        let above_threshold = threshold + 1;
        let fw = FenwickTree::from_slice(&values);
        let expected_lower_bound_below = 0;
        let expected_lower_bound_equal = 0;
        let expected_lower_bound_above = 1;

        // Act
        let lower_bound_below = fw.lower_bound(below_threshold);
        let lower_bound_equal = fw.lower_bound(equal_threshold);
        let lower_bound_above = fw.lower_bound(above_threshold);

        // Assert
        assert_eq!(lower_bound_below, expected_lower_bound_below);
        assert_eq!(lower_bound_equal, expected_lower_bound_equal);
        assert_eq!(lower_bound_above, expected_lower_bound_above);
    }

    #[test]
    fn lower_bound_returns_zero_on_empty_tree_for_positive_value() {
        // Arrange
        let fw: FenwickTree<i32> = FenwickTree::new(0);
        let value = 1i32;
        let expected_lower_bound_index = 0;

        // Act
        let lower_bound_index = fw.lower_bound(value);

        // Assert
        assert_eq!(lower_bound_index, expected_lower_bound_index);
    }

    #[test]
    fn upper_bound_with_zero_threshold_on_unmodified_tree_returns_len() {
        // Arrange
        let n = 5;
        let fw = FenwickTree::new(n);
        let v = 0;

        // Act
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, n);
    }

    #[test]
    fn upper_bound_below_only_positive_value_returns_its_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1 - 1;

        // Act
        fw.add(index1, v1);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, index1);
    }

    #[test]
    fn upper_bound_equal_to_only_positive_value_returns_len() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1;

        // Act
        fw.add(index1, v1);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, n);
    }

    #[test]
    fn upper_bound_above_only_positive_value_returns_len() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let v1 = 4;
        let v = v1 + 1;

        // Act
        fw.add(index1, v1);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, n);
    }

    #[test]
    fn upper_bound_below_total_above_first_prefix_returns_last_updated_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2 - 1;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, index2);
    }

    #[test]
    fn upper_bound_equal_to_total_returns_len() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, n);
    }

    #[test]
    fn upper_bound_above_total_returns_len() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 1;
        let index2 = 3;
        let v1 = 4;
        let v2 = 2;
        let v = v1 + v2 + 1;

        // Act
        fw.add(index1, v1);
        fw.add(index2, v2);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, n);
    }

    #[test]
    fn upper_bound_below_first_value_returns_first_index() {
        // Arrange
        let n = 5;
        let mut fw = FenwickTree::new(n);
        let index1 = 0;
        let v1 = 4;
        let v = v1 - 1;

        // Act
        fw.add(index1, v1);
        let actual_upper_bound_1 = fw.upper_bound(v);

        // Assert
        assert_eq!(actual_upper_bound_1, index1);
    }

    #[test]
    fn upper_bound_changes_result_on_x_minus_one_x_and_x_plus_one() {
        // Arrange
        let values = vec![2i32, 5, 3];
        let threshold = 2i32;
        let below_threshold = threshold - 1;
        let equal_threshold = threshold;
        let above_threshold = threshold + 1;
        let fw = FenwickTree::from_slice(&values);
        let expected_upper_bound_below = 0;
        let expected_upper_bound_equal = 1;
        let expected_upper_bound_above = 1;

        // Act
        let upper_bound_below = fw.upper_bound(below_threshold);
        let upper_bound_equal = fw.upper_bound(equal_threshold);
        let upper_bound_above = fw.upper_bound(above_threshold);

        // Assert
        assert_eq!(upper_bound_below, expected_upper_bound_below);
        assert_eq!(upper_bound_equal, expected_upper_bound_equal);
        assert_eq!(upper_bound_above, expected_upper_bound_above);
    }

    #[test]
    fn upper_bound_returns_zero_on_empty_tree_for_positive_value() {
        // Arrange
        let fw: FenwickTree<i32> = FenwickTree::new(0);
        let value = 1i32;
        let expected_upper_bound_index = 0;

        // Act
        let upper_bound_index = fw.upper_bound(value);

        // Assert
        assert_eq!(upper_bound_index, expected_upper_bound_index);
    }
}
