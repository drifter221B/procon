use std::collections::VecDeque;

const INVALID: usize = usize::MAX;

#[derive(Debug, Clone)]
pub struct Tree {
    vertex_size: usize,
    root: usize,
    msb: usize,
    parents: Vec<(usize, i64)>,
    parent_positions: Vec<usize>,
    children: Vec<Vec<(usize, i64)>>,
    dist: Vec<i64>,
    depth: Vec<usize>,
    doubling: Vec<Vec<usize>>,
}

impl Tree {
    /// Builds a rooted tree from a weighted, undirected adjacency list.
    ///
    /// Each undirected edge must occur exactly once in each direction with the
    /// same weight. The graph must be connected and acyclic.
    ///
    /// # Panics
    ///
    /// Panics if the vertex count or root is invalid, or if `graph` is not a
    /// simple, connected, undirected tree with matching weights in both
    /// directions.
    pub fn new(vertex_size: usize, graph: &[Vec<(usize, i64)>], root: usize) -> Self {
        assert!(vertex_size > 0, "a tree must contain at least one vertex");
        assert_eq!(
            vertex_size,
            graph.len(),
            "vertex_size must equal graph.len()"
        );
        assert!(root < vertex_size, "root must be a valid vertex");

        for edges in graph {
            for &(next_vertex, _) in edges {
                assert!(
                    next_vertex < vertex_size,
                    "an edge endpoint is out of range"
                );
            }
        }

        let mut tree = Self {
            vertex_size,
            root,
            msb: 0,
            parents: vec![(INVALID, 0); vertex_size],
            parent_positions: vec![INVALID; vertex_size],
            children: vec![Vec::new(); vertex_size],
            dist: vec![0; vertex_size],
            depth: vec![0; vertex_size],
            doubling: Vec::new(),
        };

        tree.build_tree(graph, root);
        tree.build_doubling_table();

        tree
    }

    pub fn new_unweighted(vertex_size: usize, graph: &[Vec<usize>], root: usize) -> Self {
        let weighted_graph = graph
            .iter()
            .map(|edges| edges.iter().map(|&next_vertex| (next_vertex, 1)).collect())
            .collect::<Vec<Vec<_>>>();

        Self::new(vertex_size, &weighted_graph, root)
    }

    fn build_tree(&mut self, graph: &[Vec<(usize, i64)>], root: usize) {
        self.root = root;

        let mut que = VecDeque::new();
        let mut visited = vec![false; self.vertex_size];
        let mut visited_count = 1;
        que.push_back(root);
        visited[root] = true;
        self.dist[root] = 0;
        self.depth[root] = 0;

        while let Some(vertex) = que.pop_front() {
            let current_dist = self.dist[vertex];
            let expected_parent = self.parents[vertex].0;
            let mut parent_position = None;

            for (edge_index, &(next_vertex, weight)) in graph[vertex].iter().enumerate() {
                if next_vertex == expected_parent {
                    assert!(
                        parent_position.is_none(),
                        "the reverse edge to a parent must occur exactly once"
                    );
                    assert_eq!(
                        weight, self.parents[vertex].1,
                        "both directions of an edge must have the same weight"
                    );
                    parent_position = Some(edge_index);
                    continue;
                }

                assert!(
                    !visited[next_vertex],
                    "the adjacency list must represent an acyclic tree"
                );

                visited[next_vertex] = true;
                visited_count += 1;
                self.parents[next_vertex] = (vertex, weight);
                self.children[vertex].push((next_vertex, weight));
                self.dist[next_vertex] = current_dist + weight;
                self.depth[next_vertex] = self.depth[vertex] + 1;

                que.push_back(next_vertex);
            }

            if vertex != root {
                self.parent_positions[vertex] = parent_position
                    .expect("each non-root vertex must contain the reverse edge to its parent");
            }
        }

        assert_eq!(
            visited_count, self.vertex_size,
            "the adjacency list must represent a connected tree"
        );
    }

    fn bit_length(n: usize) -> usize {
        if n == 0 {
            0
        } else {
            usize::BITS as usize - n.leading_zeros() as usize
        }
    }

