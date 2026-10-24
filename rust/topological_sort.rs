use std::collections::VecDeque;

fn topological_sort(graph: &mut Vec<Vec<usize>>) -> (bool, Vec<usize>) {
    let n = graph.len();
    (0..n).for_each(|i| graph[i].sort_unstable());

    let mut in_deg = vec![0i32; n];
    for next_vertices in graph.iter() {
        for &next_vertex in next_vertices.iter() {
            in_deg[next_vertex] += 1;
        }
    }

    let mut que = (0..n)
        .filter(|v| in_deg[*v] == 0)
        .fold(VecDeque::new(), |mut que, v| {
            que.push_back(v);

            que
        });

    let mut is_unique = true;
    let mut orders = Vec::with_capacity(n);

    while let Some(vertex) = que.pop_front() {
        is_unique &= que.is_empty();
        orders.push(vertex);

        for &next_vertex in graph[vertex].iter() {
            in_deg[next_vertex] -= 1;

            if in_deg[next_vertex] == 0 {
                que.push_back(next_vertex);
            }
        }
    }

    (is_unique, orders)
}

#[cfg(test)]
mod tests {
    use super::topological_sort;

    #[test]
    fn topological_sort_on_chain_returns_unique_vertex_order() {
        // Arrange
        let mut graph = vec![vec![1], vec![2], vec![]];
        let expected_result = (true, vec![0, 1, 2]);
        // Act
        let actual_result = topological_sort(&mut graph);
        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn topological_sort_on_isolated_vertices_returns_non_unique_order_containing_each_vertex() {
        // Arrange
        let mut graph = vec![vec![], vec![]];
        let expected_result = (false, vec![0, 1]);
        // Act
        let (is_unique, mut order) = topological_sort(&mut graph);
        order.sort();
        let actual_result = (is_unique, order);
        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn empty_graph_returns_unique_empty_order() {
        // Arrange
        let mut graph: Vec<Vec<usize>> = vec![];
        let expected_values = (true, Vec::<usize>::new());

        // Act
        let actual_values = topological_sort(&mut graph);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn cycle_returns_order_shorter_than_vertex_count() {
        // Arrange
        let mut graph = vec![vec![1], vec![0]];
        let vertex_count = graph.len();
        // Act
        let (actual_is_unique, actual_order) = topological_sort(&mut graph);
        let actual_order_length = actual_order.len();
        // Assert
        assert!(actual_is_unique);
        assert!(actual_order_length < vertex_count);
    }

    #[test]
    fn unsorted_adjacency_is_sorted_before_processing() {
        // Arrange
        let mut graph = vec![vec![2, 1], vec![], vec![]];
        let expected_order = vec![0, 1, 2];

        // Act
        let (_, actual_order) = topological_sort(&mut graph);

        // Assert
        assert_eq!(actual_order, expected_order);
    }

    #[test]
    fn topological_sort_on_single_isolated_vertex_returns_unique_zero_order() {
        // Arrange
        let mut graph = vec![vec![]];
        let expected_result = (true, vec![0]);
        // Act
        let actual_result = topological_sort(&mut graph);
        // Assert
        assert_eq!(actual_result, expected_result);
    }

}
