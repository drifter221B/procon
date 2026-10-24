use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Clone)]
struct LocalUnionFind {
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
}

impl LocalUnionFind {
    fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
            size: vec![1; size],
        }
    }

    fn root(&mut self, vertex: usize) -> usize {
        if self.parent[vertex] != vertex {
            let parent = self.parent[vertex];
            self.parent[vertex] = self.root(parent);
        }

        self.parent[vertex]
    }

    fn unite(&mut self, u: usize, v: usize) -> bool {
        let mut ru = self.root(u);
        let mut rv = self.root(v);

        if ru == rv {
            return false;
        }

        if self.rank[ru] < self.rank[rv] {
            std::mem::swap(&mut ru, &mut rv);
        }

        self.parent[rv] = ru;
        self.size[ru] += self.size[rv];
        if self.rank[ru] == self.rank[rv] {
            self.rank[ru] += 1;
        }

        true
    }

    fn root_size(&mut self, vertex: usize) -> usize {
        let r = self.root(vertex);
        self.size[r]
    }
}

pub struct WeightedGraph {
    vertex_number: usize,
    is_directed: bool,
    graph: Vec<Vec<(usize, i64)>>,
    edges: Vec<(usize, usize, i64)>,
}

impl WeightedGraph {
    pub fn new_with_graph(
        vertex_number: usize,
        is_directed: bool,
        graph: Vec<Vec<(usize, i64)>>,
    ) -> Self {
        Self {
            vertex_number,
            is_directed,
            graph,
            edges: Vec::new(),
        }
    }

    pub fn new_with_edges(
        vertex_number: usize,
        is_directed: bool,
        edges: Vec<(usize, usize, i64)>,
    ) -> Self {
        Self {
            vertex_number,
            is_directed,
            graph: Vec::new(),
            edges,
        }
    }

    fn ensure_graph(&self) -> Vec<Vec<(usize, i64)>> {
        if !self.graph.is_empty() {
            return self.graph.clone();
        }

        let mut graph = vec![Vec::new(); self.vertex_number];
        for &(u, v, w) in self.edges.iter() {
            if u < self.vertex_number && v < self.vertex_number {
                graph[u].push((v, w));
                if !self.is_directed {
                    graph[v].push((u, w));
                }
            }
        }

        graph
    }

    fn ensure_edges(&self) -> Vec<(usize, usize, i64)> {
        if !self.edges.is_empty() {
            return self.edges.clone();
        }

        let mut edges = Vec::new();
        for (u, adj) in self.graph.iter().enumerate() {
            for &(v, w) in adj.iter() {
                edges.push((u, v, w));
                if !self.is_directed {
                    edges.push((v, u, w));
                }
            }
        }

        edges
    }

    pub fn dijkstra(&self, start_vertex: usize) -> (Vec<i64>, Vec<isize>) {
        let inf = i64::MAX / 4;
        let mut dist = vec![inf; self.vertex_number];
        let mut parents = vec![-1isize; self.vertex_number];

        if start_vertex >= self.vertex_number {
            return (dist, parents);
        }

        let graph = self.ensure_graph();
        dist[start_vertex] = 0;

        let mut heap = BinaryHeap::new();
        heap.push((Reverse(0i64), start_vertex));

        while let Some((Reverse(cost), vertex)) = heap.pop() {
            if cost != dist[vertex] {
                continue;
            }

            for &(next_vertex, weight) in graph[vertex].iter() {
                let candidate = cost.saturating_add(weight);
                if candidate < dist[next_vertex] {
                    dist[next_vertex] = candidate;
                    parents[next_vertex] = vertex as isize;
                    heap.push((Reverse(candidate), next_vertex));
                }
            }
        }

        (dist, parents)
    }

    pub fn prim(&self, start_vertex: usize) -> Option<i64> {
        let graph = self.ensure_graph();
        if graph.is_empty() || start_vertex >= self.vertex_number {
            return None;
        }

        let mut visited = vec![false; self.vertex_number];
        let mut heap = BinaryHeap::new();
        heap.push((Reverse(0i64), start_vertex));
        let mut total = 0i64;
        let mut visited_count = 0usize;

        while let Some((Reverse(cost), vertex)) = heap.pop() {
            if visited[vertex] {
                continue;
            }

            visited[vertex] = true;
            visited_count += 1;
            total += cost;

            for &(next_vertex, weight) in graph[vertex].iter() {
                if !visited[next_vertex] {
                    heap.push((Reverse(weight), next_vertex));
                }
            }
        }

        if visited_count == self.vertex_number {
            Some(total)
        } else {
            None
        }
    }

