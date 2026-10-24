use std::collections::HashMap;

#[derive(Debug)]
pub struct WeightedDsu {
    par: Vec<usize>,
    rank: Vec<usize>,
    siz: Vec<usize>,
    weight_diff: Vec<i64>,
    is_unlimited_potential: Vec<bool>,
    group_number: usize,
}

impl WeightedDsu {

    pub fn new(size: usize) -> Self {

        Self {
            par: (0..size).collect::<Vec<usize>>(),
            rank: vec![0; size],
            siz: vec![1; size],
            weight_diff: vec![0; size],
            is_unlimited_potential: vec![false; size],
            group_number: size,
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.par.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    fn size(&self, vertex: usize) -> usize {
        self.siz[vertex]
    }

    #[inline]
    pub fn root(&mut self, vertex: usize) -> usize {

        if self.par[vertex] != vertex {
            let parent = self.par[vertex];
            let root = self.root(parent);
            self.weight_diff[vertex] += self.weight_diff[parent];
            self.par[vertex] = root;
        }

        self.par[vertex]
    }

    #[inline]
    pub fn weight(&mut self, vertex: usize) -> i64 {
        self.root(vertex);

        self.weight_diff[vertex]
    }

    #[inline]
    pub fn weight_diff(&mut self, reference_vertex: usize, vertex: usize) -> i64 {
        self.weight(vertex) - self.weight(reference_vertex)
    }

    #[inline]
    pub fn has_unlimited_potential(&mut self, vertex: usize) -> bool {
        let root = self.root(vertex);

        self.is_unlimited_potential[root]
    }

    pub fn all_roots(&mut self) -> Vec<usize> {
        (0..self.len()).filter(|&x| x == self.root(x)).collect::<Vec<usize>>()
    }

    #[inline]
    pub fn is_same_component(&mut self, x: usize, y: usize) -> bool {
        self.root(x) == self.root(y)
    }

    #[inline]
    pub fn merge(&mut self, reference_vertex: usize, vertex: usize, weight: i64) -> bool {

        let mut new_weight = self.weight(reference_vertex) - self.weight(vertex) + weight;
        let mut root_x = self.root(reference_vertex);
        let mut root_y = self.root(vertex);

        if root_x == root_y {

            if new_weight != 0 {
                self.is_unlimited_potential[root_x] = true;
            }

            return false;
        }

        if self.rank[root_x] < self.rank[root_y] {
            new_weight *= -1;
            std::mem::swap(&mut root_x, &mut root_y);
        }

        self.group_number -= 1;
        self.par[root_y] = root_x;
        self.siz[root_x] += self.siz[root_y];
        self.weight_diff[root_y] = new_weight;
        self.is_unlimited_potential[root_x] |= self.is_unlimited_potential[root_y];

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

    use super::*;

    #[test]
    fn len_returns_size_when_initialized() {
        // Arrange
        let n = 5;
        let uf = WeightedDsu::new(n);

        // Act
        let actual_check_1 = uf.len();

        // Assert
        assert_eq!(actual_check_1, n);
    }

    #[test]
    fn is_empty_true_for_zero_size() {
        // Arrange
        let input_value_1 = 0;

        let uf = WeightedDsu::new(input_value_1);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = uf.is_empty();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_empty_false_for_non_zero_size() {
        // Arrange
        let input_value_1 = 3;

        let uf = WeightedDsu::new(input_value_1);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = uf.is_empty();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn root_returns_self_in_initial_state() {
        // Arrange
        let n = 3;
        let mut uf = WeightedDsu::new(n);
        let expected_check_1 = vec![0, 1, 2];

        // Act
        let roots = (0..n).map(|v| uf.root(v)).collect::<Vec<usize>>();

        // Assert
        assert_eq!(roots, expected_check_1);
    }

    #[test]
    fn root_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.root(out_of_bounds_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn weight_returns_zero_in_initial_singleton() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let target_vertex = 0;
        let expected_check_1 = 0;

        // Act
        let actual_weight = uf.weight(target_vertex);

        // Assert
        assert_eq!(actual_weight, expected_check_1);
    }

    #[test]
    fn weight_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.weight(out_of_bounds_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn weight_diff_matches_edge_weight_after_merge() {
        // Arrange
        let input_value_1 = 2;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 7;
        let input_value_5 = 0;
        let input_value_6 = 1;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        let expected_check_1 = 7;

        // Act
        let actual_check_1 = uf.weight_diff(input_value_5, input_value_6);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn weight_diff_accumulates_along_path() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 3;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 4;
        let input_value_8 = 0;
        let input_value_9 = 2;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        let expected_check_1 = 7;

        // Act
        let actual_check_1 = uf.weight_diff(input_value_8, input_value_9);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn weight_diff_returns_zero_for_same_vertex_in_initial_singleton() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let target_vertex = 0;
        let expected_check_1 = 0;

        // Act
        let actual_weight_diff = uf.weight_diff(reference_vertex, target_vertex);

        // Assert
        assert_eq!(actual_weight_diff, expected_check_1);
    }

    #[test]
    fn weight_diff_returns_negative_when_arguments_are_swapped() {
        // Arrange
        let input_value_1 = 2;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let target_vertex = 1;
        let edge_weight = 9;
        uf.merge(reference_vertex, target_vertex, edge_weight);
        let expected_check_1 = -edge_weight;

        // Act
        let actual_weight_diff = uf.weight_diff(target_vertex, reference_vertex);

        // Assert
        assert_eq!(actual_weight_diff, expected_check_1);
    }

    #[test]
    fn weight_diff_satisfies_additivity_across_three_vertices() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 3;
        let input_value_8 = 2;
        let input_value_9 = 3;
        let input_value_10 = 4;
        let input_value_11 = 0;
        let input_value_12 = 2;
        let input_value_13 = 2;
        let input_value_14 = 3;
        let input_value_15 = 0;
        let input_value_16 = 3;
        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        uf.merge(input_value_8, input_value_9, input_value_10);

        // Act
        let weight_diff_0_2 = uf.weight_diff(input_value_11, input_value_12);
        let weight_diff_2_3 = uf.weight_diff(input_value_13, input_value_14);
        let weight_diff_0_3 = uf.weight_diff(input_value_15, input_value_16);

        let actual_check_1 = weight_diff_0_2 + weight_diff_2_3;

        // Assert
        assert_eq!(actual_check_1, weight_diff_0_3);
    }

    #[test]
    fn weight_diff_stays_same_after_repeated_root_calls_for_path_compression() {
        // Arrange
        let size = 4;
        let merges = [(0, 1, 2), (2, 3, 4), (0, 2, 3)];
        let reference = 0;
        let target = 3;
        let intermediate = 2;
        let expected_difference = 7;
        let expected_root = 0;
        let mut uf = WeightedDsu::new(size);
        for (left, right, weight) in merges {
            uf.merge(left, right, weight);
        }
        let parent_before_compression = uf.par[target];

        // Act
        let actual_root = uf.root(target);
        let repeated_root = uf.root(target);
        let parent_after_compression = uf.par[target];
        let actual_difference = uf.weight_diff(reference, target);

        // Assert
        assert_eq!(parent_before_compression, intermediate);
        assert_eq!(actual_root, expected_root);
        assert_eq!(repeated_root, expected_root);
        assert_eq!(parent_after_compression, expected_root);
        assert_eq!(actual_difference, expected_difference);
    }

    #[test]
    fn weight_diff_panics_when_reference_vertex_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_reference_vertex = uf.len();
        let target_vertex = 0;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.weight_diff(out_of_bounds_reference_vertex, target_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn weight_diff_panics_when_target_vertex_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let out_of_bounds_target_vertex = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.weight_diff(reference_vertex, out_of_bounds_target_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn has_unlimited_potential_detects_contradiction() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 3;
        let input_value_8 = 0;
        let input_value_9 = 2;
        let input_value_10 = 10;
        let input_value_11 = 0;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        uf.merge(input_value_8, input_value_9, input_value_10);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = uf.has_unlimited_potential(input_value_11);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn has_unlimited_potential_stays_false_without_contradiction() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 3;
        let input_value_8 = 2;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = uf.has_unlimited_potential(input_value_8);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn has_unlimited_potential_false_in_initial_singleton() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let target_vertex = 0;

        let expected_boolean_1 = false;

        // Act
        let actual_has_unlimited_potential = uf.has_unlimited_potential(target_vertex);

        // Assert
        assert_eq!(actual_has_unlimited_potential, expected_boolean_1);
    }

    #[test]
    fn has_unlimited_potential_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.has_unlimited_potential(out_of_bounds_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn all_roots_len_equals_vertex_count_when_no_vertices_are_connected() {
        // Arrange
        let vertex_count = 5;
        let mut uf = WeightedDsu::new(vertex_count);

        // Act
        let all_roots = uf.all_roots();
        let actual_root_count = all_roots.len();

        // Assert
        assert_eq!(actual_root_count, vertex_count);
    }

    #[test]
    fn all_roots_len_becomes_one_when_all_vertices_connected() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 1;
        let input_value_8 = 2;
        let input_value_9 = 3;
        let input_value_10 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        uf.merge(input_value_8, input_value_9, input_value_10);
        let expected_check_1 = 1;

        // Act
        let all_roots = uf.all_roots();
        let actual_root_count = all_roots.len();

        // Assert
        assert_eq!(actual_root_count, expected_check_1);
    }

    #[test]
    fn all_roots_returns_empty_for_zero_size() {
        // Arrange
        let input_value_1 = 0;
        let mut uf = WeightedDsu::new(input_value_1);

        let expected_boolean_1 = true;

        // Act
        let actual_all_roots = uf.all_roots();

        let actual_check_1 = actual_all_roots.is_empty();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn is_same_component_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();
        let in_bounds_index = 0;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.is_same_component(out_of_bounds_index, in_bounds_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn merge_reduces_group_count_by_one() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 0;

        let n = 4;
        let mut uf = WeightedDsu::new(n);
        uf.merge(input_value_1, input_value_2, input_value_3);
        let expected_check_1 = n - 1;

        // Act
        let actual_check_1 = uf.group_count();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn merge_with_self_and_nonzero_weight_marks_unlimited() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 0;
        let input_value_3 = 0;
        let input_value_4 = 5;
        let input_value_5 = 0;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = uf.has_unlimited_potential(input_value_5);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn merge_returns_true_for_disconnected_vertices() {
        // Arrange
        let input_value_1 = 2;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let target_vertex = 1;
        let edge_weight = 3;

        let expected_boolean_1 = true;

        // Act
        let actual_is_merged = uf.merge(reference_vertex, target_vertex, edge_weight);

        // Assert
        assert_eq!(actual_is_merged, expected_boolean_1);
    }

    #[test]
    fn merge_returns_false_without_contradiction_for_connected_vertices() {
        // Arrange
        let input_value_1 = 2;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let target_vertex = 1;
        let edge_weight = 5;
        uf.merge(reference_vertex, target_vertex, edge_weight);

        let expected_boolean_1 = false;
        let expected_boolean_2 = false;

        // Act
        let actual_is_merged = uf.merge(reference_vertex, target_vertex, edge_weight);
        let actual_has_unlimited_potential = uf.has_unlimited_potential(reference_vertex);

        // Assert
        assert_eq!(actual_is_merged, expected_boolean_1);
        assert_eq!(actual_has_unlimited_potential, expected_boolean_2);
    }

    #[test]
    fn merge_with_same_vertex_and_zero_weight_returns_false_without_unlimited() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let target_vertex = 0;
        let edge_weight = 0;

        let expected_boolean_1 = false;
        let expected_boolean_2 = false;

        // Act
        let actual_is_merged = uf.merge(target_vertex, target_vertex, edge_weight);
        let actual_has_unlimited_potential = uf.has_unlimited_potential(target_vertex);

        // Assert
        assert_eq!(actual_is_merged, expected_boolean_1);
        assert_eq!(actual_has_unlimited_potential, expected_boolean_2);
    }

    #[test]
    fn merge_returns_false_and_sets_unlimited_for_connected_vertices_with_contradiction() {
        // Arrange
        let input_value_1 = 2;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let target_vertex = 1;
        let expected_weight_diff = 4;
        let contradictory_weight_diff = 6;
        uf.merge(reference_vertex, target_vertex, expected_weight_diff);

        let expected_boolean_1 = false;
        let expected_boolean_2 = true;

        // Act
        let actual_is_merged = uf.merge(reference_vertex, target_vertex, contradictory_weight_diff);
        let actual_has_unlimited_potential = uf.has_unlimited_potential(reference_vertex);

        // Assert
        assert_eq!(actual_is_merged, expected_boolean_1);
        assert_eq!(actual_has_unlimited_potential, expected_boolean_2);
    }

    #[test]
    fn merge_returns_false_does_not_change_group_count_when_already_connected() {
        // Arrange
        let input_value_1 = 2;
        let mut uf = WeightedDsu::new(input_value_1);
        let reference_vertex = 0;
        let target_vertex = 1;
        let edge_weight = 2;
        uf.merge(reference_vertex, target_vertex, edge_weight);
        let expected_group_count = uf.group_count();

        let expected_boolean_1 = false;

        // Act
        let actual_is_merged = uf.merge(reference_vertex, target_vertex, edge_weight);
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_is_merged, expected_boolean_1);
        assert_eq!(actual_group_count, expected_group_count);
    }

    #[test]
    fn merge_keeps_weight_sign_consistency_when_rank_swap_branch_is_taken() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 2;
        let mut uf = WeightedDsu::new(input_value_1);
        let first_reference_vertex = 1;
        let first_target_vertex = 2;
        let first_edge_weight = 4;
        uf.merge(
            first_reference_vertex,
            first_target_vertex,
            first_edge_weight,
        );
        let second_reference_vertex = 0;
        let second_target_vertex = 1;
        let second_edge_weight = 3;
        uf.merge(
            second_reference_vertex,
            second_target_vertex,
            second_edge_weight,
        );
        let expected_check_1 = 7;

        // Act
        let actual_weight_diff = uf.weight_diff(input_value_2, input_value_3);

        // Assert
        assert_eq!(actual_weight_diff, expected_check_1);
    }

    #[test]
    fn unlimited_flag_propagates_when_merging_unlimited_and_limited_components() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 0;
        let input_value_6 = 1;
        let input_value_7 = 3;
        let input_value_8 = 2;
        let input_value_9 = 3;
        let input_value_10 = 1;
        let input_value_11 = 0;
        let input_value_12 = 2;
        let input_value_13 = 5;
        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        uf.merge(input_value_8, input_value_9, input_value_10);
        let expected_check_1 = vec![true, true, true, true];

        // Act
        uf.merge(input_value_11, input_value_12, input_value_13);
        let actual_flags = (0..4)
            .map(|vertex| uf.has_unlimited_potential(vertex))
            .collect::<Vec<bool>>();

        // Assert
        assert_eq!(actual_flags, expected_check_1);
    }

    #[test]
    fn merge_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();
        let in_bounds_index = 0;
        let edge_weight = 0;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.merge(out_of_bounds_index, in_bounds_index, edge_weight);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn component_size_accumulates_merged_vertices() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 0;
        let input_value_6 = 2;
        let input_value_7 = 3;
        let input_value_8 = 1;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        let expected_check_1 = 3;

        // Act
        let actual_check_1 = uf.component_size(input_value_8);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn component_size_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.component_size(out_of_bounds_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn all_component_size_returns_all_ones_when_no_vertices_are_connected() {
        // Arrange
        let vertex_count = 4;
        let mut uf = WeightedDsu::new(vertex_count);
        let expected_check_1 = vec![1, 1, 1, 1];

        // Act
        let actual_all_component_size = uf.all_component_size();

        // Assert
        assert_eq!(actual_all_component_size, expected_check_1);
    }

    #[test]
    fn all_component_size_sum_equals_len_in_multi_component_state() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 3;
        let input_value_7 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);

        // Act
        let all_component_size = uf.all_component_size();
        let actual_size_sum = all_component_size.iter().sum::<usize>();
        let expected_total_size = uf.len();

        // Assert
        assert_eq!(actual_size_sum, expected_total_size);
    }

    #[test]
    fn all_component_size_returns_empty_for_zero_size() {
        // Arrange
        let input_value_1 = 0;
        let mut uf = WeightedDsu::new(input_value_1);

        let expected_boolean_1 = true;

        // Act
        let actual_all_component_size = uf.all_component_size();

        let actual_check_1 = actual_all_component_size.is_empty();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn group_count_zero_when_initialized_with_zero() {
        // Arrange
        let input_value_1 = 0;
        let uf = WeightedDsu::new(input_value_1);
        let expected_check_1 = 0;

        // Act
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_group_count, expected_check_1);
    }

    #[test]
    fn group_count_one_when_initialized_with_one() {
        // Arrange
        let input_value_1 = 1;
        let uf = WeightedDsu::new(input_value_1);
        let expected_check_1 = 1;

        // Act
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_group_count, expected_check_1);
    }

    #[test]
    fn component_members_returns_component_vertices() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 1;
        let input_value_8 = 1;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);
        let expected_check_1 = vec![0, 1, 2];

        // Act
        let actual_check_1 = uf.component_members(input_value_8);

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
    }

