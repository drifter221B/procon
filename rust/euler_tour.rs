/// オイラーツアーで各頂点の入り時刻と出時刻を計算する。
pub struct EulerTour {
    graph: Vec<Vec<usize>>,
    pub tin: Vec<usize>,
    pub tout: Vec<usize>,
    pub order: Vec<usize>,
}

impl EulerTour {

    pub fn new(vertex_number: usize) -> Self {
        Self {
            graph: vec![Vec::new(); vertex_number],
            tin: vec![0; vertex_number],
            tout: vec![0; vertex_number],
            order: Vec::with_capacity(vertex_number * 2),
        }
    }

    pub fn add_edge(&mut self, u: usize, v: usize) {
        if u < self.graph.len() && v < self.graph.len() {
            self.graph[u].push(v);
            self.graph[v].push(u);
        }
    }

    pub fn build(&mut self, root: usize) {

        let mut stack = Vec::new();
        stack.push((root, usize::MAX, false));
        let mut time = 0usize;

        while let Some((v, parent, exited)) = stack.pop() {
            if exited {
                self.tout[v] = time;
                self.order.push(v);
                time += 1;
                continue;
            }

            self.tin[v] = time;
            self.order.push(v);
            time += 1;
            stack.push((v, parent, true));

            for &nv in self.graph[v].iter().rev() {
                if nv == parent {
                    continue;
                }
                stack.push((nv, v, false));
            }
        }
    }

    /// 頂点 `v` を根とする部分木に対応するオーダー区間を返す。
    pub fn subtree_range(&self, v: usize) -> Option<(usize, usize)> {
        if v < self.graph.len() {
            Some((self.tin[v], self.tout[v]))
        } else {
            None
        }
    }
}


#[cfg(test)]
mod tests {
    use super::EulerTour;

    #[test]
    fn computes_in_out_times() {
        // Arrange
        let vertex_count = 3;
        let edges = [(0, 1), (1, 2)];
        let root = 0;
        let subtree_vertex = 1;
        let mut euler_tour = EulerTour::new(vertex_count);
        for (from, to) in edges {
            euler_tour.add_edge(from, to);
        }

        // Act
        euler_tour.build(root);
        let actual_subtree_range = euler_tour.subtree_range(subtree_vertex);
        let expected_subtree_range = Some((
            euler_tour.tin[subtree_vertex],
            euler_tour.tout[subtree_vertex],
        ));

        // Assert
        assert!(euler_tour.tin[0] < euler_tour.tin[1]);
        assert!(euler_tour.tin[1] < euler_tour.tin[2]);
        assert!(euler_tour.tin[2] < euler_tour.tout[2]);
        assert_eq!(actual_subtree_range, expected_subtree_range);
    }

    #[test]
    fn handles_single_vertex() {
        // Arrange
        let vertex_count = 1;
        let root = 0;
        let expected_tin = 0;
        let expected_tout = 1;
        let expected_order_length = 2;
        let mut euler_tour = EulerTour::new(vertex_count);

        // Act
        euler_tour.build(root);
        let actual_tin = euler_tour.tin[root];
        let actual_tout = euler_tour.tout[root];
        let actual_order_length = euler_tour.order.len();

        // Assert
        assert_eq!(actual_tin, expected_tin);
        assert_eq!(actual_tout, expected_tout);
        assert_eq!(actual_order_length, expected_order_length);
    }

    #[test]
    fn build_twice_appends_order_with_current_behavior() {
        // Arrange
        let vertex_count = 2;
        let edge = (0, 1);
        let root = 0;
        let expected_first_order_length = 4;
        let expected_second_order_length = 8;
        let mut euler_tour = EulerTour::new(vertex_count);
        euler_tour.add_edge(edge.0, edge.1);
        euler_tour.build(root);
        let first_order_length = euler_tour.order.len();

        // Act
        euler_tour.build(root);
        let actual_order_length = euler_tour.order.len();

        // Assert
        assert_eq!(first_order_length, expected_first_order_length);
        assert_eq!(actual_order_length, expected_second_order_length);
    }

    #[test]
    fn non_zero_root_builds_correct_ranges() {
        // Arrange
        let vertex_count = 4;
        let root = 2;
        let leaf = 0;
        let expected_root_range = Some((0, 7));
        let expected_leaf_range = Some((2, 3));
        let mut euler_tour = EulerTour::new(vertex_count);
        euler_tour.add_edge(2, 1);
        euler_tour.add_edge(2, 3);
        euler_tour.add_edge(1, 0);

        // Act
        euler_tour.build(root);
        let actual_root_range = euler_tour.subtree_range(root);
        let actual_leaf_range = euler_tour.subtree_range(leaf);

        // Assert
        assert_eq!(actual_root_range, expected_root_range);
        assert_eq!(actual_leaf_range, expected_leaf_range);
    }

    #[test]
    fn unreachable_vertices_keep_zero_times() {
        // Arrange
        let mut euler_tour = EulerTour::new(3);
        euler_tour.add_edge(0, 1);

        // Act
        euler_tour.build(0);
        let actual_tin = euler_tour.tin[2];
        let actual_tout = euler_tour.tout[2];

        // Assert
        assert_eq!(actual_tin, 0);
        assert_eq!(actual_tout, 0);
    }

    #[test]
    fn subtree_range_covers_children_interval() {
        // Arrange
        let vertex_count = 4;
        let edges = [(0, 1), (0, 2), (2, 3)];
        let root = 0;
        let subtree_vertex = 2;
        let child_vertex = 3;
        let mut euler_tour = EulerTour::new(vertex_count);
        for (from, to) in edges {
            euler_tour.add_edge(from, to);
        }

        // Act
        euler_tour.build(root);
        let actual_subtree_range = euler_tour.subtree_range(subtree_vertex).unwrap();
        let actual_child_times = (euler_tour.tin[child_vertex], euler_tour.tout[child_vertex]);

        // Assert
        let (left, right) = actual_subtree_range;
        assert!(left < right);
        // 子 3 の出入りは 2 の区間内に含まれる
        assert!(actual_child_times.0 >= left && actual_child_times.1 <= right);
    }

    #[test]
    fn invalid_edge_and_subtree_range_boundaries_are_ignored_or_none() {
        // Arrange
        let vertex_count = 2;
        let valid_vertex = vertex_count - 1;
        let invalid_vertex = vertex_count;
        let root = 0;
        let expected_invalid_range = None;
        let mut euler_tour = EulerTour::new(vertex_count);

        // Act
        euler_tour.add_edge(root, invalid_vertex);
        euler_tour.build(root);
        let actual_valid_range = euler_tour.subtree_range(valid_vertex);
        let actual_invalid_range = euler_tour.subtree_range(invalid_vertex);

        // Assert
        assert_eq!(actual_valid_range, Some((0, 0)));
        assert_eq!(actual_invalid_range, expected_invalid_range);
    }

    #[test]
    fn empty_graph_subtree_range_returns_none_without_building() {
        // Arrange
        let vertex_count = 0;
        let queried_vertex = 0;
        let expected_range = None;
        let euler_tour = EulerTour::new(vertex_count);

        // Act
        let actual_range = euler_tour.subtree_range(queried_vertex);

        // Assert
        assert_eq!(actual_range, expected_range);
    }
}