    pub fn bellman_ford(&self, start_vertex: usize) -> (Vec<i64>, bool) {
        let edges = self.ensure_edges();
        bellman_ford_impl(self.vertex_number, start_vertex, &edges)
    }

    pub fn kruskal(&self) -> Option<i64> {
        let mut edges = self.ensure_edges();
        if self.vertex_number == 0 {
            return Some(0);
        }
        if edges.is_empty() {
            return (self.vertex_number == 1).then_some(0);
        }

        edges.sort_by_key(|e| e.2);
        let mut uf = LocalUnionFind::new(self.vertex_number);
        let mut total = 0i64;

        for &(u, v, w) in edges.iter() {
            if uf.unite(u, v) {
                total += w;
            }
        }

        if uf.root_size(0) == self.vertex_number {
            Some(total)
        } else {
            None
        }
    }

    pub fn warshall_floyd(&self) -> (Vec<Vec<i64>>, bool) {
        let edges = self.ensure_edges();
        warshall_floyd_impl(self.vertex_number, &edges, self.is_directed)
    }
}

fn bellman_ford_impl(
    vertex_number: usize,
    start_vertex: usize,
    edges: &[(usize, usize, i64)],
) -> (Vec<i64>, bool) {
    let inf = i64::MAX / 4;
    let threshold = i64::MAX / 8;
    let mut dist = vec![inf; vertex_number];
    if start_vertex < vertex_number {
        dist[start_vertex] = 0;
    }

    let mut updated = false;
    for _ in 0..vertex_number {
        updated = false;

        for &(from, to, weight) in edges.iter() {
            if vertex_number <= from || vertex_number <= to {
                continue;
            }

            if dist[from] > threshold {
                continue;
            }

            let candidate = dist[from].saturating_add(weight);
            if candidate < dist[to] {
                dist[to] = candidate;
                updated = true;
            }
        }

        if !updated {
            break;
        }
    }

    let mut has_cycle = false;
    if updated {
        for &(from, to, weight) in edges.iter() {
            if vertex_number <= from || vertex_number <= to {
                continue;
            }
            if dist[from] < inf && dist[from].saturating_add(weight) < dist[to] {
                has_cycle = true;
                break;
            }
        }
    }

    (dist, has_cycle)
}

fn warshall_floyd_impl(
    vertex_number: usize,
    edges: &[(usize, usize, i64)],
    is_directed: bool,
) -> (Vec<Vec<i64>>, bool) {
    let inf = i64::MAX / 4;
    let mut dist = vec![vec![inf; vertex_number]; vertex_number];

    for i in 0..vertex_number {
        dist[i][i] = 0;
    }

    for &(u, v, w) in edges.iter() {
        if u < vertex_number && v < vertex_number {
            if w < dist[u][v] {
                dist[u][v] = w;
            }
            if !is_directed && w < dist[v][u] {
                dist[v][u] = w;
            }
        }
    }

    for k in 0..vertex_number {
        for i in 0..vertex_number {
            for j in 0..vertex_number {
                let candidate = dist[i][k].saturating_add(dist[k][j]);
                if candidate < dist[i][j] {
                    dist[i][j] = candidate;
                }
            }
        }
    }

    let mut has_cycle = false;
    for i in 0..vertex_number {
        if dist[i][i] < 0 {
            has_cycle = true;
        }
    }

    (dist, has_cycle)
}

#[cfg(test)]
mod tests {
    use super::WeightedGraph;

    #[test]
    fn dijkstra_handles_simple_line() {
        // Arrange
        let vertex_count = 3;
        let directed = true;
        let query_vertex_1 = 2;
        let input_value_1 = 0;
        let graph = vec![vec![(1, 2)], vec![(2, 3)], vec![]];
        let wg = WeightedGraph::new_with_graph(vertex_count, directed, graph);
        let expected_check_1 = vec![0, 2, 5];
        let expected_check_2 = 1;

        // Act
        let (dist, parents) = wg.dijkstra(input_value_1);
        let actual_check_2 = parents[query_vertex_1];

        // Assert
        assert_eq!(dist, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
    }

    #[test]
    fn dijkstra_returns_inf_when_start_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let directed = false;
        let input_value_1 = 5;
        let graph = vec![vec![(1, 1)], vec![]];
        let wg = WeightedGraph::new_with_graph(vertex_count, directed, graph);
        let expected_check_1 = vec![i64::MAX / 4; 2];
        let expected_check_2 = vec![-1, -1];

        // Act
        let (dist, parents) = wg.dijkstra(input_value_1);

        // Assert
        assert_eq!(dist, expected_check_1);
        assert_eq!(parents, expected_check_2);
    }

