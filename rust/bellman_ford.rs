#[derive(Clone, Copy, Debug)]
pub struct Edge {
    pub from: usize,
    pub to: usize,
    pub weight: i64,
}

/// ベルマンフォード法で単一始点最短路と負閉路の有無を返す。
pub fn bellman_ford(vertex_number: usize, start_vertex: usize, edges: &[Edge]) -> (Vec<i64>, bool) {

    let inf = i64::MAX / 4;
    let threshold = i64::MAX / 8;
    let mut dist = vec![inf; vertex_number];
    if start_vertex < vertex_number {
        dist[start_vertex] = 0;
    }

    let mut updated = false;
    for _ in 0..vertex_number {
        updated = false;

        for edge in edges.iter() {

            if vertex_number <= edge.from || vertex_number <= edge.to {
                continue;
            }

            if dist[edge.from] > threshold {
                continue;
            }

            let candidate = dist[edge.from].saturating_add(edge.weight);
            if candidate < dist[edge.to] {
                dist[edge.to] = candidate;
                updated = true;
            }
        }

        if !updated {
            break;
        }
    }

    let mut has_negative_cycle = false;
    if updated {
        for edge in edges.iter() {

            if vertex_number <= edge.from || vertex_number <= edge.to {
                continue;
            }

            if dist[edge.from] < inf && dist[edge.from].saturating_add(edge.weight) < dist[edge.to] {
                has_negative_cycle = true;
                break;
            }
        }
    }

    (dist, has_negative_cycle)
}

#[cfg(test)]
mod tests {
    use super::{bellman_ford, Edge};

    #[test]
    fn handles_disconnected_graph_and_start_out_of_range() {
        // Arrange
        let edges = vec![Edge {
            from: 0,
            to: 1,
            weight: 5,
        }];
        let vertex_count = 3;
        let start_vertex = 3;
        let expected_distance = i64::MAX / 4;
        let expected_has_cycle = false;

        // Act
        let (actual_distances, actual_has_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_has_cycle, expected_has_cycle);
        assert_eq!(actual_distances[0], expected_distance);
    }

    #[test]
    fn computes_shortest_paths_without_negative_cycle() {
        // Arrange
        let edges = vec![
            Edge {
                from: 0,
                to: 1,
                weight: 2,
            },
            Edge {
                from: 1,
                to: 2,
                weight: 3,
            },
        ];

        let vertex_count = 3;
        let start_vertex = 0;
        let expected_distances = vec![0, 2, 5];
        let expected_has_cycle = false;

        // Act
        let (actual_distances, actual_has_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_has_cycle, expected_has_cycle);
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn detects_negative_cycle_reachable_from_start() {
        // Arrange
        let edges = vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1,
            },
            Edge {
                from: 1,
                to: 2,
                weight: -3,
            },
            Edge {
                from: 2,
                to: 1,
                weight: 1,
            },
        ];

        let vertex_count = 3;
        let start_vertex = 0;
        let expected_has_cycle = true;

        // Act
        let (_, actual_has_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_has_cycle, expected_has_cycle);
    }

    #[test]
    fn empty_graph_returns_empty_distances_and_no_negative_cycle() {
        // Arrange
        let vertex_count = 0;
        let start_vertex = 0;
        let edges = vec![];
        let expected_distances = vec![];
        let expected_has_cycle = false;

        // Act
        let (actual_distances, actual_has_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_distances, expected_distances);
        assert_eq!(actual_has_cycle, expected_has_cycle);
    }

    #[test]
    fn ignores_edges_with_out_of_bounds_vertices() {
        // Arrange
        let edges = vec![
            Edge {
                from: 0,
                to: 5,
                weight: 1,
            },
            Edge {
                from: 5,
                to: 0,
                weight: -10,
            },
        ];

        let vertex_count = 2;
        let start_vertex = 0;
        let expected_distances = vec![0, i64::MAX / 4];
        let expected_has_cycle = false;

        // Act
        let (actual_distances, actual_has_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_has_cycle, expected_has_cycle);
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn unreachable_negative_cycle_is_not_detected() {
        // Arrange
        let edges = vec![
            Edge {
                from: 0,
                to: 1,
                weight: 1,
            },
            Edge {
                from: 2,
                to: 3,
                weight: -2,
            },
            Edge {
                from: 3,
                to: 2,
                weight: -2,
            },
        ];
        let vertex_count = 4;
        let start_vertex = 0;
        let expected_has_negative_cycle = false;

        // Act
        let (_, actual_has_negative_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_has_negative_cycle, expected_has_negative_cycle);
    }

    #[test]
    fn smallest_parallel_edge_is_used_for_shortest_path() {
        // Arrange
        let edges = vec![
            Edge {
                from: 0,
                to: 1,
                weight: 10,
            },
            Edge {
                from: 0,
                to: 1,
                weight: 3,
            },
            Edge {
                from: 1,
                to: 2,
                weight: 4,
            },
        ];
        let expected_distances = vec![0, 3, 7];

        // Act
        let (actual_distances, actual_has_negative_cycle) = bellman_ford(3, 0, &edges);

        // Assert
        assert!(!actual_has_negative_cycle);
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn negative_edges_without_negative_cycle_update_distances() {
        // Arrange
        let edges = vec![
            Edge {
                from: 0,
                to: 1,
                weight: 4,
            },
            Edge {
                from: 0,
                to: 2,
                weight: 5,
            },
            Edge {
                from: 1,
                to: 2,
                weight: -2,
            },
            Edge {
                from: 2,
                to: 3,
                weight: 3,
            },
        ];
        let expected_distances = vec![0, 4, 2, 5];

        // Act
        let (actual_distances, actual_has_negative_cycle) = bellman_ford(4, 0, &edges);

        // Assert
        assert!(!actual_has_negative_cycle);
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn one_vertex_graph_without_edges_returns_zero_distance_and_no_negative_cycle() {
        // Arrange
        let vertex_count = 1;
        let start_vertex = 0;
        let edges = vec![];
        let expected_distances = vec![0];
        let expected_has_cycle = false;

        // Act
        let (actual_distances, actual_has_cycle) = bellman_ford(vertex_count, start_vertex, &edges);

        // Assert
        assert_eq!(actual_distances, expected_distances);
        assert_eq!(actual_has_cycle, expected_has_cycle);
    }
}
