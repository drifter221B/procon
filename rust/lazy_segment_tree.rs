pub trait MapMonoid {
    type S: Clone;
    type F: Clone;

    fn id() -> Self::S;
    fn identity_map() -> Self::F;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
    fn mapping(f: &Self::F, x: &Self::S) -> Self::S;
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

pub struct LazySegmentTree<M>
where
    M: MapMonoid,
{
    size: usize,
    leaf_size: usize,
    height: usize,
    data: Vec<M::S>,
    lazy: Vec<M::F>,
}

impl<M> LazySegmentTree<M>
where
    M: MapMonoid,
{

    pub fn new(size: usize) -> Self {

        let leaf_size = Self::leaf_size(size);
        let height = leaf_size.trailing_zeros() as usize;
        let data = vec![M::id(); leaf_size << 1];
        let lazy = vec![M::identity_map(); leaf_size << 1];

        Self {
            size,
            leaf_size,
            height,
            data,
            lazy,
        }
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

    pub fn select(&mut self, index: usize) -> Option<M::S> {
        if index < self.size {
            let idx = index + self.leaf_size;
            self.push_path(idx);
            Some(self.data[idx].clone())
        } else {
            None
        }
    }

    #[inline]
    pub fn select_all(&self) -> M::S {
        self.data[1].clone()
    }

    pub fn update(&mut self, index: usize, value: M::S) -> Option<()> {

        if self.size <= index {
            return None;
        }

        let idx = index + self.leaf_size;
        self.push_path(idx);
        self.data[idx] = value;
        self.pull_path(idx);

        Some(())
    }

    pub fn apply_point(&mut self, index: usize, f: M::F) -> Option<()> {
        self.apply(index, index + 1, f)
    }

    pub fn apply(&mut self, query_left: usize, query_right: usize, f: M::F) -> Option<()> {

        if query_left > query_right || self.size < query_right {
            return None;
        }

        if query_left == query_right {
            return Some(());
        }

        let left = query_left + self.leaf_size;
        let right = query_right + self.leaf_size;

        for i in (1..=self.height).rev() {

            if ((left >> i) << i) != left {
                self.push(left >> i);
            }

            if ((right >> i) << i) != right {
                self.push((right - 1) >> i);
            }
        }

        let mut tmp_left = left;
        let mut tmp_right = right;

        while tmp_left < tmp_right {

            if tmp_left & 1 == 1 {
                let f_clone = f.clone();
                self.apply_to_node(tmp_left, &f_clone);
                tmp_left += 1;
            }

            tmp_left >>= 1;

            if tmp_right & 1 == 1 {
                tmp_right -= 1;
                let f_clone = f.clone();
                self.apply_to_node(tmp_right, &f_clone);
            }

            tmp_right >>= 1;
        }

        for i in 1..=self.height {

            if ((left >> i) << i) != left {
                self.pull(left >> i);
            }

            if ((right >> i) << i) != right {
                self.pull((right - 1) >> i);
            }
        }

        Some(())
    }

    pub fn query(&mut self, query_left: usize, query_right: usize) -> Option<M::S> {

        if query_left > query_right || self.size < query_right {
            return None;
        }

        if query_left == query_right {
            return Some(M::id());
        }

        let mut left = query_left + self.leaf_size;
        let mut right = query_right + self.leaf_size;

        for i in (1..=self.height).rev() {

            if ((left >> i) << i) != left {
                self.push(left >> i);
            }

            if ((right >> i) << i) != right {
                self.push((right - 1) >> i);
            }
        }

        let mut left_result = M::id();
        let mut right_result = M::id();

        while left < right {

            if left & 1 == 1 {
                left_result = M::binary_operation(&left_result, &self.data[left]);
                left += 1;
            }

            left >>= 1;

            if right & 1 == 1 {
                right -= 1;
                right_result = M::binary_operation(&self.data[right], &right_result);
            }

            right >>= 1;
        }

        Some(M::binary_operation(&left_result, &right_result))
    }

    // query_left is inclusive, returned index is not inclusive
    pub fn max_right<P>(&mut self, query_left: usize, func: P) -> Option<usize>
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

        let mut index = query_left + self.leaf_size;
        for i in (1..=self.height).rev() {
            self.push(index >> i);
        }

        let mut result = M::id();

        loop {

            while index & 1 == 0 {
                index >>= 1;
            }

            let next = M::binary_operation(&result, &self.data[index]);
            if !func(&next) {

                while index < self.leaf_size {

                    self.push(index);
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
    pub fn min_left<P>(&mut self, query_right: usize, func: P) -> Option<usize>
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

        let mut index = query_right + self.leaf_size;
        for i in (1..=self.height).rev() {
            self.push((index - 1) >> i);
        }

        let mut result = M::id();

        loop {

            index -= 1;

            while 1 < index && index & 1 == 1 {
                index >>= 1;
            }

            let next = M::binary_operation(&self.data[index], &result);
            if !func(&next) {

                while index < self.leaf_size {

                    self.push(index);
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

    #[inline]
    fn apply_to_node(&mut self, index: usize, f: &M::F) {
        self.data[index] = M::mapping(f, &self.data[index]);
        if index < self.leaf_size {
            self.lazy[index] = M::composition(f, &self.lazy[index]);
        }
    }

    fn push(&mut self, index: usize) {
        if index < self.leaf_size {
            let f = self.lazy[index].clone();
            self.apply_to_node(index << 1, &f);
            self.apply_to_node(index << 1 | 1, &f);
            self.lazy[index] = M::identity_map();
        }
    }

    #[inline]
    fn pull(&mut self, index: usize) {
        self.data[index] = M::binary_operation(&self.data[index << 1], &self.data[index << 1 | 1]);
    }

    fn push_path(&mut self, index: usize) {
        for i in (1..=self.height).rev() {
            self.push(index >> i);
        }
    }

    fn pull_path(&mut self, index: usize) {
        for i in 1..=self.height {
            self.pull(index >> i);
        }
    }

    fn build_parents(&mut self) {
        for idx in (1..self.leaf_size).rev() {
            self.pull(idx);
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

#[derive(Clone, Debug, Default)]
struct T;

impl MapMonoid for T {
    type S = (usize, usize, usize);
    type F = usize;

    fn id() -> Self::S {
        (0, 0, 0)
    }

    fn identity_map() -> Self::F {
        0
    }

    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
        (lhs.0 + rhs.0, lhs.1 + rhs.1, lhs.2 + rhs.2 + lhs.1 * rhs.0)
    }

    fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
        if *f & 1 == 0 {
            *x
        } else {
            (x.1, x.0, x.1 * x.0 - x.2)
        }
    }

    // f(g(x))
    fn composition(f: &Self::F, g: &Self::F) -> Self::F {
        *f + *g
    }

}

#[cfg(test)]
mod tests {

    use super::{LazySegmentTree, MapMonoid};

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct SumNode {
        range_sum: i64,
        range_len: i64,
    }

    #[derive(Clone, Copy)]
    struct RangeAddSumMonoid;

    impl MapMonoid for RangeAddSumMonoid {
        type S = SumNode;
        type F = i64;

        fn id() -> Self::S {
            SumNode {
                range_sum: 0,
                range_len: 0,
            }
        }

        fn identity_map() -> Self::F {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            SumNode {
                range_sum: lhs.range_sum + rhs.range_sum,
                range_len: lhs.range_len + rhs.range_len,
            }
        }

        fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
            SumNode {
                range_sum: x.range_sum + f * x.range_len,
                range_len: x.range_len,
            }
        }

        fn composition(f: &Self::F, g: &Self::F) -> Self::F {
            *f + *g
        }
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AffineNode {
        range_sum: i64,
        range_len: i64,
    }

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct AffineMap {
        mul_value: i64,
        add_value: i64,
    }

    #[derive(Clone, Copy)]
    struct AffineSumMonoid;

    impl MapMonoid for AffineSumMonoid {
        type S = AffineNode;
        type F = AffineMap;

        fn id() -> Self::S {
            AffineNode {
                range_sum: 0,
                range_len: 0,
            }
        }

        fn identity_map() -> Self::F {
            AffineMap {
                mul_value: 1,
                add_value: 0,
            }
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            AffineNode {
                range_sum: lhs.range_sum + rhs.range_sum,
                range_len: lhs.range_len + rhs.range_len,
            }
        }

        fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
            AffineNode {
                range_sum: f.mul_value * x.range_sum + f.add_value * x.range_len,
                range_len: x.range_len,
            }
        }

        fn composition(f: &Self::F, g: &Self::F) -> Self::F {
            AffineMap {
                mul_value: f.mul_value * g.mul_value,
                add_value: f.mul_value * g.add_value + f.add_value,
            }
        }
    }

    #[derive(Clone, Copy)]
    struct StringConcatToggleMonoid;

    impl MapMonoid for StringConcatToggleMonoid {
        type S = String;
        type F = bool;

        fn id() -> Self::S {
            String::new()
        }

        fn identity_map() -> Self::F {
            false
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            let mut merged_text = lhs.clone();
            merged_text.push_str(rhs);
            merged_text
        }

        fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
            if *f {
                toggle_ascii_case_text(x)
            } else {
                x.clone()
            }
        }

        fn composition(f: &Self::F, g: &Self::F) -> Self::F {
            *f ^ *g
        }
    }

    fn toggle_ascii_case_text(text: &str) -> String {
        text.chars()
            .map(|ch| {
                if ch.is_ascii_lowercase() {
                    ch.to_ascii_uppercase()
                } else if ch.is_ascii_uppercase() {
                    ch.to_ascii_lowercase()
                } else {
                    ch
                }
            })
            .collect()
    }

    fn create_sum_node(value: i64) -> SumNode {
        SumNode {
            range_sum: value,
            range_len: 1,
        }
    }

    fn create_affine_node(value: i64) -> AffineNode {
        AffineNode {
            range_sum: value,
            range_len: 1,
        }
    }

    fn create_sum_tree(values: &[i64]) -> LazySegmentTree<RangeAddSumMonoid> {
        let sum_nodes: Vec<SumNode> = values.iter().copied().map(create_sum_node).collect();
        LazySegmentTree::from_slice(&sum_nodes)
    }

    fn create_affine_tree(values: &[i64]) -> LazySegmentTree<AffineSumMonoid> {
        let affine_nodes: Vec<AffineNode> =
            values.iter().copied().map(create_affine_node).collect();
        LazySegmentTree::from_slice(&affine_nodes)
    }

    fn create_string_tree(values: &[&str]) -> LazySegmentTree<StringConcatToggleMonoid> {
        let text_values: Vec<String> = values.iter().map(|value| (*value).to_string()).collect();
        LazySegmentTree::from_slice(&text_values)
    }

    fn query_sum(
        tree: &mut LazySegmentTree<RangeAddSumMonoid>,
        query_left: usize,
        query_right: usize,
    ) -> Option<i64> {
        tree.query(query_left, query_right)
            .map(|node| node.range_sum)
    }

    fn select_sum(tree: &mut LazySegmentTree<RangeAddSumMonoid>, index: usize) -> Option<i64> {
        tree.select(index).map(|node| node.range_sum)
    }

    fn query_affine_sum(
        tree: &mut LazySegmentTree<AffineSumMonoid>,
        query_left: usize,
        query_right: usize,
    ) -> Option<i64> {
        tree.query(query_left, query_right)
            .map(|node| node.range_sum)
    }

    fn select_affine_sum(tree: &mut LazySegmentTree<AffineSumMonoid>, index: usize) -> Option<i64> {
        tree.select(index).map(|node| node.range_sum)
    }

    fn query_text(
        tree: &mut LazySegmentTree<StringConcatToggleMonoid>,
        query_left: usize,
        query_right: usize,
    ) -> Option<String> {
        tree.query(query_left, query_right)
    }

    fn does_not_contain_ab(text: &str) -> bool {
        !text.contains("ab")
    }

    #[test]
    fn new_with_size_zero_returns_empty_state_and_identity_select_all() {
        // Arrange
        let tree_size = 0usize;
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_select_all = 0i64;
        let tree: LazySegmentTree<RangeAddSumMonoid> = LazySegmentTree::new(tree_size);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_select_all = tree.select_all().range_sum;

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_select_all, expected_select_all);
    }

    #[test]
    fn new_with_size_zero_select_returns_none_at_index_zero() {
        // Arrange
        let tree_size = 0usize;
        let select_index = 0usize;
        let expected_select_result = None;
        let mut tree: LazySegmentTree<RangeAddSumMonoid> = LazySegmentTree::new(tree_size);

        // Act
        let actual_select_result = select_sum(&mut tree, select_index);

        // Assert
        assert_eq!(actual_select_result, expected_select_result);
    }

    #[test]
    fn new_with_single_size_returns_non_empty_and_identity_values() {
        // Arrange
        let tree_size = 1usize;
        let select_index = 0usize;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_len = 1usize;
        let expected_is_empty = false;
        let expected_select_value = Some(0i64);
        let expected_query_value = Some(0i64);
        let mut tree: LazySegmentTree<RangeAddSumMonoid> = LazySegmentTree::new(tree_size);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_select_value = select_sum(&mut tree, select_index);
        let actual_query_value = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_select_value, expected_select_value);
        assert_eq!(actual_query_value, expected_query_value);
    }

    #[test]
    fn from_slice_with_empty_values_returns_identity_for_query_zero_zero() {
        // Arrange
        let values: Vec<i64> = vec![];
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_query_result = Some(0i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_query_result = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn from_slice_with_single_element_keeps_select_query_and_select_all_equal() {
        // Arrange
        let single_value = 42i64;
        let values = vec![single_value];
        let select_index = 0usize;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_selected_value = Some(single_value);
        let expected_query_value = Some(single_value);
        let expected_select_all_value = single_value;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_selected_value = select_sum(&mut tree, select_index);
        let actual_query_value = query_sum(&mut tree, query_left, query_right);
        let actual_select_all_value = tree.select_all().range_sum;

        // Assert
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_query_value, expected_query_value);
        assert_eq!(actual_select_all_value, expected_select_all_value);
    }

    #[test]
    fn from_slice_with_non_power_of_two_length_does_not_include_dummy_leaf_in_query() {
        // Arrange
        let values = vec![3i64, 5, 7];
        let all_query_left = 0usize;
        let all_query_right = 3usize;
        let tail_query_left = 2usize;
        let tail_query_right = 3usize;
        let expected_all_sum = Some(15i64);
        let expected_tail_sum = Some(7i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_all_sum = query_sum(&mut tree, all_query_left, all_query_right);
        let actual_tail_sum = query_sum(&mut tree, tail_query_left, tail_query_right);

        // Assert
        assert_eq!(actual_all_sum, expected_all_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
    }

    #[test]
    fn from_slice_with_empty_values_reports_empty_and_select_none() {
        // Arrange
        let values: Vec<i64> = vec![];
        let select_index = 0usize;
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_select_value = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_len = tree.len();
        let actual_is_empty = tree.is_empty();
        let actual_select_value = select_sum(&mut tree, select_index);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_select_value, expected_select_value);
    }

    #[test]
    fn select_boundary_indices_return_some_for_valid_range_and_none_for_size() {
        // Arrange
        let values = vec![11i64, 22, 33];
        let lower_index = 0usize;
        let upper_index = 2usize;
        let outside_index = 3usize;
        let expected_lower_value = Some(11i64);
        let expected_upper_value = Some(33i64);
        let expected_outside_value = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_lower_value = select_sum(&mut tree, lower_index);
        let actual_upper_value = select_sum(&mut tree, upper_index);
        let actual_outside_value = select_sum(&mut tree, outside_index);

        // Assert
        assert_eq!(actual_lower_value, expected_lower_value);
        assert_eq!(actual_upper_value, expected_upper_value);
        assert_eq!(actual_outside_value, expected_outside_value);
    }

    #[test]
    fn select_after_lazy_pending_value_returns_latest_value_via_push_path() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let apply_left = 0usize;
        let apply_right = 4usize;
        let apply_delta = 10i64;
        let select_index = 2usize;
        let point_query_left = 2usize;
        let point_query_right = 3usize;
        let expected_select_value = Some(13i64);
        let expected_point_query_sum = Some(13i64);
        let mut tree = create_sum_tree(&values);
        let expected_check_1 = Some(());

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_delta);
        let actual_select_value = select_sum(&mut tree, select_index);
        let actual_point_query_sum = query_sum(&mut tree, point_query_left, point_query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_check_1);
        assert_eq!(actual_select_value, expected_select_value);
        assert_eq!(actual_point_query_sum, expected_point_query_sum);
    }

    #[test]
    fn select_all_always_matches_query_from_zero_to_size() {
        // Arrange
        let values = vec![4i64, 1, 6, 2, 9];
        let query_left = 0usize;
        let query_right = values.len();
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_select_all_sum = tree.select_all().range_sum;
        let actual_query_sum = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_query_sum, Some(actual_select_all_sum));
    }

    #[test]
    fn empty_tree_update_returns_none_and_keeps_select_all_identity() {
        // Arrange
        let values: Vec<i64> = vec![];
        let update_index = 0usize;
        let updated_value = 100i64;
        let expected_update_result = None;
        let expected_select_all_sum = 0i64;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_update_result = tree.update(update_index, create_sum_node(updated_value));
        let actual_select_all_sum = tree.select_all().range_sum;

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_select_all_sum, expected_select_all_sum);
    }

    #[test]
    fn update_first_index_changes_only_head_segment_values() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let update_index = 0usize;
        let updated_value = 10i64;
        let head_query_left = 0usize;
        let head_query_right = 1usize;
        let tail_query_left = 1usize;
        let tail_query_right = 3usize;
        let expected_update_result = Some(());
        let expected_head_sum = Some(10i64);
        let expected_tail_sum = Some(5i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_update_result = tree.update(update_index, create_sum_node(updated_value));
        let actual_head_sum = query_sum(&mut tree, head_query_left, head_query_right);
        let actual_tail_sum = query_sum(&mut tree, tail_query_left, tail_query_right);

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_head_sum, expected_head_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
    }

    #[test]
    fn update_last_index_changes_only_tail_segment_values() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let update_index = 2usize;
        let updated_value = 30i64;
        let head_query_left = 0usize;
        let head_query_right = 2usize;
        let tail_query_left = 2usize;
        let tail_query_right = 3usize;
        let expected_update_result = Some(());
        let expected_head_sum = Some(3i64);
        let expected_tail_sum = Some(30i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_update_result = tree.update(update_index, create_sum_node(updated_value));
        let actual_head_sum = query_sum(&mut tree, head_query_left, head_query_right);
        let actual_tail_sum = query_sum(&mut tree, tail_query_left, tail_query_right);

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_head_sum, expected_head_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
    }

    #[test]
    fn update_with_out_of_range_index_returns_none_and_preserves_tree_state() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let update_index = 3usize;
        let updated_value = 99i64;
        let query_left = 0usize;
        let query_right = 3usize;
        let expected_update_result = None;
        let expected_total_sum = Some(6i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_before_total_sum = query_sum(&mut tree, query_left, query_right);
        let actual_update_result = tree.update(update_index, create_sum_node(updated_value));
        let actual_after_total_sum = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_before_total_sum, expected_total_sum);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_after_total_sum, expected_total_sum);
    }

    #[test]
    fn apply_point_on_first_index_updates_only_first_element() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let apply_index = 0usize;
        let apply_delta = 4i64;
        let first_query_left = 0usize;
        let first_query_right = 1usize;
        let rest_query_left = 1usize;
        let rest_query_right = 3usize;
        let expected_apply_result = Some(());
        let expected_first_sum = Some(5i64);
        let expected_rest_sum = Some(5i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply_point(apply_index, apply_delta);
        let actual_first_sum = query_sum(&mut tree, first_query_left, first_query_right);
        let actual_rest_sum = query_sum(&mut tree, rest_query_left, rest_query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_first_sum, expected_first_sum);
        assert_eq!(actual_rest_sum, expected_rest_sum);
    }

    #[test]
    fn apply_point_on_last_index_updates_only_last_element() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let apply_index = 2usize;
        let apply_delta = 4i64;
        let first_query_left = 0usize;
        let first_query_right = 2usize;
        let last_query_left = 2usize;
        let last_query_right = 3usize;
        let expected_apply_result = Some(());
        let expected_first_sum = Some(3i64);
        let expected_last_sum = Some(7i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply_point(apply_index, apply_delta);
        let actual_first_sum = query_sum(&mut tree, first_query_left, first_query_right);
        let actual_last_sum = query_sum(&mut tree, last_query_left, last_query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_first_sum, expected_first_sum);
        assert_eq!(actual_last_sum, expected_last_sum);
    }

    #[test]
    fn apply_point_with_out_of_range_index_returns_none_and_keeps_state() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let apply_index = 3usize;
        let apply_delta = 4i64;
        let query_left = 0usize;
        let query_right = 3usize;
        let expected_apply_result = None;
        let expected_total_sum = Some(6i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_before_total_sum = query_sum(&mut tree, query_left, query_right);
        let actual_apply_result = tree.apply_point(apply_index, apply_delta);
        let actual_after_total_sum = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_before_total_sum, expected_total_sum);
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_after_total_sum, expected_total_sum);
    }

    #[test]
    fn empty_tree_apply_point_returns_none_and_keeps_select_all_identity() {
        // Arrange
        let values: Vec<i64> = vec![];
        let apply_index = 0usize;
        let apply_delta = 5i64;
        let expected_apply_result = None;
        let expected_select_all_sum = 0i64;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply_point(apply_index, apply_delta);
        let actual_select_all_sum = tree.select_all().range_sum;

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_select_all_sum, expected_select_all_sum);
    }

    #[test]
    fn apply_point_and_single_range_apply_return_same_result_and_state() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let apply_index = 1usize;
        let apply_delta = 5i64;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_apply_result = Some(());
        let expected_total_sum = Some(11i64);
        let expected_target_sum = Some(7i64);
        let mut point_tree = create_sum_tree(&values);
        let mut range_tree = create_sum_tree(&values);

        // Act
        let actual_point_apply_result = point_tree.apply_point(apply_index, apply_delta);
        let actual_range_apply_result =
            range_tree.apply(apply_index, apply_index + 1usize, apply_delta);
        let actual_point_total_sum = query_sum(&mut point_tree, query_left, query_right);
        let actual_range_total_sum = query_sum(&mut range_tree, query_left, query_right);
        let actual_point_target_sum = select_sum(&mut point_tree, apply_index);
        let actual_range_target_sum = select_sum(&mut range_tree, apply_index);

        // Assert
        assert_eq!(actual_point_apply_result, expected_apply_result);
        assert_eq!(actual_range_apply_result, expected_apply_result);
        assert_eq!(actual_point_total_sum, expected_total_sum);
        assert_eq!(actual_range_total_sum, expected_total_sum);
        assert_eq!(actual_point_target_sum, expected_target_sum);
        assert_eq!(actual_range_target_sum, expected_target_sum);
    }

    #[test]
    fn non_empty_tree_apply_len_len_returns_some_and_keeps_state() {
        // Arrange
        let input_value_1 = 0;
        let values = vec![1i64, 2, 3];
        let query_left = values.len();
        let query_right = values.len();
        let apply_delta = 10i64;
        let expected_apply_result = Some(());
        let expected_total_sum = Some(6i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);
        let actual_total_sum = query_sum(&mut tree, input_value_1, values.len());

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_total_sum, expected_total_sum);
    }

    #[test]
    fn apply_with_left_greater_than_right_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 2usize;
        let query_right = 1usize;
        let apply_delta = 5i64;
        let expected_apply_result = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
    }

    #[test]
    fn apply_with_query_right_larger_than_size_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 0usize;
        let query_right = 4usize;
        let apply_delta = 5i64;
        let expected_apply_result = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
    }

    #[test]
    fn apply_with_empty_interval_returns_some_and_does_not_change_state() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 1usize;
        let query_right = 1usize;
        let apply_delta = 9i64;
        let total_query_left = 0usize;
        let total_query_right = 3usize;
        let point_query_left = 1usize;
        let point_query_right = 2usize;
        let expected_apply_result = Some(());
        let expected_total_sum = Some(6i64);
        let expected_point_sum = Some(2i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_before_total_sum = query_sum(&mut tree, total_query_left, total_query_right);
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);
        let actual_after_total_sum = query_sum(&mut tree, total_query_left, total_query_right);
        let actual_point_sum = query_sum(&mut tree, point_query_left, point_query_right);

        // Assert
        assert_eq!(actual_before_total_sum, expected_total_sum);
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_after_total_sum, expected_total_sum);
        assert_eq!(actual_point_sum, expected_point_sum);
    }

    #[test]
    fn apply_on_full_interval_applies_mapping_to_all_elements() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 0usize;
        let query_right = 3usize;
        let apply_delta = 10i64;
        let all_query_left = 0usize;
        let all_query_right = 3usize;
        let middle_query_left = 1usize;
        let middle_query_right = 2usize;
        let expected_apply_result = Some(());
        let expected_all_sum = Some(36i64);
        let expected_middle_sum = Some(12i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);
        let actual_all_sum = query_sum(&mut tree, all_query_left, all_query_right);
        let actual_middle_sum = query_sum(&mut tree, middle_query_left, middle_query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_all_sum, expected_all_sum);
        assert_eq!(actual_middle_sum, expected_middle_sum);
    }

    #[test]
    fn overlapping_apply_follows_composition_order_for_affine_maps() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let first_query_left = 0usize;
        let first_query_right = 3usize;
        let second_query_left = 0usize;
        let second_query_right = 3usize;
        let first_map = AffineMap {
            mul_value: 2,
            add_value: 1,
        };
        let second_map = AffineMap {
            mul_value: 3,
            add_value: 4,
        };
        let total_query_left = 0usize;
        let total_query_right = 3usize;
        let first_index = 0usize;
        let expected_first_apply_result = Some(());
        let expected_second_apply_result = Some(());
        let expected_total_sum = Some(57i64);
        let expected_first_value = Some(13i64);
        let mut tree = create_affine_tree(&values);

        // Act
        let actual_first_apply_result = tree.apply(first_query_left, first_query_right, first_map);
        let actual_second_apply_result =
            tree.apply(second_query_left, second_query_right, second_map);
        let actual_total_sum = query_affine_sum(&mut tree, total_query_left, total_query_right);
        let actual_first_value = select_affine_sum(&mut tree, first_index);

        // Assert
        assert_eq!(actual_first_apply_result, expected_first_apply_result);
        assert_eq!(actual_second_apply_result, expected_second_apply_result);
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_first_value, expected_first_value);
    }

    #[test]
    fn apply_on_disjoint_intervals_does_not_interfere_each_other() {
        // Arrange
        let values = vec![1i64, 2, 3, 4, 5];
        let first_query_left = 0usize;
        let first_query_right = 2usize;
        let first_apply_delta = 10i64;
        let second_query_left = 3usize;
        let second_query_right = 5usize;
        let second_apply_delta = 20i64;
        let left_check_left = 0usize;
        let left_check_right = 2usize;
        let middle_check_left = 2usize;
        let middle_check_right = 3usize;
        let right_check_left = 3usize;
        let right_check_right = 5usize;
        let expected_left_sum = Some(23i64);
        let expected_middle_sum = Some(3i64);
        let expected_right_sum = Some(49i64);
        let mut tree = create_sum_tree(&values);
        let expected_check_1 = Some(());
        let expected_check_2 = Some(());

        // Act
        let actual_first_apply_result =
            tree.apply(first_query_left, first_query_right, first_apply_delta);
        let actual_second_apply_result =
            tree.apply(second_query_left, second_query_right, second_apply_delta);
        let actual_left_sum = query_sum(&mut tree, left_check_left, left_check_right);
        let actual_middle_sum = query_sum(&mut tree, middle_check_left, middle_check_right);
        let actual_right_sum = query_sum(&mut tree, right_check_left, right_check_right);

        // Assert
        assert_eq!(actual_first_apply_result, expected_check_1);
        assert_eq!(actual_second_apply_result, expected_check_2);
        assert_eq!(actual_left_sum, expected_left_sum);
        assert_eq!(actual_middle_sum, expected_middle_sum);
        assert_eq!(actual_right_sum, expected_right_sum);
    }

    #[test]
    fn empty_tree_apply_zero_zero_returns_some_and_keeps_state() {
        // Arrange
        let values: Vec<i64> = vec![];
        let query_left = 0usize;
        let query_right = 0usize;
        let apply_delta = 7i64;
        let expected_apply_result = Some(());
        let expected_query_result = Some(0i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);
        let actual_query_result = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn empty_tree_apply_zero_one_returns_none() {
        // Arrange
        let values: Vec<i64> = vec![];
        let query_left = 0usize;
        let query_right = 1usize;
        let apply_delta = 7i64;
        let expected_apply_result = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
    }

    #[test]
    fn string_monoid_apply_with_false_keeps_range_unchanged() {
        // Arrange
        let values = vec!["Ab", "cD", "ef"];
        let apply_left = 0usize;
        let apply_right = 2usize;
        let apply_flag = false;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_apply_result = Some(());
        let expected_query_text = Some(String::from("AbcDef"));
        let mut tree = create_string_tree(&values);

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_query_text = query_text(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_query_text, expected_query_text);
    }

    #[test]
    fn string_monoid_apply_true_twice_on_same_interval_restores_original_text() {
        // Arrange
        let values = vec!["ab", "Cd", "eF"];
        let apply_left = 0usize;
        let apply_right = values.len();
        let apply_flag = true;
        let query_left = 0usize;
        let query_right = values.len();
        let expected_first_apply_result = Some(());
        let expected_second_apply_result = Some(());
        let expected_query_text = Some(String::from("abCdeF"));
        let mut tree = create_string_tree(&values);

        // Act
        let actual_first_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_second_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_query_text = query_text(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_first_apply_result, expected_first_apply_result);
        assert_eq!(actual_second_apply_result, expected_second_apply_result);
        assert_eq!(actual_query_text, expected_query_text);
    }

    #[test]
    fn affine_monoid_partial_apply_updates_only_target_interval() {
        // Arrange
        let input_value_1 = 0usize;
        let input_value_2 = 1usize;
        let input_value_3 = 1usize;
        let input_value_4 = 3usize;
        let input_value_5 = 3usize;
        let input_value_6 = 4usize;
        let input_value_7 = 0usize;
        let values = vec![1i64, 2, 3, 4];
        let apply_left = 1usize;
        let apply_right = 3usize;
        let apply_map = AffineMap {
            mul_value: 2,
            add_value: 3,
        };
        let expected_apply_result = Some(());
        let expected_left_outside_sum = Some(1i64);
        let expected_target_sum = Some(16i64);
        let expected_right_outside_sum = Some(4i64);
        let expected_total_sum = Some(21i64);
        let mut tree = create_affine_tree(&values);

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_map);
        let actual_left_outside_sum = query_affine_sum(&mut tree, input_value_1, input_value_2);
        let actual_target_sum = query_affine_sum(&mut tree, input_value_3, input_value_4);
        let actual_right_outside_sum = query_affine_sum(&mut tree, input_value_5, input_value_6);
        let actual_total_sum = query_affine_sum(&mut tree, input_value_7, values.len());

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_left_outside_sum, expected_left_outside_sum);
        assert_eq!(actual_target_sum, expected_target_sum);
        assert_eq!(actual_right_outside_sum, expected_right_outside_sum);
        assert_eq!(actual_total_sum, expected_total_sum);
    }

    #[test]
    fn apply_with_negative_delta_subtracts_values_on_target_interval() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 2;
        let input_value_4 = 2;
        let input_value_5 = 3;
        let values = vec![5i64, 6, 7];
        let query_left = 1usize;
        let query_right = 3usize;
        let apply_delta = -2i64;
        let expected_apply_result = Some(());
        let expected_total_sum = Some(14i64);
        let expected_middle_sum = Some(4i64);
        let expected_tail_sum = Some(5i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);
        let actual_total_sum = query_sum(&mut tree, input_value_1, values.len());
        let actual_middle_sum = query_sum(&mut tree, input_value_2, input_value_3);
        let actual_tail_sum = query_sum(&mut tree, input_value_4, input_value_5);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_middle_sum, expected_middle_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
    }

    #[test]
    fn apply_with_zero_delta_keeps_values_unchanged() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 2;
        let values = vec![5i64, 6, 7];
        let query_left = 0usize;
        let query_right = 3usize;
        let apply_delta = 0i64;
        let expected_apply_result = Some(());
        let expected_total_sum = Some(18i64);
        let expected_middle_sum = Some(6i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_apply_result = tree.apply(query_left, query_right, apply_delta);
        let actual_total_sum = query_sum(&mut tree, input_value_1, values.len());
        let actual_middle_sum = query_sum(&mut tree, input_value_2, input_value_3);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_middle_sum, expected_middle_sum);
    }

    #[test]
    fn non_commutative_monoid_keeps_query_order_after_apply() {
        // Arrange
        let values = vec!["ab", "cd", "ef"];
        let apply_left = 0usize;
        let apply_right = 2usize;
        let apply_flag = true;
        let query_left = 0usize;
        let query_right = 3usize;
        let expected_apply_result = Some(());
        let expected_query_text = Some(String::from("ABCDef"));
        let mut tree = create_string_tree(&values);

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_query_text = query_text(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_query_text, expected_query_text);
    }

    #[test]
    fn query_with_left_greater_than_right_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 2usize;
        let query_right = 1usize;
        let expected_query_result = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_query_result = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_with_query_right_larger_than_size_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 0usize;
        let query_right = 4usize;
        let expected_query_result = None;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_query_result = query_sum(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_query_result, expected_query_result);
    }

    #[test]
    fn query_empty_intervals_return_identity_for_zero_zero_and_size_size() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let first_query_left = 0usize;
        let first_query_right = 0usize;
        let second_query_left = 3usize;
        let second_query_right = 3usize;
        let expected_first_query_result = Some(0i64);
        let expected_second_query_result = Some(0i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_first_query_result = query_sum(&mut tree, first_query_left, first_query_right);
        let actual_second_query_result =
            query_sum(&mut tree, second_query_left, second_query_right);

        // Assert
        assert_eq!(actual_first_query_result, expected_first_query_result);
        assert_eq!(actual_second_query_result, expected_second_query_result);
    }

    #[test]
    fn query_boundary_intervals_return_expected_values() {
        // Arrange
        let values = vec![2i64, 4, 6];
        let first_query_left = 0usize;
        let first_query_right = 1usize;
        let second_query_left = 2usize;
        let second_query_right = 3usize;
        let third_query_left = 0usize;
        let third_query_right = 3usize;
        let expected_first_query_result = Some(2i64);
        let expected_second_query_result = Some(6i64);
        let expected_third_query_result = Some(12i64);
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_first_query_result = query_sum(&mut tree, first_query_left, first_query_right);
        let actual_second_query_result =
            query_sum(&mut tree, second_query_left, second_query_right);
        let actual_third_query_result = query_sum(&mut tree, third_query_left, third_query_right);

        // Assert
        assert_eq!(actual_first_query_result, expected_first_query_result);
        assert_eq!(actual_second_query_result, expected_second_query_result);
        assert_eq!(actual_third_query_result, expected_third_query_result);
    }

    #[test]
    fn max_right_with_query_left_larger_than_size_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 4usize;
        let expected_max_right = None;
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 100;

        // Act
        let actual_max_right =
            tree.max_right(query_left, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn max_right_with_query_left_equal_size_returns_size() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 3usize;
        let expected_max_right = Some(3usize);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 100;

        // Act
        let actual_max_right =
            tree.max_right(query_left, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn max_right_with_always_true_predicate_returns_size() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 0usize;
        let expected_max_right = Some(3usize);
        let mut tree = create_sum_tree(&values);

        let predicate_result_1 = true;

        // Act
        let actual_max_right = tree.max_right(query_left, |_node| predicate_result_1);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn max_right_returns_query_left_when_first_element_breaks_predicate() {
        // Arrange
        let values = vec![5i64, 1, 1];
        let query_left = 1usize;
        let expected_max_right = Some(1usize);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 0;

        // Act
        let actual_max_right =
            tree.max_right(query_left, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn max_right_after_apply_uses_lazy_values_and_returns_correct_boundary() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let apply_left = 1usize;
        let apply_right = 4usize;
        let apply_delta = 2i64;
        let query_left = 1usize;
        let expected_max_right = Some(2usize);
        let mut tree = create_sum_tree(&values);
        let expected_check_1 = Some(());

        let predicate_threshold_1 = 8;

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_delta);
        let actual_max_right =
            tree.max_right(query_left, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_apply_result, expected_check_1);
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn max_right_and_min_left_with_same_predicate_return_consistent_boundaries() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 1;
        let input_value_3 = 3;
        let input_value_4 = 0;
        let input_value_5 = 3;
        let values = vec![1i64, 2, 3, 4];
        let query_left = 1usize;
        let expected_max_right = Some(3usize);
        let expected_min_left = Some(1usize);
        let expected_target_range_sum = Some(5i64);
        let expected_extended_range_sum = Some(6i64);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 5;
        let predicate_threshold_2 = 5;

        // Act
        let actual_max_right =
            tree.max_right(query_left, |node| node.range_sum <= predicate_threshold_1);
        let actual_min_left = tree.min_left(input_value_1, |node| {
            node.range_sum <= predicate_threshold_2
        });
        let actual_target_range_sum = query_sum(&mut tree, input_value_2, input_value_3);
        let actual_extended_range_sum = query_sum(&mut tree, input_value_4, input_value_5);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
        assert_eq!(actual_min_left, expected_min_left);
        assert_eq!(actual_target_range_sum, expected_target_range_sum);
        assert_eq!(actual_extended_range_sum, expected_extended_range_sum);
    }

    #[test]
    fn empty_tree_max_right_from_zero_returns_zero() {
        // Arrange
        let values: Vec<i64> = vec![];
        let query_left = 0usize;
        let expected_max_right = Some(0usize);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 0;

        // Act
        let actual_max_right =
            tree.max_right(query_left, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn non_commutative_monoid_max_right_after_apply_uses_lazy_values() {
        // Arrange
        let values = vec!["ab", "cd", "ef"];
        let apply_left = 0usize;
        let apply_right = 2usize;
        let apply_flag = true;
        let max_right_query_left = 0usize;
        let expected_apply_result = Some(());
        let expected_max_right = Some(1usize);
        let expected_selected_text = Some(String::from("AB"));
        let expected_prefix = String::from("AB");
        let mut tree = create_string_tree(&values);

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_max_right = tree.max_right(max_right_query_left, |aggregated_text| {
            expected_prefix.starts_with(aggregated_text.as_str())
        });
        let actual_selected_text = actual_max_right
            .and_then(|boundary_index| query_text(&mut tree, max_right_query_left, boundary_index));

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_max_right, expected_max_right);
        assert_eq!(actual_selected_text, expected_selected_text);
    }

    #[test]
    fn max_right_with_middle_left_and_always_true_predicate_returns_size() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let query_left = 2usize;
        let expected_max_right = Some(values.len());
        let mut tree = create_sum_tree(&values);

        let predicate_result_1 = true;

        // Act
        let actual_max_right = tree.max_right(query_left, |_node| predicate_result_1);

        // Assert
        assert_eq!(actual_max_right, expected_max_right);
    }

    #[test]
    fn max_right_panics_when_predicate_is_false_for_identity() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_left = 0usize;
        let mut tree = create_sum_tree(&values);

        let predicate_result_1 = false;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = tree.max_right(query_left, |_node| predicate_result_1);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn min_left_with_query_right_larger_than_size_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_right = 4usize;
        let expected_min_left = None;
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 100;

        // Act
        let actual_min_left =
            tree.min_left(query_right, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn min_left_with_query_right_zero_returns_zero() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_right = 0usize;
        let expected_min_left = Some(0usize);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 100;

        // Act
        let actual_min_left =
            tree.min_left(query_right, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn min_left_with_always_true_predicate_and_query_right_size_returns_zero() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_right = 3usize;
        let expected_min_left = Some(0usize);
        let mut tree = create_sum_tree(&values);

        let predicate_result_1 = true;

        // Act
        let actual_min_left = tree.min_left(query_right, |_node| predicate_result_1);

        // Assert
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn min_left_returns_query_right_when_first_reverse_element_breaks_predicate() {
        // Arrange
        let values = vec![5i64, 1, 1];
        let query_right = 2usize;
        let expected_min_left = Some(2usize);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 0;

        // Act
        let actual_min_left =
            tree.min_left(query_right, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn min_left_after_apply_uses_lazy_values_and_returns_correct_boundary() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let apply_left = 0usize;
        let apply_right = 3usize;
        let apply_delta = 2i64;
        let query_right = 3usize;
        let expected_min_left = Some(2usize);
        let mut tree = create_sum_tree(&values);
        let expected_check_1 = Some(());

        let predicate_threshold_1 = 8;

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_delta);
        let actual_min_left =
            tree.min_left(query_right, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_apply_result, expected_check_1);
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn empty_tree_min_left_from_zero_returns_zero() {
        // Arrange
        let values: Vec<i64> = vec![];
        let query_right = 0usize;
        let expected_min_left = Some(0usize);
        let mut tree = create_sum_tree(&values);

        let predicate_threshold_1 = 0;

        // Act
        let actual_min_left =
            tree.min_left(query_right, |node| node.range_sum <= predicate_threshold_1);

        // Assert
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn non_commutative_monoid_min_left_after_apply_uses_lazy_values() {
        // Arrange
        let values = vec!["ab", "cd", "ef"];
        let apply_left = 0usize;
        let apply_right = 2usize;
        let apply_flag = true;
        let min_left_query_right = 2usize;
        let expected_apply_result = Some(());
        let expected_min_left = Some(1usize);
        let expected_selected_text = Some(String::from("CD"));
        let expected_suffix = String::from("CD");
        let mut tree = create_string_tree(&values);

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_min_left = tree.min_left(min_left_query_right, |aggregated_text| {
            expected_suffix.ends_with(aggregated_text.as_str())
        });
        let actual_selected_text = actual_min_left
            .and_then(|boundary_index| query_text(&mut tree, boundary_index, min_left_query_right));

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_min_left, expected_min_left);
        assert_eq!(actual_selected_text, expected_selected_text);
    }

    #[test]
    fn min_left_with_middle_right_and_always_true_predicate_returns_zero() {
        // Arrange
        let values = vec![1i64, 2, 3, 4];
        let query_right = 2usize;
        let expected_min_left = Some(0usize);
        let mut tree = create_sum_tree(&values);

        let predicate_result_1 = true;

        // Act
        let actual_min_left = tree.min_left(query_right, |_node| predicate_result_1);

        // Assert
        assert_eq!(actual_min_left, expected_min_left);
    }

    #[test]
    fn min_left_panics_when_predicate_is_false_for_identity() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let query_right = 3usize;
        let mut tree = create_sum_tree(&values);

        let predicate_result_1 = false;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = tree.min_left(query_right, |_node| predicate_result_1);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn single_element_tree_keeps_select_query_and_select_all_consistent_after_update_and_apply() {
        // Arrange
        let values = vec![5i64];
        let update_index = 0usize;
        let updated_value = 10i64;
        let apply_left = 0usize;
        let apply_right = 1usize;
        let apply_delta = 3i64;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_update_result = Some(());
        let expected_apply_result = Some(());
        let expected_after_update_sum = Some(10i64);
        let expected_after_update_select_all_sum = 10i64;
        let expected_after_apply_sum = Some(13i64);
        let expected_after_apply_select_all_sum = 13i64;
        let mut tree = create_sum_tree(&values);

        // Act
        let actual_update_result = tree.update(update_index, create_sum_node(updated_value));
        let actual_after_update_select = select_sum(&mut tree, update_index);
        let actual_after_update_query = query_sum(&mut tree, query_left, query_right);
        let actual_after_update_select_all_sum = tree.select_all().range_sum;
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_delta);
        let actual_after_apply_select = select_sum(&mut tree, update_index);
        let actual_after_apply_query = query_sum(&mut tree, query_left, query_right);
        let actual_after_apply_select_all_sum = tree.select_all().range_sum;

        // Assert
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_after_update_select, expected_after_update_sum);
        assert_eq!(actual_after_update_query, expected_after_update_sum);
        assert_eq!(
            actual_after_update_select_all_sum,
            expected_after_update_select_all_sum
        );
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_after_apply_select, expected_after_apply_sum);
        assert_eq!(actual_after_apply_query, expected_after_apply_sum);
        assert_eq!(
            actual_after_apply_select_all_sum,
            expected_after_apply_select_all_sum
        );
    }

    #[test]
    fn non_commutative_monoid_keeps_query_order_after_apply_and_update() {
        // Arrange
        let values = vec!["ab", "cd", "ef"];
        let apply_left = 0usize;
        let apply_right = 2usize;
        let apply_flag = true;
        let update_index = 1usize;
        let updated_value = String::from("xy");
        let query_left = 0usize;
        let query_right = values.len();
        let expected_apply_result = Some(());
        let expected_update_result = Some(());
        let expected_query_text = Some(String::from("ABxyef"));
        let mut tree = create_string_tree(&values);

        // Act
        let actual_apply_result = tree.apply(apply_left, apply_right, apply_flag);
        let actual_update_result = tree.update(update_index, updated_value);
        let actual_query_text = query_text(&mut tree, query_left, query_right);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_query_text, expected_query_text);
    }

    #[test]
    fn alternating_apply_and_update_keeps_query_and_select_consistent() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let first_apply_left = 0usize;
        let first_apply_right = 3usize;
        let first_apply_delta = 10i64;
        let update_index = 1usize;
        let updated_value = 100i64;
        let second_apply_left = 1usize;
        let second_apply_right = 3usize;
        let second_apply_delta = 5i64;
        let total_query_left = 0usize;
        let total_query_right = 3usize;
        let first_select_index = 0usize;
        let second_select_index = 1usize;
        let third_select_index = 2usize;
        let expected_total_sum = Some(134i64);
        let expected_first_select = Some(11i64);
        let expected_second_select = Some(105i64);
        let expected_third_select = Some(18i64);
        let mut tree = create_sum_tree(&values);
        let expected_check_1 = Some(());
        let expected_check_2 = Some(());
        let expected_check_3 = Some(());

        // Act
        let actual_first_apply_result =
            tree.apply(first_apply_left, first_apply_right, first_apply_delta);
        let actual_update_result = tree.update(update_index, create_sum_node(updated_value));
        let actual_second_apply_result =
            tree.apply(second_apply_left, second_apply_right, second_apply_delta);
        let actual_total_sum = query_sum(&mut tree, total_query_left, total_query_right);
        let actual_first_select = select_sum(&mut tree, first_select_index);
        let actual_second_select = select_sum(&mut tree, second_select_index);
        let actual_third_select = select_sum(&mut tree, third_select_index);

        // Assert
        assert_eq!(actual_first_apply_result, expected_check_1);
        assert_eq!(actual_update_result, expected_check_2);
        assert_eq!(actual_second_apply_result, expected_check_3);
        assert_eq!(actual_total_sum, expected_total_sum);
        assert_eq!(actual_first_select, expected_first_select);
        assert_eq!(actual_second_select, expected_second_select);
        assert_eq!(actual_third_select, expected_third_select);
    }

    #[test]
    fn non_commutative_monoid_keeps_query_max_right_and_min_left_order_dependent() {
        // Arrange
        let values = vec!["a", "b", "c"];
        let query_left = 0usize;
        let query_right = 3usize;
        let max_right_query_left = 0usize;
        let min_left_query_right = 3usize;
        let expected_query_text = Some(String::from("abc"));
        let expected_max_right = Some(1usize);
        let expected_min_left = Some(1usize);
        let mut tree = create_string_tree(&values);

        // Act
        let actual_query_text = query_text(&mut tree, query_left, query_right);
        let actual_max_right =
            tree.max_right(max_right_query_left, |text| does_not_contain_ab(text));
        let actual_min_left = tree.min_left(min_left_query_right, |text| does_not_contain_ab(text));

        // Assert
        assert_eq!(actual_query_text, expected_query_text);
        assert_eq!(actual_max_right, expected_max_right);
        assert_eq!(actual_min_left, expected_min_left);
    }
}