    #[test]
    fn dijkstra_with_undirected_edges_uses_reverse_edges_from_edge_list() {
        // Arrange
        let vertex_number = 3;
        let is_directed = false;
        let edges = vec![(0, 1, 2), (1, 2, 3)];
        let start_vertex = 2;
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);
        let expected_distances = vec![5, 3, 0];

        // Act
        let (actual_distances, _) = weighted_graph.dijkstra(start_vertex);

        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn dijkstra_keeps_unreachable_vertex_distance_and_parent_as_initial_values() {
        // Arrange
        let query_vertex_1 = 2;
        let query_vertex_2 = 2;

        let vertex_number = 3;
        let is_directed = true;
        let graph = vec![vec![(1, 2)], vec![], vec![]];
        let start_vertex = 0;
        let weighted_graph = WeightedGraph::new_with_graph(vertex_number, is_directed, graph);
        let expected_distance = i64::MAX / 4;
        let expected_parent = -1isize;

        // Act
        let (actual_distances, actual_parents) = weighted_graph.dijkstra(start_vertex);

        let actual_check_1 = actual_distances[query_vertex_1];
        let actual_check_2 = actual_parents[query_vertex_2];

        // Assert
        assert_eq!(actual_check_1, expected_distance);
        assert_eq!(actual_check_2, expected_parent);
    }

    #[test]
    fn dijkstra_ignores_stale_heap_entry_after_shorter_path_is_found() {
        // Arrange
        let query_vertex_1 = 1;
        let query_vertex_2 = 1;

        let vertex_number = 3;
        let is_directed = true;
        let graph = vec![vec![(1, 10), (2, 1)], vec![], vec![(1, 1)]];
        let start_vertex = 0;
        let weighted_graph = WeightedGraph::new_with_graph(vertex_number, is_directed, graph);
        let expected_distance_to_one = 2;
        let expected_parent_of_one = 2;

        // Act
        let (actual_distances, actual_parents) = weighted_graph.dijkstra(start_vertex);

        let actual_check_1 = actual_distances[query_vertex_1];
        let actual_check_2 = actual_parents[query_vertex_2];

        // Assert
        assert_eq!(actual_check_1, expected_distance_to_one);
        assert_eq!(actual_check_2, expected_parent_of_one);
    }

