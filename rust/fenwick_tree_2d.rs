use std::ops::{Add, AddAssign, Mul, Neg, Sub, SubAssign};

#[derive(Clone)]
pub struct FenwickTree<T>
where
    T: Copy + Default + AddAssign + Add<Output = T> + Sub<Output = T>,
{
    size: usize,
    tree: Vec<T>,
    bit_ceil: u32,
}

impl<T> FenwickTree<T>
where
    T: Copy + Default + AddAssign + Add<Output = T> + Sub<Output = T>,
{
    pub fn new(n: usize) -> Self {
        Self {
            size: n,
            tree: vec![T::default(); n],
            bit_ceil: (n + 1).next_power_of_two().trailing_zeros(),
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    #[inline]
    fn lsb(index: usize) -> usize {
        index & index.wrapping_neg()
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

}

pub struct FenwickTree2D<T>
where
    T: Copy + Default + AddAssign + SubAssign + Add<Output = T> + Sub<Output = T> + Mul<i128, Output = T> + Neg<Output = T>,
{
    row: usize,
    col: usize,
    b1: Vec<FenwickTree<T>>,
    bx: Vec<FenwickTree<T>>,
    by: Vec<FenwickTree<T>>,
    bxy: Vec<FenwickTree<T>>,
}

impl<T> FenwickTree2D<T>
where
    T: Copy + Default + AddAssign + SubAssign + Add<Output = T> + Sub<Output = T> + Mul<i128, Output = T> + Neg<Output = T>,
{
    pub fn new(row: usize, col: usize) -> Self {
        Self {
            row,
            col,
            b1: (0..row).map(|_| FenwickTree::<T>::new(col)).collect(),
            bx: (0..row).map(|_| FenwickTree::<T>::new(col)).collect(),
            by: (0..row).map(|_| FenwickTree::<T>::new(col)).collect(),
            bxy: (0..row).map(|_| FenwickTree::<T>::new(col)).collect(),
        }
    }

    #[inline]
    fn lsb(index: usize) -> usize {
        index & index.wrapping_neg()
    }

    #[inline]
    // 1-indexed: x, y
    fn point_add(layer: &mut [FenwickTree<T>], x: usize, y: usize, delta: T) {
        let row = layer.len();
        let col = layer.first().map(FenwickTree::len).unwrap_or(0);

        if x == 0 || y == 0 || row < x || col < y {
            return;
        }

        let mut x = x;
        while x <= row {
            layer[x - 1].add(y - 1, delta);
            x += Self::lsb(x);
        }

    }

    #[inline]
    // 0-indexed: [0, x) [0, y)
    fn point_sum(layer: &[FenwickTree<T>], x: usize, y: usize) -> T {
        let col = layer.first().map(FenwickTree::len).unwrap_or(0);

        if x == 0 || y == 0 || layer.is_empty() || col < y {
            return T::default();
        }

        let mut x = x;
        let mut res = T::default();

        while 0 < x {
            res += layer[x - 1].sum(0, y).unwrap_or_default();
            x -= Self::lsb(x);
        }

        res
    }

    #[inline]
    // 0-indexed boundary for diff injection; fenwick uses (x+1, y+1) internally
    fn apply(&mut self, x: usize, y: usize, delta: T) {
        if self.row == 0 || self.col == 0 || x > self.row || y > self.col {
            return;
        }

        let xi = x as i128;
        let yi = y as i128;
        let fx = x + 1;
        let fy = y + 1;

        Self::point_add(&mut self.b1, fx, fy, delta);
        Self::point_add(&mut self.bx, fx, fy, delta * xi);
        Self::point_add(&mut self.by, fx, fy, delta * yi);
        Self::point_add(&mut self.bxy, fx, fy, delta * xi * yi);
    }

    // 0-indexed; [0, x), [0, y)
    fn prefix_sum(&self, x: usize, y: usize) -> T {
        if x == 0 || y == 0 || self.row == 0 || self.col == 0 {
            return T::default();
        }

        let xi = x as i128;
        let yi = y as i128;

        Self::point_sum(&self.b1, x, y) * xi * yi - Self::point_sum(&self.bx, x, y) * yi - Self::point_sum(&self.by, x, y) * xi + Self::point_sum(&self.bxy, x, y)
    }

    #[inline]
    // 0-indexed; [x1, x2), [y1, y2)
    pub fn add(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, delta: T) {
        if x1 < x2 && y1 < y2 && x2 <= self.row && y2 <= self.col {
            self.apply(x1, y1, delta);
            self.apply(x1, y2, -delta);
            self.apply(x2, y1, -delta);
            self.apply(x2, y2, delta);
        }

    }

    // 0-indexed; [x1, x2), [y1, y2) O(row * col)
    pub fn update(&mut self, x1: usize, y1: usize, x2: usize, y2: usize, value: T) {
        if x1 < x2 && y1 < y2 && x2 <= self.row && y2 <= self.col {
            for x in x1..x2 {
                for y in y1..y2 {
                    if let Some(current) = self.select(x, y) {
                        self.add(x, y, x + 1, y + 1, value - current);
                    }
                }
            }
        }
    }

    #[inline]
    // 0-indexed; [x1, x2), [y1, y2)
    pub fn sum(&self, x1: usize, y1: usize, x2: usize, y2: usize) -> Option<T> {
        if x1 < x2 && y1 < y2 && x2 <= self.row && y2 <= self.col {
            Some(self.prefix_sum(x2, y2) - self.prefix_sum(x1, y2) - self.prefix_sum(x2, y1) + self.prefix_sum(x1, y1))
        } else {
            None
        }
    }

    #[inline]
    // 0-indexed; x, y
    pub fn select(&self, x: usize, y: usize) -> Option<T> {
        if x < self.row && y < self.col {
            Some(Self::point_sum(&self.b1, x + 1, y + 1))
        } else {
            None
        }
    }

    #[inline]
    pub fn select_all(&self) -> Option<T> {
        self.sum(0, 0, self.row, self.col)
    }
}

#[cfg(test)]
mod tests {

    use super::{FenwickTree, FenwickTree2D};

    type TestValue = i128;

    fn create_fenwick_tree(size: usize) -> FenwickTree<TestValue> {
        FenwickTree::new(size)
    }

    fn create_fenwick_tree_2d(row: usize, col: usize) -> FenwickTree2D<TestValue> {
        FenwickTree2D::new(row, col)
    }

    #[test]
    fn fenwick_tree_new_with_zero_size_returns_zero_length_and_empty() {
        // Arrange
        let size = 0;
        let tree = create_fenwick_tree(size);
        let expected_length = 0;
        let expected_is_empty = true;

        // Act
        let actual_length = tree.len();
        let actual_is_empty = tree.is_empty();

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn fenwick_tree_new_with_one_size_returns_default_sum_for_initial_range() {
        // Arrange
        let size = 1;
        let tree = create_fenwick_tree(size);
        let left_index = 0;
        let right_index = 1;
        let expected_sum = Some(TestValue::default());

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_add_on_first_index_updates_single_element_range_sum() {
        // Arrange
        let size = 1;
        let mut tree = create_fenwick_tree(size);
        let add_index = 0;
        let add_value = 7;
        let left_index = 0;
        let right_index = 1;
        let expected_sum = Some(add_value);

        // Act
        tree.add(add_index, add_value);
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_add_on_last_index_changes_only_ranges_that_include_tail() {
        // Arrange
        let size = 4;
        let mut tree = create_fenwick_tree(size);
        let add_index = size - 1;
        let add_value = 9;
        let expected_prefix_sum = Some(0);
        let expected_tail_sum = Some(add_value);
        let expected_total_sum = Some(add_value);
        let sum_range_1 = (0, size - 1);
        let sum_range_2 = (size - 1, size);
        let sum_range_3 = (0, size);

        // Act
        tree.add(add_index, add_value);
        let actual_prefix_sum = tree.sum(sum_range_1.0, sum_range_1.1);
        let actual_tail_sum = tree.sum(sum_range_2.0, sum_range_2.1);
        let actual_total_sum = tree.sum(sum_range_3.0, sum_range_3.1);

        // Assert
        assert_eq!(actual_prefix_sum, expected_prefix_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
        assert_eq!(actual_total_sum, expected_total_sum);
    }

    #[test]
    fn fenwick_tree_add_with_out_of_range_index_keeps_state_unchanged() {
        // Arrange
        let size = 3;
        let mut tree = create_fenwick_tree(size);
        let initial_index = 1;
        let initial_value = 5;
        let out_of_range_index = size;
        let ignored_value = 11;
        let left_index = 0;
        let right_index = size;
        let expected_sum = Some(initial_value);

        // Act
        tree.add(initial_index, initial_value);
        tree.add(out_of_range_index, ignored_value);
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_multiple_additions_produce_expected_range_sums() {
        // Arrange
        let size = 5;
        let mut tree = create_fenwick_tree(size);
        let first_index = 0;
        let second_index = 1;
        let third_index = 3;
        let first_value = 2;
        let second_value = 3;
        let third_value = 5;
        let expected_total_sum = Some(10);
        let expected_inner_sum = Some(8);
        let sum_range_1 = (0, size);
        let sum_range_2 = (1, 4);

        // Act
        tree.add(first_index, first_value);
        tree.add(second_index, second_value);
        tree.add(third_index, third_value);
        let actual_total_sum = tree.sum(sum_range_1.0, sum_range_1.1);
        let actual_inner_sum = tree.sum(sum_range_2.0, sum_range_2.1);

        // Assert
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_inner_sum, expected_inner_sum);
    }

    #[test]
    fn fenwick_tree_sum_with_equal_bounds_returns_default_value() {
        // Arrange
        let size = 5;
        let tree = create_fenwick_tree(size);
        let left_index = 3;
        let right_index = 3;
        let expected_sum = Some(TestValue::default());

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_sum_with_right_less_than_left_returns_none() {
        // Arrange
        let size = 5;
        let tree = create_fenwick_tree(size);
        let left_index = 4;
        let right_index = 3;
        let expected_sum: Option<TestValue> = None;

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_sum_with_right_exceeding_length_returns_none() {
        // Arrange
        let size = 5;
        let tree = create_fenwick_tree(size);
        let left_index = 0;
        let right_index = size + 1;
        let expected_sum: Option<TestValue> = None;

        // Act
        let actual_sum = tree.sum(left_index, right_index);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_new_with_zero_size_returns_none_for_select_all() {
        // Arrange
        let row = 0;
        let col = 0;
        let tree = create_fenwick_tree_2d(row, col);
        let expected_sum: Option<TestValue> = None;

        // Act
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_new_with_one_by_one_returns_default_for_select() {
        // Arrange
        let row = 1;
        let col = 1;
        let tree = create_fenwick_tree_2d(row, col);
        let select_x = 0;
        let select_y = 0;
        let expected_value = Some(TestValue::default());

        // Act
        let actual_value = tree.select(select_x, select_y);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_new_with_one_by_one_returns_default_for_select_all() {
        // Arrange
        let row = 1;
        let col = 1;
        let tree = create_fenwick_tree_2d(row, col);
        let expected_sum = Some(TestValue::default());

        // Act
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_add_single_cell_range_updates_that_cell_value() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let x1 = 1;
        let y1 = 1;
        let x2 = 2;
        let y2 = 2;
        let delta = 6;
        let expected_value = Some(delta);

        // Act
        tree.add(x1, y1, x2, y2, delta);
        let actual_value = tree.select(x1, y1);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_add_whole_range_updates_select_all_by_area_times_delta() {
        // Arrange
        let row = 2;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let x1 = 0;
        let y1 = 0;
        let x2 = row;
        let y2 = col;
        let delta = 4;
        let expected_sum = Some(delta * (row * col) as TestValue);

        // Act
        tree.add(x1, y1, x2, y2, delta);
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_add_overlapping_rectangles_accumulates_only_overlap_region() {
        // Arrange
        let row = 4;
        let col = 4;
        let mut tree = create_fenwick_tree_2d(row, col);
        let first_delta = 2;
        let second_delta = 5;
        let expected_overlap_value = Some(7);
        let expected_first_only_value = Some(2);
        let expected_second_only_value = Some(5);
        let add_range_1 = (0, 0, 3, 3);
        let add_range_2 = (1, 1, 4, 4);
        let select_point_1 = (1, 1);
        let select_point_2 = (0, 0);
        let select_point_3 = (3, 3);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            first_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            second_delta,
        );
        let actual_overlap_value = tree.select(select_point_1.0, select_point_1.1);
        let actual_first_only_value = tree.select(select_point_2.0, select_point_2.1);
        let actual_second_only_value = tree.select(select_point_3.0, select_point_3.1);

        // Assert
        assert_eq!(actual_overlap_value, expected_overlap_value);
        assert_eq!(actual_first_only_value, expected_first_only_value);
        assert_eq!(actual_second_only_value, expected_second_only_value);
    }

    #[test]
    fn fenwick_tree_2d_add_with_negative_delta_decreases_cell_value() {
        // Arrange
        let row = 2;
        let col = 2;
        let mut tree = create_fenwick_tree_2d(row, col);
        let positive_delta = 7;
        let negative_delta = -3;
        let expected_value = Some(4);
        let add_range_1 = (0, 0, 2, 2);
        let select_point_1 = (1, 1);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            positive_delta,
        );
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            negative_delta,
        );
        let actual_value = tree.select(select_point_1.0, select_point_1.1);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_add_with_x1_equal_x2_keeps_state_unchanged() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let baseline_delta = 2;
        let ignored_delta = 9;
        let expected_sum = Some(18);
        let add_range_1 = (0, 0, 3, 3);
        let add_range_2 = (1, 0, 1, 2);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            baseline_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            ignored_delta,
        );
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_add_with_y1_equal_y2_keeps_state_unchanged() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let baseline_delta = 2;
        let ignored_delta = 9;
        let expected_sum = Some(18);
        let add_range_1 = (0, 0, 3, 3);
        let add_range_2 = (0, 1, 2, 1);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            baseline_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            ignored_delta,
        );
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_add_with_out_of_bounds_range_keeps_state_unchanged() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let baseline_delta = 2;
        let ignored_delta = 9;
        let expected_sum = Some(18);
        let add_range_1 = (0, 0, 3, 3);
        let add_range_2 = (0, 0, 4, 3);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            baseline_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            ignored_delta,
        );
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_add_and_update_mixed_operations_keep_expected_final_sum() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let first_delta = 1;
        let second_delta = 2;
        let update_value = 4;
        let third_delta = -1;
        let expected_sum = Some(24);
        let add_range_1 = (0, 0, 3, 3);
        let add_range_2 = (0, 0, 2, 2);
        let update_range_1 = (1, 1, 3, 3);
        let add_range_3 = (0, 2, 3, 3);
        let sum_range_1 = (0, 0, row, col);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            first_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            second_delta,
        );
        tree.update(
            update_range_1.0,
            update_range_1.1,
            update_range_1.2,
            update_range_1.3,
            update_value,
        );
        tree.add(
            add_range_3.0,
            add_range_3.1,
            add_range_3.2,
            add_range_3.3,
            third_delta,
        );
        let actual_sum = tree.sum(sum_range_1.0, sum_range_1.1, sum_range_1.2, sum_range_1.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_update_sets_target_rectangle_to_uniform_value() {
        // Arrange
        let row = 4;
        let col = 4;
        let mut tree = create_fenwick_tree_2d(row, col);
        let baseline_delta = 1;
        let update_value = 7;
        let expected_value = Some(update_value);
        let add_range_1 = (0, 0, 4, 4);
        let update_range_1 = (1, 1, 3, 3);
        let select_point_1 = (1, 1);
        let select_point_2 = (1, 2);
        let select_point_3 = (2, 1);
        let select_point_4 = (2, 2);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            baseline_delta,
        );
        tree.update(
            update_range_1.0,
            update_range_1.1,
            update_range_1.2,
            update_range_1.3,
            update_value,
        );
        let actual_value_11 = tree.select(select_point_1.0, select_point_1.1);
        let actual_value_12 = tree.select(select_point_2.0, select_point_2.1);
        let actual_value_21 = tree.select(select_point_3.0, select_point_3.1);
        let actual_value_22 = tree.select(select_point_4.0, select_point_4.1);

        // Assert
        assert_eq!(actual_value_11, expected_value);
        assert_eq!(actual_value_12, expected_value);
        assert_eq!(actual_value_21, expected_value);
        assert_eq!(actual_value_22, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_update_replaces_existing_values_only_in_target_rectangle() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let base_delta = 1;
        let overlap_delta = 2;
        let update_value = 5;
        let expected_value = Some(update_value);
        let add_range_1 = (0, 0, 3, 3);
        let add_range_2 = (1, 1, 3, 3);
        let select_point_1 = (1, 1);
        let select_point_2 = (2, 2);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            base_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            overlap_delta,
        );
        tree.update(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            update_value,
        );
        let actual_value_11 = tree.select(select_point_1.0, select_point_1.1);
        let actual_value_22 = tree.select(select_point_2.0, select_point_2.1);

        // Assert
        assert_eq!(actual_value_11, expected_value);
        assert_eq!(actual_value_22, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_update_keeps_outside_cells_unchanged() {
        // Arrange
        let row = 4;
        let col = 4;
        let mut tree = create_fenwick_tree_2d(row, col);
        let base_delta = 4;
        let update_value = 9;
        let expected_outside_value = Some(base_delta);
        let add_range_1 = (0, 0, 4, 4);
        let update_range_1 = (1, 1, 3, 3);
        let select_point_1 = (0, 0);
        let select_point_2 = (3, 3);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            base_delta,
        );
        tree.update(
            update_range_1.0,
            update_range_1.1,
            update_range_1.2,
            update_range_1.3,
            update_value,
        );
        let actual_outside_value_top_left = tree.select(select_point_1.0, select_point_1.1);
        let actual_outside_value_bottom_right = tree.select(select_point_2.0, select_point_2.1);

        // Assert
        assert_eq!(actual_outside_value_top_left, expected_outside_value);
        assert_eq!(actual_outside_value_bottom_right, expected_outside_value);
    }

    #[test]
    fn fenwick_tree_2d_update_with_x1_equal_x2_keeps_state_unchanged() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let base_delta = 2;
        let update_value = 7;
        let expected_sum = Some(18);
        let add_range_1 = (0, 0, 3, 3);
        let update_range_1 = (1, 0, 1, 2);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            base_delta,
        );
        tree.update(
            update_range_1.0,
            update_range_1.1,
            update_range_1.2,
            update_range_1.3,
            update_value,
        );
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_update_with_out_of_bounds_range_keeps_state_unchanged() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let base_delta = 2;
        let update_value = 7;
        let expected_sum = Some(18);
        let add_range_1 = (0, 0, 3, 3);
        let update_range_1 = (0, 0, 4, 3);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            base_delta,
        );
        tree.update(
            update_range_1.0,
            update_range_1.1,
            update_range_1.2,
            update_range_1.3,
            update_value,
        );
        let actual_sum = tree.select_all();

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_sum_for_whole_range_returns_total_sum() {
        // Arrange
        let row = 3;
        let col = 2;
        let mut tree = create_fenwick_tree_2d(row, col);
        let first_delta = 4;
        let second_delta = 1;
        let expected_sum = Some(26);
        let add_range_1 = (0, 0, 3, 2);
        let add_range_2 = (1, 1, 3, 2);
        let sum_range_1 = (0, 0, row, col);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            first_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            second_delta,
        );
        let actual_sum = tree.sum(sum_range_1.0, sum_range_1.1, sum_range_1.2, sum_range_1.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_sum_for_sub_rectangle_returns_partial_sum() {
        // Arrange
        let row = 4;
        let col = 5;
        let mut tree = create_fenwick_tree_2d(row, col);
        let first_delta = 1;
        let second_delta = 4;
        let expected_sum = Some(22);
        let add_range_1 = (0, 0, 4, 5);
        let add_range_2 = (2, 1, 4, 3);
        let sum_range_1 = (1, 1, 4, 3);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            first_delta,
        );
        tree.add(
            add_range_2.0,
            add_range_2.1,
            add_range_2.2,
            add_range_2.3,
            second_delta,
        );
        let actual_sum = tree.sum(sum_range_1.0, sum_range_1.1, sum_range_1.2, sum_range_1.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_sum_with_x2_less_than_x1_returns_none() {
        // Arrange
        let row = 3;
        let col = 3;
        let tree = create_fenwick_tree_2d(row, col);
        let expected_sum: Option<TestValue> = None;
        let sum_range_1 = (2, 0, 1, 1);

        // Act
        let actual_sum = tree.sum(sum_range_1.0, sum_range_1.1, sum_range_1.2, sum_range_1.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_sum_with_y2_less_than_y1_returns_none() {
        // Arrange
        let row = 3;
        let col = 3;
        let tree = create_fenwick_tree_2d(row, col);
        let expected_sum: Option<TestValue> = None;
        let sum_range_1 = (0, 2, 1, 1);

        // Act
        let actual_sum = tree.sum(sum_range_1.0, sum_range_1.1, sum_range_1.2, sum_range_1.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_sum_with_out_of_bounds_range_returns_none() {
        // Arrange
        let row = 3;
        let col = 3;
        let tree = create_fenwick_tree_2d(row, col);
        let expected_sum: Option<TestValue> = None;
        let sum_range_1 = (0, 0, 4, 3);

        // Act
        let actual_sum = tree.sum(sum_range_1.0, sum_range_1.1, sum_range_1.2, sum_range_1.3);

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn fenwick_tree_2d_select_returns_same_value_as_single_cell_sum() {
        // Arrange
        let row = 3;
        let col = 3;
        let mut tree = create_fenwick_tree_2d(row, col);
        let delta = 5;
        let x = 1;
        let y = 2;
        let expected_value = Some(delta);
        let add_range_1 = (x, y, x + 1, y + 1);

        // Act
        tree.add(
            add_range_1.0,
            add_range_1.1,
            add_range_1.2,
            add_range_1.3,
            delta,
        );
        let actual_select_value = tree.select(x, y);
        let actual_sum_value = tree.sum(add_range_1.0, add_range_1.1, add_range_1.2, add_range_1.3);

        // Assert
        assert_eq!(actual_select_value, expected_value);
        assert_eq!(actual_sum_value, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_select_with_x_equal_row_returns_none() {
        // Arrange
        let row = 2;
        let col = 3;
        let tree = create_fenwick_tree_2d(row, col);
        let x = row;
        let y = 1;
        let expected_value: Option<TestValue> = None;

        // Act
        let actual_value = tree.select(x, y);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_select_with_y_equal_col_returns_none() {
        // Arrange
        let row = 2;
        let col = 3;
        let tree = create_fenwick_tree_2d(row, col);
        let x = 1;
        let y = col;
        let expected_value: Option<TestValue> = None;

        // Act
        let actual_value = tree.select(x, y);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn fenwick_tree_2d_add_accepts_column_upper_bound_and_ignores_one_beyond() {
        // Arrange
        let row = 2;
        let col = 3;
        let range = (0, 0, row, col);
        let invalid_range = (0, 0, row, col + 1);
        let value = 5;
        let ignored_value = 99;
        let expected_sum = Some(30);
        let expected_tail = Some(value);
        let last_cell = (row - 1, col - 1);
        let mut tree = create_fenwick_tree_2d(row, col);

        // Act
        tree.add(range.0, range.1, range.2, range.3, value);
        let actual_at_boundary = tree.select_all();
        tree.add(
            invalid_range.0,
            invalid_range.1,
            invalid_range.2,
            invalid_range.3,
            ignored_value,
        );
        let actual_after_invalid = tree.select_all();
        let actual_tail = tree.select(last_cell.0, last_cell.1);

        // Assert
        assert_eq!(actual_at_boundary, expected_sum);
        assert_eq!(actual_after_invalid, expected_sum);
        assert_eq!(actual_tail, expected_tail);
    }

    #[test]
    fn fenwick_tree_2d_update_accepts_column_upper_bound_and_ignores_one_beyond() {
        // Arrange
        let row = 2;
        let col = 3;
        let range = (0, 0, row, col);
        let invalid_range = (0, 0, row, col + 1);
        let value = 5;
        let ignored_value = 99;
        let expected_sum = Some(30);
        let expected_tail = Some(value);
        let last_cell = (row - 1, col - 1);
        let mut tree = create_fenwick_tree_2d(row, col);

        // Act
        tree.update(range.0, range.1, range.2, range.3, value);
        let actual_at_boundary = tree.select_all();
        tree.update(
            invalid_range.0,
            invalid_range.1,
            invalid_range.2,
            invalid_range.3,
            ignored_value,
        );
        let actual_after_invalid = tree.select_all();
        let actual_tail = tree.select(last_cell.0, last_cell.1);

        // Assert
        assert_eq!(actual_at_boundary, expected_sum);
        assert_eq!(actual_after_invalid, expected_sum);
        assert_eq!(actual_tail, expected_tail);
    }

    #[test]
    fn fenwick_tree_2d_sum_accepts_column_upper_bound_and_rejects_one_beyond() {
        // Arrange
        let row = 2;
        let col = 3;
        let range = (0, 0, row, col);
        let query_ends = [col - 1, col, col + 1];
        let value = 5;
        let expected_sums = [Some(20), Some(30), None];
        let mut tree = create_fenwick_tree_2d(row, col);
        tree.add(range.0, range.1, range.2, range.3, value);

        // Act
        let actual_sums = query_ends.map(|right| tree.sum(range.0, range.1, range.2, right));

        // Assert
        assert_eq!(actual_sums, expected_sums);
    }

    #[test]
    fn fenwick_tree_2d_zero_rows_returns_none_and_ignores_mutations() {
        // Arrange
        let row = 0;
        let col = 3;
        let point = (0, 0);
        let bounds = (0, 0, row, col);
        let value = 7;
        let expected_states = [(None, None); 3];
        let mut tree = create_fenwick_tree_2d(row, col);

        // Act
        let initial_state = (tree.select_all(), tree.select(point.0, point.1));
        tree.add(bounds.0, bounds.1, bounds.2, bounds.3, value);
        let after_add = (tree.select_all(), tree.select(point.0, point.1));
        tree.update(bounds.0, bounds.1, bounds.2, bounds.3, value);
        let after_update = (tree.select_all(), tree.select(point.0, point.1));
        let actual_states = [initial_state, after_add, after_update];

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn fenwick_tree_2d_zero_columns_returns_none_and_ignores_mutations() {
        // Arrange
        let row = 3;
        let col = 0;
        let point = (0, 0);
        let bounds = (0, 0, row, col);
        let value = 7;
        let expected_states = [(None, None); 3];
        let mut tree = create_fenwick_tree_2d(row, col);

        // Act
        let initial_state = (tree.select_all(), tree.select(point.0, point.1));
        tree.add(bounds.0, bounds.1, bounds.2, bounds.3, value);
        let after_add = (tree.select_all(), tree.select(point.0, point.1));
        tree.update(bounds.0, bounds.1, bounds.2, bounds.3, value);
        let after_update = (tree.select_all(), tree.select(point.0, point.1));
        let actual_states = [initial_state, after_add, after_update];

        // Assert
        assert_eq!(actual_states, expected_states);
    }
}
