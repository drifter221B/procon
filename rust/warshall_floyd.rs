pub fn warshall_floyd(vertex_number: usize, edges: &[(usize, usize, i64)], is_directed: bool) -> (Vec<Vec<i64>>, bool) {

    let inf = i64::MAX / 4;
    let mut dist = vec![vec![inf; vertex_number]; vertex_number];

    for i in 0..vertex_number {
        dist[i][i] = 0;
    }

    for &(u, v, w) in edges {
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
                if dist[i][k] == inf || dist[k][j] == inf {
                    continue;
                }
                let candidate = dist[i][k].saturating_add(dist[k][j]);
                if candidate < dist[i][j] {
                    dist[i][j] = candidate;
                }
            }
        }
    }

    let mut has_negative_cycle = false;
    for i in 0..vertex_number {
        if dist[i][i] < 0 {
            has_negative_cycle = true;
        }
    }

    (dist, has_negative_cycle)
}

#[cfg(test)]
mod tests {
    use super::warshall_floyd;

    #[test]
    fn empty_graph_returns_empty_distances_without_negative_cycle() {
        // Arrange
        let vertex_count = 0;
        let edges = [];
        let is_directed = false;
        let expected_distances = Vec::<Vec<i64>>::new();

        // Act
        let (actual_distances, actual_has_cycle) =
            warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert_eq!(actual_distances, expected_distances);
        assert!(!actual_has_cycle);
    }

    #[test]
    fn computes_all_pairs_shortest_paths() {
        // Arrange
        let vertex_count = 3;
        let edges = [(0, 1, 2), (1, 2, 3), (0, 2, 10)];
        let is_directed = false;
        let expected_distances = vec![vec![0, 2, 5], vec![2, 0, 3], vec![5, 3, 0]];

        // Act
        let (actual_distances, actual_has_cycle) =
            warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert_eq!(actual_distances, expected_distances);
        assert!(!actual_has_cycle);
    }

    #[test]
    fn detects_negative_cycle() {
        // Arrange
        let vertex_count = 2;
        let edges = [(0, 1, 1), (1, 0, -3)];
        let is_directed = true;

        // Act
        let (_, actual_has_cycle) = warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert!(actual_has_cycle);
    }

    #[test]
    fn respects_directed_flag_for_edge_duplication() {
        // Arrange
        let vertex_count = 2;
        let edges = [(0, 1, 5)];
        let cases = [(false, 5), (true, i64::MAX / 4)];
        let from_vertex = 1;
        let to_vertex = 0;
        let expected_reverse_distances = cases.map(|(_, distance)| distance);

        // Act
        let actual_reverse_distances = cases.map(|(is_directed, _)| {
            let (distances, _) = warshall_floyd(vertex_count, &edges, is_directed);
            distances[from_vertex][to_vertex]
        });

        // Assert
        assert_eq!(actual_reverse_distances, expected_reverse_distances);
    }

    #[test]
    fn parallel_edges_keep_smallest_weight() {
        // Arrange
        let edges = vec![(0, 1, 10), (0, 1, 3)];
        let expected_distance = 3;

        let vertex_count = 2;
        let is_directed = true;

        // Act
        let (actual_distances, _) = warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert_eq!(actual_distances[0][1], expected_distance);
    }

    #[test]
    fn unreachable_pair_keeps_inf_value() {
        // Arrange
        let edges = vec![(0, 1, 2)];
        let expected_distance = i64::MAX / 4;

        let vertex_count = 3;
        let is_directed = true;

        // Act
        let (actual_distances, _) = warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert_eq!(actual_distances[2][0], expected_distance);
    }

    #[test]
    fn out_of_range_edges_are_ignored() {
        // Arrange
        let edges = vec![(0, 5, -1), (5, 0, -1)];
        let expected_distances = vec![vec![0, i64::MAX / 4], vec![i64::MAX / 4, 0]];

        let vertex_count = 2;
        let is_directed = true;

        // Act
        let (actual_distances, actual_has_negative_cycle) =
            warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert!(!actual_has_negative_cycle);
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn negative_self_loop_is_negative_cycle() {
        // Arrange
        let edges = vec![(0, 0, -1)];
        let expected_has_negative_cycle = true;

        let vertex_count = 1;
        let is_directed = true;

        // Act
        let (_, actual_has_negative_cycle) = warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert_eq!(actual_has_negative_cycle, expected_has_negative_cycle);
    }

    #[test]
    fn unreachable_pair_with_negative_edge_keeps_inf_value() {
        // Arrange
        let vertex_count = 3;
        let edges = [(1, 2, -1)];
        let is_directed = true;
        let inf = i64::MAX / 4;
        let expected_distances = vec![vec![0, inf, inf], vec![inf, 0, -1], vec![inf, inf, 0]];

        // Act
        let (actual_distances, actual_has_cycle) =
            warshall_floyd(vertex_count, &edges, is_directed);

        // Assert
        assert_eq!(actual_distances, expected_distances);
        assert!(!actual_has_cycle);
    }

}
