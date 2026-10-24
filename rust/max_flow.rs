use std::cmp::min;
use std::collections::VecDeque;

#[derive(Clone, Debug)]
struct ResidualEdge {
    to: usize,
    rev: usize,
    capacity: i64,
}

/// Computes the maximum flow in a directed graph using Dinic's algorithm.
///
/// # Arguments
///
/// * `vertex_count` - The total number of vertices in the graph.
///   **This count must include both the source and sink vertices.**
///   Therefore, valid vertex indices are in the range `0..vertex_count`.
/// * `edges` - A list of directed edges represented as `(from, to, capacity)`.
/// * `source_vertex` - The index of the source vertex.
/// * `sink_vertex` - The index of the sink vertex.
///
/// # Important
///
/// `vertex_count` must represent the total number of vertices in the graph,
/// including the source and sink vertices.
///
/// For example, if the graph has `m + n` regular vertices and one additional
/// source and sink vertex, the vertex layout can be:
///
/// ```text
/// 0 .. m + n - 1 : regular vertices
/// m + n          : source
/// m + n + 1      : sink
/// ```
///
/// In this case, `vertex_count` must be `m + n + 2`.
///
/// # Example
///
/// ```ignore
/// let source = m + n;
/// let sink = m + n + 1;
/// let vertex_count = m + n + 2;
///
/// let max_flow = dinic_max_flow(
///     vertex_count,
///     &edges,
///     source,
///     sink,
/// );
/// ```
pub fn dinic_max_flow(vertex_count: usize, edges: &[(usize, usize, i64)], source_vertex: usize, sink_vertex: usize) -> i64 {
    let (total_flow, _) = run_dinic(vertex_count, edges, source_vertex, sink_vertex);

    total_flow
}

/// Computes the minimum cut in a directed graph using Dinic's algorithm.
///
/// # Arguments
///
/// * `vertex_count` - The total number of vertices in the graph.
///   **This count must include both the source and sink vertices.**
///   Therefore, valid vertex indices are in the range `0..vertex_count`.
/// * `edges` - A list of directed edges represented as `(from, to, capacity)`.
/// * `source_vertex` - The index of the source vertex.
/// * `sink_vertex` - The index of the sink vertex.
///
/// # Returns
///
/// Returns a tuple containing:
///
/// * The minimum cut value, which is equal to the maximum flow.
/// * A `Vec<bool>` indicating whether each vertex is reachable from the source
///   in the final residual graph.
/// * A list of edges `(from, to, capacity)` that cross the minimum cut from
///   the source side to the sink side.
///
/// # Important
///
/// `vertex_count` must represent the total number of vertices in the graph,
/// including the source and sink vertices.
///
/// For example, if the graph has `m + n` regular vertices and one additional
/// source and sink vertex, the vertex layout can be:
///
/// ```text
/// 0 .. m + n - 1 : regular vertices
/// m + n          : source
/// m + n + 1      : sink
/// ```
///
/// In this case, `vertex_count` must be `m + n + 2`.
///
/// # Example
///
/// ```ignore
/// let source = m + n;
/// let sink = m + n + 1;
/// let vertex_count = m + n + 2;
///
/// let (cut_value, reachable, cut_edges) = dinic_min_cut(
///     vertex_count,
///     &edges,
///     source,
///     sink,
/// );
/// ```
pub fn dinic_min_cut(vertex_count: usize, edges: &[(usize, usize, i64)], source_vertex: usize, sink_vertex: usize) -> (i64, Vec<bool>, Vec<(usize, usize, i64)>) {

    let (cut_value, residual_graph) = run_dinic(vertex_count, edges, source_vertex, sink_vertex);

    if vertex_count == 0 {
        return (0, Vec::new(), Vec::new());
    }

    let reachable_from_source = collect_reachable_vertices(source_vertex, &residual_graph);
    let cut_edges = extract_cut_edges(edges, &reachable_from_source);

    (cut_value, reachable_from_source, cut_edges)
}

