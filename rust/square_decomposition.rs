use std::cmp::min;

pub trait MapMonoid {
    type S: Clone;
    type F: Clone;

    fn id() -> Self::S;
    fn identity_map() -> Self::F;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
    fn mapping(f: &Self::F, x: &Self::S) -> Self::S;
    fn composition(f: &Self::F, g: &Self::F) -> Self::F;
}

pub struct SquareDecomposition<M>
where
    M: MapMonoid,
{
    size: usize,
    bucket_size: usize,
    bucket_count: usize,
    values: Vec<M::S>,
    bucket_values: Vec<M::S>,
    lazy: Vec<M::F>,
    has_lazy: Vec<bool>,
}

impl<M> SquareDecomposition<M>
where
    M: MapMonoid,
{
    pub fn new(size: usize) -> Self {
        let bucket_size = Self::compute_bucket_size(size);
        let bucket_count = Self::compute_bucket_count(size, bucket_size);
        let padded_size = bucket_size.saturating_mul(bucket_count);

        Self {
            size,
            bucket_size,
            bucket_count,
            values: vec![M::id(); padded_size],
            bucket_values: vec![M::id(); bucket_count],
            lazy: vec![M::identity_map(); bucket_count],
            has_lazy: vec![false; bucket_count],
        }
    }

    pub fn from_slice(values: &[M::S]) -> Self {
        let mut structure = Self::new(values.len());

        for (index, value) in values.iter().enumerate() {
            structure.values[index] = value.clone();
        }

        structure.build_buckets();

        structure
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
        if self.size <= index {
            return None;
        }

        self.push_bucket_for_index(index);
        Some(self.values[index].clone())
    }

    pub fn select_all(&mut self) -> Vec<M::S> {
        for bucket_index in 0..self.bucket_count {
            self.push_bucket(bucket_index);
        }

        self.values[..self.size].to_vec()
    }

    pub fn query_all(&self) -> M::S {
        let mut result = M::id();

        for bucket_value in &self.bucket_values {
            result = M::binary_operation(&result, bucket_value);
        }

        result
    }

    pub fn update(&mut self, index: usize, value: M::S) -> Option<()> {
        if self.size <= index {
            return None;
        }

        let bucket_index = self.bucket_index(index);
        self.push_bucket(bucket_index);
        self.values[index] = value;
        self.rebuild_bucket(bucket_index);

        Some(())
    }

    pub fn apply_point(&mut self, index: usize, map_value: M::F) -> Option<()> {
        self.apply(index, index + 1, map_value)
    }

    pub fn apply_range(&mut self, left: usize, right: usize, map_value: M::F) -> Option<()> {
        self.apply(left, right, map_value)
    }

    pub fn apply(&mut self, left: usize, right: usize, map_value: M::F) -> Option<()> {
        if left > right || self.size < right {
            return None;
        }

        if left == right {
            return Some(());
        }

        let left_bucket_index = self.bucket_index(left);
        let right_bucket_index = self.bucket_index(right - 1);

        if left_bucket_index == right_bucket_index {
            self.apply_in_bucket(left, right, &map_value);
            return Some(());
        }

        if left % self.bucket_size != 0 {
            let left_bucket_right = self.bucket_right_boundary(left_bucket_index);
            self.apply_in_bucket(left, left_bucket_right, &map_value);
        }

        let first_full_bucket = if left % self.bucket_size == 0 {
            left_bucket_index
        } else {
            left_bucket_index + 1
        };
        let full_bucket_right = if right % self.bucket_size == 0 {
            right / self.bucket_size
        } else {
            right_bucket_index
        };

        for bucket_index in first_full_bucket..full_bucket_right {
            self.apply_to_bucket(bucket_index, &map_value);
        }

        if right % self.bucket_size != 0 {
            let right_bucket_left = right_bucket_index * self.bucket_size;
            self.apply_in_bucket(right_bucket_left, right, &map_value);
        }

        Some(())
    }

    pub fn query(&mut self, left: usize, right: usize) -> Option<M::S> {
        if left > right || self.size < right {
            return None;
        }

        if left == right {
            return Some(M::id());
        }

        let mut result = M::id();
        let mut index = left;

        while index < right && index % self.bucket_size != 0 {
            let bucket_right = min(self.bucket_right_boundary(self.bucket_index(index)), right);
            let bucket_result = self.query_in_bucket(index, bucket_right);
            result = M::binary_operation(&result, &bucket_result);
            index = bucket_right;
        }

        while index + self.bucket_size <= right {
            let bucket_index = self.bucket_index(index);
            result = M::binary_operation(&result, &self.bucket_values[bucket_index]);
            index += self.bucket_size;
        }

        while index < right {
            let bucket_right = min(self.bucket_right_boundary(self.bucket_index(index)), right);
            let bucket_result = self.query_in_bucket(index, bucket_right);
            result = M::binary_operation(&result, &bucket_result);
            index = bucket_right;
        }

        Some(result)
    }

    #[inline]
    fn compute_bucket_size(length: usize) -> usize {
        if length <= 1 {
            1
        } else {
            ((length as f64).sqrt().floor() as usize).max(1)
        }
    }

    #[inline]
    fn compute_bucket_count(length: usize, bucket_size: usize) -> usize {
        if length == 0 {
            0
        } else {
            length.div_ceil(bucket_size)
        }
    }

    #[inline]
    fn bucket_index(&self, index: usize) -> usize {
        index / self.bucket_size
    }

    #[inline]
    fn bucket_right_boundary(&self, bucket_index: usize) -> usize {
        min(self.size, (bucket_index + 1) * self.bucket_size)
    }

    fn push_bucket_for_index(&mut self, index: usize) {
        let bucket_index = self.bucket_index(index);
        self.push_bucket(bucket_index);
    }

    fn push_bucket(&mut self, bucket_index: usize) {
        if bucket_index >= self.bucket_count || !self.has_lazy[bucket_index] {
            return;
        }

        let map_value = self.lazy[bucket_index].clone();
        let start = bucket_index * self.bucket_size;
        let end = self.bucket_right_boundary(bucket_index);

        for index in start..end {
            self.values[index] = M::mapping(&map_value, &self.values[index]);
        }

        self.lazy[bucket_index] = M::identity_map();
        self.has_lazy[bucket_index] = false;
    }

    fn apply_in_bucket(&mut self, left: usize, right: usize, map_value: &M::F) {
        let bucket_index = self.bucket_index(left);

        self.push_bucket(bucket_index);

        for index in left..right {
            self.values[index] = M::mapping(map_value, &self.values[index]);
        }

        self.rebuild_bucket(bucket_index);
    }

    fn query_in_bucket(&mut self, left: usize, right: usize) -> M::S {
        let bucket_index = self.bucket_index(left);
        let mut result = M::id();

        self.push_bucket(bucket_index);

        for index in left..right {
            result = M::binary_operation(&result, &self.values[index]);
        }

        result
    }

    fn apply_to_bucket(&mut self, bucket_index: usize, map_value: &M::F) {
        self.bucket_values[bucket_index] = M::mapping(map_value, &self.bucket_values[bucket_index]);

        if self.has_lazy[bucket_index] {
            self.lazy[bucket_index] = M::composition(map_value, &self.lazy[bucket_index]);
        } else {
            self.lazy[bucket_index] = map_value.clone();
            self.has_lazy[bucket_index] = true;
        }
    }

    fn build_buckets(&mut self) {
        for bucket_index in 0..self.bucket_count {
            self.rebuild_bucket(bucket_index);
        }
    }

    fn rebuild_bucket(&mut self, bucket_index: usize) {
        let start = bucket_index * self.bucket_size;

        if self.size <= start {
            return;
        }

        let end = self.bucket_right_boundary(bucket_index);
        let mut bucket_value = M::id();

        for index in start..end {
            bucket_value = M::binary_operation(&bucket_value, &self.values[index]);
        }

        if self.has_lazy[bucket_index] {
            bucket_value = M::mapping(&self.lazy[bucket_index], &bucket_value);
        }

        self.bucket_values[bucket_index] = bucket_value;
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
    use super::{MapMonoid, SquareDecomposition};

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

    #[derive(Clone, Copy)]
    struct StringPrefixMonoid;

    impl MapMonoid for StringPrefixMonoid {
        type S = String;
        type F = String;

        fn id() -> Self::S {
            String::new()
        }

        fn identity_map() -> Self::F {
            String::new()
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            let mut merged_text = lhs.clone();
            merged_text.push_str(rhs);
            merged_text
        }

        fn mapping(f: &Self::F, x: &Self::S) -> Self::S {
            let mut mapped_text = f.clone();
            mapped_text.push_str(x);
            mapped_text
        }

        fn composition(f: &Self::F, g: &Self::F) -> Self::F {
            let mut composed_text = f.clone();
            composed_text.push_str(g);
            composed_text
        }
    }

    fn create_sum_node(value: i64) -> SumNode {
        SumNode {
            range_sum: value,
            range_len: 1,
        }
    }

    fn create_sum_structure(values: &[i64]) -> SquareDecomposition<RangeAddSumMonoid> {
        let sum_nodes: Vec<SumNode> = values.iter().copied().map(create_sum_node).collect();
        SquareDecomposition::from_slice(&sum_nodes)
    }

    fn create_text_structure(values: &[&str]) -> SquareDecomposition<StringPrefixMonoid> {
        let text_values: Vec<String> = values.iter().map(|value| (*value).to_string()).collect();
        SquareDecomposition::from_slice(&text_values)
    }

    fn select_sum(
        structure: &mut SquareDecomposition<RangeAddSumMonoid>,
        index: usize,
    ) -> Option<i64> {
        structure.select(index).map(|node| node.range_sum)
    }

    fn select_all_sum(structure: &mut SquareDecomposition<RangeAddSumMonoid>) -> Vec<i64> {
        structure
            .select_all()
            .into_iter()
            .map(|node| node.range_sum)
            .collect()
    }

    fn query_sum(
        structure: &mut SquareDecomposition<RangeAddSumMonoid>,
        left: usize,
        right: usize,
    ) -> Option<i64> {
        structure.query(left, right).map(|node| node.range_sum)
    }

    #[test]
    fn new_with_size_zero_returns_empty_values_and_identity_query() {
        // Arrange
        let structure_size = 0usize;
        let select_index = 0usize;
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_selected_value = None;
        let expected_query_value = Some(0i64);
        let expected_query_all_value = 0i64;
        let expected_all_values: Vec<i64> = vec![];
        let mut structure: SquareDecomposition<RangeAddSumMonoid> =
            SquareDecomposition::new(structure_size);

        // Act
        let actual_len = structure.len();
        let actual_is_empty = structure.is_empty();
        let actual_selected_value = select_sum(&mut structure, select_index);
        let actual_query_value = query_sum(&mut structure, query_left, query_right);
        let actual_query_all_value = structure.query_all().range_sum;
        let actual_all_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_query_value, expected_query_value);
        assert_eq!(actual_query_all_value, expected_query_all_value);
        assert_eq!(actual_all_values, expected_all_values);
    }

    #[test]
    fn from_slice_with_single_element_supports_select_query_and_query_all() {
        // Arrange
        let values = vec![42i64];
        let select_index = 0usize;
        let out_of_bounds_index = 1usize;
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_selected_value = Some(42i64);
        let expected_out_of_bounds_value = None;
        let expected_query_value = Some(42i64);
        let expected_query_all_value = 42i64;
        let expected_all_values = vec![42i64];
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_selected_value = select_sum(&mut structure, select_index);
        let actual_out_of_bounds_value = select_sum(&mut structure, out_of_bounds_index);
        let actual_query_value = query_sum(&mut structure, query_left, query_right);
        let actual_query_all_value = structure.query_all().range_sum;
        let actual_all_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_selected_value, expected_selected_value);
        assert_eq!(actual_out_of_bounds_value, expected_out_of_bounds_value);
        assert_eq!(actual_query_value, expected_query_value);
        assert_eq!(actual_query_all_value, expected_query_all_value);
        assert_eq!(actual_all_values, expected_all_values);
    }

    #[test]
    fn from_slice_with_last_partial_bucket_excludes_padding_from_query_results() {
        // Arrange
        let values = vec![3i64, 5, 7, 11, 13];
        let all_query_left = 0usize;
        let all_query_right = 5usize;
        let tail_query_left = 4usize;
        let tail_query_right = 5usize;
        let expected_all_sum = Some(39i64);
        let expected_tail_sum = Some(13i64);
        let expected_query_all_sum = 39i64;
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_all_sum = query_sum(&mut structure, all_query_left, all_query_right);
        let actual_tail_sum = query_sum(&mut structure, tail_query_left, tail_query_right);
        let actual_query_all_sum = structure.query_all().range_sum;

        // Assert
        assert_eq!(actual_all_sum, expected_all_sum);
        assert_eq!(actual_tail_sum, expected_tail_sum);
        assert_eq!(actual_query_all_sum, expected_query_all_sum);
    }

    #[test]
    fn update_after_full_bucket_apply_pushes_lazy_value_before_assignment() {
        // Arrange
        let values = vec![1i64, 2, 3, 4, 5];
        let apply_left = 0usize;
        let apply_right = 5usize;
        let apply_map_value = 2i64;
        let update_index = 1usize;
        let updated_value = create_sum_node(10i64);
        let expected_update_result = Some(());
        let expected_prefix_sum = Some(13i64);
        let expected_query_all_sum = 31i64;
        let expected_all_values = vec![3i64, 10, 5, 6, 7];
        let mut structure = create_sum_structure(&values);

        let prefix_left = 0;
        let prefix_right = 2;
        let expected_apply_result = Some(());

        // Act
        let actual_apply_result = structure.apply(apply_left, apply_right, apply_map_value);
        let actual_update_result = structure.update(update_index, updated_value);
        let actual_prefix_sum = query_sum(&mut structure, prefix_left, prefix_right);
        let actual_query_all_sum = structure.query_all().range_sum;
        let actual_all_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_update_result, expected_update_result);
        assert_eq!(actual_prefix_sum, expected_prefix_sum);
        assert_eq!(actual_query_all_sum, expected_query_all_sum);
        assert_eq!(actual_all_values, expected_all_values);
    }

    #[test]
    fn apply_point_updates_only_target_index() {
        // Arrange
        let values = vec![3i64, 4];
        let update_index = 1usize;
        let map_value = 2i64;
        let expected_apply_result = Some(());
        let expected_values = vec![3i64, 6];
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_apply_result = structure.apply_point(update_index, map_value);
        let actual_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn apply_with_multiple_buckets_updates_range_and_keeps_boundaries() {
        // Arrange
        let values = vec![0i64, 1, 2, 3, 4];
        let query_left = 1usize;
        let query_right = 4usize;
        let map_value = 1i64;
        let expected_apply_result = Some(());
        let expected_all_values = vec![0i64, 2, 3, 4, 4];
        let expected_query_value = Some(9i64);
        let expected_query_all_value = 13i64;
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_apply_result = structure.apply(query_left, query_right, map_value);
        let actual_all_values = select_all_sum(&mut structure);
        let actual_query_value = query_sum(&mut structure, query_left, query_right);
        let actual_query_all_value = structure.query_all().range_sum;

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_all_values, expected_all_values);
        assert_eq!(actual_query_value, expected_query_value);
        assert_eq!(actual_query_all_value, expected_query_all_value);
    }

    #[test]
    fn apply_with_empty_range_returns_some_without_changes() {
        // Arrange
        let values = vec![1i64, 2];
        let query_left = 1usize;
        let query_right = 1usize;
        let map_value = 3i64;
        let expected_apply_result = Some(());
        let expected_all_values = vec![1i64, 2];
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_apply_result = structure.apply(query_left, query_right, map_value);
        let actual_all_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_all_values, expected_all_values);
    }

    #[test]
    fn apply_with_string_mapping_keeps_composition_order() {
        // Arrange
        let values = vec!["x", "y", "z", "w"];
        let first_map_value = "A".to_string();
        let second_map_value = "B".to_string();
        let expected_all_values = vec![
            "BAx".to_string(),
            "BAy".to_string(),
            "BAz".to_string(),
            "BAw".to_string(),
        ];
        let mut structure = create_text_structure(&values);

        let apply_left = 0;
        let apply_right = 4;
        let apply_range_left = 0;
        let apply_range_right = 4;

        let expected_apply_result = Some(());

        // Act
        let actual_apply_result = structure.apply(apply_left, apply_right, first_map_value);
        let actual_apply_range_result =
            structure.apply_range(apply_range_left, apply_range_right, second_map_value);
        let actual_all_values = structure.select_all();

        // Assert
        assert_eq!(actual_apply_result, expected_apply_result);
        assert_eq!(actual_apply_range_result, expected_apply_result);
        assert_eq!(actual_all_values, expected_all_values);
    }

    #[test]
    fn query_with_invalid_range_returns_none() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let reverse_left = 2usize;
        let reverse_right = 1usize;
        let overflow_left = 0usize;
        let overflow_right = 4usize;
        let expected_reverse_result = None;
        let expected_overflow_result = None;
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_reverse_result = query_sum(&mut structure, reverse_left, reverse_right);
        let actual_overflow_result = query_sum(&mut structure, overflow_left, overflow_right);

        // Assert
        assert_eq!(actual_reverse_result, expected_reverse_result);
        assert_eq!(actual_overflow_result, expected_overflow_result);
    }

    #[test]
    fn apply_point_with_index_equal_to_len_returns_none_without_changing_values() {
        // Arrange
        let values = vec![3i64, 4];
        let invalid_index = values.len();
        let update_value = 99i64;
        let expected_result = None;
        let expected_values = values.clone();
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_result = structure.apply_point(invalid_index, update_value);
        let actual_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }


    #[test]
    fn update_with_index_equal_to_len_returns_none_without_changing_values() {
        // Arrange
        let values = vec![3i64, 4];
        let invalid_index = values.len();
        let update_value = create_sum_node(99);
        let expected_result = None;
        let expected_values = values.clone();
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_result = structure.update(invalid_index, update_value);
        let actual_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }


    #[test]
    fn apply_range_with_reversed_range_returns_none_without_changing_values() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let left = 2usize;
        let right = 1usize;
        let mapping = 99i64;
        let expected_result = None;
        let expected_values = values.clone();
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_result = structure.apply_range(left, right, mapping);
        let actual_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }


    #[test]
    fn apply_range_with_right_greater_than_len_returns_none_without_changing_values() {
        // Arrange
        let values = vec![1i64, 2, 3];
        let left = 0usize;
        let right = 4usize;
        let mapping = 99i64;
        let expected_result = None;
        let expected_values = values.clone();
        let mut structure = create_sum_structure(&values);

        // Act
        let actual_result = structure.apply_range(left, right, mapping);
        let actual_values = select_all_sum(&mut structure);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

}