    fn build_doubling_table(&mut self) {
        self.msb = Self::bit_length(self.vertex_size - 1);
        self.doubling = vec![vec![INVALID; self.vertex_size]; self.msb + 1];

        for vertex in 0..self.vertex_size {
            self.doubling[0][vertex] = self.parents[vertex].0;
        }

        for k in 0..self.msb {
            for vertex in 0..self.vertex_size {
                let parent = self.doubling[k][vertex];
                if parent == INVALID {
                    continue;
                }

                self.doubling[k + 1][vertex] = self.doubling[k][parent];
            }
        }
    }

    #[inline]
    pub fn contains(&self, vertex: usize) -> bool {
        vertex < self.vertex_size
    }

    #[inline]
    pub fn is_root(&self, vertex: usize) -> bool {
        vertex == self.root
    }

    #[inline]
    pub fn is_leaf(&self, vertex: usize) -> bool {
        self.children[vertex].is_empty()
    }

    pub fn degree(&self, vertex: usize) -> usize {
        if self.is_root(vertex) {
            self.children[vertex].len()
        } else {
            self.children[vertex].len() + 1
        }
    }

    pub fn parent_vertex(&self, vertex: usize) -> Option<usize> {
        let (parent, _) = self.parents[vertex];

        if parent == INVALID {
            None
        } else {
            Some(parent)
        }
    }

    pub fn children_vertices(&self, vertex: usize) -> Vec<usize> {
        self.children[vertex]
            .iter()
            .map(|(child, _)| *child)
            .collect()
    }

    pub fn children_size(&self, vertex: usize) -> usize {
        self.children[vertex].len()
    }

    pub fn depth_from_root(&self, vertex: usize) -> usize {
        self.depth[vertex]
    }

    pub fn distance_from_root(&self, vertex: usize) -> i64 {
        self.dist[vertex]
    }

    pub fn minimum_distance_with_lca(
        &self,
        vertex1: usize,
        vertex2: usize,
        lca_vertex: Option<usize>,
    ) -> i64 {
        let lca_vertex =
            lca_vertex.unwrap_or_else(|| self.lowest_common_ancestor(vertex1, vertex2));
        self.dist[vertex1] + self.dist[vertex2] - 2 * self.dist[lca_vertex]
    }

    pub fn minimum_distance(&self, vertex1: usize, vertex2: usize) -> i64 {
        self.minimum_distance_with_lca(vertex1, vertex2, None)
    }

    pub fn subtree_vertex_count(&self, vertex: usize) -> usize {
        let mut result = 1;
        let mut que = VecDeque::new();
        que.push_back(vertex);

        while let Some(vertex) = que.pop_front() {
            for &(next_vertex, _) in &self.children[vertex] {
                result += 1;
                que.push_back(next_vertex);
            }
        }

        result
    }

    pub fn descendant_count(&self, vertex: usize) -> usize {
        self.subtree_vertex_count(vertex) - 1
    }

    pub fn subtree_edge_weight_sum(&self, vertex: usize) -> i64 {
        let mut result = 0;
        let mut que = VecDeque::new();
        que.push_back(vertex);

        while let Some(vertex) = que.pop_front() {
            for &(next_vertex, weight) in &self.children[vertex] {
                result += weight;
                que.push_back(next_vertex);
            }
        }

        result
    }

    pub fn has_same_parent(&self, x: usize, y: usize) -> bool {
        if self.is_root(x) || self.is_root(y) {
            return false;
        }

        self.parents[x].0 == self.parents[y].0
    }

    pub fn shallower_vertex(&self, vertex1: usize, vertex2: usize) -> usize {
        if self.depth[vertex1] < self.depth[vertex2] {
            vertex1
        } else if self.depth[vertex2] < self.depth[vertex1] {
            vertex2
        } else {
            vertex1.min(vertex2)
        }
    }

    pub fn deeper_vertex(&self, vertex1: usize, vertex2: usize) -> usize {
        if self.depth[vertex1] < self.depth[vertex2] {
            vertex2
        } else if self.depth[vertex2] < self.depth[vertex1] {
            vertex1
        } else {
            vertex1.max(vertex2)
        }
    }

    pub fn is_descendant_and_ancestor(
        &self,
        descendant_vertex: usize,
        ancestor_vertex: usize,
    ) -> bool {
        if self.depth[descendant_vertex] < self.depth[ancestor_vertex] {
            return false;
        }

        let distance = self.depth[descendant_vertex] - self.depth[ancestor_vertex];
        let vertex = self.level_ancestor(descendant_vertex, distance);

        vertex == ancestor_vertex
    }