fn run_dinic(vertex_count: usize, edges: &[(usize, usize, i64)], source_vertex: usize, sink_vertex: usize) -> (i64, Vec<Vec<ResidualEdge>>) {

    if vertex_count == 0 {
        return (0, Vec::new());
    }

    if source_vertex == sink_vertex {
        return (0, vec![Vec::new(); vertex_count]);
    }

    if source_vertex < vertex_count && sink_vertex < vertex_count {
        // valid
    } else {
        panic!("source_vertex と sink_vertex は 0 以上かつ vertex_count 未満にしてください");
    }

    let mut residual_graph = build_residual_graph(vertex_count, edges);
    let mut total_flow = 0;
    let mut level_of_vertex = vec![None; vertex_count];
    let mut next_search_start = vec![0usize; vertex_count];

    loop {

        let has_path_to_sink = build_level_graph(source_vertex, sink_vertex, &residual_graph, &mut level_of_vertex);

        if !has_path_to_sink {
            break;
        }

        next_search_start.fill(0);

        loop {

            let additional_flow = push_augmenting_flow(
                source_vertex,
                sink_vertex,
                i64::MAX,
                &mut residual_graph,
                &level_of_vertex,
                &mut next_search_start,
            );

            if additional_flow == 0 {
                break;
            }

            total_flow += additional_flow;
        }
    }

    (total_flow, residual_graph)
}

fn build_residual_graph(vertex_count: usize, edges: &[(usize, usize, i64)]) -> Vec<Vec<ResidualEdge>> {

    let mut graph = vec![Vec::new(); vertex_count];

    for &(from, to, capacity) in edges.iter() {

        if from < vertex_count && to < vertex_count {
            // valid
        } else {
            panic!("from, to は 0 以上かつ vertex_count 未満にしてください");
        }

        if capacity < 0 {
            panic!("capacity は 0 以上にしてください");
        }

        append_edge(&mut graph, from, to, capacity);
    }

    graph
}

fn append_edge(graph: &mut [Vec<ResidualEdge>], from: usize, to: usize, capacity: i64) {

    let forward_edge = ResidualEdge {
        to,
        rev: graph[to].len(),
        capacity,
    };

    let backward_edge = ResidualEdge {
        to: from,
        rev: graph[from].len(),
        capacity: 0,
    };

    graph[from].push(forward_edge);
    graph[to].push(backward_edge);
}

fn collect_reachable_vertices(source_vertex: usize, residual_graph: &[Vec<ResidualEdge>]) -> Vec<bool> {

    if residual_graph.is_empty() {
        return Vec::new();
    }

    let mut reachable = vec![false; residual_graph.len()];
    let mut queue = VecDeque::new();

    reachable[source_vertex] = true;
    queue.push_back(source_vertex);

    while let Some(current_vertex) = queue.pop_front() {

        for edge in residual_graph[current_vertex].iter() {

            if edge.capacity <= 0 {
                continue;
            }

            if !reachable[edge.to] {
                reachable[edge.to] = true;
                queue.push_back(edge.to);
            }
        }
    }

    reachable
}

fn extract_cut_edges(edges: &[(usize, usize, i64)], reachable_from_source: &[bool]) -> Vec<(usize, usize, i64)> {

    let mut cut_edges = Vec::new();

    for &(from, to, capacity) in edges.iter() {

        if capacity <= 0 {
            continue;
        }

        if let (Some(&true), Some(&false)) = (reachable_from_source.get(from), reachable_from_source.get(to)) {
            cut_edges.push((from, to, capacity));
        }
    }

    cut_edges
}

fn build_level_graph(
    source_vertex: usize,
    sink_vertex: usize,
    graph: &[Vec<ResidualEdge>],
    level_of_vertex: &mut [Option<usize>],
) -> bool {

    level_of_vertex.fill(None);

    let mut queue = VecDeque::new();
    level_of_vertex[source_vertex] = Some(0);
    queue.push_back(source_vertex);

    while let Some(current_vertex) = queue.pop_front() {

        let current_level = level_of_vertex[current_vertex].expect("レベル未設定の頂点をキューに入れないでください");

        for edge in graph[current_vertex].iter() {

            if edge.capacity <= 0 {
                continue;
            }

            if level_of_vertex[edge.to].is_none() {
                level_of_vertex[edge.to] = Some(current_level + 1);
                queue.push_back(edge.to);
            }
        }
    }

    level_of_vertex[sink_vertex].is_some()
}

