#[derive(Debug, Clone)]
pub struct LowestCommonAncestor {
    size: usize,
    msb: usize,
    parents: Vec<Option<usize>>,
    depth: Vec<Option<usize>>,
    doubling: Vec<Vec<Option<usize>>>,
}

impl LowestCommonAncestor {
    pub fn new(vertex_size: usize, parent_list: &[isize]) -> Self {
        let parents = Self::normalize_parents(vertex_size, parent_list);
        let msb = Self::bit_length(vertex_size.saturating_sub(1));

        let mut lca = Self {
            size: vertex_size,
            msb,
            parents,
            depth: vec![None; vertex_size],
            doubling: vec![vec![None; vertex_size]; msb + 1],
        };

        lca.all_depths();
        lca.build_doubling_table();

        lca
    }

    #[inline]
    pub fn contains(&self, vertex: usize) -> bool {
        vertex < self.size
    }

    pub fn level_ancestor(&self, vertex: usize, height: usize) -> Option<usize> {
        if !self.contains(vertex) {
            return None;
        }
        if height > self.depth[vertex]? {
            return None;
        }

        let mut current = Some(vertex);

        for k in (0..=self.msb).rev() {
            if (height >> k) & 1 == 1 {
                current = current.and_then(|v| self.doubling[k][v]);
            }
        }

        current
    }

    pub fn lowest_common_ancestor(&self, mut u: usize, mut v: usize) -> Option<usize> {
        if !self.contains(u) || !self.contains(v) {
            return None;
        }

        let depth_u = self.depth[u]?;
        let depth_v = self.depth[v]?;

        if depth_u < depth_v {
            std::mem::swap(&mut u, &mut v);
        }

        let diff = self.depth[u]? - self.depth[v]?;
        u = self.level_ancestor(u, diff)?;

        if u == v {
            return Some(u);
        }

        for k in (0..=self.msb).rev() {
            let parent_u = self.doubling[k][u];
            let parent_v = self.doubling[k][v];

            if parent_u.is_some() && parent_v.is_some() && parent_u != parent_v {
                u = parent_u.unwrap();
                v = parent_v.unwrap();
            }
        }

        self.doubling[0][u]
    }

    pub fn distance(&self, u: usize, v: usize) -> Option<usize> {
        let lca = self.lowest_common_ancestor(u, v)?;
        let depth_u = self.depth[u]?;
        let depth_v = self.depth[v]?;
        let depth_lca = self.depth[lca]?;

        let left = depth_u.checked_sub(depth_lca)?;
        let right = depth_v.checked_sub(depth_lca)?;

        Some(left + right)
    }

    fn build_doubling_table(&mut self) {
        for vertex in 0..self.size {
            self.doubling[0][vertex] = self.parents[vertex];
        }

        for k in 0..self.msb {
            for vertex in 0..self.size {
                let parent = self.doubling[k][vertex];
                if parent.is_none() {
                    continue;
                }

                self.doubling[k + 1][vertex] = self.doubling[k][parent.unwrap()];
            }
        }
    }

    fn all_depths(&mut self) {
        let mut state = vec![VisitState::Unvisited; self.size];

        for vertex in 0..self.size {
            if self.depth[vertex].is_some() {
                continue;
            }
            self.depth(vertex, &mut state);
        }
    }

    fn depth(&mut self, vertex: usize, state: &mut [VisitState]) -> usize {
        if let Some(depth) = self.depth[vertex] {
            return depth;
        }

        match state[vertex] {
            VisitState::Visiting => {
                self.parents[vertex] = None;
                self.depth[vertex] = Some(0);
                state[vertex] = VisitState::Done;
                return 0;
            }

            VisitState::Done => {
                return self.depth[vertex].unwrap_or(0);
            }

            VisitState::Unvisited => {}
        }

        state[vertex] = VisitState::Visiting;

        let depth = match self.parents[vertex] {
            Some(parent) => self.depth(parent, state) + 1,
            None => 0,
        };

        self.depth[vertex] = Some(depth);
        state[vertex] = VisitState::Done;

        depth
    }

    fn bit_length(n: usize) -> usize {
        if n == 0 {
            0
        } else {
            usize::BITS as usize - n.leading_zeros() as usize
        }
    }