    pub fn level_ancestor(&self, mut vertex: usize, mut height: usize) -> usize {
        assert!(height <= self.depth[vertex]);

        for k in (0..=self.msb).rev() {
            if height >> k & 1 == 1 {
                vertex = self.doubling[k][vertex];
                height ^= 1 << k;
            }
        }

        vertex
    }

    pub fn lowest_common_ancestor(
        &self,
        mut descendant_vertex: usize,
        mut ancestor_vertex: usize,
    ) -> usize {
        if self.depth[descendant_vertex] < self.depth[ancestor_vertex] {
            std::mem::swap(&mut descendant_vertex, &mut ancestor_vertex);
        }

        descendant_vertex = self.level_ancestor(
            descendant_vertex,
            self.depth[descendant_vertex] - self.depth[ancestor_vertex],
        );

        if descendant_vertex == ancestor_vertex {
            return descendant_vertex;
        }

        for height in (0..=self.msb).rev() {
            if self.doubling[height][descendant_vertex] != self.doubling[height][ancestor_vertex] {
                descendant_vertex = self.doubling[height][descendant_vertex];
                ancestor_vertex = self.doubling[height][ancestor_vertex];
            }
        }

        self.doubling[0][descendant_vertex]
    }

    pub fn pre_order(&self) -> Vec<usize> {
        let mut stack = vec![self.root];
        let mut node_list = Vec::new();

        while let Some(vertex) = stack.pop() {
            node_list.push(vertex);

            for &(next_vertex, _) in self.children[vertex].iter().rev() {
                stack.push(next_vertex);
            }
        }

        node_list
    }

    pub fn post_order(&self) -> Vec<usize> {
        let mut stack = vec![(self.root, false)];
        let mut node_list = Vec::new();

        while let Some((vertex, is_visited)) = stack.pop() {
            if is_visited {
                node_list.push(vertex);
                continue;
            }

            stack.push((vertex, true));

            for &(next_vertex, _) in self.children[vertex].iter().rev() {
                stack.push((next_vertex, false));
            }
        }

        node_list
    }

    pub fn level_order(&self) -> Vec<usize> {
        let mut que = VecDeque::new();
        que.push_back(self.root);
        let mut node_list = Vec::new();

        while let Some(vertex) = que.pop_front() {
            node_list.push(vertex);

            for &(next_vertex, _) in &self.children[vertex] {
                que.push_back(next_vertex);
            }
        }

        node_list
    }

    pub fn path(&self, start_vertex: usize, end_vertex: usize) -> Vec<usize> {
        let lca_vertex = self.lowest_common_ancestor(start_vertex, end_vertex);
        let mut path_list_f = Vec::new();
        let mut path_list_b = Vec::new();

        let mut current_vertex_f = start_vertex;
        while current_vertex_f != lca_vertex {
            path_list_f.push(current_vertex_f);
            current_vertex_f = self.parents[current_vertex_f].0;
        }

        let mut current_vertex_b = end_vertex;
        while current_vertex_b != lca_vertex {
            path_list_b.push(current_vertex_b);
            current_vertex_b = self.parents[current_vertex_b].0;
        }

        path_list_b.reverse();
        path_list_f.push(lca_vertex);
        path_list_f.extend(path_list_b);

        path_list_f
    }

    pub fn diameter_pair(&self) -> (i64, (usize, usize)) {
        let mut best_down = vec![(0, self.root); self.vertex_size];
        let mut diameter = (0, (self.root, self.root));

        for vertex in self.post_order() {
            let mut longest = (0, vertex);
            let mut second_longest = (0, vertex);

            for &(child_vertex, weight) in &self.children[vertex] {
                let candidate = (
                    weight + best_down[child_vertex].0,
                    best_down[child_vertex].1,
                );

                if longest.0 < candidate.0 {
                    second_longest = longest;
                    longest = candidate;
                } else if second_longest.0 < candidate.0 {
                    second_longest = candidate;
                }
            }

            best_down[vertex] = longest;

            let through_vertex = longest.0 + second_longest.0;
            if diameter.0 < through_vertex {
                diameter = (through_vertex, (longest.1, second_longest.1));
            }
        }

        diameter
    }