fn push_augmenting_flow(
    current_vertex: usize,
    sink_vertex: usize,
    flow_limit: i64,
    graph: &mut [Vec<ResidualEdge>],
    level_of_vertex: &[Option<usize>],
    next_search_start: &mut [usize],
) -> i64 {

    if current_vertex == sink_vertex {
        return flow_limit;
    }

    while next_search_start[current_vertex] < graph[current_vertex].len() {

        let edge_index = next_search_start[current_vertex];

        let (to, remaining_capacity, reverse_index) = {
            let edge = &graph[current_vertex][edge_index];
            (edge.to, edge.capacity, edge.rev)
        };

        if remaining_capacity <= 0 {
            next_search_start[current_vertex] += 1;
            continue;
        }

        let current_level = level_of_vertex[current_vertex].unwrap_or(usize::MAX);
        let next_level = level_of_vertex[to].unwrap_or(usize::MAX);

        if next_level != current_level + 1 {
            next_search_start[current_vertex] += 1;
            continue;
        }

        let possible_flow = push_augmenting_flow(
            to,
            sink_vertex,
            min(flow_limit, remaining_capacity),
            graph,
            level_of_vertex,
            next_search_start,
        );

        if possible_flow > 0 {

            graph[current_vertex][edge_index].capacity -= possible_flow;
            graph[to][reverse_index].capacity += possible_flow;

            return possible_flow;
        }

        next_search_start[current_vertex] += 1;
    }

    0
}

#[cfg(test)]
mod tests {

    use super::{dinic_max_flow, dinic_min_cut};

    #[test]
    fn dinic_max_flow_returns_zero_when_graph_has_no_edges() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;

        let vertex_count = 2;
        let edges: Vec<(usize, usize, i64)> = vec![];
        let expected_check_1 = 0;

        // Act
        let flow = dinic_max_flow(vertex_count, &edges, input_value_1, input_value_2);

