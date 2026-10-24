pub trait Monoid {
    type S: Clone;

    fn id() -> Self::S;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
    fn base(vertex: usize) -> Self::S;
    fn composition(value: &Self::S, parent: usize, child: usize, direction: i32) -> Self::S;
}

pub fn rerooting<M>(
    vertex_number: usize,
    edges: &[(usize, usize)],
    root: usize,
) -> Vec<M::S>
where
    M: Monoid,
{

    let mut graph = vec![Vec::new(); vertex_number];
    for &(u, v) in edges.iter() {
        if u < vertex_number && v < vertex_number {
            graph[u].push(v);
            graph[v].push(u);
        }
    }

    let mut parent = vec![usize::MAX; vertex_number];
    let mut order = Vec::with_capacity(vertex_number);
    let mut stack = vec![root];
    parent[root] = root;
    while let Some(v) = stack.pop() {
        order.push(v);
        for &nv in graph[v].iter() {
            if parent[nv] == usize::MAX {
                parent[nv] = v;
                stack.push(nv);
            }
        }
    }

    let mut down = vec![M::id(); vertex_number];
    for &v in order.iter().rev() {
        let mut acc = M::base(v);
        for &nv in graph[v].iter() {
            if parent[v] == nv {
                continue;
            }
            let mapped = M::composition(&down[nv], v, nv, 0);
            acc = M::binary_operation(&acc, &mapped);
        }
        down[v] = acc;
    }

    let mut result = vec![M::id(); vertex_number];
    let mut up = vec![M::id(); vertex_number];
    up[root] = M::id();

    for &v in order.iter() {
        let children = graph[v]
            .iter()
            .copied()
            .filter(|&nv| parent[v] != nv)
            .collect::<Vec<_>>();

        let child_count = children.len();
        let mut contributions = Vec::with_capacity(child_count);
        for &nv in children.iter() {
            contributions.push(M::composition(&down[nv], v, nv, 0));
        }

        let mut prefix = vec![M::id(); child_count + 1];
        for i in 0..child_count {
            prefix[i + 1] = M::binary_operation(&prefix[i], &contributions[i]);
        }

        let mut suffix = vec![M::id(); child_count + 1];
        for i in (0..child_count).rev() {
            suffix[i] = M::binary_operation(&contributions[i], &suffix[i + 1]);
        }

        let parent_base = M::binary_operation(&up[v], &M::base(v));
        let total = M::binary_operation(&parent_base, &prefix[child_count]);
        result[v] = total.clone();

        for (i, &nv) in children.iter().enumerate() {
            let merged = M::binary_operation(
                &M::binary_operation(&parent_base, &prefix[i]),
                &suffix[i + 1],
            );
            up[nv] = M::composition(&merged, v, nv, 1);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::{rerooting, Monoid};

    struct SubtreeSizeMonoid;

    impl Monoid for SubtreeSizeMonoid {
        type S = i64;

        fn id() -> Self::S {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            *lhs + *rhs
        }

        fn base(_vertex: usize) -> Self::S {
            1
        }

        fn composition(value: &Self::S, _parent: usize, _child: usize, _direction: i32) -> Self::S {
            *value
        }
    }

    struct VertexWeightSumMonoid;

    impl Monoid for VertexWeightSumMonoid {
        type S = i64;

        fn id() -> Self::S {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            *lhs + *rhs
        }

        fn base(vertex: usize) -> Self::S {
            vertex as i64 + 1
        }

        fn composition(value: &Self::S, _parent: usize, _child: usize, _direction: i32) -> Self::S {
            *value
        }
    }

    struct NonCommutativeTextMonoid;

    impl Monoid for NonCommutativeTextMonoid {
        type S = String;

        fn id() -> Self::S {
            String::new()
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            let mut merged_text = lhs.clone();
            merged_text.push_str(rhs);
            merged_text
        }

        fn base(vertex: usize) -> Self::S {
            match vertex {
                0 => "A".to_string(),
                1 => "B".to_string(),
                2 => "C".to_string(),
                3 => "D".to_string(),
                _ => format!("V{vertex}"),
            }
        }

        fn composition(value: &Self::S, _parent: usize, _child: usize, _direction: i32) -> Self::S {
            value.clone()
        }
    }

    struct DirectionSensitiveMonoid;

    impl Monoid for DirectionSensitiveMonoid {
        type S = i64;

        fn id() -> Self::S {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            *lhs + *rhs
        }

        fn base(_vertex: usize) -> Self::S {
            1
        }

        fn composition(value: &Self::S, _parent: usize, _child: usize, direction: i32) -> Self::S {
            if direction == 0 {
                *value + 10
            } else {
                *value + 1000
            }
        }
    }

    struct ParentChildAwareMonoid;

    impl Monoid for ParentChildAwareMonoid {
        type S = i64;

        fn id() -> Self::S {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            *lhs + *rhs
        }

        fn base(_vertex: usize) -> Self::S {
            0
        }

        fn composition(value: &Self::S, parent: usize, child: usize, direction: i32) -> Self::S {
            if direction == 1 {
                *value + parent as i64 * 10 + child as i64
            } else {
                *value
            }
        }
    }

    #[test]
    fn rerooting_with_non_commutative_binary_operation_keeps_adjacency_order() {
        // Arrange
        let vertex_number = 3usize;
        let edges = vec![(0usize, 1usize), (0usize, 2usize)];
        let root = 0usize;
        let expected_values = vec![
            "ABC".to_string(),
            "ACB".to_string(),
            "ABC".to_string(),
        ];

        // Act
        let actual_values = rerooting::<NonCommutativeTextMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_with_vertex_dependent_base_returns_expected_totals() {
        // Arrange
        let vertex_number = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let root = 0usize;
        let expected_values = vec![6i64, 6i64, 6i64];

        // Act
        let actual_values = rerooting::<VertexWeightSumMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_uses_direction_specific_composition_result() {
        // Arrange
        let vertex_number = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let expected_values = vec![12i64, 1002i64];

        // Act
        let actual_values = rerooting::<DirectionSensitiveMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_passes_parent_and_child_arguments_to_composition_in_expected_order() {
        // Arrange
        let vertex_number = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let expected_values = vec![0i64, 1i64];

        // Act
        let actual_values = rerooting::<ParentChildAwareMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_with_non_zero_root_returns_expected_values_for_all_vertices() {
        // Arrange
        let vertex_number = 4usize;
        let edges = vec![(0usize, 1usize), (0usize, 2usize), (1usize, 3usize)];
        let root = 3usize;
        let expected_values = vec![4i64, 4i64, 4i64, 4i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_on_star_tree_excludes_self_contribution_in_each_child_up_propagation() {
        // Arrange
        let vertex_number = 4usize;
        let edges = vec![(0usize, 1usize), (0usize, 2usize), (0usize, 3usize)];
        let root = 0usize;
        let expected_values = vec![10i64, 10i64, 10i64, 10i64];

        // Act
        let actual_values = rerooting::<VertexWeightSumMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_with_leaf_root_returns_expected_values() {
        // Arrange
        let vertex_number = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let root = 2usize;
        let expected_values = vec![3i64, 3i64, 3i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_two_node_tree_with_root_zero_returns_expected_values() {
        // Arrange
        let vertex_number = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let expected_values = vec![2i64, 2i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_two_node_tree_with_root_one_returns_expected_values() {
        // Arrange
        let vertex_number = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 1usize;
        let expected_values = vec![2i64, 2i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_with_root_equal_vertex_number_minus_one_does_not_panic() {
        // Arrange
        let vertex_number = 4usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize), (2usize, 3usize)];
        let root = vertex_number - 1;

        // Act
        let run_result = panic::catch_unwind(|| rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root));

        // Assert
        assert!(run_result.is_ok());
    }

    #[test]
    fn rerooting_with_root_equal_vertex_number_panics() {
        // Arrange
        let vertex_number = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let root = vertex_number;

        // Act
        let run_result = panic::catch_unwind(|| rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root));

        // Assert
        assert!(run_result.is_err());
    }

    #[test]
    fn rerooting_with_vertex_number_zero_panics() {
        // Arrange
        let vertex_number = 0usize;
        let edges = vec![];
        let root = 0usize;

        // Act
        let run_result = panic::catch_unwind(|| rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root));

        // Assert
        assert!(run_result.is_err());
    }

    #[test]
    fn rerooting_on_disconnected_graph_keeps_unreachable_vertices_as_identity() {
        // Arrange
        let vertex_number = 4usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let expected_values = vec![2i64, 2i64, 0i64, 0i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_ignores_edges_with_out_of_range_vertices() {
        // Arrange
        let vertex_number = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 3usize), (2usize, 5usize)];
        let root = 0usize;
        let expected_values = vec![2i64, 2i64, 0i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_with_duplicate_edges_aggregates_duplicate_contributions() {
        // Arrange
        let vertex_number = 2usize;
        let edges = vec![(0usize, 1usize), (0usize, 1usize)];
        let root = 0usize;
        let expected_values = vec![3i64, 3i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_with_self_loop_preserves_two_vertex_component_totals() {
        // Arrange
        let vertex_number = 2usize;
        let edges = vec![(0usize, 0usize), (0usize, 1usize)];
        let root = 0usize;
        let expected_values = vec![2i64, 2i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_on_triangle_cycle_returns_current_totals_four_six_four() {
        // 木以外の入力は木 DP の意味を保証せず、既存の返却値のみを回帰確認する。
        // Arrange
        let vertex_number = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize), (2usize, 0usize)];
        let root = 0usize;
        let expected_values = vec![4i64, 6i64, 4i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn rerooting_on_single_vertex_without_edges_returns_base_value() {
        // Arrange
        let vertex_number = 1;
        let edges: Vec<(usize, usize)> = vec![];
        let root = 0;
        let expected_values = vec![1i64];

        // Act
        let actual_values = rerooting::<SubtreeSizeMonoid>(vertex_number, &edges, root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

}
