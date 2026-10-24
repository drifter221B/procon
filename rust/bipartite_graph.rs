use std::collections::VecDeque;

/// 隣接リストに片方向だけ登録された辺も、無向の色制約として扱う。
fn is_bipartite(graph: &[Vec<usize>]) -> Option<Vec<usize>> {
    let vertex_count = graph.len();
    let constraint_graph = build_undirected_constraint_graph(graph);
    let mut colors = vec![None; vertex_count];

    for start_vertex in 0..vertex_count {
        if colors[start_vertex].is_some() {
            continue;
        }

        if !color_component(start_vertex, &constraint_graph, &mut colors) {
            return None;
        }
    }

    Some(colors.into_iter().map(|color| color.unwrap_or(0)).collect())
}

fn build_undirected_constraint_graph(graph: &[Vec<usize>]) -> Vec<Vec<usize>> {
    let mut constraint_graph = vec![Vec::new(); graph.len()];

    for (vertex, next_vertices) in graph.iter().enumerate() {
        for &next_vertex in next_vertices {
            constraint_graph[vertex].push(next_vertex);
            constraint_graph[next_vertex].push(vertex);
        }
    }

    constraint_graph
}

fn color_component(
    start_vertex: usize,
    graph: &[Vec<usize>],
    colors: &mut [Option<usize>],
) -> bool {
    let mut queue = VecDeque::new();
    colors[start_vertex] = Some(0);
    queue.push_back(start_vertex);

    while let Some(vertex) = queue.pop_front() {
        let current_color = colors[vertex].unwrap_or(0);
        let next_color = current_color ^ 1;

        for &next_vertex in graph[vertex].iter() {
            match colors[next_vertex] {
                Some(color) if color == current_color => {
                    return false;
                }
                Some(_) => {}
                None => {
                    colors[next_vertex] = Some(next_color);
                    queue.push_back(next_vertex);
                }
            }
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::is_bipartite;

    fn are_all_adjacent_vertices_colored_differently(
        graph: &[Vec<usize>],
        vertex_colors: &[usize],
    ) -> bool {
        for (vertex, next_vertices) in graph.iter().enumerate() {
            for &next_vertex in next_vertices {
                if vertex_colors[vertex] == vertex_colors[next_vertex] {
                    return false;
                }
            }
        }
        true
    }

    fn simple_undirected_graph_from_edge_mask(
        vertex_count: usize,
        edge_mask: usize,
    ) -> Vec<Vec<usize>> {
        let mut graph = vec![vec![]; vertex_count];
        let mut edge_index = 0;

        for vertex in 0..vertex_count {
            for next_vertex in (vertex + 1)..vertex_count {
                if edge_mask & (1 << edge_index) != 0 {
                    graph[vertex].push(next_vertex);
                    graph[next_vertex].push(vertex);
                }
                edge_index += 1;
            }
        }

        graph
    }

    fn has_valid_two_coloring(graph: &[Vec<usize>]) -> bool {
        let vertex_count = graph.len();
        let coloring_count = 1 << vertex_count;

        (0..coloring_count).any(|color_mask| {
            graph.iter().enumerate().all(|(vertex, next_vertices)| {
                let vertex_color = (color_mask >> vertex) & 1;
                next_vertices.iter().all(|&next_vertex| {
                    let next_vertex_color = (color_mask >> next_vertex) & 1;
                    vertex_color != next_vertex_color
                })
            })
        })
    }

    #[test]
    fn is_bipartite_with_empty_graph_returns_some_empty_vector() {
        // Arrange
        let graph: Vec<Vec<usize>> = vec![];
        let expected_colors = Some(vec![]);

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_single_vertex_without_edges_returns_zero_color() {
        // Arrange
        let graph = vec![vec![]];
        let expected_colors = Some(vec![0]);

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_three_isolated_vertices_returns_all_zero_colors() {
        // Arrange
        let graph = vec![vec![], vec![], vec![]];
        let expected_colors = Some(vec![0, 0, 0]);

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_two_vertices_one_edge_returns_some_with_different_neighbor_colors() {
        // Arrange
        let graph = vec![vec![1], vec![0]];

        // Act
        let actual_colors = is_bipartite(&graph);
        let actual_vertex_colors = actual_colors
            .as_ref()
            .expect("2 頂点 1 辺のグラフは二部グラフなので Some を返すべき");
        let are_neighbors_colored_differently = actual_vertex_colors[0] != actual_vertex_colors[1];

        // Assert
        assert!(actual_colors.is_some());
        assert!(are_neighbors_colored_differently);
    }

    #[test]
    fn is_bipartite_with_three_vertex_path_returns_alternating_colors() {
        // Arrange
        let graph = vec![vec![1], vec![0, 2], vec![1]];
        let expected_colors = Some(vec![0, 1, 0]);

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_even_cycle_returns_some() {
        // Arrange
        let graph = vec![vec![1, 3], vec![0, 2], vec![1, 3], vec![0, 2]];

        // Act
        let actual_colors = is_bipartite(&graph);
        let actual_vertex_colors = actual_colors
            .as_ref()
            .expect("偶数サイクルは二部グラフなので Some を返すべき");
        let are_all_edges_valid =
            are_all_adjacent_vertices_colored_differently(&graph, actual_vertex_colors);

        // Assert
        assert!(actual_colors.is_some());
        assert!(are_all_edges_valid);
    }

    #[test]
    fn is_bipartite_with_odd_cycle_returns_none() {
        // Arrange
        let graph = vec![vec![1, 2], vec![0, 2], vec![0, 1]];
        let expected_colors = None;

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_self_loop_returns_none() {
        // Arrange
        let graph = vec![vec![0]];
        let expected_colors = None;

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_disconnected_bipartite_components_returns_some() {
        // Arrange
        let graph = vec![vec![1], vec![0], vec![3], vec![2]];

        // Act
        let actual_colors = is_bipartite(&graph);
        let actual_vertex_colors = actual_colors
            .as_ref()
            .expect("全成分が二部グラフであれば Some を返すべき");
        let are_all_edges_valid =
            are_all_adjacent_vertices_colored_differently(&graph, actual_vertex_colors);

        // Assert
        assert!(actual_colors.is_some());
        assert!(are_all_edges_valid);
    }

    #[test]
    fn is_bipartite_with_disconnected_graph_containing_odd_cycle_returns_none() {
        // Arrange
        let graph = vec![vec![1], vec![0], vec![3, 4], vec![2, 4], vec![2, 3]];
        let expected_colors = None;

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_with_multiple_edges_between_two_vertices_returns_some() {
        // Arrange
        let graph = vec![vec![1, 1], vec![0, 0]];

        // Act
        let actual_colors = is_bipartite(&graph);
        let actual_vertex_colors = actual_colors
            .as_ref()
            .expect("多重辺のみの 2 部分割可能グラフは Some を返すべき");
        let are_all_edges_valid =
            are_all_adjacent_vertices_colored_differently(&graph, actual_vertex_colors);

        // Assert
        assert!(actual_colors.is_some());
        assert!(are_all_edges_valid);
    }

    #[test]
    fn is_bipartite_with_out_of_range_neighbor_index_panics() {
        // Arrange
        let graph = vec![vec![2], vec![]];

        // Act
        let actual_result = std::panic::catch_unwind(|| is_bipartite(&graph));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn is_bipartite_with_reordered_even_cycle_neighbors_returns_expected_colors() {
        // Arrange
        let graph = vec![vec![3, 1], vec![2, 0], vec![3, 1], vec![2, 0]];
        let expected_colors = Some(vec![0, 1, 0, 1]);

        // Act
        let actual_colors = is_bipartite(&graph);

        // Assert
        assert_eq!(actual_colors, expected_colors);
    }

    #[test]
    fn is_bipartite_for_all_simple_graphs_up_to_five_vertices_matches_brute_force() {
        // Arrange
        let minimum_vertex_count = 0usize;
        let maximum_vertex_count = 5usize;
        let expected_allowed_colors = [0, 1];
        let mut test_cases = Vec::new();

        for vertex_count in minimum_vertex_count..=maximum_vertex_count {
            let possible_edge_count = vertex_count * vertex_count.saturating_sub(1) / 2;
            let graph_count = 1 << possible_edge_count;

            for edge_mask in 0..graph_count {
                let graph = simple_undirected_graph_from_edge_mask(vertex_count, edge_mask);
                let expected_is_bipartite = has_valid_two_coloring(&graph);
                test_cases.push((graph, expected_is_bipartite));
            }
        }

        // Act
        let actual_results: Vec<Option<Vec<usize>>> = test_cases
            .iter()
            .map(|(graph, _)| is_bipartite(graph))
            .collect();

        // Assert
        for ((graph, expected_is_bipartite), actual_colors) in
            test_cases.iter().zip(&actual_results)
        {
            assert_eq!(actual_colors.is_some(), *expected_is_bipartite);

            if let Some(colors) = actual_colors {
                assert_eq!(colors.len(), graph.len());
                assert!(colors
                    .iter()
                    .all(|color| expected_allowed_colors.contains(color)));
                assert!(are_all_adjacent_vertices_colored_differently(graph, colors));
            }
        }
    }

    #[test]
    fn is_bipartite_with_asymmetric_edge_is_invariant_under_vertex_renaming() {
        // Arrange
        let graph_with_forward_edge = vec![vec![1], vec![]];
        let graph_with_reversed_edge = vec![vec![], vec![0]];
        let expected_forward_colors = Some(vec![0, 1]);
        let expected_reversed_colors = Some(vec![0, 1]);
        let expected_same_judgment = true;

        // Act
        let actual_forward_colors = is_bipartite(&graph_with_forward_edge);
        let actual_reversed_colors = is_bipartite(&graph_with_reversed_edge);
        let actual_same_judgment =
            actual_forward_colors.is_some() == actual_reversed_colors.is_some();

        // Assert
        assert_eq!(actual_forward_colors, expected_forward_colors);
        assert_eq!(actual_reversed_colors, expected_reversed_colors);
        assert_eq!(actual_same_judgment, expected_same_judgment);
    }
}
