use std::collections::HashSet;

const INVALID_VERTEX: usize = usize::MAX;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PathSegment {
    pub start: usize,
    pub end: usize,
    pub is_reversed: bool,
}


#[derive(Debug, Clone)]
pub struct HeavyLightDecomposition {
    size: usize,
    root: usize,
    parent: Vec<usize>,
    depth: Vec<usize>,
    heavy_child: Vec<usize>,
    head: Vec<usize>,
    position: Vec<usize>,
    subtree_size: Vec<usize>,
    order: Vec<usize>,
    is_attached: Vec<bool>,
}

impl HeavyLightDecomposition {

    pub fn new(vertex_size: usize, edges: &[(usize, usize)], root: usize) -> Self {

        assert!(vertex_size < INVALID_VERTEX);
        assert!(root < vertex_size);
        Self::validate_input_graph(vertex_size, edges);

        let adjacency = Self::build_adjacency(vertex_size, edges);

        let (parent, depth, heavy_child, subtree_size, is_attached) =
            Self::compute_sizes_and_heavy(vertex_size, root, &adjacency);

        let (head, position, order) =
            Self::assign_heads_and_positions(vertex_size, root, &adjacency, &parent, &heavy_child, &is_attached);

        Self {
            size: vertex_size,
            root,
            parent,
            depth,
            heavy_child,
            head,
            position,
            subtree_size,
            order,
            is_attached,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    #[inline]
    pub fn contains(&self, vertex: usize) -> bool {
        vertex < self.size && self.is_attached[vertex]
    }

    #[inline]
    pub fn position(&self, vertex: usize) -> Option<usize> {
        if self.contains(vertex) {
            Some(self.position[vertex])
        } else {
            None
        }
    }

    #[inline]
    pub fn head(&self, vertex: usize) -> Option<usize> {
        if self.contains(vertex) {
            Some(self.head[vertex])
        } else {
            None
        }
    }

    #[inline]
    pub fn depth(&self, vertex: usize) -> Option<usize> {
        if self.contains(vertex) {
            Some(self.depth[vertex])
        } else {
            None
        }
    }

    pub fn lowest_common_ancestor(&self, mut u: usize, mut v: usize) -> Option<usize> {

        if !self.contains(u) || !self.contains(v) {
            return None;
        }

        while self.head[u] != self.head[v] {

            if self.depth[self.head[u]] < self.depth[self.head[v]] {
                v = self.parent[self.head[v]];
            } else {
                u = self.parent[self.head[u]];
            }
        }

        if self.depth[u] < self.depth[v] {
            Some(u)
        } else {
            Some(v)
        }
    }

    pub fn path_segments(&self, mut u: usize, mut v: usize) -> Option<Vec<PathSegment>> {

        if !self.contains(u) || !self.contains(v) {
            return None;
        }

        let mut upper_segments = Vec::new();
        let mut lower_segments = Vec::new();

        while self.head[u] != self.head[v] {

            if self.depth[self.head[u]] < self.depth[self.head[v]] {
                lower_segments.push(PathSegment {
                    start: self.position[self.head[v]],
                    end: self.position[v] + 1,
                    is_reversed: false,
                });
                v = self.parent[self.head[v]];
            } else {
                upper_segments.push(PathSegment {
                    start: self.position[self.head[u]],
                    end: self.position[u] + 1,
                    is_reversed: true,
                });
                u = self.parent[self.head[u]];
            }
        }

        if self.depth[u] < self.depth[v] {
            lower_segments.push(PathSegment {
                start: self.position[u],
                end: self.position[v] + 1,
                is_reversed: false,
            });
        } else {
            upper_segments.push(PathSegment {
                start: self.position[v],
                end: self.position[u] + 1,
                is_reversed: true,
            });
        }

        lower_segments.reverse();
        upper_segments.extend(lower_segments);

        Some(upper_segments)
    }

    pub fn subtree_range(&self, vertex: usize) -> Option<(usize, usize)> {

        if !self.contains(vertex) {
            return None;
        }

        let start = self.position[vertex];
        Some((start, start + self.subtree_size[vertex]))
    }

    #[inline]
    pub fn vertex_at(&self, position: usize) -> Option<usize> {
        if position < self.order.len() {
            Some(self.order[position])
        } else {
            None
        }
    }

    fn build_adjacency(vertex_size: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {

        let mut adjacency = vec![Vec::new(); vertex_size];

        for &(u, v) in edges {

            assert!(u < vertex_size && v < vertex_size);
            adjacency[u].push(v);
            adjacency[v].push(u);
        }

        adjacency
    }

    fn normalize_undirected_edge(left_vertex: usize, right_vertex: usize) -> (usize, usize) {
        if left_vertex < right_vertex {
            (left_vertex, right_vertex)
        } else {
            (right_vertex, left_vertex)
        }
    }

    fn find_disjoint_set_root(vertex: usize, union_find_parent: &mut [usize]) -> usize {
        let mut current_vertex = vertex;
        while union_find_parent[current_vertex] != current_vertex {
            current_vertex = union_find_parent[current_vertex];
        }

        let root_vertex = current_vertex;
        let mut compressed_vertex = vertex;
        while union_find_parent[compressed_vertex] != compressed_vertex {
            let next_vertex = union_find_parent[compressed_vertex];
            union_find_parent[compressed_vertex] = root_vertex;
            compressed_vertex = next_vertex;
        }

        root_vertex
    }

    fn unite_disjoint_sets(
        left_vertex: usize,
        right_vertex: usize,
        union_find_parent: &mut [usize],
        union_find_size: &mut [usize],
    ) -> bool {
        let mut left_root = Self::find_disjoint_set_root(left_vertex, union_find_parent);
        let mut right_root = Self::find_disjoint_set_root(right_vertex, union_find_parent);

        if left_root == right_root {
            return false;
        }

        if union_find_size[left_root] < union_find_size[right_root] {
            std::mem::swap(&mut left_root, &mut right_root);
        }

        union_find_parent[right_root] = left_root;
        union_find_size[left_root] += union_find_size[right_root];
        true
    }

    fn validate_input_graph(vertex_size: usize, edges: &[(usize, usize)]) {
        let mut normalized_edges = HashSet::with_capacity(edges.len());
        let mut union_find_parent = (0..vertex_size).collect::<Vec<_>>();
        let mut union_find_size = vec![1; vertex_size];

        for &(left_vertex, right_vertex) in edges {
            assert!(left_vertex < vertex_size && right_vertex < vertex_size);
            assert!(left_vertex != right_vertex, "self-loop edge is not allowed");

            let normalized_edge = Self::normalize_undirected_edge(left_vertex, right_vertex);
            assert!(
                normalized_edges.insert(normalized_edge),
                "multiple edges are not allowed"
            );

            let is_merged = Self::unite_disjoint_sets(
                left_vertex,
                right_vertex,
                &mut union_find_parent,
                &mut union_find_size,
            );
            assert!(is_merged, "cycle edge is not allowed");
        }
    }

    fn compute_sizes_and_heavy(
        vertex_size: usize,
        root: usize,
        adjacency: &[Vec<usize>],
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>, Vec<usize>, Vec<bool>) {

        let mut parent = vec![INVALID_VERTEX; vertex_size];
        let mut depth = vec![0; vertex_size];
        let mut subtree_size = vec![1; vertex_size];
        let mut heavy_child = vec![INVALID_VERTEX; vertex_size];
        let mut is_attached = vec![false; vertex_size];

        let mut stack = Vec::new();
        stack.push((root, root, false));

        while let Some((vertex, par, is_visited)) = stack.pop() {

            if is_visited {

                let mut size = 1;
                let mut largest = 0;

                for &next_vertex in &adjacency[vertex] {

                    if next_vertex == par {
                        continue;
                    }

                    size += subtree_size[next_vertex];

                    if largest < subtree_size[next_vertex] {
                        largest = subtree_size[next_vertex];
                        heavy_child[vertex] = next_vertex;
                    }
                }

                subtree_size[vertex] = size;
                continue;
            }

            parent[vertex] = par;
            is_attached[vertex] = true;

            if vertex != par {
                depth[vertex] = depth[par] + 1;
            }

            stack.push((vertex, par, true));

            for &next_vertex in &adjacency[vertex] {

                if next_vertex == par {
                    continue;
                }

                stack.push((next_vertex, vertex, false));
            }
        }

        (parent, depth, heavy_child, subtree_size, is_attached)
    }

    fn assign_heads_and_positions(
        vertex_size: usize,
        root: usize,
        adjacency: &[Vec<usize>],
        parent: &[usize],
        heavy_child: &[usize],
        is_attached: &[bool],
    ) -> (Vec<usize>, Vec<usize>, Vec<usize>) {

        let mut head = vec![INVALID_VERTEX; vertex_size];
        let mut position = vec![0; vertex_size];
        let mut order = vec![0; vertex_size];
        let mut current_position = 0;

        let mut stack = Vec::new();
        stack.push((root, root));

        while let Some((start_vertex, start_head)) = stack.pop() {

            let mut vertex = start_vertex;
            let current_head = start_head;

            while vertex != INVALID_VERTEX {

                head[vertex] = current_head;
                position[vertex] = current_position;
                order[current_position] = vertex;
                current_position += 1;

                let heavy = heavy_child[vertex];

                for &next_vertex in adjacency[vertex].iter().rev() {

                    if next_vertex == parent[vertex] || next_vertex == heavy {
                        continue;
                    }

                    if is_attached[next_vertex] {
                        stack.push((next_vertex, next_vertex));
                    }
                }

                if heavy != INVALID_VERTEX && is_attached[heavy] {
                    vertex = heavy;
                } else {
                    vertex = INVALID_VERTEX;
                }
            }
        }

        (head, position, order)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;
    use std::collections::VecDeque;
    use std::panic;

    fn create_single_vertex_hld() -> HeavyLightDecomposition {
        let vertex_size = 1;
        let edges: [(usize, usize); 0] = [];
        let root = 0;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn create_chain_hld() -> HeavyLightDecomposition {
        let vertex_size = 4;
        let edges = [(0, 1), (1, 2), (2, 3)];
        let root = 0;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn balanced_tree_edges() -> Vec<(usize, usize)> {
        vec![(0, 1), (0, 2), (1, 3), (1, 4), (2, 5), (2, 6)]
    }

    fn create_balanced_tree_hld() -> HeavyLightDecomposition {
        let vertex_size = 7;
        let edges = balanced_tree_edges();
        let root = 0;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn create_light_child_head_hld() -> HeavyLightDecomposition {
        let vertex_size = 3;
        let edges = [(0, 1), (0, 2)];
        let root = 0;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn create_disconnected_hld() -> HeavyLightDecomposition {
        let vertex_size = 5;
        let edges = [(0, 1), (1, 2), (3, 4)];
        let root = 0;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn create_multi_segment_hld() -> HeavyLightDecomposition {
        let vertex_size = 7;
        let edges = [(0, 1), (0, 2), (1, 3), (3, 4), (1, 6), (2, 5)];
        let root = 0;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn create_non_zero_root_hld() -> HeavyLightDecomposition {
        let vertex_size = 5;
        let edges = [(2, 1), (1, 0), (2, 3), (3, 4)];
        let root = 2;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn asymmetric_tree_edges() -> Vec<(usize, usize)> {
        vec![(0, 1), (0, 2), (1, 3), (1, 4), (3, 5), (2, 6), (6, 7)]
    }

    fn create_disconnected_non_zero_root_hld() -> HeavyLightDecomposition {
        let vertex_size = 7;
        let edges = [(0, 1), (2, 3), (3, 4), (3, 5)];
        let root = 3;
        HeavyLightDecomposition::new(vertex_size, &edges, root)
    }

    fn build_vertices_from_path_segments(
        hld: &HeavyLightDecomposition,
        path_segments: &[PathSegment],
    ) -> Vec<usize> {
        let mut vertices = Vec::new();
        for segment in path_segments {
            if segment.is_reversed {
                for position in (segment.start..segment.end).rev() {
                    let vertex = hld
                        .vertex_at(position)
                        .expect("position in path segment must be assigned");
                    vertices.push(vertex);
                }
            } else {
                for position in segment.start..segment.end {
                    let vertex = hld
                        .vertex_at(position)
                        .expect("position in path segment must be assigned");
                    vertices.push(vertex);
                }
            }
        }
        vertices
    }

    fn calculate_naive_tree_path(
        vertex_size: usize,
        edges: &[(usize, usize)],
        start_vertex: usize,
        goal_vertex: usize,
    ) -> Vec<usize> {
        let mut adjacency = vec![Vec::new(); vertex_size];
        for &(left_vertex, right_vertex) in edges {
            adjacency[left_vertex].push(right_vertex);
            adjacency[right_vertex].push(left_vertex);
        }

        let mut parent = vec![INVALID_VERTEX; vertex_size];
        let mut queue = VecDeque::new();
        parent[start_vertex] = start_vertex;
        queue.push_back(start_vertex);

        while let Some(vertex) = queue.pop_front() {
            if vertex == goal_vertex {
                break;
            }

            for &next_vertex in &adjacency[vertex] {
                if parent[next_vertex] != INVALID_VERTEX {
                    continue;
                }

                parent[next_vertex] = vertex;
                queue.push_back(next_vertex);
            }
        }

        let mut reverse_path = Vec::new();
        let mut current_vertex = goal_vertex;
        while current_vertex != start_vertex {
            reverse_path.push(current_vertex);
            current_vertex = parent[current_vertex];
        }
        reverse_path.push(start_vertex);
        reverse_path.reverse();
        reverse_path
    }

    fn calculate_naive_parent_and_depth(
        vertex_size: usize,
        edges: &[(usize, usize)],
        root: usize,
    ) -> (Vec<usize>, Vec<usize>) {
        let mut adjacency = vec![Vec::new(); vertex_size];
        for &(left_vertex, right_vertex) in edges {
            adjacency[left_vertex].push(right_vertex);
            adjacency[right_vertex].push(left_vertex);
        }

        let mut parent = vec![INVALID_VERTEX; vertex_size];
        let mut depth = vec![INVALID_VERTEX; vertex_size];
        let mut queue = VecDeque::new();
        parent[root] = root;
        depth[root] = 0;
        queue.push_back(root);

        while let Some(vertex) = queue.pop_front() {
            for &next_vertex in &adjacency[vertex] {
                if parent[next_vertex] != INVALID_VERTEX {
                    continue;
                }

                parent[next_vertex] = vertex;
                depth[next_vertex] = depth[vertex] + 1;
                queue.push_back(next_vertex);
            }
        }

        (parent, depth)
    }

    fn calculate_naive_lowest_common_ancestor(
        mut left_vertex: usize,
        mut right_vertex: usize,
        parent: &[usize],
        depth: &[usize],
    ) -> Option<usize> {
        if left_vertex >= parent.len()
            || right_vertex >= parent.len()
            || parent[left_vertex] == INVALID_VERTEX
            || parent[right_vertex] == INVALID_VERTEX
        {
            return None;
        }

        while depth[left_vertex] > depth[right_vertex] {
            left_vertex = parent[left_vertex];
        }
        while depth[right_vertex] > depth[left_vertex] {
            right_vertex = parent[right_vertex];
        }
        while left_vertex != right_vertex {
            left_vertex = parent[left_vertex];
            right_vertex = parent[right_vertex];
        }

        Some(left_vertex)
    }

    fn calculate_naive_subtree_vertices(parent: &[usize], subtree_root: usize) -> Vec<usize> {
        let mut children = vec![Vec::new(); parent.len()];
        for (vertex, &parent_vertex) in parent.iter().enumerate() {
            if parent_vertex != INVALID_VERTEX && parent_vertex != vertex {
                children[parent_vertex].push(vertex);
            }
        }

        let mut subtree_vertices = Vec::new();
        let mut stack = vec![subtree_root];
        while let Some(vertex) = stack.pop() {
            subtree_vertices.push(vertex);
            stack.extend(children[vertex].iter().copied());
        }
        subtree_vertices.sort_unstable();
        subtree_vertices
    }

    fn vertices_in_subtree_range(hld: &HeavyLightDecomposition, vertex: usize) -> Vec<usize> {
        let (start, end) = hld
            .subtree_range(vertex)
            .expect("attached vertex must have subtree range");
        let mut vertices = (start..end)
            .map(|position| {
                hld.vertex_at(position)
                    .expect("subtree position must be assigned")
            })
            .collect::<Vec<_>>();
        vertices.sort_unstable();
        vertices
    }

    fn panic_payload_message(payload: Box<dyn Any + Send>) -> String {
        if let Some(message) = payload.downcast_ref::<String>() {
            return message.clone();
        }
        if let Some(message) = payload.downcast_ref::<&str>() {
            return (*message).to_owned();
        }
        "non-string panic payload".to_owned()
    }

    #[test]
    fn new_with_zero_vertex_size_panics() {
        // Arrange
        let vertex_size = 0;
        let edges: [(usize, usize); 0] = [];
        let root = 0;
        let expected_is_err = true;

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_is_err = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err, expected_is_err);
    }

    #[test]
    fn new_with_maximum_vertex_size_panics_before_allocation() {
        // Arrange
        let vertex_size = usize::MAX;
        let edges: [(usize, usize); 0] = [];
        let root = 0;
        let expected_is_err = true;

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_is_err = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err, expected_is_err);
    }

    #[test]
    fn new_with_root_boundary_values_panics_only_for_out_of_range_root() {
        // Arrange
        let vertex_size = 3;
        let edges: [(usize, usize); 0] = [];
        let boundary_root = vertex_size - 1;
        let lower_root = boundary_root - 1;
        let upper_root = boundary_root + 1;
        let expected_lower_is_err = false;
        let expected_boundary_is_err = false;
        let expected_upper_is_err = true;

        // Act
        let lower_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, lower_root));
        let boundary_result = panic::catch_unwind(|| {
            HeavyLightDecomposition::new(vertex_size, &edges, boundary_root)
        });
        let upper_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, upper_root));
        let lower_is_err = lower_result.is_err();
        let boundary_is_err = boundary_result.is_err();
        let upper_is_err = upper_result.is_err();

        // Assert
        assert_eq!(lower_is_err, expected_lower_is_err);
        assert_eq!(boundary_is_err, expected_boundary_is_err);
        assert_eq!(upper_is_err, expected_upper_is_err);
    }

    #[test]
    fn new_with_out_of_range_edge_endpoint_panics() {
        // Arrange
        let vertex_size = 3;
        let edges = [(0, 1), (1, 3)];
        let root = 0;
        let expected_is_err = true;

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_is_err = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err, expected_is_err);
    }

    #[test]
    fn new_with_single_vertex_returns_len_one_and_contains_root_true() {
        // Arrange
        let expected_len = 1;
        let target_vertex = 0;
        let expected_contains = true;
        let hld = create_single_vertex_hld();

        // Act
        let actual_len = hld.len();
        let actual_contains = hld.contains(target_vertex);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn new_with_non_zero_root_in_disconnected_forest_attaches_only_root_component() {
        // Arrange
        let expected_len = 7;
        let expected_contains = vec![false, false, true, true, true, true, false];
        let hld = create_disconnected_non_zero_root_hld();

        // Act
        let actual_len = hld.len();
        let actual_contains = (0..actual_len)
            .map(|vertex| hld.contains(vertex))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn new_with_cycle_input_panics() {
        // Arrange
        let vertex_size = 3;
        let edges = [(0, 1), (1, 2), (2, 0)];
        let root = 0;
        let expected_is_err = true;

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_is_err = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err, expected_is_err);
    }

    #[test]
    fn new_with_multiple_edges_input_panics() {
        // Arrange
        let vertex_size = 2;
        let edges = [(0, 1), (0, 1)];
        let root = 0;
        let expected_is_err = true;

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_is_err = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err, expected_is_err);
    }

    #[test]
    fn new_with_self_loop_input_panics() {
        // Arrange
        let vertex_size = 1;
        let edges = [(0, 0)];
        let root = 0;
        let expected_is_err = true;

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_is_err = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err, expected_is_err);
    }

    #[test]
    fn new_with_reverse_duplicate_edge_panics_as_multiple_edge() {
        // Arrange
        let vertex_size = 2;
        let edges = [(0, 1), (1, 0)];
        let root = 0;
        let expected_message = "multiple edges are not allowed";

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_message = panic_payload_message(
            panic_result.expect_err("reverse duplicate edge must cause a panic"),
        );

        // Assert
        assert_eq!(actual_message, expected_message);
    }

    #[test]
    fn new_with_cycle_only_in_component_detached_from_root_panics() {
        // Arrange
        let vertex_size = 5;
        let edges = [(0, 1), (2, 3), (3, 4), (4, 2)];
        let root = 0;
        let expected_message = "cycle edge is not allowed";

        // Act
        let panic_result =
            panic::catch_unwind(|| HeavyLightDecomposition::new(vertex_size, &edges, root));
        let actual_message = panic_payload_message(
            panic_result.expect_err("cycle outside the root component must cause a panic"),
        );

        // Assert
        assert_eq!(actual_message, expected_message);
    }

    #[test]
    fn long_chain_construction_and_endpoint_queries_complete_without_stack_overflow() {
        // Arrange
        const VERTEX_SIZE: usize = 100_000;
        let edges = (0..VERTEX_SIZE - 1)
            .map(|vertex| (vertex, vertex + 1))
            .collect::<Vec<_>>();
        let root = 0;
        let last_vertex = VERTEX_SIZE - 1;
        let expected_depth = Some(last_vertex);
        let expected_head = Some(root);
        let expected_lca = Some(root);
        let expected_segments = Some(vec![PathSegment {
            start: 0,
            end: VERTEX_SIZE,
            is_reversed: false,
        }]);

        // Act
        let hld = HeavyLightDecomposition::new(VERTEX_SIZE, &edges, root);
        let actual_depth = hld.depth(last_vertex);
        let actual_head = hld.head(last_vertex);
        let actual_lca = hld.lowest_common_ancestor(root, last_vertex);
        let actual_segments = hld.path_segments(root, last_vertex);

        // Assert
        assert_eq!(actual_depth, expected_depth);
        assert_eq!(actual_head, expected_head);
        assert_eq!(actual_lca, expected_lca);
        assert_eq!(actual_segments, expected_segments);
    }

    #[test]
    fn contains_returns_true_for_all_vertices_in_connected_tree() {
        // Arrange
        let hld = create_balanced_tree_hld();
        let expected_all_contained = true;

        // Act
        let contains_results = (0..hld.len())
            .map(|vertex| hld.contains(vertex))
            .collect::<Vec<_>>();
        let actual_all_contained = contains_results.iter().all(|&result| result);

        // Assert
        assert_eq!(actual_all_contained, expected_all_contained);
    }

    #[test]
    fn contains_returns_false_for_vertex_in_different_component() {
        // Arrange
        let detached_vertex = 3;
        let expected_contains = false;
        let hld = create_disconnected_hld();

        // Act
        let actual_contains = hld.contains(detached_vertex);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn contains_returns_expected_values_for_boundary_vertices() {
        // Arrange
        let vertex_size = 5;
        let edges = [(0, 1), (1, 2), (2, 3), (3, 4)];
        let root = 0;
        let boundary_vertex = vertex_size - 1;
        let lower_vertex = boundary_vertex - 1;
        let upper_vertex = boundary_vertex + 1;
        let expected_lower_contains = true;
        let expected_boundary_contains = true;
        let expected_upper_contains = false;
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let lower_contains = hld.contains(lower_vertex);
        let boundary_contains = hld.contains(boundary_vertex);
        let upper_contains = hld.contains(upper_vertex);

        // Assert
        assert_eq!(lower_contains, expected_lower_contains);
        assert_eq!(boundary_contains, expected_boundary_contains);
        assert_eq!(upper_contains, expected_upper_contains);
    }

    #[test]
    fn single_vertex_position_head_depth_and_vertex_at_are_zero() {
        // Arrange
        let target_vertex = 0;
        let target_position = 0;
        let expected_position = Some(0);
        let expected_head = Some(0);
        let expected_depth = Some(0);
        let expected_vertex = Some(0);
        let hld = create_single_vertex_hld();

        // Act
        let actual_position = hld.position(target_vertex);
        let actual_head = hld.head(target_vertex);
        let actual_depth = hld.depth(target_vertex);
        let actual_vertex = hld.vertex_at(target_position);

        // Assert
        assert_eq!(actual_position, expected_position);
        assert_eq!(actual_head, expected_head);
        assert_eq!(actual_depth, expected_depth);
        assert_eq!(actual_vertex, expected_vertex);
    }

    #[test]
    fn position_values_are_unique_for_all_attached_vertices() {
        // Arrange
        let expected_is_unique = true;
        let hld = create_balanced_tree_hld();

        // Act
        let mut positions = (0..hld.len())
            .filter(|&vertex| hld.contains(vertex))
            .map(|vertex| {
                hld.position(vertex)
                    .expect("attached vertex must have position")
            })
            .collect::<Vec<_>>();
        positions.sort_unstable();
        positions.dedup();
        let actual_is_unique = positions.len() == hld.len();

        // Assert
        assert_eq!(actual_is_unique, expected_is_unique);
    }

    #[test]
    fn position_head_and_depth_return_none_for_out_of_range_vertex() {
        // Arrange
        let invalid_vertex = 7;
        let expected_position = None;
        let expected_head = None;
        let expected_depth = None;
        let hld = create_balanced_tree_hld();

        // Act
        let actual_position = hld.position(invalid_vertex);
        let actual_head = hld.head(invalid_vertex);
        let actual_depth = hld.depth(invalid_vertex);

        // Assert
        assert_eq!(actual_position, expected_position);
        assert_eq!(actual_head, expected_head);
        assert_eq!(actual_depth, expected_depth);
    }

    #[test]
    fn position_head_and_depth_return_none_for_unattached_vertex() {
        // Arrange
        let detached_vertex = 3;
        let expected_position = None;
        let expected_head = None;
        let expected_depth = None;
        let hld = create_disconnected_hld();

        // Act
        let actual_position = hld.position(detached_vertex);
        let actual_head = hld.head(detached_vertex);
        let actual_depth = hld.depth(detached_vertex);

        // Assert
        assert_eq!(actual_position, expected_position);
        assert_eq!(actual_head, expected_head);
        assert_eq!(actual_depth, expected_depth);
    }

    #[test]
    fn chain_tree_assigns_same_head_to_all_vertices() {
        // Arrange
        let expected_heads = vec![Some(0), Some(0), Some(0), Some(0)];
        let hld = create_chain_hld();

        // Act
        let actual_heads = (0..hld.len())
            .map(|vertex| hld.head(vertex))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_heads, expected_heads);
    }

    #[test]
    fn light_child_vertex_has_itself_as_head() {
        // Arrange
        let target_vertex = 2;
        let expected_head = Some(2);
        let hld = create_light_child_head_hld();

        // Act
        let actual_head = hld.head(target_vertex);

        // Assert
        assert_eq!(actual_head, expected_head);
    }

    #[test]
    fn later_larger_subtree_child_becomes_heavy_child() {
        // Arrange
        let vertex_size = 5;
        let edges = [(0, 1), (0, 2), (2, 3), (2, 4)];
        let root = 0;
        let larger_child = 2;
        let smaller_child = 1;
        let expected_larger_child_head = Some(0);
        let expected_smaller_child_head = Some(1);
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let actual_larger_child_head = hld.head(larger_child);
        let actual_smaller_child_head = hld.head(smaller_child);

        // Assert
        assert_eq!(actual_larger_child_head, expected_larger_child_head);
        assert_eq!(actual_smaller_child_head, expected_smaller_child_head);
    }

    #[test]
    fn depth_matches_distance_from_non_zero_root_in_branching_tree() {
        // Arrange
        let vertex_size = 8;
        let edges = asymmetric_tree_edges();
        let root = 3;
        let (_, naive_depth) = calculate_naive_parent_and_depth(vertex_size, &edges, root);
        let expected_depths = naive_depth.into_iter().map(Some).collect::<Vec<_>>();
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let actual_depths = (0..vertex_size)
            .map(|vertex| hld.depth(vertex))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_depths, expected_depths);
    }

    #[test]
    fn lowest_common_ancestor_returns_same_vertex_for_identical_inputs() {
        // Arrange
        let left_vertex = 3;
        let right_vertex = 3;
        let expected_lca = Some(3);
        let hld = create_chain_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(left_vertex, right_vertex);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_returns_ancestor_for_ancestor_descendant_pair() {
        // Arrange
        let ancestor_vertex = 1;
        let descendant_vertex = 3;
        let expected_lca = Some(1);
        let hld = create_chain_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(ancestor_vertex, descendant_vertex);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_returns_parent_for_sibling_vertices() {
        // Arrange
        let left_sibling = 3;
        let right_sibling = 4;
        let expected_lca = Some(1);
        let hld = create_balanced_tree_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(left_sibling, right_sibling);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_returns_correct_vertex_for_different_heavy_paths() {
        // Arrange
        let left_vertex = 6;
        let right_vertex = 5;
        let expected_lca = Some(0);
        let hld = create_multi_segment_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(left_vertex, right_vertex);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_is_invariant_to_argument_order() {
        // Arrange
        let left_vertex = 4;
        let right_vertex = 6;
        let expected_lca = Some(0);
        let hld = create_balanced_tree_hld();

        // Act
        let forward_result = hld.lowest_common_ancestor(left_vertex, right_vertex);
        let reverse_result = hld.lowest_common_ancestor(right_vertex, left_vertex);

        // Assert
        assert_eq!(forward_result, expected_lca);
        assert_eq!(forward_result, reverse_result);
    }

    #[test]
    fn lowest_common_ancestor_returns_none_for_out_of_range_vertex() {
        // Arrange
        let invalid_vertex = 7;
        let valid_vertex = 2;
        let expected_lca = None;
        let hld = create_balanced_tree_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(invalid_vertex, valid_vertex);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_returns_none_for_unattached_vertex() {
        // Arrange
        let attached_vertex = 1;
        let detached_vertex = 3;
        let expected_lca = None;
        let hld = create_disconnected_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(attached_vertex, detached_vertex);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_works_with_non_zero_root_tree() {
        // Arrange
        let left_vertex = 0;
        let right_vertex = 4;
        let expected_lca = Some(2);
        let hld = create_non_zero_root_hld();

        // Act
        let actual_lca = hld.lowest_common_ancestor(left_vertex, right_vertex);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn lowest_common_ancestor_matches_naive_parent_tracking_for_all_vertex_pairs() {
        // Arrange
        let vertex_size = 8;
        let edges = asymmetric_tree_edges();
        let root = 0;
        let (parent, depth) = calculate_naive_parent_and_depth(vertex_size, &edges, root);
        let mut expected_lcas = Vec::with_capacity(vertex_size * vertex_size);
        for left_vertex in 0..vertex_size {
            for right_vertex in 0..vertex_size {
                expected_lcas.push(calculate_naive_lowest_common_ancestor(
                    left_vertex,
                    right_vertex,
                    &parent,
                    &depth,
                ));
            }
        }
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let mut actual_lcas = Vec::with_capacity(vertex_size * vertex_size);
        for left_vertex in 0..vertex_size {
            for right_vertex in 0..vertex_size {
                actual_lcas.push(hld.lowest_common_ancestor(left_vertex, right_vertex));
            }
        }

        // Assert
        assert_eq!(actual_lcas, expected_lcas);
    }

    #[test]
    fn path_segments_for_identical_vertices_returns_single_length_one_segment() {
        // Arrange
        let left_vertex = 2;
        let right_vertex = 2;
        let expected_segment_count = 1;
        let expected_start = 2;
        let expected_end = 3;
        let expected_is_reversed = true;
        let hld = create_chain_hld();

        // Act
        let segments = hld
            .path_segments(left_vertex, right_vertex)
            .expect("valid path must exist");
        let actual_segment_count = segments.len();
        let actual_segment = segments[0];

        // Assert
        assert_eq!(actual_segment_count, expected_segment_count);
        assert_eq!(actual_segment.start, expected_start);
        assert_eq!(actual_segment.end, expected_end);
        assert_eq!(actual_segment.is_reversed, expected_is_reversed);
    }

    #[test]
    fn path_segments_for_ancestor_to_descendant_on_same_heavy_path_is_forward() {
        // Arrange
        let ancestor_vertex = 1;
        let descendant_vertex = 3;
        let expected_segment_count = 1;
        let expected_start = 1;
        let expected_end = 4;
        let expected_is_reversed = false;
        let hld = create_chain_hld();

        // Act
        let segments = hld
            .path_segments(ancestor_vertex, descendant_vertex)
            .expect("valid path must exist");
        let actual_segment_count = segments.len();
        let actual_segment = segments[0];

        // Assert
        assert_eq!(actual_segment_count, expected_segment_count);
        assert_eq!(actual_segment.start, expected_start);
        assert_eq!(actual_segment.end, expected_end);
        assert_eq!(actual_segment.is_reversed, expected_is_reversed);
    }

    #[test]
    fn path_segments_for_descendant_to_ancestor_on_same_heavy_path_is_reversed() {
        // Arrange
        let descendant_vertex = 3;
        let ancestor_vertex = 1;
        let expected_segment_count = 1;
        let expected_start = 1;
        let expected_end = 4;
        let expected_is_reversed = true;
        let hld = create_chain_hld();

        // Act
        let segments = hld
            .path_segments(descendant_vertex, ancestor_vertex)
            .expect("valid path must exist");
        let actual_segment_count = segments.len();
        let actual_segment = segments[0];

        // Assert
        assert_eq!(actual_segment_count, expected_segment_count);
        assert_eq!(actual_segment.start, expected_start);
        assert_eq!(actual_segment.end, expected_end);
        assert_eq!(actual_segment.is_reversed, expected_is_reversed);
    }

    #[test]
    fn path_segments_between_branch_vertices_preserve_u_to_v_order() {
        // Arrange
        let start_vertex = 6;
        let end_vertex = 5;
        let expected_vertex_path = vec![6, 1, 0, 2, 5];
        let hld = create_multi_segment_hld();

        // Act
        let segments = hld
            .path_segments(start_vertex, end_vertex)
            .expect("valid path must exist");
        let actual_vertex_path = build_vertices_from_path_segments(&hld, &segments);

        // Assert
        assert_eq!(actual_vertex_path, expected_vertex_path);
    }

    #[test]
    fn path_segments_for_multi_switch_path_returns_three_or_more_segments() {
        // Arrange
        let start_vertex = 6;
        let end_vertex = 5;
        let expected_minimum_segment_count = 3;
        let hld = create_multi_segment_hld();

        // Act
        let segments = hld
            .path_segments(start_vertex, end_vertex)
            .expect("valid path must exist");
        let actual_segment_count = segments.len();

        // Assert
        assert!(expected_minimum_segment_count <= actual_segment_count);
    }

    #[test]
    fn path_segments_always_return_segments_with_start_less_than_end() {
        // Arrange
        let start_vertex = 6;
        let end_vertex = 5;
        let expected_all_valid = true;
        let hld = create_multi_segment_hld();

        // Act
        let segments = hld
            .path_segments(start_vertex, end_vertex)
            .expect("valid path must exist");
        let actual_all_valid = segments.iter().all(|segment| segment.start < segment.end);

        // Assert
        assert_eq!(actual_all_valid, expected_all_valid);
    }

    #[test]
    fn path_segments_return_none_for_out_of_range_vertex() {
        // Arrange
        let invalid_vertex = 7;
        let valid_vertex = 1;
        let expected_segments = None;
        let hld = create_multi_segment_hld();

        // Act
        let actual_segments = hld.path_segments(invalid_vertex, valid_vertex);

        // Assert
        assert_eq!(actual_segments, expected_segments);
    }

    #[test]
    fn path_segments_return_none_for_unattached_vertex() {
        // Arrange
        let attached_vertex = 1;
        let detached_vertex = 3;
        let expected_segments = None;
        let hld = create_disconnected_hld();

        // Act
        let actual_segments = hld.path_segments(attached_vertex, detached_vertex);

        // Assert
        assert_eq!(actual_segments, expected_segments);
    }

    #[test]
    fn path_segments_vertex_sequence_matches_naive_tree_path() {
        // Arrange
        let vertex_size = 7;
        let edges = balanced_tree_edges();
        let root = 0;
        let start_vertex = 4;
        let end_vertex = 6;
        let expected_vertex_path =
            calculate_naive_tree_path(vertex_size, &edges, start_vertex, end_vertex);
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let segments = hld
            .path_segments(start_vertex, end_vertex)
            .expect("valid path must exist");
        let actual_vertex_path = build_vertices_from_path_segments(&hld, &segments);

        // Assert
        assert_eq!(actual_vertex_path, expected_vertex_path);
    }

    #[test]
    fn path_segments_match_naive_path_for_all_ordered_vertex_pairs() {
        // Arrange
        let vertex_size = 8;
        let edges = asymmetric_tree_edges();
        let root = 0;
        let mut expected_paths = Vec::with_capacity(vertex_size * vertex_size);
        for start_vertex in 0..vertex_size {
            for goal_vertex in 0..vertex_size {
                expected_paths.push(calculate_naive_tree_path(
                    vertex_size,
                    &edges,
                    start_vertex,
                    goal_vertex,
                ));
            }
        }
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let mut actual_paths = Vec::with_capacity(vertex_size * vertex_size);
        for start_vertex in 0..vertex_size {
            for goal_vertex in 0..vertex_size {
                let segments = hld
                    .path_segments(start_vertex, goal_vertex)
                    .expect("all vertices belong to the connected tree");
                actual_paths.push(build_vertices_from_path_segments(&hld, &segments));
            }
        }

        // Assert
        assert_eq!(actual_paths, expected_paths);
    }

    #[test]
    fn subtree_range_for_leaf_vertex_has_length_one() {
        // Arrange
        let target_vertex = 4;
        let expected_range_length = 1;
        let hld = create_balanced_tree_hld();

        // Act
        let (start_position, end_position) = hld
            .subtree_range(target_vertex)
            .expect("attached vertex must have subtree range");
        let actual_range_length = end_position - start_position;

        // Assert
        assert_eq!(actual_range_length, expected_range_length);
    }

    #[test]
    fn subtree_range_for_internal_vertex_matches_subtree_vertex_count() {
        // Arrange
        let target_vertex = 1;
        let expected_range_length = 3;
        let hld = create_balanced_tree_hld();

        // Act
        let (start_position, end_position) = hld
            .subtree_range(target_vertex)
            .expect("attached vertex must have subtree range");
        let actual_range_length = end_position - start_position;

        // Assert
        assert_eq!(actual_range_length, expected_range_length);
    }

    #[test]
    fn subtree_range_for_root_matches_attached_component_size() {
        // Arrange
        let root_vertex = 0;
        let expected_range_length = 3;
        let hld = create_disconnected_hld();

        // Act
        let (start_position, end_position) = hld
            .subtree_range(root_vertex)
            .expect("attached root must have subtree range");
        let actual_range_length = end_position - start_position;

        // Assert
        assert_eq!(actual_range_length, expected_range_length);
    }

    #[test]
    fn subtree_range_returns_none_for_out_of_range_vertex() {
        // Arrange
        let invalid_vertex = 7;
        let expected_range = None;
        let hld = create_balanced_tree_hld();

        // Act
        let actual_range = hld.subtree_range(invalid_vertex);

        // Assert
        assert_eq!(actual_range, expected_range);
    }

    #[test]
    fn subtree_range_returns_none_for_unattached_vertex() {
        // Arrange
        let detached_vertex = 3;
        let expected_range = None;
        let hld = create_disconnected_hld();

        // Act
        let actual_range = hld.subtree_range(detached_vertex);

        // Assert
        assert_eq!(actual_range, expected_range);
    }

    #[test]
    fn subtree_range_of_descendant_is_contained_within_ancestor_range() {
        // Arrange
        let ancestor_vertex = 1;
        let descendant_vertex = 3;
        let expected_is_contained = true;
        let hld = create_balanced_tree_hld();

        // Act
        let (ancestor_start, ancestor_end) = hld
            .subtree_range(ancestor_vertex)
            .expect("ancestor must be attached");
        let (descendant_start, descendant_end) = hld
            .subtree_range(descendant_vertex)
            .expect("descendant must be attached");
        let actual_is_contained =
            ancestor_start <= descendant_start && descendant_end <= ancestor_end;

        // Assert
        assert_eq!(actual_is_contained, expected_is_contained);
    }

    #[test]
    fn subtree_ranges_of_distinct_subtrees_do_not_overlap() {
        // Arrange
        let left_subtree_vertex = 3;
        let right_subtree_vertex = 4;
        let expected_non_overlap = true;
        let hld = create_balanced_tree_hld();

        // Act
        let (left_start, left_end) = hld
            .subtree_range(left_subtree_vertex)
            .expect("left subtree vertex must be attached");
        let (right_start, right_end) = hld
            .subtree_range(right_subtree_vertex)
            .expect("right subtree vertex must be attached");
        let actual_non_overlap = left_end <= right_start || right_end <= left_start;

        // Assert
        assert_eq!(actual_non_overlap, expected_non_overlap);
    }

    #[test]
    fn subtree_ranges_match_naive_descendant_sets_with_non_zero_root() {
        // Arrange
        let vertex_size = 8;
        let edges = asymmetric_tree_edges();
        let root = 3;
        let (parent, _) = calculate_naive_parent_and_depth(vertex_size, &edges, root);
        let expected_subtrees = (0..vertex_size)
            .map(|vertex| calculate_naive_subtree_vertices(&parent, vertex))
            .collect::<Vec<_>>();
        let hld = HeavyLightDecomposition::new(vertex_size, &edges, root);

        // Act
        let actual_subtrees = (0..vertex_size)
            .map(|vertex| vertices_in_subtree_range(&hld, vertex))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_subtrees, expected_subtrees);
    }

    #[test]
    fn vertex_at_of_position_of_vertex_returns_original_vertex() {
        // Arrange
        let expected_all_match = true;
        let hld = create_balanced_tree_hld();

        // Act
        let actual_all_match =
            (0..hld.len())
                .filter(|&vertex| hld.contains(vertex))
                .all(|vertex| {
                    let position = hld
                        .position(vertex)
                        .expect("attached vertex must have position");
                    let restored_vertex = hld.vertex_at(position);
                    restored_vertex == Some(vertex)
                });

        // Assert
        assert_eq!(actual_all_match, expected_all_match);
    }

    #[test]
    fn vertex_at_returns_none_only_for_upper_out_of_range_boundary_position() {
        // Arrange
        let boundary_position = 3;
        let lower_position = boundary_position - 1;
        let upper_position = boundary_position + 1;
        let expected_lower_is_some = true;
        let expected_boundary_is_some = true;
        let expected_upper = None;
        let hld = create_chain_hld();

        // Act
        let lower_result = hld.vertex_at(lower_position);
        let boundary_result = hld.vertex_at(boundary_position);
        let upper_result = hld.vertex_at(upper_position);
        let lower_is_some = lower_result.is_some();
        let boundary_is_some = boundary_result.is_some();

        // Assert
        assert_eq!(lower_is_some, expected_lower_is_some);
        assert_eq!(boundary_is_some, expected_boundary_is_some);
        assert_eq!(upper_result, expected_upper);
    }

    #[test]
    fn vertex_at_returns_default_zero_for_unassigned_position_in_disconnected_input() {
        // Arrange
        let unassigned_position = 3;
        let expected_vertex = Some(0);
        let hld = create_disconnected_hld();

        // Act
        let actual_vertex = hld.vertex_at(unassigned_position);

        // Assert
        assert_eq!(actual_vertex, expected_vertex);
    }

    #[test]
    fn vertex_at_of_position_restores_every_vertex_in_non_zero_root_forest_component() {
        // Arrange
        let attached_vertices = [2, 3, 4, 5];
        let expected_all_match = true;
        let hld = create_disconnected_non_zero_root_hld();

        // Act
        let actual_all_match = attached_vertices.into_iter().all(|vertex| {
            let position = hld
                .position(vertex)
                .expect("root component vertex must have a position");
            hld.vertex_at(position) == Some(vertex)
        });

        // Assert
        assert_eq!(actual_all_match, expected_all_match);
    }
}
