use std::cmp::min;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
struct Edge {
    to: usize,
    rev: usize,
    capacity: i64,
}

#[derive(Debug)]
pub struct Dinic {
    graph: Vec<Vec<Edge>>,
    level: Vec<i64>,
    next_edge_index: Vec<usize>,
}

impl Dinic {
    pub fn new(vertex_count: usize) -> Self {
        Self {
            graph: vec![Vec::new(); vertex_count],
            level: vec![-1; vertex_count],
            next_edge_index: vec![0; vertex_count],
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize, capacity: i64) {
        assert!(from < self.graph.len());
        assert!(to < self.graph.len());
        assert!(0 <= capacity);

        let forward_index = self.graph[to].len();
        let backward_index = self.graph[from].len();

        self.graph[from].push(Edge {
            to,
            rev: forward_index,
            capacity,
        });

        self.graph[to].push(Edge {
            to: from,
            rev: backward_index,
            capacity: 0,
        });
    }

    pub fn max_flow(&mut self, source: usize, sink: usize) -> i64 {
        if self.graph.is_empty() {
            return 0;
        }
        assert!(source < self.graph.len());
        assert!(sink < self.graph.len());
        if source == sink {
            return 0;
        }

        let mut flow = 0;
        loop {
            if !self.build_level_graph(source, sink) {
                break;
            }
            self.next_edge_index.fill(0);

            loop {
                let pushed = self.dfs(source, sink, i64::MAX);
                if pushed == 0 {
                    break;
                }
                flow += pushed;
            }
        }
        flow
    }

    fn build_level_graph(&mut self, source: usize, sink: usize) -> bool {
        self.level.fill(-1);
        let mut queue = VecDeque::new();
        self.level[source] = 0;
        queue.push_back(source);

        while let Some(vertex) = queue.pop_front() {
            let current_level = self.level[vertex];
            for edge in self.graph[vertex].iter() {
                if edge.capacity <= 0 {
                    continue;
                }
                if self.level[edge.to] != -1 {
                    continue;
                }
                self.level[edge.to] = current_level + 1;
                queue.push_back(edge.to);
            }
        }

        self.level[sink] != -1
    }

    fn dfs(&mut self, vertex: usize, sink: usize, flow_limit: i64) -> i64 {
        if vertex == sink {
            return flow_limit;
        }

        while self.next_edge_index[vertex] < self.graph[vertex].len() {
            let edge_index = self.next_edge_index[vertex];
            let to;
            let remaining_capacity;
            let rev;
            {
                let edge = &self.graph[vertex][edge_index];
                to = edge.to;
                remaining_capacity = edge.capacity;
                rev = edge.rev;
            }

            if remaining_capacity <= 0 {
                self.next_edge_index[vertex] += 1;
                continue;
            }
            if self.level[to] <= self.level[vertex] {
                self.next_edge_index[vertex] += 1;
                continue;
            }

            let possible_flow = self.dfs(to, sink, min(flow_limit, remaining_capacity));

            if possible_flow > 0 {
                self.graph[vertex][edge_index].capacity -= possible_flow;
                self.graph[to][rev].capacity += possible_flow;
                return possible_flow;
            }

            self.next_edge_index[vertex] += 1;
        }

        0
    }
}

#[cfg(test)]
mod tests {
    use super::Dinic;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn build_dinic_with_edges(vertex_count: usize, edges: &[(usize, usize, i64)]) -> Dinic {
        let mut dinic = Dinic::new(vertex_count);
        for &(from_vertex, to_vertex, capacity_value) in edges.iter() {
            dinic.add_edge(from_vertex, to_vertex, capacity_value);
        }
        dinic
    }

    fn compute_single_edge_max_flow(capacity_value: i64) -> i64 {
        let vertex_count = 2;
        let source_vertex = 0;
        let sink_vertex = 1;

        let mut dinic = Dinic::new(vertex_count);
        dinic.add_edge(source_vertex, sink_vertex, capacity_value);
        dinic.max_flow(source_vertex, sink_vertex)
    }