        // Assert
        assert_eq!(flow, expected_check_1);
    }

    #[test]
    fn dinic_max_flow_respects_single_edge_capacity_at_0_1_and_2() {
        // Arrange
        let vertex_count = 2;
        let source = 0;
        let sink = 1;
        let capacities = [0, 1, 2];
        let expected = [0, 1, 2];
        let edge_cases = capacities.map(|capacity| vec![(source, sink, capacity)]);

        // Act
        let actual = edge_cases.map(|edges| dinic_max_flow(vertex_count, &edges, source, sink));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn dinic_max_flow_combines_parallel_edges_with_and_without_zero_capacity() {
        // Arrange
        let vertex_count = 2;
        let source = 0;
        let sink = 1;
        let edge_cases = [vec![(0, 1, 2), (0, 1, 3)], vec![(0, 1, 0), (0, 1, 3)]];
        let expected = [5, 3];

        // Act
        let actual = edge_cases.map(|edges| dinic_max_flow(vertex_count, &edges, source, sink));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn dinic_max_flow_handles_two_step_paths_with_bottleneck_boundaries() {
        // Arrange
        let vertex_count = 4;
        let source = 0;
        let sink = 3;
        let edge_cases = [
            vec![(0, 1, 2), (0, 2, 2), (1, 3, 0), (2, 3, 2)],
            vec![(0, 1, 2), (0, 2, 2), (1, 3, 1), (2, 3, 2)],
            vec![(0, 1, 2), (0, 2, 2), (1, 3, 3), (2, 3, 3)],
        ];
        let expected = [2, 3, 4];

        // Act
        let actual = edge_cases.map(|edges| dinic_max_flow(vertex_count, &edges, source, sink));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn dinic_max_flow_returns_zero_for_empty_vertex_set() {
        // Arrange
        let vertex_count = 0;
        let edges: Vec<(usize, usize, i64)> = vec![];
        let source_vertex = 0;
        let sink_vertex = 0;
        let expected_check_1 = 0;

        // Act
        let total_flow = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(total_flow, expected_check_1);
    }

    #[test]
    fn dinic_max_flow_returns_zero_when_source_equals_sink() {
        // Arrange
        let vertex_count = 3;
        let edges = vec![(0, 1, 5), (1, 2, 4)];
        let source_vertex = 1;
        let sink_vertex = 1;
        let expected_check_1 = 0;

        // Act
        let total_flow = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(total_flow, expected_check_1);
    }

    #[test]
    fn dinic_max_flow_panics_when_source_vertex_is_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 1, 1)];
        let source_vertex = 2;
        let sink_vertex = 1;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_max_flow_panics_when_sink_vertex_is_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 1, 1)];
        let source_vertex = 0;
        let sink_vertex = 2;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_max_flow_panics_when_edge_from_is_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(2, 1, 1)];
        let source_vertex = 0;
        let sink_vertex = 1;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_max_flow_panics_when_edge_to_is_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 2, 1)];
        let source_vertex = 0;
        let sink_vertex = 1;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_max_flow_panics_when_edge_capacity_is_negative() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 1, -1)];
        let source_vertex = 0;
        let sink_vertex = 1;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_max_flow_returns_zero_when_sink_is_unreachable_from_source() {
        // Arrange
        let vertex_count = 4;
        let edges = vec![(2, 3, 10)];
        let source_vertex = 0;
        let sink_vertex = 1;
        let expected_check_1 = 0;

        // Act
        let total_flow = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(total_flow, expected_check_1);
    }

    #[test]
    fn dinic_max_flow_computes_correct_value_on_graph_with_cycle() {
        // Arrange
        let vertex_count = 4;
        let edges = vec![
            (0, 1, 2),
            (0, 2, 2),
            (1, 2, 1),
            (2, 1, 1),
            (1, 3, 2),
            (2, 3, 2),
        ];
        let source_vertex = 0;
        let sink_vertex = 3;
        let expected_check_1 = 4;

        // Act
        let total_flow = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(total_flow, expected_check_1);
    }

    #[test]
    fn dinic_max_flow_rebuilds_level_graph_until_optimal_flow() {
        // Arrange
        let vertex_count = 4;
        let edges = vec![(0, 1, 2), (0, 2, 1), (1, 2, 1), (1, 3, 1), (2, 3, 2)];
        let source_vertex = 0;
        let sink_vertex = 3;
        let expected_check_1 = 3;

        // Act
        let total_flow = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(total_flow, expected_check_1);
    }

    #[test]
    fn dinic_max_flow_handles_single_edge_with_i64_max_capacity() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 1, i64::MAX)];
        let source_vertex = 0;
        let sink_vertex = 1;
        let expected_check_1 = i64::MAX;

        // Act
        let total_flow = dinic_max_flow(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(total_flow, expected_check_1);
    }

    #[test]
    fn dinic_min_cut_returns_zero_and_empty_cut_when_graph_has_no_edges() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;

        let vertex_count = 2;
        let edges: Vec<(usize, usize, i64)> = vec![];
        let expected_check_1 = 0;
        let expected_check_2 = vec![true, false];

        let expected_boolean_1 = true;

        // Act
        let (cut_value, reachable, cut_edges) =
            dinic_min_cut(vertex_count, &edges, input_value_1, input_value_2);

        let actual_check_3 = cut_edges.is_empty();

        // Assert
        assert_eq!(cut_value, expected_check_1);
        assert_eq!(reachable, expected_check_2);
        assert_eq!(actual_check_3, expected_boolean_1);
    }

    #[test]
    fn dinic_min_cut_identifies_single_edge_cut_at_capacity_0_1_and_2() {
        // Arrange
        let vertex_count = 2;
        let source = 0;
        let sink = 1;
        let edge_cases = [vec![(0, 1, 0)], vec![(0, 1, 1)], vec![(0, 1, 2)]];
        let expected = [
            (0, vec![true, false], vec![]),
            (1, vec![true, false], vec![(0, 1, 1)]),
            (2, vec![true, false], vec![(0, 1, 2)]),
        ];

        // Act
        let actual = edge_cases.map(|edges| dinic_min_cut(vertex_count, &edges, source, sink));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn dinic_min_cut_collects_parallel_edges_leaving_reachable_side() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 2;
        let input_value_3 = 0;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let input_value_6 = 0;
        let input_value_7 = 2;
        let input_value_8 = 2;

        let vertex_count = 3;
        let edges = vec![(0, 1, 2), (0, 2, 2), (1, 2, 2)];
        let expected_check_1 = 4;
        let expected_check_2 = vec![true, false, false];
        let expected_check_3 = 2;

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        let (cut_value, reachable, cut_edges) =
            dinic_min_cut(vertex_count, &edges, input_value_1, input_value_2);

        let actual_check_3 = cut_edges.len();
        let actual_check_4 = cut_edges.contains(&(input_value_3, input_value_4, input_value_5));
        let actual_check_5 = cut_edges.contains(&(input_value_6, input_value_7, input_value_8));

        // Assert
        assert_eq!(cut_value, expected_check_1);
        assert_eq!(reachable, expected_check_2);
        assert_eq!(actual_check_3, expected_check_3);
        assert_eq!(actual_check_4, expected_boolean_1);
        assert_eq!(actual_check_5, expected_boolean_2);
    }

    #[test]
    fn dinic_min_cut_separates_two_step_paths_with_bottleneck_boundary() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 3;
        let input_value_3 = 0;
        let input_value_4 = 2;
        let input_value_5 = 2;
        let input_value_6 = 1;
        let input_value_7 = 3;
        let input_value_8 = 1;

        let vertex_count = 4;
        let edges: Vec<(usize, usize, i64)> = vec![(0, 1, 2), (0, 2, 2), (1, 3, 1), (2, 3, 2)];
        let expected_check_1 = 3;
        let expected_check_2 = vec![true, true, false, false];
        let expected_check_3 = 2;

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        let (cut_value, reachable, cut_edges) =
            dinic_min_cut(vertex_count, &edges, input_value_1, input_value_2);

        let actual_check_3 = cut_edges.len();
        let actual_check_4 = cut_edges.contains(&(input_value_3, input_value_4, input_value_5));
        let actual_check_5 = cut_edges.contains(&(input_value_6, input_value_7, input_value_8));

        // Assert
        assert_eq!(cut_value, expected_check_1);
        assert_eq!(reachable, expected_check_2);
        assert_eq!(actual_check_3, expected_check_3);
        assert_eq!(actual_check_4, expected_boolean_1);
        assert_eq!(actual_check_5, expected_boolean_2);
    }

    #[test]
    fn dinic_min_cut_returns_empty_structures_for_empty_vertex_set() {
        // Arrange
        let vertex_count = 0;
        let edges: Vec<(usize, usize, i64)> = vec![];
        let source_vertex = 0;
        let sink_vertex = 0;
        let expected_check_1 = 0;
        let expected_check_2 = Vec::<bool>::new();
        let expected_check_3 = Vec::<(usize, usize, i64)>::new();

        // Act
        let (cut_value, reachable, cut_edges) =
            dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);

        // Assert
        assert_eq!(cut_value, expected_check_1);
        assert_eq!(reachable, expected_check_2);
        assert_eq!(cut_edges, expected_check_3);
    }

    #[test]
    fn dinic_min_cut_marks_source_reachable_when_source_equals_sink() {
        // Arrange
        let vertex_count = 3;
        let edges = vec![(1, 0, 7), (1, 2, 6)];
        let source_vertex = 1;
        let sink_vertex = 1;
        let expected_check_1 = 0;

        let expected_boolean_1 = true;

        // Act
        let (cut_value, reachable, _) =
            dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);

        let actual_check_2 = reachable[source_vertex];

        // Assert
        assert_eq!(cut_value, expected_check_1);
        assert_eq!(actual_check_2, expected_boolean_1);
    }

    #[test]
    fn dinic_min_cut_panics_when_source_vertex_is_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 1, 1)];
        let source_vertex = 2;
        let sink_vertex = 1;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_min_cut_panics_when_sink_vertex_is_out_of_range() {
        // Arrange
        let vertex_count = 2;
        let edges = vec![(0, 1, 1)];
        let source_vertex = 0;
        let sink_vertex = 2;

        let expected_boolean_1 = true;

        // Act
        let panic_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let _ = dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_min_cut_excludes_zero_capacity_edges_from_cut_edges() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 0;
        let vertex_count = 3;
        let edges = vec![(0, 1, 0), (0, 2, 2), (1, 2, 1)];
        let source_vertex = 0;
        let sink_vertex = 2;

        let expected_boolean_1 = false;

        // Act
        let (_, _, cut_edges) = dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);

        let actual_check_1 = cut_edges.contains(&(input_value_1, input_value_2, input_value_3));

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_min_cut_collects_edges_only_from_reachable_side_to_non_reachable_side() {
        // Arrange
        let vertex_count = 4;
        let edges = vec![(0, 1, 3), (1, 0, 4), (1, 3, 2), (0, 2, 1), (2, 3, 1)];
        let source_vertex = 0;
        let sink_vertex = 3;

        let expected_boolean_1 = true;

        // Act
        let (_, reachable, cut_edges) =
            dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);

        let actual_check_1 = cut_edges
            .iter()
            .all(|(from, to, _)| reachable[*from] && !reachable[*to]);

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn dinic_min_cut_cut_value_matches_sum_of_cut_edge_capacities() {
        // Arrange
        let vertex_count = 4;
        let edges = vec![(0, 1, 2), (0, 2, 3), (1, 3, 2), (2, 3, 3)];
        let source_vertex = 0;
        let sink_vertex = 3;

        // Act
        let (cut_value, _, cut_edges) =
            dinic_min_cut(vertex_count, &edges, source_vertex, sink_vertex);
        let sum_of_cut_capacities: i64 = cut_edges.iter().map(|edge| edge.2).sum();

        // Assert
        assert_eq!(cut_value, sum_of_cut_capacities);
    }
}
