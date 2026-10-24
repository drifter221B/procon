use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologicalResult {
    pub is_unique: bool,
    pub order: Vec<usize>,
}

/// Kahn 法によるトポロジカルソート。
/// - 戻り値は (一意性, トポロジカル順)。閉路がある場合は None を返す。
pub fn topological_sort_bfs(graph: &[Vec<usize>]) -> Option<TopologicalResult> {
    let vertex_count = graph.len();
    let mut sorted_graph = graph.to_vec();
    sorted_graph.iter_mut().for_each(|edges| edges.sort_unstable());

    let mut in_degrees = vec![0usize; vertex_count];
    sorted_graph.iter().for_each(|edges| {
        edges.iter().for_each(|&next_vertex| {
            in_degrees[next_vertex] += 1;
        });
    });

    let mut queue = (0..vertex_count)
        .filter(|&vertex| in_degrees[vertex] == 0)
        .fold(VecDeque::new(), |mut que, vertex| {
            que.push_back(vertex);
            que
        });

    let mut order = Vec::with_capacity(vertex_count);
    let mut is_unique = true;

    while let Some(vertex) = queue.pop_front() {
        is_unique &= queue.is_empty();
        order.push(vertex);

        for &next_vertex in &sorted_graph[vertex] {
            in_degrees[next_vertex] -= 1;
            if in_degrees[next_vertex] == 0 {
                queue.push_back(next_vertex);
            }
        }
    }

    if order.len() == vertex_count {
        Some(TopologicalResult { is_unique, order })
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::topological_sort_bfs;

    #[test]
    fn topological_sort_bfs_returns_empty_order_for_zero_vertices() {
        // Arrange
        let graph: Vec<Vec<usize>> = vec![];
        let expected_is_unique = true;
        let expected_order = vec![];

        // Act
        let actual_result = topological_sort_bfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_is_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn topological_sort_bfs_sorts_single_vertex_graph() {
        // Arrange
        let graph: Vec<Vec<usize>> = vec![vec![]];
        let expected_is_unique = true;
        let expected_order = vec![0];

        // Act
        let actual_result = topological_sort_bfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_is_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn topological_sort_bfs_detects_cycle_and_returns_none() {
        // Arrange
        let graph = vec![vec![1], vec![0]];
        let expected_result = None;

        // Act
        let actual_result = topological_sort_bfs(&graph);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn topological_sort_bfs_marks_non_unique_when_multiple_choices() {
        // Arrange
        let graph = vec![vec![2], vec![2], vec![]];
        let expected_is_unique = false;
        let expected_order = vec![0, 1, 2];

        // Act
        let actual_result = topological_sort_bfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_is_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn disconnected_dag_is_not_unique() {
        // Arrange
        let graph = vec![vec![1], vec![], vec![3], vec![]];
        let expected_is_unique = false;
        let expected_order = vec![0, 2, 1, 3];

        // Act
        let actual_result = topological_sort_bfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_is_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn multi_edges_do_not_break_order() {
        // Arrange
        let graph = vec![vec![1, 1], vec![]];
        let expected_order = vec![0, 1];

        // Act
        let actual_result = topological_sort_bfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn self_loop_returns_none() {
        // Arrange
        let graph = vec![vec![0]];

        // Act
        let actual_result = topological_sort_bfs(&graph);

        // Assert
        assert!(actual_result.is_none());
    }
}