    #[test]
    fn new_with_zero_vertex_count_creates_empty_internal_vectors() {
        // Arrange
        let vertex_count = 0;

        // Act
        let dinic = Dinic::new(vertex_count);
        let graph_is_empty = dinic.graph.is_empty();
        let level_is_empty = dinic.level.is_empty();
        let next_edge_index_is_empty = dinic.next_edge_index.is_empty();

        // Assert
        assert!(graph_is_empty);
        assert!(level_is_empty);
        assert!(next_edge_index_is_empty);
    }

    #[test]
    fn new_with_one_vertex_count_initializes_default_internal_values() {
        // Arrange
        let vertex_count = 1;
        let expected_graph_len = 1;
        let expected_level = vec![-1];
        let expected_next_edge_index = vec![0];

        // Act
        let dinic = Dinic::new(vertex_count);
        let graph_len = dinic.graph.len();
        let level = dinic.level.clone();
        let next_edge_index = dinic.next_edge_index.clone();

        // Assert
        assert_eq!(graph_len, expected_graph_len);
        assert_eq!(level, expected_level);
        assert_eq!(next_edge_index, expected_next_edge_index);
    }

    #[test]
    fn new_with_vertex_count_boundary_0_1_2_sets_internal_vector_lengths() {
        // Arrange
        let lower_vertex_count = 0;
        let center_vertex_count = 1;
        let upper_vertex_count = 2;
        let expected_lengths = vec![(0, 0, 0), (1, 1, 1), (2, 2, 2)];

        // Act
        let lower_dinic = Dinic::new(lower_vertex_count);
        let center_dinic = Dinic::new(center_vertex_count);
        let upper_dinic = Dinic::new(upper_vertex_count);
        let actual_lengths = vec![
            (
                lower_dinic.graph.len(),
                lower_dinic.level.len(),
                lower_dinic.next_edge_index.len(),
            ),
            (
                center_dinic.graph.len(),
                center_dinic.level.len(),
                center_dinic.next_edge_index.len(),
            ),
            (
                upper_dinic.graph.len(),
                upper_dinic.level.len(),
                upper_dinic.next_edge_index.len(),
            ),
        ];

        // Assert
        assert_eq!(actual_lengths, expected_lengths);
    }

    #[test]
    fn add_edge_adds_forward_and_backward_edges_with_expected_capacities() {
        // Arrange
        let vertex_count = 2;
        let from_vertex = 0;
        let to_vertex = 1;
        let capacity_value = 7;
        let expected_forward_capacity = 7;
        let expected_backward_capacity = 0;
        let expected_reverse_index = 0;
        let mut dinic = Dinic::new(vertex_count);

        // Act
        dinic.add_edge(from_vertex, to_vertex, capacity_value);
        let forward_edge = &dinic.graph[from_vertex][0];
        let backward_edge = &dinic.graph[to_vertex][0];
        let forward_to = forward_edge.to;
        let forward_reverse_index = forward_edge.rev;
        let forward_capacity = forward_edge.capacity;
        let backward_to = backward_edge.to;
        let backward_reverse_index = backward_edge.rev;
        let backward_capacity = backward_edge.capacity;

        // Assert
        assert_eq!(forward_to, to_vertex);
        assert_eq!(forward_reverse_index, expected_reverse_index);
        assert_eq!(forward_capacity, expected_forward_capacity);
        assert_eq!(backward_to, from_vertex);
        assert_eq!(backward_reverse_index, expected_reverse_index);
        assert_eq!(backward_capacity, expected_backward_capacity);
    }

    #[test]
    fn add_edge_with_capacity_boundary_0_1_2_sets_forward_edge_capacity_as_input() {
        // Arrange
        let vertex_count = 2;
        let from_vertex = 0;
        let to_vertex = 1;
        let capacity_values = vec![0, 1, 2];
        let expected_forward_capacities = vec![0, 1, 2];
        let mut dinic = Dinic::new(vertex_count);

        // Act
        for &capacity_value in capacity_values.iter() {
            dinic.add_edge(from_vertex, to_vertex, capacity_value);
        }
        let actual_forward_capacities: Vec<i64> = dinic.graph[from_vertex]
            .iter()
            .map(|edge| edge.capacity)
            .collect();

        // Assert
        assert_eq!(actual_forward_capacities, expected_forward_capacities);
    }

