fn bfs(start_vertex: usize, graph: &Vec<Vec<usize>>) -> Vec<usize> {

    let n = graph.len();

    let mut dist = vec![!0; n];
    dist[start_vertex] = 0;
    let mut que = std::collections::VecDeque::new();
    que.push_back(start_vertex);

    while let Some(vertex) = que.pop_front() {

        for &next_vertex in graph[vertex].iter() {

            if dist[next_vertex] != !0 {
                continue;
            }

            dist[next_vertex] = dist[vertex] + 1;
            que.push_back(next_vertex);
        }
    }

    dist
}

#[cfg(test)]
mod tests {
    use super::bfs;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn bfs_from_isolated_vertex_returns_zero_for_start_and_inf_for_other() {
        // Arrange
        let graph = vec![vec![], vec![]];
        let start = 0;
        let expected_distances = vec![0, usize::MAX];
        // Act
        let actual_distances = bfs(start, &graph);
        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn bfs_on_branching_graph_returns_shortest_edge_counts() {
        // Arrange
        let graph = vec![vec![1, 2], vec![2], vec![3], vec![]];
        let start = 0;
        let expected_distances = vec![0, 1, 1, 2];
        // Act
        let actual_distances = bfs(start, &graph);
        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn non_zero_start_vertex_computes_shortest_edge_counts() {
        // Arrange
        let graph = vec![vec![1], vec![2], vec![3], vec![]];
        let start_vertex = 1;
        let expected_distances = vec![usize::MAX, 0, 1, 2];
        // Act
        let actual_distances = bfs(start_vertex, &graph);
        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn cycle_graph_finishes_without_revisiting_vertices() {
        // Arrange
        let graph = vec![vec![1], vec![2], vec![0, 3], vec![]];
        let start = 0;
        let expected_distances = vec![0, 1, 2, 3];
        // Act
        let actual_distances = bfs(start, &graph);
        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn out_of_range_start_vertex_panics() {
        // Arrange
        let graph = vec![vec![]];
        let start = graph.len();
        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| bfs(start, &graph)));
        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bfs_on_empty_graph_panics_for_start_zero() {
        // Arrange
        let graph: Vec<Vec<usize>> = vec![];
        let start = 0;
        // Act
        let actual_result = catch_unwind(|| bfs(start, &graph));
        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn bfs_on_single_vertex_returns_zero_distance() {
        // Arrange
        let graph = vec![vec![]];
        let start = 0;
        let expected_distances = vec![0];
        // Act
        let actual_distances = bfs(start, &graph);
        // Assert
        assert_eq!(actual_distances, expected_distances);
    }
}
