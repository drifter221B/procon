use std::collections::VecDeque;

/// 左右二部グラフの抽象インターフェース。
/// 左頂点ごとの隣接右頂点をイテレータとして提供する。
pub trait BipartiteGraph {
    type RightNeighbors<'a>: Iterator<Item = usize>
    where
        Self: 'a;

    fn left_size(&self) -> usize;
    fn right_size(&self) -> usize;
    fn right_neighbors(&self, left_vertex: usize) -> Self::RightNeighbors<'_>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatchingResult {
    matching_size: usize,
    left_pairs: Vec<Option<usize>>,
    right_pairs: Vec<Option<usize>>,
}

impl MatchingResult {
    pub fn matching_size(&self) -> usize {
        self.matching_size
    }

    pub fn left_pairs(&self) -> &[Option<usize>] {
        &self.left_pairs
    }

    pub fn right_pairs(&self) -> &[Option<usize>] {
        &self.right_pairs
    }

    pub fn into_parts(self) -> (usize, Vec<Option<usize>>, Vec<Option<usize>>) {
        (self.matching_size, self.left_pairs, self.right_pairs)
    }
}

/// `Vec<Vec<usize>>` の隣接リストを `BipartiteGraph` として扱うためのアダプタ。
pub struct AdjacencyListBipartiteGraph<'a> {
    right_size: usize,
    adjacency: &'a [Vec<usize>],
}

impl<'a> AdjacencyListBipartiteGraph<'a> {
    pub fn new(right_size: usize, adjacency: &'a [Vec<usize>]) -> Self {
        Self {
            right_size,
            adjacency,
        }
    }
}

impl BipartiteGraph for AdjacencyListBipartiteGraph<'_> {
    type RightNeighbors<'a>
        = std::iter::Copied<std::slice::Iter<'a, usize>>
    where
        Self: 'a;

    fn left_size(&self) -> usize {
        self.adjacency.len()
    }

    fn right_size(&self) -> usize {
        self.right_size
    }

    fn right_neighbors(&self, left_vertex: usize) -> Self::RightNeighbors<'_> {
        self.adjacency[left_vertex].iter().copied()
    }
}

/// Hopcroft-Karp 法の最大マッチング計算器。
pub struct HopcroftKarpMatcher<'a, G>
where
    G: BipartiteGraph,
{
    graph: &'a G,
    pair_left: Vec<Option<usize>>,
    pair_right: Vec<Option<usize>>,
    distances: Vec<usize>,
}