    #[test]
    fn add_edge_with_parallel_edges_keeps_all_parallel_edges() {
        // Arrange
        let vertex_count = 2;
        let from_vertex = 0;
        let to_vertex = 1;
        let first_capacity = 2;
        let second_capacity = 3;
        let expected_parallel_count = 2;
        let expected_capacities = vec![2, 3];
        let mut dinic = Dinic::new(vertex_count);

        // Act
        dinic.add_edge(from_vertex, to_vertex, first_capacity);
        dinic.add_edge(from_vertex, to_vertex, second_capacity);
        let actual_parallel_count = dinic.graph[from_vertex].len();
        let actual_capacities: Vec<i64> = dinic.graph[from_vertex]
            .iter()
            .map(|edge| edge.capacity)
            .collect();

        // Assert
        assert_eq!(actual_parallel_count, expected_parallel_count);
        assert_eq!(actual_capacities, expected_capacities);
    }

    #[test]
    fn add_edge_with_vertex_boundary_from_zero_to_last_does_not_panic() {
        // Arrange
        let vertex_count = 4;
        let from_vertex = 0;
        let to_vertex = vertex_count - 1;
        let capacity_value = 1;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut dinic = Dinic::new(vertex_count);
            dinic.add_edge(from_vertex, to_vertex, capacity_value);
        }));
        let did_not_panic = panic_result.is_ok();

        // Assert
        assert!(did_not_panic);
    }

    #[test]
    fn add_edge_with_from_equal_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let from_vertex = vertex_count;
        let to_vertex = 1;
        let capacity_value = 1;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut dinic = Dinic::new(vertex_count);
            dinic.add_edge(from_vertex, to_vertex, capacity_value);
        }));
        let did_panic = panic_result.is_err();

        // Assert
        assert!(did_panic);
    }

    #[test]
    fn add_edge_with_to_equal_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let from_vertex = 0;
        let to_vertex = vertex_count;
        let capacity_value = 1;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut dinic = Dinic::new(vertex_count);
            dinic.add_edge(from_vertex, to_vertex, capacity_value);
        }));
        let did_panic = panic_result.is_err();

        // Assert
        assert!(did_panic);
    }

    #[test]
    fn add_edge_with_negative_capacity_panics() {
        // Arrange
        let vertex_count = 2;
        let from_vertex = 0;
        let to_vertex = 1;
        let capacity_value = -1;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut dinic = Dinic::new(vertex_count);
            dinic.add_edge(from_vertex, to_vertex, capacity_value);
        }));
        let did_panic = panic_result.is_err();

        // Assert
        assert!(did_panic);
    }

    #[test]
    fn max_flow_on_empty_graph_returns_zero() {
        // Arrange
        let vertex_count = 0;
        let source_vertex = 0;
        let sink_vertex = 0;
        let expected_flow = 0;
        let mut dinic = Dinic::new(vertex_count);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_when_source_equals_sink_returns_zero_on_non_empty_graph() {
        // Arrange
        let vertex_count = 2;
        let source_vertex = 0;
        let sink_vertex = 0;
        let expected_flow = 0;
        let mut dinic = Dinic::new(vertex_count);
        dinic.add_edge(0, 1, 5);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_with_source_equal_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let source_vertex = vertex_count;
        let sink_vertex = 1;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut dinic = Dinic::new(vertex_count);
            let _ = dinic.max_flow(source_vertex, sink_vertex);
        }));
        let did_panic = panic_result.is_err();

        // Assert
        assert!(did_panic);
    }

    #[test]
    fn max_flow_with_sink_equal_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let source_vertex = 0;
        let sink_vertex = vertex_count;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let mut dinic = Dinic::new(vertex_count);
            let _ = dinic.max_flow(source_vertex, sink_vertex);
        }));
        let did_panic = panic_result.is_err();

        // Assert
        assert!(did_panic);
    }

    #[test]
    fn max_flow_with_empty_edge_set_and_distinct_source_sink_returns_zero() {
        // Arrange
        let vertex_count = 2;
        let source_vertex = 0;
        let sink_vertex = 1;
        let expected_flow = 0;
        let mut dinic = Dinic::new(vertex_count);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_with_single_edge_capacity_boundary_0_1_2_returns_expected_flows() {
        // Arrange
        let lower_capacity = 0;
        let center_capacity = 1;
        let upper_capacity = 2;
        let expected_flows = vec![0, 1, 2];

        // Act
        let lower_flow = compute_single_edge_max_flow(lower_capacity);
        let center_flow = compute_single_edge_max_flow(center_capacity);
        let upper_flow = compute_single_edge_max_flow(upper_capacity);
        let actual_flows = vec![lower_flow, center_flow, upper_flow];

        // Assert
        assert_eq!(actual_flows, expected_flows);
    }

    #[test]
    fn max_flow_with_parallel_edges_returns_sum_of_parallel_capacities() {
        // Arrange
        let vertex_count = 2;
        let source_vertex = 0;
        let sink_vertex = 1;
        let edges = vec![(0, 1, 2), (0, 1, 3)];
        let expected_flow = 5;
        let mut dinic = build_dinic_with_edges(vertex_count, &edges);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_with_two_disjoint_paths_returns_sum_of_path_bottlenecks() {
        // Arrange
        let vertex_count = 4;
        let source_vertex = 0;
        let sink_vertex = 3;
        let edges = vec![(0, 1, 3), (1, 3, 3), (0, 2, 2), (2, 3, 2)];
        let expected_flow = 5;
        let mut dinic = build_dinic_with_edges(vertex_count, &edges);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_with_intermediate_bottleneck_returns_bottleneck_capacity() {
        // Arrange
        let vertex_count = 4;
        let source_vertex = 0;
        let sink_vertex = 3;
        let edges = vec![(0, 1, 5), (1, 2, 2), (2, 3, 5)];
        let expected_flow = 2;
        let mut dinic = build_dinic_with_edges(vertex_count, &edges);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_with_cycle_graph_returns_correct_flow_without_infinite_loop() {
        // Arrange
        let vertex_count = 4;
        let source_vertex = 0;
        let sink_vertex = 3;
        let edges = vec![(0, 1, 3), (1, 2, 3), (2, 1, 1), (2, 3, 3)];
        let expected_flow = 3;
        let mut dinic = build_dinic_with_edges(vertex_count, &edges);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_rebuilds_level_graph_when_shortest_path_length_changes() {
        // Arrange
        let vertex_count = 5;
        let source_vertex = 0;
        let sink_vertex = 4;
        let edges = vec![(0, 1, 1), (1, 4, 1), (0, 2, 1), (2, 3, 1), (3, 4, 1)];
        let expected_flow = 2;
        let mut dinic = build_dinic_with_edges(vertex_count, &edges);

        // Act
        let actual_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_with_single_edge_capacity_i64_max_returns_i64_max() {
        // Arrange
        let max_capacity = i64::MAX;
        let expected_flow = i64::MAX;

        // Act
        let actual_flow = compute_single_edge_max_flow(max_capacity);

        // Assert
        assert_eq!(actual_flow, expected_flow);
    }

    #[test]
    fn max_flow_called_twice_on_same_instance_returns_zero_on_second_call() {
        // Arrange
        let vertex_count = 2;
        let source_vertex = 0;
        let sink_vertex = 1;
        let capacity_value = 3;
        let expected_first_flow = 3;
        let expected_second_flow = 0;
        let mut dinic = Dinic::new(vertex_count);
        dinic.add_edge(source_vertex, sink_vertex, capacity_value);

        // Act
        let first_flow = dinic.max_flow(source_vertex, sink_vertex);
        let second_flow = dinic.max_flow(source_vertex, sink_vertex);

        // Assert
        assert_eq!(first_flow, expected_first_flow);
        assert_eq!(second_flow, expected_second_flow);
    }
}
