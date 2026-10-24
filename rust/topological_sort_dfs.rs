#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TopologicalResult {
    pub is_unique: bool,
    pub order: Vec<usize>,
}

/// DFS によるトポロジカルソート。
/// - 閉路がある場合は None を返す。
pub fn topological_sort_dfs(graph: &[Vec<usize>]) -> Option<TopologicalResult> {
    let vertex_count = graph.len();
    let mut sorted_graph = graph.to_vec();
    sorted_graph.iter_mut().for_each(|edges| edges.sort_unstable());

    let mut states = vec![0u8; vertex_count]; // 0: 未訪問, 1: 処理中, 2: 処理済み
    let mut order = Vec::with_capacity(vertex_count);
    let mut has_cycle = false;

    for start in 0..vertex_count {
        if states[start] != 0 {
            continue;
        }
        dfs(start, &sorted_graph, &mut states, &mut order, &mut has_cycle);
        if has_cycle {
            return None;
        }
    }

    order.reverse();
    let is_unique = is_unique_order(&sorted_graph, &order);
    Some(TopologicalResult { is_unique, order })
}

fn dfs(
    vertex: usize,
    graph: &[Vec<usize>],
    states: &mut [u8],
    order: &mut Vec<usize>,
    has_cycle: &mut bool,
) {
    states[vertex] = 1;

    for &next_vertex in &graph[vertex] {
        match states[next_vertex] {
            0 => dfs(next_vertex, graph, states, order, has_cycle),
            1 => {
                *has_cycle = true;
                return;
            }
            _ => {}
        }
        if *has_cycle {
            return;
        }
    }

    states[vertex] = 2;
    order.push(vertex);
}

fn is_unique_order(graph: &[Vec<usize>], order: &[usize]) -> bool {
    if order.len() <= 1 {
        return true;
    }
    order.windows(2).all(|pair| {
        let from = pair[0];
        let to = pair[1];
        graph[from].binary_search(&to).is_ok()
    })
}

#[cfg(test)]
mod tests {
    use super::topological_sort_dfs;

    #[test]
    fn topological_sort_dfs_returns_empty_order_for_zero_vertices() {
        // Arrange
        let graph: Vec<Vec<usize>> = vec![];
        let expected_order: Vec<usize> = vec![];
        let expected_unique = true;

        // Act
        let actual_result = topological_sort_dfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn topological_sort_dfs_on_single_vertex_returns_unique_zero_order() {
        // Arrange
        let graph = vec![vec![]];
        let expected_order = vec![0];
        let expected_unique = true;

        // Act
        let actual_result = topological_sort_dfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn topological_sort_dfs_detects_cycle_and_returns_none() {
        // Arrange
        let graph = vec![vec![1], vec![0]];
        let expected_result = None;

        // Act
        let actual_result = topological_sort_dfs(&graph);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn topological_sort_dfs_marks_non_unique_when_multiple_linearizations_exist() {
        // Arrange
        let graph = vec![vec![2], vec![2], vec![]];
        let expected_unique = false;
        let expected_order = vec![1, 0, 2];

        // Act
        let actual_result = topological_sort_dfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn disconnected_dag_is_not_unique() {
        // Arrange
        let graph = vec![vec![1], vec![], vec![3], vec![]];
        let expected_unique = false;
        let expected_order = vec![2, 3, 0, 1];

        // Act
        let actual_result = topological_sort_dfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.is_unique, expected_unique);
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn multi_edges_do_not_break_order() {
        // Arrange
        let graph = vec![vec![1, 1], vec![]];
        let expected_order = vec![0, 1];

        // Act
        let actual_result = topological_sort_dfs(&graph).unwrap();

        // Assert
        assert_eq!(actual_result.order, expected_order);
    }

    #[test]
    fn self_loop_returns_none() {
        // Arrange
        let graph = vec![vec![0]];

        // Act
        let actual_result = topological_sort_dfs(&graph);

        // Assert
        assert!(actual_result.is_none());
    }
}