    fn normalize_parents(vertex_size: usize, parent_list: &[isize]) -> Vec<Option<usize>> {
        let mut normalized = vec![None; vertex_size];

        if parent_list.len() < vertex_size {
            for (offset, &raw_parent) in parent_list.iter().enumerate() {
                let vertex = offset + 1;
                if vertex >= vertex_size {
                    break;
                }

                normalized[vertex] = Self::sanitize_parent(vertex, raw_parent, vertex_size);
            }
        } else {
            for (vertex, &raw_parent) in parent_list.iter().take(vertex_size).enumerate() {
                normalized[vertex] = Self::sanitize_parent(vertex, raw_parent, vertex_size);
            }
        }

        normalized
    }

    fn sanitize_parent(vertex: usize, raw_parent: isize, vertex_size: usize) -> Option<usize> {
        if raw_parent < 0 {
            return None;
        }

        let parent = raw_parent as usize;
        if parent >= vertex_size || parent == vertex {
            None
        } else {
            Some(parent)
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum VisitState {
    Unvisited,
    Visiting,
    Done,
}

#[cfg(test)]
mod tests {

    use super::LowestCommonAncestor;

    fn create_chain_lca_with_compact_parent_format() -> LowestCommonAncestor {
        let vertex_size = 5;
        let parent_list = [0, 1, 2, 3];
        LowestCommonAncestor::new(vertex_size, &parent_list)
    }

    fn create_chain_lca_with_full_parent_format() -> LowestCommonAncestor {
        let vertex_size = 5;
        let parent_list = [-1, 0, 1, 2, 3];
        LowestCommonAncestor::new(vertex_size, &parent_list)
    }

    fn create_branch_tree_lca() -> LowestCommonAncestor {
        let vertex_size = 7;
        let parent_list = [-1, 0, 0, 1, 1, 2, 2];
        LowestCommonAncestor::new(vertex_size, &parent_list)
    }

    fn create_disconnected_lca() -> LowestCommonAncestor {
        let vertex_size = 4;
        let parent_list = [-1, 0, -1, 2];
        LowestCommonAncestor::new(vertex_size, &parent_list)
    }

    #[test]
    fn new_with_empty_vertex_set_contains_returns_false_and_level_ancestor_returns_none() {
        // Arrange
        let vertex_size = 0;
        let parent_list: [isize; 0] = [];
        let target_vertex = 0;
        let target_height = 0;
        let expected_contains = false;
        let expected_ancestor = None;
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let contains_result = lca.contains(target_vertex);
        let ancestor_result = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(contains_result, expected_contains);
        assert_eq!(ancestor_result, expected_ancestor);
    }

    #[test]
    fn new_with_single_vertex_and_empty_parent_list_returns_root_for_lca_and_zero_distance() {
        // Arrange
        let vertex_size = 1;
        let parent_list: [isize; 0] = [];
        let left_vertex = 0;
        let right_vertex = 0;
        let expected_lca = Some(0);
        let expected_distance = Some(0);
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let lca_result = lca.lowest_common_ancestor(left_vertex, right_vertex);
        let distance_result = lca.distance(left_vertex, right_vertex);

        // Assert
        assert_eq!(lca_result, expected_lca);
        assert_eq!(distance_result, expected_distance);
    }

    #[test]
    fn new_with_single_vertex_and_negative_root_parent_matches_empty_parent_list_behavior() {
        // Arrange
        let vertex_size = 1;
        let empty_parent_list: [isize; 0] = [];
        let full_parent_list = [-1];
        let target_vertex = 0;
        let target_height = 0;
        let left_vertex = 0;
        let right_vertex = 0;
        let lca_with_empty_parent = LowestCommonAncestor::new(vertex_size, &empty_parent_list);
        let lca_with_full_parent = LowestCommonAncestor::new(vertex_size, &full_parent_list);

        // Act
        let ancestor_with_empty_parent =
            lca_with_empty_parent.level_ancestor(target_vertex, target_height);
        let ancestor_with_full_parent =
            lca_with_full_parent.level_ancestor(target_vertex, target_height);
        let lca_with_empty_parent_result =
            lca_with_empty_parent.lowest_common_ancestor(left_vertex, right_vertex);
        let lca_with_full_parent_result =
            lca_with_full_parent.lowest_common_ancestor(left_vertex, right_vertex);
        let distance_with_empty_parent = lca_with_empty_parent.distance(left_vertex, right_vertex);
        let distance_with_full_parent = lca_with_full_parent.distance(left_vertex, right_vertex);

        // Assert
        assert_eq!(ancestor_with_empty_parent, ancestor_with_full_parent);
        assert_eq!(lca_with_empty_parent_result, lca_with_full_parent_result);
        assert_eq!(distance_with_empty_parent, distance_with_full_parent);
    }

    #[test]
    fn new_with_compact_and_full_parent_formats_produces_identical_level_ancestor_results() {
        // Arrange
        let target_vertices = [0, 1, 2, 3, 4];
        let target_heights = [0, 1, 2, 3, 4, 5];
        let lca_with_compact_parent = create_chain_lca_with_compact_parent_format();
        let lca_with_full_parent = create_chain_lca_with_full_parent_format();

        // Act
        let mut compact_results = Vec::new();
        let mut full_results = Vec::new();
        for &vertex in &target_vertices {
            for &height in &target_heights {
                compact_results.push(lca_with_compact_parent.level_ancestor(vertex, height));
                full_results.push(lca_with_full_parent.level_ancestor(vertex, height));
            }
        }

        // Assert
        assert_eq!(compact_results, full_results);
    }

    #[test]
    fn new_with_negative_parent_value_treats_target_vertex_as_root() {
        // Arrange
        let vertex_size = 3;
        let parent_list = [-1, 0];
        let target_vertex = 1;
        let target_height = 1;
        let expected_ancestor = None;
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let ancestor_result = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(ancestor_result, expected_ancestor);
    }

    #[test]
    fn new_with_out_of_range_parent_value_treats_target_vertex_as_root() {
        // Arrange
        let vertex_size = 3;
        let parent_list = [0, 3];
        let target_vertex = 2;
        let target_height = 1;
        let expected_ancestor = None;
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let ancestor_result = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(ancestor_result, expected_ancestor);
    }

    #[test]
    fn new_with_self_referential_parent_value_treats_target_vertex_as_root_without_recursion_loop()
    {
        // Arrange
        let vertex_size = 3;
        let parent_list = [0, 2];
        let target_vertex = 2;
        let zero_height = 0;
        let parent_height = 1;
        let expected_self_ancestor = Some(2);
        let expected_parent_ancestor = None;
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let self_ancestor_result = lca.level_ancestor(target_vertex, zero_height);
        let parent_ancestor_result = lca.level_ancestor(target_vertex, parent_height);

        // Assert
        assert_eq!(self_ancestor_result, expected_self_ancestor);
        assert_eq!(parent_ancestor_result, expected_parent_ancestor);
    }

    #[test]
    fn new_ignores_extra_parent_entries_when_parent_list_is_longer_than_vertex_size() {
        // Arrange
        let vertex_size = 3;
        let parent_list = [-1, 0, 1, 99, 99];
        let target_vertex = 2;
        let target_height = 2;
        let expected_ancestor = Some(0);
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let actual_ancestor = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(actual_ancestor, expected_ancestor);
    }

    #[test]
    fn new_with_cycle_in_parent_list_does_not_loop_and_breaks_cycle_at_current_vertex() {
        // Arrange
        let vertex_size = 3;
        let parent_list = [-1, 2, 1];
        let cycle_vertex = 1;
        let child_vertex = 2;
        let parent_height = 1;
        let expected_cycle_vertex_parent = None;
        let expected_child_parent = Some(cycle_vertex);
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let actual_cycle_vertex_parent = lca.level_ancestor(cycle_vertex, parent_height);
        let actual_child_parent = lca.level_ancestor(child_vertex, parent_height);

        // Assert
        assert_eq!(actual_cycle_vertex_parent, expected_cycle_vertex_parent);
        assert_eq!(actual_child_parent, expected_child_parent);
    }

    #[test]
    fn new_treats_vertices_without_parent_entry_as_roots() {
        // Arrange
        let vertex_size = 4;
        let parent_list = [0];
        let unspecified_vertex = 2;
        let parent_height = 1;
        let expected_ancestor = None;
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let actual_ancestor = lca.level_ancestor(unspecified_vertex, parent_height);

        // Assert
        assert_eq!(actual_ancestor, expected_ancestor);
    }

    #[test]
    fn lowest_common_ancestor_returns_same_vertex_for_identical_inputs() {
        // Arrange
        let left_vertex = 3;
        let right_vertex = 3;
        let expected_result = Some(3);
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let lca_result = lca.lowest_common_ancestor(left_vertex, right_vertex);

        // Assert
        assert_eq!(lca_result, expected_result);
    }

    #[test]
    fn lowest_common_ancestor_returns_ancestor_when_one_vertex_is_ancestor_of_other() {
        // Arrange
        let ancestor_vertex = 1;
        let descendant_vertex = 4;
        let expected_result = Some(1);
        let lca = create_branch_tree_lca();

        // Act
        let lca_result = lca.lowest_common_ancestor(ancestor_vertex, descendant_vertex);

        // Assert
        assert_eq!(lca_result, expected_result);
    }

    #[test]
    fn lowest_common_ancestor_returns_parent_for_sibling_vertices() {
        // Arrange
        let left_sibling = 3;
        let right_sibling = 4;
        let expected_result = Some(1);
        let lca = create_branch_tree_lca();

        // Act
        let lca_result = lca.lowest_common_ancestor(left_sibling, right_sibling);

        // Assert
        assert_eq!(lca_result, expected_result);
    }

    #[test]
    fn lowest_common_ancestor_returns_none_for_vertices_in_different_components() {
        // Arrange
        let left_vertex = 1;
        let right_vertex = 3;
        let expected_result = None;
        let lca = create_disconnected_lca();

        // Act
        let lca_result = lca.lowest_common_ancestor(left_vertex, right_vertex);

        // Assert
        assert_eq!(lca_result, expected_result);
    }

    #[test]
    fn lowest_common_ancestor_returns_none_when_any_input_vertex_is_out_of_range() {
        // Arrange
        let invalid_vertex = 5;
        let valid_vertex = 2;
        let expected_left_invalid_result = None;
        let expected_right_invalid_result = None;
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let left_invalid_result = lca.lowest_common_ancestor(invalid_vertex, valid_vertex);
        let right_invalid_result = lca.lowest_common_ancestor(valid_vertex, invalid_vertex);

        // Assert
        assert_eq!(left_invalid_result, expected_left_invalid_result);
        assert_eq!(right_invalid_result, expected_right_invalid_result);
    }

    #[test]
    fn lowest_common_ancestor_returns_same_result_regardless_of_argument_order() {
        // Arrange
        let left_vertex = 4;
        let right_vertex = 6;
        let expected_result = Some(0);
        let lca = create_branch_tree_lca();

        // Act
        let forward_result = lca.lowest_common_ancestor(left_vertex, right_vertex);
        let reverse_result = lca.lowest_common_ancestor(right_vertex, left_vertex);

        // Assert
        assert_eq!(forward_result, reverse_result);
        assert_eq!(forward_result, expected_result);
    }

    #[test]
    fn distance_returns_zero_for_identical_vertices() {
        // Arrange
        let left_vertex = 6;
        let right_vertex = 6;
        let expected_result = Some(0);
        let lca = create_branch_tree_lca();

        // Act
        let distance_result = lca.distance(left_vertex, right_vertex);

        // Assert
        assert_eq!(distance_result, expected_result);
    }

    #[test]
    fn distance_returns_one_for_parent_child_vertices() {
        // Arrange
        let parent_vertex = 1;
        let child_vertex = 4;
        let expected_result = Some(1);
        let lca = create_branch_tree_lca();

        // Act
        let distance_result = lca.distance(parent_vertex, child_vertex);

        // Assert
        assert_eq!(distance_result, expected_result);
    }

    #[test]
    fn distance_matches_manual_depth_formula_for_vertices_in_different_branches() {
        // Arrange
        let left_vertex = 3;
        let right_vertex = 5;
        let left_depth = 2;
        let right_depth = 2;
        let common_ancestor_depth = 0;
        let expected_distance = left_depth + right_depth - 2 * common_ancestor_depth;
        let expected_result = Some(expected_distance);
        let lca = create_branch_tree_lca();

        // Act
        let distance_result = lca.distance(left_vertex, right_vertex);

        // Assert
        assert_eq!(distance_result, expected_result);
    }

    #[test]
    fn distance_returns_none_for_vertices_in_different_components() {
        // Arrange
        let left_vertex = 1;
        let right_vertex = 3;
        let expected_result = None;
        let lca = create_disconnected_lca();

        // Act
        let distance_result = lca.distance(left_vertex, right_vertex);

        // Assert
        assert_eq!(distance_result, expected_result);
    }

    #[test]
    fn distance_returns_none_when_any_input_vertex_is_out_of_range() {
        // Arrange
        let invalid_vertex = 5;
        let valid_vertex = 0;
        let expected_left_invalid_result = None;
        let expected_right_invalid_result = None;
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let left_invalid_result = lca.distance(invalid_vertex, valid_vertex);
        let right_invalid_result = lca.distance(valid_vertex, invalid_vertex);

        // Assert
        assert_eq!(left_invalid_result, expected_left_invalid_result);
        assert_eq!(right_invalid_result, expected_right_invalid_result);
    }

    #[test]
    fn level_ancestor_returns_none_for_out_of_range_vertex() {
        // Arrange
        let target_vertex = 5;
        let target_height = 0;
        let expected_result = None;
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let ancestor_result = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(ancestor_result, expected_result);
    }

    #[test]
    fn level_ancestor_with_zero_height_returns_same_vertex() {
        // Arrange
        let target_vertex = 4;
        let target_height = 0;
        let expected_result = Some(4);
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let ancestor_result = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(ancestor_result, expected_result);
    }

    #[test]
    fn level_ancestor_returns_expected_values_for_depth_boundary_heights() {
        // Arrange
        let target_vertex = 4;
        let target_depth = 4;
        let lower_height = target_depth - 1;
        let exact_height = target_depth;
        let upper_height = target_depth + 1;
        let expected_lower_result = Some(1);
        let expected_exact_result = Some(0);
        let expected_upper_result = None;
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let lower_result = lca.level_ancestor(target_vertex, lower_height);
        let exact_result = lca.level_ancestor(target_vertex, exact_height);
        let upper_result = lca.level_ancestor(target_vertex, upper_height);

        // Assert
        assert_eq!(lower_result, expected_lower_result);
        assert_eq!(exact_result, expected_exact_result);
        assert_eq!(upper_result, expected_upper_result);
    }

    #[test]
    fn level_ancestor_returns_none_when_height_exceeds_doubling_table_bit_width() {
        // Arrange
        let target_vertex = 4;
        let target_height = 16;
        let expected_result = None;
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let actual_result = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn level_ancestor_returns_none_for_root_with_height_one() {
        // Arrange
        let root_vertex = 0;
        let target_height = 1;
        let expected_result = None;
        let lca = create_chain_lca_with_compact_parent_format();

        // Act
        let ancestor_result = lca.level_ancestor(root_vertex, target_height);

        // Assert
        assert_eq!(ancestor_result, expected_result);
    }

    #[test]
    fn level_ancestor_returns_expected_ancestor_across_power_of_two_jump() {
        // Arrange
        let target_vertex = 6;
        let target_height = 2;
        let expected_ancestor = Some(0);
        let lca = create_branch_tree_lca();

        // Act
        let actual_ancestor = lca.level_ancestor(target_vertex, target_height);

        // Assert
        assert_eq!(actual_ancestor, expected_ancestor);
    }

    #[test]
    fn level_ancestor_returns_none_for_disconnected_component_root_height_one() {
        // Arrange
        let disconnected_root = 2;
        let target_height = 1;
        let expected_ancestor = None;
        let lca = create_disconnected_lca();

        // Act
        let actual_ancestor = lca.level_ancestor(disconnected_root, target_height);

        // Assert
        assert_eq!(actual_ancestor, expected_ancestor);
    }

    #[test]
    fn contains_returns_expected_values_for_boundary_vertices() {
        // Arrange
        let vertex_size = 5;
        let parent_list = [0, 1, 2, 3];
        let boundary_vertex = vertex_size - 1;
        let lower_vertex = boundary_vertex - 1;
        let upper_vertex = boundary_vertex + 1;
        let expected_lower_result = true;
        let expected_boundary_result = true;
        let expected_upper_result = false;
        let lca = LowestCommonAncestor::new(vertex_size, &parent_list);

        // Act
        let lower_result = lca.contains(lower_vertex);
        let boundary_result = lca.contains(boundary_vertex);
        let upper_result = lca.contains(upper_vertex);

        // Assert
        assert_eq!(lower_result, expected_lower_result);
        assert_eq!(boundary_result, expected_boundary_result);
        assert_eq!(upper_result, expected_upper_result);
    }
}
