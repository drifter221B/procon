use std::collections::HashMap;

#[derive(Debug)]
pub struct Dsu {
    siz: Vec<usize>,
    rank: Vec<usize>,
    par: Vec<usize>,
    group_number: usize,
}

impl Dsu {

    pub fn new(size: usize) -> Self {

        Self {
            par: (0..size).collect::<Vec<usize>>(),
            rank: vec![0; size],
            siz: vec![1; size],
            group_number: size,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.par.len()
    }

    #[inline]
    fn size(&self, vertex: usize) -> usize {
        self.siz[vertex]
    }

    #[inline]
    pub fn root(&mut self, x: usize) -> usize {

        if self.par[x] != x {
            self.par[x] = self.root(self.par[x]);
        }

        self.par[x]
    }

    pub fn all_roots(&mut self) -> Vec<usize> {
        (0..self.len()).filter(|&x| x == self.root(x)).collect::<Vec<usize>>()
    }

    #[inline]
    pub fn is_same_component(&mut self, x: usize, y: usize) -> bool {
        self.root(x) == self.root(y)
    }

    pub fn merge(&mut self, x: usize, y: usize) -> bool {

        let mut root_x = self.root(x);
        let mut root_y = self.root(y);

        if root_x == root_y {
            return false;
        }

        self.group_number -= 1;

        if self.rank[root_x] < self.rank[root_y] {
            std::mem::swap(&mut root_x, &mut root_y);
        }

        self.par[root_y] = root_x;
        self.siz[root_x] += self.siz[root_y];

        if self.rank[root_x] == self.rank[root_y] {
            self.rank[root_x] += 1;
        }

        true
    }

    #[inline]
    pub fn component_size(&mut self, index: usize) -> usize {
        let root = self.root(index);

        self.size(root)
    }

    pub fn all_component_size(&mut self) -> Vec<usize> {
        self.all_roots().into_iter().map(|root| self.size(root)).collect()
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_number
    }

    pub fn component_members(&mut self, x: usize) -> Vec<usize> {

        let root_x = self.root(x);
        let mut results = Vec::with_capacity(self.size(root_x));

        for v in 0..self.len() {
            if self.root(v) == root_x {
                results.push(v);
            }
        }

        results
    }

    pub fn components_by_root(&mut self) -> HashMap<usize, Vec<usize>> {

        let mut group_member = HashMap::new();

        for vertex in 0..self.len() {
            let root = self.root(vertex);
            let members = group_member.entry(root).or_insert_with(|| Vec::with_capacity(self.size(root)));
            members.push(vertex);
        }

        group_member
    }

}


#[cfg(test)]
mod tests {

    use std::collections::HashMap;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    #[test]
    fn new_with_size_one_returns_non_empty_and_single_group() {

        // Arrange
        let size = 1;
        let expected_is_empty = false;
        let expected_len = 1;
        let expected_group_count = 1;

        // Act
        let uf = Dsu::new(size);
        let actual_is_empty = uf.is_empty();
        let actual_len = uf.len();
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_group_count, expected_group_count);
    }