    #[test]
    fn prim_returns_none_when_disconnected() {
        // Arrange
        let vertex_count = 3;
        let directed = false;
        let input_value_1 = 0;
        let graph = vec![vec![(1, 1)], vec![], vec![]];
        let wg = WeightedGraph::new_with_graph(vertex_count, directed, graph);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = wg.prim(input_value_1).is_none();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn prim_and_kruskal_compute_mst_weight() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = false;
        let input_value_3 = 0;
        let edges = vec![(0, 1, 1), (1, 2, 2), (0, 2, 4)];
        let wg_edges = WeightedGraph::new_with_edges(input_value_1, input_value_2, edges.clone());
        let expected_check_1 = Some(3);
        let expected_check_2 = Some(3);

        // Act
        let actual_check_1 = wg_edges.prim(input_value_3);
        let actual_check_2 = wg_edges.kruskal();

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_check_2, expected_check_2);
    }

    #[test]
    fn prim_returns_none_when_start_vertex_is_out_of_range() {
        // Arrange
        let vertex_number = 2;
        let is_directed = false;
        let edges = vec![(0, 1, 1)];
        let start_vertex = vertex_number;
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);
        let expected_result = None;

        // Act
        let actual_result = weighted_graph.prim(start_vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn negative_edges_without_cycle_return_expected_distances_and_false_cycle_flags() {
        // Arrange
        let vertex_count = 3;
        let directed = true;
        let edges = vec![(0, 1, 3), (1, 2, -2)];
        let source = 0;
        let expected_distances = vec![0, 3, 1];
        let expected_cycle = false;
        let graph = WeightedGraph::new_with_edges(vertex_count, directed, edges);
        let expected_check_1 = &expected_distances;

        // Act
        let (all_distances, warshall_cycle) = graph.warshall_floyd();
        let (bellman_distances, bellman_cycle) = graph.bellman_ford(source);
        let warshall_distances = &all_distances[source];

        // Assert
        assert_eq!(warshall_distances, expected_check_1);
        assert_eq!(bellman_distances, expected_distances);
        assert_eq!(warshall_cycle, expected_cycle);
        assert_eq!(bellman_cycle, expected_cycle);
    }

    #[test]
    fn bellman_ford_detects_reachable_negative_cycle() {
        // Arrange
        let vertex_number = 3;
        let is_directed = true;
        let edges = vec![(0, 1, 1), (1, 2, -3), (2, 1, 1)];
        let start_vertex = 0;
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);
        let expected_has_cycle = true;

        // Act
        let (_, actual_has_cycle) = weighted_graph.bellman_ford(start_vertex);

        // Assert
        assert_eq!(actual_has_cycle, expected_has_cycle);
    }

    #[test]
    fn kruskal_returns_none_when_disconnected_graph() {
        // Arrange
        let edges = vec![(0, 1, 1)];
        let wg = WeightedGraph::new_with_edges(3, false, edges);

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = wg.kruskal().is_none();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn kruskal_returns_zero_for_single_vertex_graph_without_edges() {
        // Arrange
        let vertex_number = 1;
        let is_directed = false;
        let edges = vec![];
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);
        let expected_result = Some(0);

        // Act
        let actual_result = weighted_graph.kruskal();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn kruskal_returns_none_for_multiple_vertices_without_edges() {
        // Arrange
        let vertex_number = 2;
        let is_directed = false;
        let edges = vec![];
        let expected_result = None;
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);

        // Act
        let actual_result = weighted_graph.kruskal();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn kruskal_with_graph_representation_returns_mst_weight() {
        // Arrange
        let vertex_number = 3;
        let is_directed = false;
        let graph = vec![vec![(1, 1), (2, 4)], vec![(2, 2)], vec![]];
        let weighted_graph = WeightedGraph::new_with_graph(vertex_number, is_directed, graph);
        let expected_result = Some(3);

        // Act
        let actual_result = weighted_graph.kruskal();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn warshall_floyd_detects_negative_cycle() {
        // Arrange
        let vertex_number = 3;
        let is_directed = true;
        let edges = vec![(0, 1, 1), (1, 2, -3), (2, 1, 1)];
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);
        let expected_has_cycle = true;

        // Act
        let (_, actual_has_cycle) = weighted_graph.warshall_floyd();

        // Assert
        assert_eq!(actual_has_cycle, expected_has_cycle);
    }

    #[test]
    fn shortest_path_algorithms_ignore_out_of_range_edges_without_panicking() {
        // Arrange
        let query_vertex_1 = 1;
        let query_vertex_2 = 0;
        let query_vertex_3 = 1;

        let vertex_number = 2;
        let is_directed = true;
        let edges = vec![(0, 1, 3), (0, 2, 1), (3, 1, 1)];
        let start_vertex = 0;
        let weighted_graph = WeightedGraph::new_with_edges(vertex_number, is_directed, edges);
        let expected_bellman_cycle = false;
        let expected_warshall_cycle = false;
        let expected_check_1 = 3;
        let expected_check_3 = 3;

        // Act
        let (bellman_distances, actual_bellman_cycle) = weighted_graph.bellman_ford(start_vertex);
        let (warshall_distances, actual_warshall_cycle) = weighted_graph.warshall_floyd();

        let actual_check_1 = bellman_distances[query_vertex_1];
        let actual_check_3 = warshall_distances[query_vertex_2][query_vertex_3];

        // Assert
        assert_eq!(actual_check_1, expected_check_1);
        assert_eq!(actual_bellman_cycle, expected_bellman_cycle);
        assert_eq!(actual_check_3, expected_check_3);
        assert_eq!(actual_warshall_cycle, expected_warshall_cycle);
    }
    #[test]
    fn kruskal_returns_zero_for_empty_vertex_set() {
        // Arrange
        let vertex_count = 0;
        let directed = false;
        let edges = vec![];
        let expected = Some(0);
        let graph = WeightedGraph::new_with_edges(vertex_count, directed, edges);

        // Act
        let actual = graph.kruskal();

        // Assert
        assert_eq!(actual, expected);
    }
}