    #[test]
    fn component_members_returns_singleton_for_isolated_vertex() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        let isolated_vertex = 2;
        let expected_check_1 = vec![isolated_vertex];

        // Act
        let actual_component_members = uf.component_members(isolated_vertex);

        // Assert
        assert_eq!(actual_component_members, expected_check_1);
    }

    #[test]
    fn component_members_panics_when_index_is_len() {
        // Arrange
        let input_value_1 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        let out_of_bounds_index = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            uf.component_members(out_of_bounds_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn components_by_root_collects_each_component() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;

        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);

        let mut expected = HashMap::new();
        expected.insert(0, vec![0, 1]);
        expected.insert(2, vec![2]);
        expected.insert(3, vec![3]);

        // Act
        let actual_check_1 = uf.components_by_root();

        // Assert
        assert_eq!(actual_check_1, expected);
    }

    #[test]
    fn components_by_root_len_matches_group_count() {
        // Arrange
        let input_value_1 = 5;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 3;
        let input_value_6 = 4;
        let input_value_7 = 1;
        let mut uf = WeightedDsu::new(input_value_1);
        uf.merge(input_value_2, input_value_3, input_value_4);
        uf.merge(input_value_5, input_value_6, input_value_7);

        // Act
        let components_by_root = uf.components_by_root();
        let actual_group_member_count = components_by_root.len();
        let expected_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_group_member_count, expected_group_count);
    }

    #[test]
    fn components_by_root_union_covers_all_vertices() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 3;
        let input_value_6 = 1;
        let vertex_count = 5;
        let mut uf = WeightedDsu::new(vertex_count);
        uf.merge(input_value_1, input_value_2, input_value_3);
        uf.merge(input_value_4, input_value_5, input_value_6);

        // Act
        let components_by_root = uf.components_by_root();
        let mut actual_vertices = components_by_root
            .values()
            .flat_map(|members| members.iter().copied())
            .collect::<Vec<usize>>();
        actual_vertices.sort_unstable();
        let expected_vertices = (0..vertex_count).collect::<Vec<usize>>();

        // Assert
        assert_eq!(actual_vertices, expected_vertices);
    }

    #[test]
    fn components_by_root_returns_empty_for_zero_size() {
        // Arrange
        let input_value_1 = 0;
        let mut uf = WeightedDsu::new(input_value_1);

        let expected_boolean_1 = true;

        // Act
        let actual_components_by_root = uf.components_by_root();

        let actual_check_1 = actual_components_by_root.is_empty();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }
}