    fn dp_bottom_up_internal<T, F, M, C>(
        &self,
        e: T,
        func: &F,
        mapping: &M,
        composition: &C,
    ) -> Vec<T>
    where
        T: Clone,
        F: Fn(T, T) -> T,
        M: Fn(T, usize, usize, i64) -> T,
        C: Fn(T, usize) -> T,
    {
        let mut dp = vec![e.clone(); self.vertex_size];
        let mut node_list = self.level_order();
        node_list.reverse();

        for vertex in node_list {
            for &(next_vertex, weight) in &self.children[vertex] {
                let mapped = mapping(dp[next_vertex].clone(), vertex, next_vertex, weight);
                dp[vertex] = func(dp[vertex].clone(), mapped);
            }

            dp[vertex] = composition(dp[vertex].clone(), vertex);
        }

        dp
    }

    pub fn dp_top_down<T, F, M, C>(&self, e: T, func: F, mapping: M, composition: C) -> Vec<T>
    where
        T: Clone,
        F: Fn(T, T) -> T,
        M: Fn(T, usize, usize, i64) -> T,
        C: Fn(T, usize) -> T,
    {
        let mut dp = vec![e.clone(); self.vertex_size];
        dp[self.root] = composition(e.clone(), self.root);

        for vertex in self.level_order() {
            for &(child_vertex, weight) in &self.children[vertex] {
                let mapped = mapping(dp[vertex].clone(), child_vertex, vertex, weight);
                dp[child_vertex] = composition(func(e.clone(), mapped), child_vertex);
            }
        }

        dp
    }

    pub fn dp_bottom_up<T, F, M, C>(&self, e: T, func: F, mapping: M, composition: C) -> Vec<T>
    where
        T: Clone,
        F: Fn(T, T) -> T,
        M: Fn(T, usize, usize, i64) -> T,
        C: Fn(T, usize) -> T,
    {
        self.dp_bottom_up_internal(e, &func, &mapping, &composition)
    }

    pub fn rerooting<T, F, M, C>(&self, e: T, func: F, mapping: M, composition: C) -> Vec<T>
    where
        T: Clone,
        F: Fn(T, T) -> T,
        M: Fn(T, usize, usize, i64) -> T,
        C: Fn(T, usize) -> T,
    {
        let dp_down = self.dp_bottom_up_internal(e.clone(), &func, &mapping, &composition);
        let mut dp_up = vec![e.clone(); self.vertex_size];
        let mut dp = vec![e.clone(); self.vertex_size];

        for vertex in self.level_order() {
            let children = &self.children[vertex];
            let degree = children.len() + usize::from(vertex != self.root);
            let parent_position = self.parent_positions[vertex];
            let mut contributions = Vec::with_capacity(degree);
            let mut child_index = 0;

            for edge_index in 0..degree {
                if edge_index == parent_position {
                    let (parent, weight) = self.parents[vertex];
                    contributions.push(mapping(dp_up[vertex].clone(), vertex, parent, weight));
                } else {
                    let (child_vertex, weight) = children[child_index];
                    contributions.push(mapping(
                        dp_down[child_vertex].clone(),
                        vertex,
                        child_vertex,
                        weight,
                    ));
                    child_index += 1;
                }
            }

            let mut prefix = vec![e.clone(); degree + 1];
            for edge_index in 0..degree {
                prefix[edge_index + 1] = func(
                    prefix[edge_index].clone(),
                    contributions[edge_index].clone(),
                );
            }

            let mut suffix = vec![e.clone(); degree + 1];
            for edge_index in (0..degree).rev() {
                suffix[edge_index] = func(
                    contributions[edge_index].clone(),
                    suffix[edge_index + 1].clone(),
                );
            }

            dp[vertex] = composition(prefix[degree].clone(), vertex);

            for (index, &(child_vertex, _)) in children.iter().enumerate() {
                let edge_index = if vertex == self.root || index < parent_position {
                    index
                } else {
                    index + 1
                };
                let without_child =
                    func(prefix[edge_index].clone(), suffix[edge_index + 1].clone());
                dp_up[child_vertex] = composition(without_child, vertex);
            }
        }

        dp
    }