impl<'a, G> HopcroftKarpMatcher<'a, G>
where
    G: BipartiteGraph,
{
    pub fn new(graph: &'a G) -> Self {
        Self {
            graph,
            pair_left: vec![None; graph.left_size()],
            pair_right: vec![None; graph.right_size()],
            distances: vec![usize::MAX; graph.left_size()],
        }
    }

    pub fn solve(mut self) -> MatchingResult {
        if self.graph.left_size() == 0 || self.graph.right_size() == 0 {
            for left_vertex in 0..self.graph.left_size() {
                for right_vertex in self.graph.right_neighbors(left_vertex) {
                    self.validate_right_vertex_index(right_vertex);
                }
            }

            return MatchingResult {
                matching_size: 0,
                left_pairs: self.pair_left,
                right_pairs: self.pair_right,
            };
        }

        let matching_size = self.run_while_augmenting_path_exists();
        MatchingResult {
            matching_size,
            left_pairs: self.pair_left,
            right_pairs: self.pair_right,
        }
    }

    fn run_while_augmenting_path_exists(&mut self) -> usize {
        let mut matching_size = 0;

        loop {
            let has_path_to_free_right = self.build_layered_distances();
            if !has_path_to_free_right {
                break;
            }

            for left_vertex in 0..self.graph.left_size() {
                if self.pair_left[left_vertex].is_none() && self.try_augment_from_left(left_vertex)
                {
                    matching_size += 1;
                }
            }
        }

        matching_size
    }

    fn build_layered_distances(&mut self) -> bool {
        self.distances.fill(usize::MAX);
        let mut queue = VecDeque::new();
        let mut has_path_to_free_right = false;

        for left_vertex in 0..self.graph.left_size() {
            if self.pair_left[left_vertex].is_none() {
                self.distances[left_vertex] = 0;
                queue.push_back(left_vertex);
            }
        }

        while let Some(left_vertex) = queue.pop_front() {
            let current_distance = self.distances[left_vertex];

            for right_vertex in self.graph.right_neighbors(left_vertex) {
                self.validate_right_vertex_index(right_vertex);

                if let Some(next_left_vertex) = self.pair_right[right_vertex] {
                    if self.distances[next_left_vertex] == usize::MAX {
                        self.distances[next_left_vertex] = current_distance + 1;
                        queue.push_back(next_left_vertex);
                    }
                } else {
                    has_path_to_free_right = true;
                }
            }
        }

        has_path_to_free_right
    }

    fn try_augment_from_left(&mut self, left_vertex: usize) -> bool {
        let right_vertices: Vec<usize> = self.graph.right_neighbors(left_vertex).collect();

        for right_vertex in right_vertices {
            self.validate_right_vertex_index(right_vertex);

            if let Some(next_left_vertex) = self.pair_right[right_vertex] {
                if self.is_reachable_in_next_layer(left_vertex, next_left_vertex)
                    && self.try_augment_from_left(next_left_vertex)
                {
                    self.record_new_matching_edge(left_vertex, right_vertex);
                    return true;
                }
            } else {
                self.record_new_matching_edge(left_vertex, right_vertex);
                return true;
            }
        }

        self.distances[left_vertex] = usize::MAX;
        false
    }

    fn is_reachable_in_next_layer(&self, left_vertex: usize, next_left_vertex: usize) -> bool {
        self.distances[left_vertex].saturating_add(1) == self.distances[next_left_vertex]
    }

    fn record_new_matching_edge(&mut self, left_vertex: usize, right_vertex: usize) {
        self.pair_left[left_vertex] = Some(right_vertex);
        self.pair_right[right_vertex] = Some(left_vertex);
    }

    fn validate_right_vertex_index(&self, right_vertex: usize) {
        if self.graph.right_size() <= right_vertex {
            panic!("right_size 未満のインデックスのみを指定してください");
        }
    }
}

/// 任意の `BipartiteGraph` 実装に対して Hopcroft-Karp を実行する。
pub fn hopcroft_karp_on_graph<G>(graph: &G) -> MatchingResult
where
    G: BipartiteGraph,
{
    HopcroftKarpMatcher::new(graph).solve()
}

/// `Vec<Vec<usize>>` を受け取る互換 API。
pub fn hopcroft_karp(
    left_size: usize,
    right_size: usize,
    adjacency: &[Vec<usize>],
) -> (usize, Vec<Option<usize>>, Vec<Option<usize>>) {
    assert_eq!(
        adjacency.len(),
        left_size,
        "left_size と adjacency の長さは一致させてください"
    );

    let graph = AdjacencyListBipartiteGraph::new(right_size, adjacency);
    let result = hopcroft_karp_on_graph(&graph);
    result.into_parts()
}

#[cfg(test)]
mod tests {
    use super::{
        hopcroft_karp, hopcroft_karp_on_graph, AdjacencyListBipartiteGraph, BipartiteGraph,
        HopcroftKarpMatcher,
    };

    #[derive(Clone)]
    struct TestBipartiteGraph {
        right_size: usize,
        adjacency: Vec<Vec<usize>>,
    }

    impl TestBipartiteGraph {
        fn new(right_size: usize, adjacency: Vec<Vec<usize>>) -> Self {
            Self {
                right_size,
                adjacency,
            }
        }
    }