    #[test]
    fn new_with_size_zero_returns_zero_group_count() {

        // Arrange
        let size = 0;
        let expected_group_count = 0;

        // Act
        let uf = Dsu::new(size);
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_group_count, expected_group_count);
    }

    #[test]
    fn new_with_positive_size_returns_non_empty() {
        // Arrange
        let size = 5;
        let expected_is_empty = false;
        let uf = Dsu::new(size);

        // Act
        let actual_is_empty = uf.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn new_with_zero_size_returns_empty() {
        // Arrange
        let size = 0;
        let expected_is_empty = true;
        let uf = Dsu::new(size);

        // Act
        let actual_is_empty = uf.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn len_remains_initial_size_after_merges() {
        // Arrange
        let size = 5;
        let merge_pairs = [(0, 1), (0, 2)];
        let expected_len = size;
        let mut uf = Dsu::new(size);

        // Act
        let length_before_merges = uf.len();
        for (left, right) in merge_pairs {
            uf.merge(left, right);
        }
        let length_after_merges = uf.len();

        // Assert
        assert_eq!(length_before_merges, expected_len);
        assert_eq!(length_after_merges, expected_len);
    }

    #[test]
    fn merged_vertices_return_component_root_and_others_return_themselves() {
        // Arrange
        let size = 5;
        let root_vertex = 0;
        let mut uf = Dsu::new(size);
        let vertices = [0, 2, 4];
        for vertex in vertices.iter() {
            uf.merge(root_vertex, *vertex);
        }
        let expected_roots: Vec<usize> = (0..size)
            .map(|vertex| if vertices.contains(&vertex) { root_vertex } else { vertex })
            .collect();

        // Act
        let actual_roots: Vec<usize> = (0..size).map(|vertex| uf.root(vertex)).collect();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn root_on_initial_state_returns_each_vertex_itself() {
        // Arrange
        let size = 5;
        let expected_roots: Vec<usize> = (0..size).collect();
        let mut uf = Dsu::new(size);

        // Act
        let actual_roots: Vec<usize> = (0..uf.len()).map(|vertex| uf.root(vertex)).collect();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn root_compresses_path_after_multi_stage_merge() {

        // Arrange
        let size = 4;
        let reference_vertex_1 = 0;
        let vertex_1 = 1;
        let reference_vertex_2 = 2;
        let vertex_2 = 3;
        let reference_vertex_3 = 0;
        let vertex_3 = 2;
        let target_vertex = 3;
        let expected_parent_before_root = 2;
        let expected_root = 0;
        let expected_parent_after_root = 0;

        let mut uf = Dsu::new(size);
        uf.merge(reference_vertex_1, vertex_1);
        uf.merge(reference_vertex_2, vertex_2);
        uf.merge(reference_vertex_3, vertex_3);
        let parent_before_root = uf.par[target_vertex];

        // Act
        let actual_root = uf.root(target_vertex);
        let parent_after_root = uf.par[target_vertex];

        // Assert
        assert_eq!(parent_before_root, expected_parent_before_root);
        assert_eq!(actual_root, expected_root);
        assert_eq!(parent_after_root, expected_parent_after_root);
    }

    #[test]
    fn root_with_lower_bound_index_zero_returns_root() {

        // Arrange
        let size = 3;
        let index = 0;
        let expected_root = 0;
        let mut uf = Dsu::new(size);

        // Act
        let actual_root = uf.root(index);

        // Assert
        assert_eq!(actual_root, expected_root);
    }

    #[test]
    fn root_with_upper_bound_index_len_minus_one_returns_root() {

        // Arrange
        let size = 3;
        let mut uf = Dsu::new(size);
        let index = uf.len() - 1;
        let expected_root = index;

        // Act
        let actual_root = uf.root(index);

        // Assert
        assert_eq!(actual_root, expected_root);
    }

    #[test]
    fn root_with_out_of_range_index_len_panics() {

        // Arrange
        let size = 3;
        let mut uf = Dsu::new(size);
        let out_of_range_index = uf.len();

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.root(out_of_range_index);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn root_with_size_zero_panics() {

        // Arrange
        let size = 0;
        let mut uf = Dsu::new(size);
        let index = 0;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.root(index);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn all_roots_returns_one_root_per_component_after_merges() {
        // Arrange
        let size = 5;
        let root_vertex = 0;
        let mut uf = Dsu::new(size);
        let vertices = [0, 2, 4];
        for vertex in vertices.iter() {
            uf.merge(root_vertex, *vertex);
        }
        let expected_roots = vec![0, 1, 3];

        // Act
        let actual_roots = uf.all_roots();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn all_roots_on_initial_state_returns_all_vertices() {
        // Arrange
        let size = 5;
        let expected_roots: Vec<usize> = (0..size).collect();
        let mut uf = Dsu::new(size);

        // Act
        let actual_roots = uf.all_roots();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn all_roots_with_size_zero_returns_empty_vector() {

        // Arrange
        let size = 0;
        let expected_roots = Vec::<usize>::new();
        let mut uf = Dsu::new(size);

        // Act
        let actual_roots = uf.all_roots();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn all_roots_after_merging_all_vertices_returns_single_root() {

        // Arrange
        let size = 5;
        let expected_root_count = 1;
        let mut uf = Dsu::new(size);
        for vertex in 1..size {
            uf.merge(0, vertex);
        }

        // Act
        let actual_roots = uf.all_roots();
        let actual_root_count = actual_roots.len();

        // Assert
        assert_eq!(actual_root_count, expected_root_count);
    }

    #[test]
    fn is_same_component_distinguishes_merged_and_unmerged_vertices() {
        // Arrange
        let size = 5;
        let root_vertex = 0;
        let mut uf = Dsu::new(size);
        let vertices = [0, 2, 4];
        for vertex in vertices.iter() {
            uf.merge(root_vertex, *vertex);
        }
        let query_vertices: Vec<usize> = (0..size).collect();
        let expected_results: Vec<bool> = query_vertices
            .iter()
            .map(|vertex| vertices.contains(vertex))
            .collect();

        // Act
        let actual_results: Vec<bool> = query_vertices
            .iter()
            .map(|vertex| uf.is_same_component(root_vertex, *vertex))
            .collect();

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn is_same_component_on_initial_state_is_false_for_distinct_adjacent_vertices() {
        // Arrange
        let size = 5;
        let expected_results = vec![false; size];
        let mut uf = Dsu::new(size);

        // Act
        let actual_results: Vec<bool> = (0..size)
            .map(|vertex| uf.is_same_component(vertex, (vertex + 1) % size))
            .collect();

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn is_same_component_with_same_index_returns_true() {

        // Arrange
        let size = 4;
        let reference_vertex = 0;
        let vertex = 1;
        let target_index = 1;
        let expected_is_same_component = true;
        let mut uf = Dsu::new(size);
        uf.merge(reference_vertex, vertex);

        // Act
        let actual_is_same_component = uf.is_same_component(target_index, target_index);

        // Assert
        assert_eq!(actual_is_same_component, expected_is_same_component);
    }

    #[test]
    fn is_same_component_with_out_of_range_index_len_panics() {

        // Arrange
        let size = 3;
        let mut uf = Dsu::new(size);
        let in_range_index = 0;
        let out_of_range_index = uf.len();

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.is_same_component(in_range_index, out_of_range_index);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn is_same_component_with_size_zero_panics() {

        // Arrange
        let size = 0;
        let left_vertex = 0;
        let right_vertex = 0;
        let mut uf = Dsu::new(size);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.is_same_component(left_vertex, right_vertex);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn merge_connects_components_rejects_duplicate_merge_and_preserves_length() {
        // Arrange
        let size = 5;
        let merges = [(0, 1), (0, 1), (3, 4), (0, 3)];
        let expected_merge_results = vec![true, false, true, true];
        let expected_group_counts = vec![4, 4, 3, 2];
        let expected_roots = vec![0, 0, 2, 0, 0];
        let mut uf = Dsu::new(size);

        // Act
        let mut actual_merge_results = Vec::with_capacity(merges.len());
        let mut actual_group_counts = Vec::with_capacity(merges.len());
        for (left, right) in merges {
            actual_merge_results.push(uf.merge(left, right));
            actual_group_counts.push(uf.group_count());
        }
        let actual_roots: Vec<usize> = (0..size).map(|vertex| uf.root(vertex)).collect();
        let actual_len = uf.len();

        // Assert
        assert_eq!(actual_merge_results, expected_merge_results);
        assert_eq!(actual_group_counts, expected_group_counts);
        assert_eq!(actual_roots, expected_roots);
        assert_eq!(actual_len, size);
    }

    #[test]
    fn merge_with_same_vertex_returns_false_and_group_count_unchanged() {

        // Arrange
        let size = 4;
        let index = 2;
        let mut uf = Dsu::new(size);
        let group_count_before_merge = uf.group_count();
        let expected_merge_result = false;

        // Act
        let actual_merge_result = uf.merge(index, index);
        let group_count_after_merge = uf.group_count();

        // Assert
        assert_eq!(actual_merge_result, expected_merge_result);
        assert_eq!(group_count_after_merge, group_count_before_merge);
    }

    #[test]
    fn merge_success_n_minus_one_times_reduces_group_count_to_one() {

        // Arrange
        let size = 6;
        let expected_group_count = 1;
        let mut uf = Dsu::new(size);
        for vertex in 1..size {
            uf.merge(0, vertex);
        }

        // Act
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_group_count, expected_group_count);
    }

    #[test]
    fn merge_with_out_of_range_index_len_panics() {

        // Arrange
        let size = 3;
        let mut uf = Dsu::new(size);
        let in_range_index = 0;
        let out_of_range_index = uf.len();

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.merge(in_range_index, out_of_range_index);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn merge_with_size_zero_panics() {

        // Arrange
        let size = 0;
        let reference_vertex = 0;
        let vertex = 0;
        let mut uf = Dsu::new(size);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.merge(reference_vertex, vertex);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn merge_with_already_connected_different_vertices_returns_false_and_group_count_unchanged() {

        // Arrange
        let size = 4;
        let first_reference_vertex = 0;
        let first_vertex = 1;
        let second_reference_vertex = 1;
        let second_vertex = 2;
        let connected_vertex_1 = 0;
        let connected_vertex_2 = 2;
        let expected_merge_result = false;
        let mut uf = Dsu::new(size);
        uf.merge(first_reference_vertex, first_vertex);
        uf.merge(second_reference_vertex, second_vertex);
        let group_count_before_merge = uf.group_count();

        // Act
        let actual_merge_result = uf.merge(connected_vertex_1, connected_vertex_2);
        let group_count_after_merge = uf.group_count();

        // Assert
        assert_eq!(actual_merge_result, expected_merge_result);
        assert_eq!(group_count_after_merge, group_count_before_merge);
    }

    #[test]
    fn component_size_returns_merged_and_singleton_component_sizes() {
        // Arrange
        let size = 5;
        let merged_vertices = (0, 1);
        let queried_vertices = (0, 1, 4);
        let expected_sizes = (2, 2, 1);
        let mut uf = Dsu::new(size);
        uf.merge(merged_vertices.0, merged_vertices.1);

        // Act
        let actual_sizes = (
            uf.component_size(queried_vertices.0),
            uf.component_size(queried_vertices.1),
            uf.component_size(queried_vertices.2),
        );

        // Assert
        assert_eq!(actual_sizes, expected_sizes);
    }

    #[test]
    fn component_size_after_two_merges_returns_three() {

        // Arrange
        let size = 4;
        let first_reference_vertex = 0;
        let first_vertex = 1;
        let second_reference_vertex = 1;
        let second_vertex = 2;
        let target_vertex = 2;
        let expected_component_size = 3;
        let mut uf = Dsu::new(size);
        uf.merge(first_reference_vertex, first_vertex);
        uf.merge(second_reference_vertex, second_vertex);

        // Act
        let actual_component_size = uf.component_size(target_vertex);

        // Assert
        assert_eq!(actual_component_size, expected_component_size);
    }

    #[test]
    fn component_size_with_out_of_range_index_len_panics() {

        // Arrange
        let size = 3;
        let mut uf = Dsu::new(size);
        let out_of_range_index = uf.len();

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.component_size(out_of_range_index);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn component_size_with_size_zero_panics() {

        // Arrange
        let size = 0;
        let vertex = 0;
        let mut uf = Dsu::new(size);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.component_size(vertex);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn all_component_size_returns_sizes_for_merged_and_singleton_components() {
        // Arrange
        let size = 5;
        let root_vertex = 0;
        let mut uf = Dsu::new(size);
        let vertices = [0, 2, 4];
        for vertex in vertices.iter() {
            uf.merge(root_vertex, *vertex);
        }
        let expected_component_sizes = vec![3, 1, 1];

        // Act
        let actual_component_sizes = uf.all_component_size();

        // Assert
        assert_eq!(actual_component_sizes, expected_component_sizes);
    }

    #[test]
    fn all_component_size_on_initial_state_returns_one_per_vertex() {
        // Arrange
        let size = 5;
        let expected_component_sizes = vec![1; size];
        let mut uf = Dsu::new(size);

        // Act
        let actual_component_sizes = uf.all_component_size();

        // Assert
        assert_eq!(actual_component_sizes, expected_component_sizes);
    }

    #[test]
    fn all_component_size_with_size_zero_returns_empty_vector() {

        // Arrange
        let size = 0;
        let expected_component_sizes = Vec::<usize>::new();
        let mut uf = Dsu::new(size);

        // Act
        let actual_component_sizes = uf.all_component_size();

        // Assert
        assert_eq!(actual_component_sizes, expected_component_sizes);
    }

    #[test]
    fn all_component_size_after_merging_all_vertices_returns_single_entry_with_len_value() {

        // Arrange
        let size = 6;
        let expected_component_size_count = 1;
        let expected_only_component_size = size;
        let mut uf = Dsu::new(size);
        for vertex in 1..size {
            uf.merge(0, vertex);
        }

        // Act
        let actual_component_sizes = uf.all_component_size();
        let actual_component_size_count = actual_component_sizes.len();
        let actual_only_component_size = actual_component_sizes[0];

        // Assert
        assert_eq!(actual_component_size_count, expected_component_size_count);
        assert_eq!(actual_only_component_size, expected_only_component_size);
    }

    #[test]
    fn group_views_count_always_matches_group_count_after_each_operation_step() {

        // Arrange
        let size = 6;
        let merge_operations = vec![(0, 1), (2, 3), (1, 2), (4, 5), (0, 5), (0, 5)];
        let expected_group_counts = vec![5, 4, 3, 2, 1, 1];
        let mut uf = Dsu::new(size);

        // Act
        let mut actual_group_view_counts = Vec::with_capacity(merge_operations.len());
        for (left_vertex, right_vertex) in merge_operations {
            uf.merge(left_vertex, right_vertex);
            actual_group_view_counts.push((
                uf.group_count(),
                uf.all_roots().len(),
                uf.all_component_size().len(),
                uf.components_by_root().len(),
            ));
        }

        // Assert
        for (actual_counts, expected_group_count) in actual_group_view_counts.into_iter().zip(expected_group_counts) {
            assert_eq!(actual_counts.0, expected_group_count);
            assert_eq!(actual_counts.1, expected_group_count);
            assert_eq!(actual_counts.2, expected_group_count);
            assert_eq!(actual_counts.3, expected_group_count);
        }
    }

    #[test]
    fn component_members_returns_singleton_and_merged_component_members() {
        // Arrange
        let size = 5;
        let merge_pairs = [(1, 2), (2, 3)];
        let queried_vertices = (0, 1, 4);
        let expected_members = (vec![0], vec![1, 2, 3], vec![4]);
        let mut uf = Dsu::new(size);
        for (left, right) in merge_pairs {
            uf.merge(left, right);
        }

        // Act
        let actual_members = (
            uf.component_members(queried_vertices.0),
            uf.component_members(queried_vertices.1),
            uf.component_members(queried_vertices.2),
        );

        // Assert
        assert_eq!(actual_members, expected_members);
    }

    #[test]
    fn component_members_after_merging_all_vertices_returns_all_vertices() {

        // Arrange
        let size = 5;
        let target_vertex = 3;
        let expected_members = (0..size).collect::<Vec<usize>>();
        let mut uf = Dsu::new(size);
        for vertex in 1..size {
            uf.merge(0, vertex);
        }

        // Act
        let actual_members = uf.component_members(target_vertex);

        // Assert
        assert_eq!(actual_members, expected_members);
    }

    #[test]
    fn component_members_with_out_of_range_index_len_panics() {

        // Arrange
        let size = 3;
        let mut uf = Dsu::new(size);
        let out_of_range_index = uf.len();

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.component_members(out_of_range_index);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn component_members_with_size_zero_panics() {

        // Arrange
        let size = 0;
        let vertex = 0;
        let mut uf = Dsu::new(size);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.component_members(vertex);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn components_by_root_returns_members_grouped_by_representative() {
        // Arrange
        let size = 5;
        let merged_vertices = (1, 2);
        let mut expected_components = HashMap::new();
        expected_components.insert(0, vec![0]);
        expected_components.insert(1, vec![1, 2]);
        expected_components.insert(3, vec![3]);
        expected_components.insert(4, vec![4]);
        let mut uf = Dsu::new(size);
        uf.merge(merged_vertices.0, merged_vertices.1);

        // Act
        let actual_components = uf.components_by_root();

        // Assert
        assert_eq!(actual_components, expected_components);
    }

    #[test]
    fn components_by_root_with_size_zero_returns_empty_hash_map() {

        // Arrange
        let size = 0;
        let expected_group_members = HashMap::<usize, Vec<usize>>::new();
        let mut uf = Dsu::new(size);

        // Act
        let actual_group_members = uf.components_by_root();

        // Assert
        assert_eq!(actual_group_members, expected_group_members);
    }

    #[test]
    fn components_by_root_len_always_matches_group_count() {

        // Arrange
        let size = 5;
        let first_reference_vertex = 0;
        let first_vertex = 1;
        let second_reference_vertex = 3;
        let second_vertex = 4;
        let mut uf = Dsu::new(size);
        uf.merge(first_reference_vertex, first_vertex);
        uf.merge(second_reference_vertex, second_vertex);
        let expected_group_count = uf.group_count();

        // Act
        let actual_group_members = uf.components_by_root();
        let actual_group_count_from_map = actual_group_members.len();

        // Assert
        assert_eq!(actual_group_count_from_map, expected_group_count);
    }

    #[test]
    fn components_by_root_after_merging_all_vertices_returns_single_entry_with_all_vertices() {

        // Arrange
        let size = 5;
        let expected_group_count = 1;
        let expected_members = (0..size).collect::<Vec<usize>>();
        let mut uf = Dsu::new(size);
        for vertex in 1..size {
            uf.merge(0, vertex);
        }

        // Act
        let actual_group_members = uf.components_by_root();
        let actual_group_count = actual_group_members.len();
        let actual_only_members = actual_group_members.values().next().cloned().unwrap_or_default();

        // Assert
        assert_eq!(actual_group_count, expected_group_count);
        assert_eq!(actual_only_members, expected_members);
    }

}
