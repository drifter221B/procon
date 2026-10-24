use std::cmp::Reverse;
use std::collections::BinaryHeap;

/// プリム法で最小全域木の重み合計を求める。連結でない場合は None。
pub fn prim(vertex_number: usize, graph: &[Vec<(usize, i64)>], start_vertex: usize) -> Option<i64> {

    if vertex_number == 0 || start_vertex >= vertex_number || graph.len() < vertex_number {
        return None;
    }

    let mut visited = vec![false; vertex_number];
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

    if visited_count == vertex_number {
        Some(total)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::prim;

    #[test]
    fn returns_none_for_empty_vertex_set() {
        // Arrange
        let vertex_number = 0;
        let graph: Vec<Vec<(usize, i64)>> = vec![];
        let start_vertex = 0;
        let expected_value = None;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn returns_none_when_start_vertex_is_out_of_range() {
        // Arrange
        let vertex_number = 2;
        let graph = vec![vec![(1, 1)], vec![(0, 1)]];
        let start_vertex = 2;
        let expected_value = None;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn returns_none_when_graph_has_fewer_adjacency_lists_than_vertices() {
        // Arrange
        let vertex_number = 2;
        let graph = vec![vec![(1, 1)]];
        let start_vertex = 0;
        let expected_value = None;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn computes_mst_weight_on_small_graph() {
        // Arrange
        let vertex_number = 3;
        let graph = vec![
            vec![(1, 1), (2, 4)],
            vec![(0, 1), (2, 2)],
            vec![(0, 4), (1, 2)],
        ];
        let start_vertex = 0;
        let expected_value = Some(3);

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn returns_none_when_disconnected() {
        // Arrange
        let vertex_number = 3;
        let graph = vec![vec![(1, 1)], vec![(0, 1)], vec![]];
        let start_vertex = 0;
        let expected_value = None;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn handles_single_vertex_without_edges() {
        // Arrange
        let vertex_number = 1;
        let graph = vec![vec![]];
        let start_vertex = 0;
        let expected_value = Some(0);

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn non_zero_start_vertex_keeps_same_mst_weight() {
        // Arrange
        let vertex_number = 3;
        let graph = vec![
            vec![(1, 1), (2, 4)],
            vec![(0, 1), (2, 2)],
            vec![(0, 4), (1, 2)],
        ];
        let expected_value = Some(3);
        let start_vertex = 1;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn parallel_edges_choose_smallest_weight() {
        // Arrange
        let vertex_number = 3;
        let graph = vec![
            vec![(1, 5), (1, 1)],
            vec![(0, 5), (0, 1), (2, 2)],
            vec![(1, 2)],
        ];
        let expected_value = Some(3);
        let start_vertex = 0;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn negative_edges_are_supported() {
        // Arrange
        let vertex_number = 3;
        let graph = vec![
            vec![(1, -2), (2, 10)],
            vec![(0, -2), (2, 3)],
            vec![(0, 10), (1, 3)],
        ];
        let expected_value = Some(1);
        let start_vertex = 0;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn self_loop_is_not_used_in_mst() {
        // Arrange
        let vertex_number = 2;
        let graph = vec![
            vec![(0, -100), (1, 1)],
            vec![(0, 1)],
        ];
        let expected_value = Some(1);
        let start_vertex = 0;

        // Act
        let actual_value = prim(vertex_number, &graph, start_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