    pub fn euler_tour(&self) -> (Vec<isize>, Vec<isize>, Vec<usize>, Vec<usize>) {
        let mut ins = vec![0; self.vertex_size];
        let mut outs = vec![0; self.vertex_size];

        let mut stack = Vec::new();
        stack.push((self.root, false));

        let mut parents = vec![-1; self.vertex_size];
        let mut vertex_orders = vec![-1; 2 * self.vertex_size];
        let mut edge_orders = vec![-1; 2 * self.vertex_size];
        let mut current_time: isize = -1;

        while let Some((vertex, is_visited)) = stack.pop() {
            current_time += 1;

            if !is_visited {
                ins[vertex] = current_time as usize;
                stack.push((vertex, true));
                vertex_orders[current_time as usize] = vertex as isize;
                edge_orders[current_time as usize] = vertex as isize;

                for &(next_vertex, _) in self.children[vertex].iter().rev() {
                    parents[next_vertex] = vertex as isize;
                    stack.push((next_vertex, false));
                }
            } else {
                outs[vertex] = current_time as usize;
                vertex_orders[current_time as usize] = parents[vertex];
                edge_orders[current_time as usize] = -(vertex as isize);
            }
        }

        vertex_orders.pop();

        (vertex_orders, edge_orders, ins, outs)
    }

    pub fn default_func(x: i64, y: i64) -> i64 {
        x.max(y)
    }

    pub fn default_mapping(f: i64, _x: usize, _y: usize, c: i64) -> i64 {
        f + c
    }

    pub fn default_composition(f: i64, _x: usize) -> i64 {
        f
    }
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::Tree;

    fn sample_tree() -> Tree {
        // 0-1 (1), 1-2 (2)
        let graph = vec![vec![(1, 1)], vec![(0, 1), (2, 2)], vec![(1, 2)]];
        Tree::new(3, &graph, 0)
    }

    fn branching_tree() -> Tree {
        let graph = vec![
            vec![(1, 2), (2, 3)],
            vec![(0, 2), (3, 4), (4, 1)],
            vec![(0, 3)],
            vec![(1, 4)],
            vec![(1, 1)],
        ];
        Tree::new(5, &graph, 0)
    }

    fn ordered_func(mut lhs: String, rhs: String) -> String {
        lhs.push_str(&rhs);
        lhs
    }

    fn ordered_mapping(value: String, receiver: usize, sender: usize, weight: i64) -> String {
        format!("<{receiver}<-{sender}@{weight}:{value}>")
    }

    fn ordered_composition(value: String, vertex: usize) -> String {
        format!("[{vertex}:{value}]")
    }

    #[test]
    fn new_unweighted_assigns_unit_edge_weights() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 0;
        let input_value_6 = 2;
        let input_value_7 = 0;
        let graph = vec![vec![1], vec![0, 2], vec![1]];
        let tree = Tree::new_unweighted(graph.len(), &graph, input_value_1);
        let expected_check_1 = (1, 0, 1, 2);
        let expected_check_2 = vec![2, 1, 2];

        // Act
        let actual_distances = (
            tree.distance_from_root(input_value_2),
            tree.distance_from_root(input_value_3),
            tree.distance_from_root(input_value_4),
            tree.minimum_distance(input_value_5, input_value_6),
        );
        let actual_rerooting = tree.rerooting(
            input_value_7,
            Tree::default_func,
            Tree::default_mapping,
            Tree::default_composition,
        );

