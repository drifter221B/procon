fn dijkstra(start_vertex: usize, graph: &[Vec<(usize, usize)>]) -> Vec<usize> {
    let n = graph.len();

    let mut dist = vec![usize::MAX; n];
    dist[start_vertex] = 0;

    let mut heap = std::collections::BinaryHeap::new();
    heap.push(std::cmp::Reverse((0, start_vertex)));

    while let Some(std::cmp::Reverse((cost, vertex))) = heap.pop() {
        if dist[vertex] < cost {
            continue;
        }

        for &(next_vertex, weight) in graph[vertex].iter() {
            if let Some(next_cost) = cost.checked_add(weight) {
                if next_cost < dist[next_vertex] {
                    dist[next_vertex] = next_cost;
                    heap.push(std::cmp::Reverse((next_cost, next_vertex)));
                }
            }
        }
    }

    dist
}

#[cfg(test)]
mod tests {
    use super::dijkstra;

    #[test]
    fn computes_shortest_paths_on_simple_graph() {
        // Arrange
        let graph = vec![vec![(1, 2), (2, 5)], vec![(2, 1)], vec![]];
        let start_vertex = 0;
        let expected_distances = vec![0, 2, 3];

        // Act
        let actual_distances = dijkstra(start_vertex, &graph);

        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn non_zero_start_vertex_computes_shortest_path() {
        // Arrange
        let graph = vec![vec![], vec![(2, 4), (3, 10)], vec![(3, 1)], vec![]];
        let start_vertex = 1;
        let expected_distances = vec![usize::MAX, 0, 4, 5];

        // Act
        let actual_distances = dijkstra(start_vertex, &graph);

        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn returns_inf_for_unreachable() {
        // Arrange
        let graph = vec![vec![], vec![]];
        let start_vertex = 0;
        let unreachable_vertex = 1;
        let expected_distance = usize::MAX;

        // Act
        let actual_distances = dijkstra(start_vertex, &graph);
        let actual_distance = actual_distances[unreachable_vertex];

        // Assert
        assert_eq!(actual_distance, expected_distance);
    }

    #[test]
    fn overflowing_path_is_ignored_by_checked_add() {
        // Arrange
        let graph = vec![vec![(1, usize::MAX - 1), (2, 7)], vec![(2, 2)], vec![]];
        let start_vertex = 0;
        let last_vertex = 2;
        let expected_distance_to_last = 7;

        // Act
        let actual_distances = dijkstra(start_vertex, &graph);

        // Assert
        assert_eq!(actual_distances[last_vertex], expected_distance_to_last);
    }

    #[test]
    fn stale_heap_entries_do_not_break_shortest_path() {
        // Arrange
        let graph = vec![
            vec![(1, 10), (2, 1)],
            vec![(3, 1)],
            vec![(1, 1), (3, 10)],
            vec![],
        ];
        let start_vertex = 0;
        let expected_distances = vec![0, 2, 1, 3];

        // Act
        let actual_distances = dijkstra(start_vertex, &graph);

        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn empty_graph_with_start_zero_panics() {
        // Arrange
        let graph: Vec<Vec<(usize, usize)>> = vec![];
        let start_vertex = 0;

        // Act
        let actual_result = std::panic::catch_unwind(|| dijkstra(start_vertex, &graph));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn one_vertex_graph_without_edges_returns_zero_distance() {
        // Arrange
        let graph = vec![vec![]];
        let start_vertex = 0;
        let expected_distances = vec![0];

        // Act
        let actual_distances = dijkstra(start_vertex, &graph);

        // Assert
        assert_eq!(actual_distances, expected_distances);
    }

    #[test]
    fn start_vertex_equal_to_graph_length_panics() {
        // Arrange
        let graph = vec![vec![]];
        let start_vertex = graph.len();

        // Act
        let actual_result = std::panic::catch_unwind(|| dijkstra(start_vertex, &graph));

        // Assert
        assert!(actual_result.is_err());
    }
}