    impl BipartiteGraph for TestBipartiteGraph {
        type RightNeighbors<'a>
            = std::iter::Copied<std::slice::Iter<'a, usize>>
        where
            Self: 'a;

        fn left_size(&self) -> usize {
            self.adjacency.len()
        }

        fn right_size(&self) -> usize {
            self.right_size
        }

        fn right_neighbors(&self, left_vertex: usize) -> Self::RightNeighbors<'_> {
            self.adjacency[left_vertex].iter().copied()
        }
    }

    fn assert_pair_mapping_is_consistent(
        adjacency: &[Vec<usize>],
        left_pairs: &[Option<usize>],
        right_pairs: &[Option<usize>],
    ) {
        for (left_vertex, matched_right_vertex) in left_pairs.iter().enumerate() {
            if let Some(right_vertex) = matched_right_vertex {
                assert_eq!(right_pairs[*right_vertex], Some(left_vertex));
                assert!(adjacency[left_vertex].contains(right_vertex));
            }
        }

        for (right_vertex, matched_left_vertex) in right_pairs.iter().enumerate() {
            if let Some(left_vertex) = matched_left_vertex {
                assert_eq!(left_pairs[*left_vertex], Some(right_vertex));
                assert!(adjacency[*left_vertex].contains(&right_vertex));
            }
        }
    }

    fn reference_maximum_matching_size(adjacency: &[Vec<usize>], right_size: usize) -> usize {
        fn search(
            left_vertex: usize,
            adjacency: &[Vec<usize>],
            used_right_vertices: &mut [bool],
        ) -> usize {
            if left_vertex == adjacency.len() {
                return 0;
            }

            let mut maximum_matching_size = search(left_vertex + 1, adjacency, used_right_vertices);

            for &right_vertex in &adjacency[left_vertex] {
                if used_right_vertices[right_vertex] {
                    continue;
                }

                used_right_vertices[right_vertex] = true;
                maximum_matching_size = maximum_matching_size
                    .max(1 + search(left_vertex + 1, adjacency, used_right_vertices));
                used_right_vertices[right_vertex] = false;
            }

            maximum_matching_size
        }

        let mut used_right_vertices = vec![false; right_size];
        search(0, adjacency, &mut used_right_vertices)
    }

    fn adjacency_from_edge_mask(
        left_size: usize,
        right_size: usize,
        edge_mask: usize,
    ) -> Vec<Vec<usize>> {
        (0..left_size)
            .map(|left_vertex| {
                (0..right_size)
                    .filter(|&right_vertex| {
                        let edge_index = left_vertex * right_size + right_vertex;
                        edge_mask & (1usize << edge_index) != 0
                    })
                    .collect()
            })
            .collect()
    }

    fn verify_all_simple_graphs_up_to_size(maximum_size: usize) -> Result<(), String> {
        for left_size in 0..=maximum_size {
            for right_size in 0..=maximum_size {
                let edge_count = left_size * right_size;
                let graph_count = 1usize << edge_count;

                for edge_mask in 0..graph_count {
                    let adjacency = adjacency_from_edge_mask(left_size, right_size, edge_mask);
                    let expected_matching_size =
                        reference_maximum_matching_size(&adjacency, right_size);
                    let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
                        hopcroft_karp(left_size, right_size, &adjacency);
                    let actual_left_pair_count = actual_left_pairs
                        .iter()
                        .filter(|pair| pair.is_some())
                        .count();
                    let actual_right_pair_count = actual_right_pairs
                        .iter()
                        .filter(|pair| pair.is_some())
                        .count();

                    if actual_matching_size != expected_matching_size {
                        return Err(format!(
                            "matching size differs: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, expected={expected_matching_size}, actual={actual_matching_size}"
                        ));
                    }
                    if actual_left_pairs.len() != left_size {
                        return Err(format!(
                            "left pair length differs: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, actual={}",
                            actual_left_pairs.len()
                        ));
                    }
                    if actual_right_pairs.len() != right_size {
                        return Err(format!(
                            "right pair length differs: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, actual={}",
                            actual_right_pairs.len()
                        ));
                    }
                    if actual_left_pair_count != actual_matching_size {
                        return Err(format!(
                            "left pair count differs: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, matching_size={actual_matching_size}, pair_count={actual_left_pair_count}"
                        ));
                    }
                    if actual_right_pair_count != actual_matching_size {
                        return Err(format!(
                            "right pair count differs: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, matching_size={actual_matching_size}, pair_count={actual_right_pair_count}"
                        ));
                    }

                    for (left_vertex, &matched_right_vertex) in actual_left_pairs.iter().enumerate()
                    {
                        if let Some(right_vertex) = matched_right_vertex {
                            if !adjacency[left_vertex].contains(&right_vertex) {
                                return Err(format!(
                                    "matched edge does not exist: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, left_vertex={left_vertex}, right_vertex={right_vertex}"
                                ));
                            }
                            if actual_right_pairs.get(right_vertex) != Some(&Some(left_vertex)) {
                                return Err(format!(
                                    "left-to-right pair is inconsistent: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, left_vertex={left_vertex}, right_vertex={right_vertex}"
                                ));
                            }
                        }
                    }

                    for (right_vertex, &matched_left_vertex) in
                        actual_right_pairs.iter().enumerate()
                    {
                        if let Some(left_vertex) = matched_left_vertex {
                            if !adjacency[left_vertex].contains(&right_vertex) {
                                return Err(format!(
                                    "reverse matched edge does not exist: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, left_vertex={left_vertex}, right_vertex={right_vertex}"
                                ));
                            }
                            if actual_left_pairs.get(left_vertex) != Some(&Some(right_vertex)) {
                                return Err(format!(
                                    "right-to-left pair is inconsistent: left_size={left_size}, right_size={right_size}, edge_mask={edge_mask}, left_vertex={left_vertex}, right_vertex={right_vertex}"
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    fn catch_panic_message<F, T>(operation: F) -> Option<String>
    where
        F: FnOnce() -> T + std::panic::UnwindSafe,
    {
        match std::panic::catch_unwind(operation) {
            Ok(_) => None,
            Err(payload) => payload
                .downcast_ref::<&str>()
                .map(|message| (*message).to_owned())
                .or_else(|| payload.downcast_ref::<String>().cloned()),
        }
    }

    #[test]
    fn hopcroft_karp_on_graph_supports_custom_graph_implementation() {
        // Arrange
        let right_size = 3;
        let adjacency = vec![vec![0, 1], vec![1, 2], vec![0, 2]];
        let graph = TestBipartiteGraph::new(right_size, adjacency.clone());
        let expected_matching_size = 3;

        // Act
        let result = hopcroft_karp_on_graph(&graph);
        let actual_matching_size = result.matching_size();
        let actual_left_pairs = result.left_pairs().to_vec();
        let actual_right_pairs = result.right_pairs().to_vec();

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_pair_mapping_is_consistent(&adjacency, &actual_left_pairs, &actual_right_pairs);
    }

    #[test]
    fn matching_result_getters_return_distinct_left_and_right_pairs() {
        // Arrange
        let right_size = 3;
        let adjacency = vec![vec![1], vec![]];
        let graph = AdjacencyListBipartiteGraph::new(right_size, &adjacency);
        let expected_matching_size = 1;
        let expected_left_pairs = vec![Some(1), None];
        let expected_right_pairs = vec![None, Some(0), None];

        // Act
        let result = hopcroft_karp_on_graph(&graph);
        let actual_matching_size = result.matching_size();
        let actual_left_pairs = result.left_pairs();
        let actual_right_pairs = result.right_pairs();

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn matching_result_into_parts_returns_owned_parts_in_order() {
        // Arrange
        let right_size = 2;
        let adjacency = vec![vec![0], vec![1]];
        let graph = AdjacencyListBipartiteGraph::new(right_size, &adjacency);
        let expected_parts = (2, vec![Some(0), Some(1)], vec![Some(0), Some(1)]);

        // Act
        let result = hopcroft_karp_on_graph(&graph);
        let actual_parts = result.into_parts();

        // Assert
        assert_eq!(actual_parts, expected_parts);
    }

    #[test]
    fn matching_result_into_parts_returns_distinct_left_and_right_pairs_in_order() {
        // Arrange
        let right_size = 3;
        let adjacency = vec![vec![1], vec![]];
        let graph = AdjacencyListBipartiteGraph::new(right_size, &adjacency);
        let expected_parts = (1, vec![Some(1), None], vec![None, Some(0), None]);

        // Act
        let result = hopcroft_karp_on_graph(&graph);
        let actual_parts = result.into_parts();

        // Assert
        assert_eq!(actual_parts, expected_parts);
    }

    #[test]
    fn hopcroft_karp_on_graph_with_adjacency_adapter_returns_expected_matching() {
        // Arrange
        let right_size = 2;
        let adjacency = vec![vec![0], vec![1]];
        let graph = AdjacencyListBipartiteGraph::new(right_size, &adjacency);
        let expected_matching_size = 2;
        let expected_left_pairs = vec![Some(0), Some(1)];
        let expected_right_pairs = vec![Some(0), Some(1)];

        // Act
        let result = HopcroftKarpMatcher::new(&graph).solve();
        let actual_matching_size = result.matching_size();
        let actual_left_pairs = result.left_pairs().to_vec();
        let actual_right_pairs = result.right_pairs().to_vec();

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn adjacency_list_bipartite_graph_preserves_sizes_neighbor_order_and_duplicates() {
        // Arrange
        let right_size = 4;
        let adjacency = vec![vec![2, 0, 2], vec![]];
        let graph = AdjacencyListBipartiteGraph::new(right_size, &adjacency);
        let left_vertex_zero = 0;
        let left_vertex_one = 1;
        let expected_left_size = 2;
        let expected_right_size = 4;
        let expected_left_zero_neighbors = vec![2, 0, 2];
        let expected_left_one_neighbors: Vec<usize> = vec![];

        // Act
        let actual_left_size = graph.left_size();
        let actual_right_size = graph.right_size();
        let actual_left_zero_neighbors: Vec<usize> =
            graph.right_neighbors(left_vertex_zero).collect();
        let actual_left_one_neighbors: Vec<usize> =
            graph.right_neighbors(left_vertex_one).collect();

        // Assert
        assert_eq!(actual_left_size, expected_left_size);
        assert_eq!(actual_right_size, expected_right_size);
        assert_eq!(actual_left_zero_neighbors, expected_left_zero_neighbors);
        assert_eq!(actual_left_one_neighbors, expected_left_one_neighbors);
    }

    #[test]
    fn hopcroft_karp_returns_zero_when_graph_is_empty() {
        // Arrange
        let left_size = 0;
        let right_size = 0;
        let adjacency: Vec<Vec<usize>> = vec![];
        let expected_matching_size = 0;
        let expected_left_pairs: Vec<Option<usize>> = vec![];
        let expected_right_pairs: Vec<Option<usize>> = vec![];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_returns_zero_when_right_side_is_empty() {
        // Arrange
        let left_size = 1;
        let right_size = 0;
        let adjacency = vec![vec![]];
        let expected_matching_size = 0;
        let expected_left_pairs = vec![None];
        let expected_right_pairs: Vec<Option<usize>> = vec![];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_matches_single_edge() {
        // Arrange
        let left_size = 1;
        let right_size = 1;
        let adjacency = vec![vec![0]];
        let expected_matching_size = 1;
        let expected_left_pairs = vec![Some(0)];
        let expected_right_pairs = vec![Some(0)];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_finds_perfect_matching_in_three_by_three_graph() {
        // Arrange
        let left_size = 3;
        let right_size = 3;
        let adjacency = vec![vec![0], vec![0, 1], vec![1, 2]];
        let expected_matching_size = 3;
        let expected_left_pairs = vec![Some(0), Some(1), Some(2)];
        let expected_right_pairs = vec![Some(0), Some(1), Some(2)];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_does_not_overcount_when_duplicate_edges_exist() {
        // Arrange
        let left_size = 2;
        let right_size = 2;
        let adjacency = vec![vec![0, 0], vec![1]];
        let expected_matching_size = 2;

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_pair_mapping_is_consistent(&adjacency, &actual_left_pairs, &actual_right_pairs);
    }

    #[test]
    fn hopcroft_karp_keeps_isolated_left_vertices_unmatched() {
        // Arrange
        let left_size = 4;
        let right_size = 3;
        let adjacency = vec![vec![0], vec![], vec![1, 2], vec![]];
        let expected_matching_size = 2;
        let expected_left_vertex_one_pair = None;
        let expected_left_vertex_three_pair = None;

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);
        let actual_left_vertex_one_pair = actual_left_pairs[1];
        let actual_left_vertex_three_pair = actual_left_pairs[3];

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_vertex_one_pair, expected_left_vertex_one_pair);
        assert_eq!(
            actual_left_vertex_three_pair,
            expected_left_vertex_three_pair
        );
        assert_pair_mapping_is_consistent(&adjacency, &actual_left_pairs, &actual_right_pairs);
    }

    #[test]
    fn hopcroft_karp_panics_when_adjacency_length_is_less_than_left_size() {
        // Arrange
        let left_size = 2;
        let right_size = 2;
        let adjacency = vec![vec![0]];
        let expected_message = "left_size と adjacency の長さは一致させてください";

        // Act
        let actual_message =
            catch_panic_message(|| hopcroft_karp(left_size, right_size, &adjacency));

        // Assert
        assert!(actual_message.is_some_and(|message| message.contains(expected_message)));
    }

    #[test]
    fn hopcroft_karp_panics_when_adjacency_length_is_greater_than_left_size() {
        // Arrange
        let left_size = 1;
        let right_size = 2;
        let adjacency = vec![vec![0], vec![1]];
        let expected_message = "left_size と adjacency の長さは一致させてください";

        // Act
        let actual_message =
            catch_panic_message(|| hopcroft_karp(left_size, right_size, &adjacency));

        // Assert
        assert!(actual_message.is_some_and(|message| message.contains(expected_message)));
    }

    #[test]
    fn hopcroft_karp_panics_when_right_index_is_equal_to_right_size_boundary() {
        // Arrange
        let left_size = 1;
        let right_size = 2;
        let adjacency = vec![vec![2]];
        let expected_message = "right_size 未満のインデックスのみを指定してください";

        // Act
        let actual_message =
            catch_panic_message(|| hopcroft_karp(left_size, right_size, &adjacency));

        // Assert
        assert!(actual_message.is_some_and(|message| message.contains(expected_message)));
    }

    #[test]
    fn hopcroft_karp_returns_unmatched_right_vertices_when_left_side_is_empty() {
        // Arrange
        let left_size = 0;
        let right_size = 1;
        let adjacency: Vec<Vec<usize>> = vec![];
        let expected_matching_size = 0;
        let expected_left_pairs: Vec<Option<usize>> = vec![];
        let expected_right_pairs = vec![None];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_keeps_all_vertices_unmatched_when_all_adjacencies_are_empty() {
        // Arrange
        let left_size = 3;
        let right_size = 3;
        let adjacency = vec![vec![], vec![], vec![]];
        let expected_matching_size = 0;
        let expected_left_pairs = vec![None, None, None];
        let expected_right_pairs = vec![None, None, None];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_leaves_extra_right_vertices_unmatched_when_left_side_is_smaller() {
        // Arrange
        let left_size = 2;
        let right_size = 4;
        let adjacency = vec![vec![0], vec![2]];
        let expected_matching_size = 2;
        let expected_right_pairs = vec![Some(0), None, Some(1), None];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_right_pairs, expected_right_pairs);
        assert_pair_mapping_is_consistent(&adjacency, &actual_left_pairs, &actual_right_pairs);
    }

    #[test]
    fn hopcroft_karp_sums_matching_size_across_multiple_components() {
        // Arrange
        let left_size = 4;
        let right_size = 4;
        let adjacency = vec![vec![0], vec![1], vec![2], vec![]];
        let expected_matching_size = 3;

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_pair_mapping_is_consistent(&adjacency, &actual_left_pairs, &actual_right_pairs);
    }

    #[test]
    fn hopcroft_karp_reassigns_an_existing_pair_to_complete_matching() {
        // Arrange
        let left_size = 2;
        let right_size = 2;
        let adjacency = vec![vec![0, 1], vec![0]];
        let expected_matching_size = 2;
        let expected_left_pairs = vec![Some(1), Some(0)];
        let expected_right_pairs = vec![Some(1), Some(0)];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_backtracks_from_a_dead_end_to_use_a_multistep_augmenting_path() {
        // Arrange
        let left_size = 4;
        let right_size = 4;
        let adjacency = vec![vec![0], vec![1, 3], vec![2, 1], vec![0, 2]];
        let expected_matching_size = 4;
        let expected_left_pairs = vec![Some(0), Some(3), Some(1), Some(2)];
        let expected_right_pairs = vec![Some(0), Some(2), Some(3), Some(1)];

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_left_pairs, expected_left_pairs);
        assert_eq!(actual_right_pairs, expected_right_pairs);
    }

    #[test]
    fn hopcroft_karp_returns_maximum_matching_when_hall_condition_is_not_satisfied() {
        // Arrange
        let left_size = 3;
        let right_size = 2;
        let adjacency = vec![vec![0, 1], vec![0, 1], vec![0, 1]];
        let expected_matching_size = 2;
        let expected_matched_left_count = 2;
        let expected_unmatched_left_count = 1;
        let expected_matched_right_count = 2;

        // Act
        let (actual_matching_size, actual_left_pairs, actual_right_pairs) =
            hopcroft_karp(left_size, right_size, &adjacency);
        let actual_matched_left_count = actual_left_pairs
            .iter()
            .filter(|pair| pair.is_some())
            .count();
        let actual_unmatched_left_count = actual_left_pairs
            .iter()
            .filter(|pair| pair.is_none())
            .count();
        let actual_matched_right_count = actual_right_pairs
            .iter()
            .filter(|pair| pair.is_some())
            .count();

        // Assert
        assert_eq!(actual_matching_size, expected_matching_size);
        assert_eq!(actual_matched_left_count, expected_matched_left_count);
        assert_eq!(actual_unmatched_left_count, expected_unmatched_left_count);
        assert_eq!(actual_matched_right_count, expected_matched_right_count);
        assert_pair_mapping_is_consistent(&adjacency, &actual_left_pairs, &actual_right_pairs);
    }

    #[test]
    fn hopcroft_karp_matches_reference_for_all_simple_graphs_up_to_four_by_four() {
        // Arrange
        let maximum_size = 4;
        let expected_verification = Ok(());

        // Act
        let actual_verification = verify_all_simple_graphs_up_to_size(maximum_size);

        // Assert
        assert_eq!(actual_verification, expected_verification);
    }

    #[test]
    fn hopcroft_karp_panics_for_out_of_range_edge_when_right_side_is_empty() {
        // Arrange
        let left_size = 1;
        let right_size = 0;
        let adjacency = vec![vec![0]];
        let expected_panic_message =
            Some("right_size 未満のインデックスのみを指定してください".to_owned());

        // Act
        let actual_panic_message =
            catch_panic_message(|| hopcroft_karp(left_size, right_size, &adjacency));

        // Assert
        assert_eq!(actual_panic_message, expected_panic_message);
    }
}