        // Assert
        assert_eq!(actual_distances, expected_check_1);
        assert_eq!(actual_rerooting, expected_check_2);
    }

    #[test]
    fn new_rejects_empty_graph() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 0;
        let graph: Vec<Vec<(usize, i64)>> = vec![];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(input_value_1, &graph, input_value_2));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_vertex_size_different_from_graph_length() {
        // Arrange
        let input_value_1 = 2;
        let input_value_2 = 0;
        let graph: Vec<Vec<(usize, i64)>> = vec![vec![]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(input_value_1, &graph, input_value_2));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_root_out_of_range() {
        // Arrange
        let input_value_1 = 1;
        let graph: Vec<Vec<(usize, i64)>> = vec![vec![]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_adjacent_vertex_out_of_range() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1)]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_self_loop() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(0, 1)]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_missing_reverse_edge() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1)], vec![]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_reverse_edge_with_different_weight() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1)], vec![(0, 2)]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_cycle() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![
            vec![(1, 1), (2, 1)],
            vec![(0, 1), (2, 1)],
            vec![(0, 1), (1, 1)],
        ];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_disconnected_graph() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1)], vec![(0, 1)], vec![(3, 1)], vec![(2, 1)]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_parallel_edges() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1), (1, 1)], vec![(0, 1), (0, 1)]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn new_rejects_duplicate_reverse_edge_to_parent() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1)], vec![(0, 1), (0, 1)]];

        let expected_boolean_1 = true;

        // Act
        let actual = panic::catch_unwind(|| Tree::new(graph.len(), &graph, input_value_1));

        let actual_check_1 = actual.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn contains_is_root_and_is_leaf_handle_boundaries_and_representatives() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 4;
        let input_value_3 = 5;
        let input_value_4 = 0;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 1;
        let tree = branching_tree();
        let expected_check_1 = (true, true, false, true, false, true, false);

        // Act
        let actual_values = (
            tree.contains(input_value_1),
            tree.contains(input_value_2),
            tree.contains(input_value_3),
            tree.is_root(input_value_4),
            tree.is_root(input_value_5),
            tree.is_leaf(input_value_6),
            tree.is_leaf(input_value_7),
        );

        // Assert
        assert_eq!(actual_values, expected_check_1);
    }

    #[test]
    fn lowest_common_ancestor_returns_expected_vertex() {
        // Arrange
        let tree = sample_tree();
        let lca_vertices = (1, 2);
        let expected_lca = 1;

        // Act
        let actual_lca = tree.lowest_common_ancestor(lca_vertices.0, lca_vertices.1);

        // Assert
        assert_eq!(actual_lca, expected_lca);
    }

    #[test]
    fn degree_of_chain_root_returns_one() {
        // Arrange
        let tree = sample_tree();
        let vertex = 0;
        let expected_degree = 1;

        // Act
        let actual_degree = tree.degree(vertex);
        // Assert
        assert_eq!(actual_degree, expected_degree);
    }

    #[test]
    fn has_same_parent_ignores_different_edge_weights() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 2;
        let graph = vec![vec![(1, 1), (2, 2)], vec![(0, 1)], vec![(0, 2)]];
        let tree = Tree::new(graph.len(), &graph, input_value_1);

        let expected_boolean_1 = true;

        // Act
        let actual = tree.has_same_parent(input_value_2, input_value_3);

        // Assert
        assert_eq!(actual, expected_boolean_1);
    }

    #[test]
    fn shallower_and_deeper_vertex_compare_different_depths() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 3;
        let input_value_3 = 0;
        let input_value_4 = 3;
        let tree = branching_tree();
        let expected_check_1 = (0, 3);

        // Act
        let actual = (
            tree.shallower_vertex(input_value_1, input_value_2),
            tree.deeper_vertex(input_value_3, input_value_4),
        );

        // Assert
        assert_eq!(actual, expected_check_1);
    }

    #[test]
    fn shallower_and_deeper_vertex_keep_legacy_tie_breaking() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 3;
        let input_value_3 = 4;
        let input_value_4 = 3;
        let tree = branching_tree();
        let expected_check_1 = (3, 4);

        // Act
        let actual = (
            tree.shallower_vertex(input_value_1, input_value_2),
            tree.deeper_vertex(input_value_3, input_value_4),
        );

        // Assert
        assert_eq!(actual, expected_check_1);
    }

    #[test]
    fn level_ancestor_returns_self_at_zero_and_root_at_exact_depth() {
        // Arrange
        let tree = branching_tree();
        let target_vertex = 4;
        let heights = [0, 1, 2];
        let expected_values = [4, 1, 0];

        // Act
        let actual_values = heights.map(|height| tree.level_ancestor(target_vertex, height));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn level_ancestor_panics_when_height_exceeds_vertex_depth() {
        // Arrange
        let tree = branching_tree();
        let target_vertex = 4;
        let excessive_height = 3;

        let expected_boolean_1 = true;

        // Act
        let actual_result =
            panic::catch_unwind(|| tree.level_ancestor(target_vertex, excessive_height));

        let actual_check_1 = actual_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn path_and_diameter_pair_are_consistent_on_weighted_tree() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 2;
        let input_value_3 = 3;
        let input_value_4 = 2;
        let tree = branching_tree();
        let expected_check_1 = vec![3, 1, 0, 2];
        let expected_check_2 = 9;
        let expected_check_3 = (9, (3, 2));

        // Act
        let actual_path = tree.path(input_value_1, input_value_2);
        let actual_distance = tree.minimum_distance(input_value_3, input_value_4);
        let actual_diameter_pair = tree.diameter_pair();

        // Assert
        assert_eq!(actual_path, expected_check_1);
        assert_eq!(actual_distance, expected_check_2);
        assert_eq!(actual_diameter_pair, expected_check_3);
    }

    #[test]
    fn diameter_pair_finds_positive_path_beyond_negative_prefix() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, -100)], vec![(0, -100), (2, 60)], vec![(1, 60)]];
        let tree = Tree::new(graph.len(), &graph, input_value_1);
        let expected_check_1 = 60;
        let expected_check_2 = [1, 2];

        // Act
        let (actual_distance, (first, second)) = tree.diameter_pair();
        let mut actual_endpoints = [first, second];
        actual_endpoints.sort_unstable();

        let actual_minimum_distance = tree.minimum_distance(first, second);

        // Assert
        assert_eq!(actual_distance, expected_check_1);
        assert_eq!(actual_endpoints, expected_check_2);
        assert_eq!(actual_minimum_distance, actual_distance);
    }

    #[test]
    fn diameter_pair_keeps_empty_path_when_all_edges_are_negative() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, -4)], vec![(0, -4), (2, -2)], vec![(1, -2)]];
        let tree = Tree::new(graph.len(), &graph, input_value_1);
        let expected_check_1 = (0, (0, 0));

        // Act
        let actual = tree.diameter_pair();

        // Assert
        assert_eq!(actual, expected_check_1);
    }

    #[test]
    fn dp_top_down_propagates_parent_value_with_receiver_sender_order() {
        // Arrange
        let input_value_1 = 0;
        let graph = vec![vec![(1, 2)], vec![(0, 2), (2, 3)], vec![(1, 3)]];
        let tree = Tree::new(graph.len(), &graph, input_value_1);

        let identity = 0_i64;
        let receiver_scale = 100;
        let sender_scale = 10;
        let vertex_offset = 1;
        let vertex_scale = 1_000;
        let expected_check_1 = vec![1_000, 3_102, 6_315];

        // Act
        let actual = tree.dp_top_down(
            identity,
            |lhs, rhs| lhs + rhs,
            |value, receiver, sender, weight| {
                value + receiver as i64 * receiver_scale + sender as i64 * sender_scale + weight
            },
            |value, vertex| value + (vertex as i64 + vertex_offset) * vertex_scale,
        );

        // Assert
        assert_eq!(actual, expected_check_1);
    }

    #[test]
    fn rerooting_returns_hand_calculated_values_on_branching_tree() {
        // Arrange
        let tree = branching_tree();
        let identity = 0;
        let expected = vec![6, 5, 9, 9, 6];

        // Act
        let actual = tree.rerooting(
            identity,
            Tree::default_func,
            Tree::default_mapping,
            Tree::default_composition,
        );

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn rerooting_matches_bottom_up_for_every_root_with_non_commutative_operation() {
        // Arrange
        let input_value_1 = 2;
        // With construction root 2, vertex 1 has its parent edge in the middle:
        // [child 3, parent 0, child 4].
        let graph = vec![
            vec![(2, 5), (1, 1)],
            vec![(3, 2), (0, 1), (4, 3)],
            vec![(5, 4), (0, 5)],
            vec![(1, 2)],
            vec![(1, 3)],
            vec![(2, 4)],
        ];
        let tree = Tree::new(graph.len(), &graph, input_value_1);
        let expected = (0..graph.len())
            .map(|root| {
                let rooted_tree = Tree::new(graph.len(), &graph, root);
                let rooted_values = rooted_tree.dp_bottom_up(
                    String::new(),
                    ordered_func,
                    ordered_mapping,
                    ordered_composition,
                );
                rooted_values[root].clone()
            })
            .collect::<Vec<_>>();

        // Act
        let actual = tree.rerooting(
            String::new(),
            ordered_func,
            ordered_mapping,
            ordered_composition,
        );

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn depth_and_distance_return_expected_root_and_leaf_values() {
        // Arrange
        let tree = sample_tree();
        let root_vertex = 0;
        let target_vertex = 2;
        let expected_values = (0, 2, 3, 3);

        // Act
        let actual_values = (
            tree.depth_from_root(root_vertex),
            tree.depth_from_root(target_vertex),
            tree.distance_from_root(target_vertex),
            tree.minimum_distance(root_vertex, target_vertex),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn traversals_return_expected_orders_on_branching_tree() {
        // Arrange
        let tree = branching_tree();
        let expected_check_1 = vec![0, 1, 3, 4, 2];
        let expected_check_2 = vec![3, 4, 1, 2, 0];
        let expected_check_3 = vec![0, 1, 2, 3, 4];

        // Act
        let actual_values = (tree.pre_order(), tree.post_order(), tree.level_order());

        let actual_check_1 = actual_values.0;
        let actual_check_2 = actual_values.1;
        let actual_check_3 = actual_values.2;

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_check_3);
    }

    #[test]
    fn negative_distance_equal_to_sentinel_does_not_corrupt_tree_structure() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 2;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 0;
        let input_value_7 = 2;
        let graph = vec![vec![(1, -1)], vec![(0, -1), (2, 3)], vec![(1, 3)]];
        let tree = Tree::new(graph.len(), &graph, input_value_1);
        let expected_check_1 = (Some(0), Some(1), -1, 2, vec![0, 1, 2], 2);

        // Act
        let actual = (
            tree.parent_vertex(input_value_2),
            tree.parent_vertex(input_value_3),
            tree.distance_from_root(input_value_4),
            tree.distance_from_root(input_value_5),
            tree.level_order(),
            tree.minimum_distance(input_value_6, input_value_7),
        );

        // Assert
        assert_eq!(actual, expected_check_1);
    }

    #[test]
    fn single_vertex_tree_supports_dynamic_programming_and_diameter() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 0;
        let input_value_3 = 0;
        let input_value_4 = 0;
        let graph: Vec<Vec<(usize, i64)>> = vec![vec![]];
        let tree = Tree::new(graph.len(), &graph, input_value_1);
        let expected_check_1 = vec![0];
        let expected_check_2 = vec![0];
        let expected_check_3 = vec![0];
        let expected_check_4 = (0, (0, 0));

        // Act
        let actual_top_down = tree.dp_top_down(
            input_value_2,
            Tree::default_func,
            Tree::default_mapping,
            Tree::default_composition,
        );
        let actual_bottom_up = tree.dp_bottom_up(
            input_value_3,
            Tree::default_func,
            Tree::default_mapping,
            Tree::default_composition,
        );
        let actual_rerooting = tree.rerooting(
            input_value_4,
            Tree::default_func,
            Tree::default_mapping,
            Tree::default_composition,
        );
        let actual_diameter = tree.diameter_pair();

        // Assert
        assert_eq!(actual_top_down, expected_check_1);
        assert_eq!(actual_bottom_up, expected_check_2);
        assert_eq!(actual_rerooting, expected_check_3);
        assert_eq!(actual_diameter, expected_check_4);
    }

    #[test]
    fn subtree_queries_expose_counts_and_edge_weight_sum_separately() {
        // Arrange
        let input_value_1 = 1;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let tree = branching_tree();
        let expected_check_1 = (2, 3, 5);

        // Act
        let actual = (
            tree.descendant_count(input_value_1),
            tree.subtree_vertex_count(input_value_2),
            tree.subtree_edge_weight_sum(input_value_3),
        );

        // Assert
        assert_eq!(actual, expected_check_1);
    }
    #[test]
    fn euler_tour_returns_expected_visits_events_and_subtree_boundaries() {
        // Arrange
        let tree = branching_tree();
        let expected = (
            vec![0, 1, 3, 1, 4, 1, 0, 2, 0],
            vec![0, 1, 3, -3, 4, -4, -1, 2, -2, 0],
            vec![0, 1, 7, 2, 4],
            vec![9, 6, 8, 3, 5],
        );

        // Act
        let actual = tree.euler_tour();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn leaf_and_descendant_count_match_chain_subtrees() {
        // Arrange
        let tree = sample_tree();
        let leaf = 2;
        let parent = 1;
        let expected = (true, 1);

        // Act
        let actual = (tree.is_leaf(leaf), tree.descendant_count(parent));
        // Assert
        assert_eq!(actual, expected);
    }
}
