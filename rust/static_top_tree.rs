//! Static Top Tree.
//! `compress / rake / add_edge / add_vertex / singleton` の 5 演算を差し込んで、
//! 根付き木上の静的 Top Tree DP を再計算するためのライブラリ。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ClusterKind {
    Vertex {
        vertex: usize,
    },
    Compress,
    Rake,
    AddEdge {
        parent_vertex: usize,
        child_vertex: usize,
    },
    AddVertex {
        vertex: usize,
    },
}

#[derive(Debug, Clone)]
struct ClusterNode {
    kind: ClusterKind,
    left: Option<usize>,
    right: Option<usize>,
    parent: Option<usize>,
    size: usize,
}

/// 木の形だけを保持する Static Top Tree の静的構造。
#[derive(Debug, Clone)]
pub struct StaticTopTreeStructure {
    vertex_size: usize,
    root: usize,
    parent: Vec<Option<usize>>,
    children: Vec<Vec<usize>>,
    heavy_child: Vec<Option<usize>>,
    subtree_size: Vec<usize>,
    cluster_root: usize,
    cluster_nodes: Vec<ClusterNode>,
    vertex_update_node: Vec<usize>,
}

impl StaticTopTreeStructure {
    pub fn new(vertex_size: usize, edges: &[(usize, usize)], root: usize) -> Self {
        assert!(0 < vertex_size, "vertex_size must be positive");
        assert!(root < vertex_size, "root must be in range");
        assert_eq!(edges.len(), vertex_size - 1, "input graph must be a tree");

        let adjacency = Self::build_adjacency(vertex_size, edges);
        let (parent, children, order) = Self::build_rooted_tree(vertex_size, &adjacency, root);
        let (subtree_size, heavy_child) =
            Self::compute_subtree_size_and_heavy_child(vertex_size, &children, &order);

        let mut builder =
            StaticTopTreeBuilder::new(children.clone(), heavy_child.clone(), subtree_size.clone());
        let cluster_root = builder.build_path_cluster(root);

        Self {
            vertex_size,
            root,
            parent,
            children,
            heavy_child,
            subtree_size,
            cluster_root,
            cluster_nodes: builder.cluster_nodes,
            vertex_update_node: builder
                .vertex_update_node
                .into_iter()
                .map(|node_index| {
                    node_index.expect("every vertex must have a corresponding update node")
                })
                .collect(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.vertex_size
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.vertex_size == 0
    }

    #[inline]
    pub fn contains(&self, vertex: usize) -> bool {
        vertex < self.vertex_size
    }

    #[inline]
    pub fn root(&self) -> usize {
        self.root
    }

    #[inline]
    pub fn parent_vertex(&self, vertex: usize) -> Option<usize> {
        if self.contains(vertex) {
            self.parent[vertex]
        } else {
            None
        }
    }

    #[inline]
    pub fn children_vertices(&self, vertex: usize) -> Option<&[usize]> {
        if self.contains(vertex) {
            Some(&self.children[vertex])
        } else {
            None
        }
    }

    #[inline]
    pub fn heavy_child(&self, vertex: usize) -> Option<usize> {
        if self.contains(vertex) {
            self.heavy_child[vertex]
        } else {
            None
        }
    }

    #[inline]
    pub fn subtree_size(&self, vertex: usize) -> Option<usize> {
        if self.contains(vertex) {
            Some(self.subtree_size[vertex])
        } else {
            None
        }
    }

    #[inline]
    pub fn cluster_count(&self) -> usize {
        self.cluster_nodes.len()
    }

    #[inline]
    pub fn cluster_root(&self) -> usize {
        self.cluster_root
    }

    fn build_adjacency(vertex_size: usize, edges: &[(usize, usize)]) -> Vec<Vec<usize>> {
        let mut adjacency = vec![Vec::new(); vertex_size];

        for &(left_vertex, right_vertex) in edges.iter() {
            assert!(left_vertex < vertex_size && right_vertex < vertex_size);
            assert_ne!(left_vertex, right_vertex, "self-loop edge is not allowed");

            adjacency[left_vertex].push(right_vertex);
            adjacency[right_vertex].push(left_vertex);
        }

        adjacency
    }

    fn build_rooted_tree(
        vertex_size: usize,
        adjacency: &[Vec<usize>],
        root: usize,
    ) -> (Vec<Option<usize>>, Vec<Vec<usize>>, Vec<usize>) {
        let mut parent = vec![None; vertex_size];
        let mut children = vec![Vec::new(); vertex_size];
        let mut order = Vec::with_capacity(vertex_size);
        let mut visited = vec![false; vertex_size];
        let mut stack = vec![root];

        visited[root] = true;

        while let Some(vertex) = stack.pop() {
            order.push(vertex);

            for &next_vertex in adjacency[vertex].iter() {
                if parent[vertex] == Some(next_vertex) {
                    continue;
                }

                assert!(!visited[next_vertex], "input graph must be a tree");

                visited[next_vertex] = true;
                parent[next_vertex] = Some(vertex);
                children[vertex].push(next_vertex);
                stack.push(next_vertex);
            }
        }

        assert_eq!(order.len(), vertex_size, "input graph must be connected");

        (parent, children, order)
    }

    fn compute_subtree_size_and_heavy_child(
        vertex_size: usize,
        children: &[Vec<usize>],
        order: &[usize],
    ) -> (Vec<usize>, Vec<Option<usize>>) {
        let mut subtree_size = vec![1; vertex_size];
        let mut heavy_child = vec![None; vertex_size];

        for &vertex in order.iter().rev() {
            let mut heavy_size = 0usize;

            for &child_vertex in children[vertex].iter() {
                subtree_size[vertex] += subtree_size[child_vertex];

                if heavy_size < subtree_size[child_vertex] {
                    heavy_size = subtree_size[child_vertex];
                    heavy_child[vertex] = Some(child_vertex);
                }
            }
        }

        (subtree_size, heavy_child)
    }
}

struct StaticTopTreeBuilder {
    children: Vec<Vec<usize>>,
    heavy_child: Vec<Option<usize>>,
    cluster_nodes: Vec<ClusterNode>,
    vertex_update_node: Vec<Option<usize>>,
}

impl StaticTopTreeBuilder {
    fn new(
        children: Vec<Vec<usize>>,
        heavy_child: Vec<Option<usize>>,
        subtree_size: Vec<usize>,
    ) -> Self {
        let vertex_size = children.len();

        Self {
            children,
            heavy_child,
            cluster_nodes: Vec::with_capacity(subtree_size.len() * 4),
            vertex_update_node: vec![None; vertex_size],
        }
    }

    fn build_path_cluster(&mut self, start_vertex: usize) -> usize {
        let mut path_unit_nodes = Vec::new();
        let mut current_vertex = Some(start_vertex);

        while let Some(vertex) = current_vertex {
            let mut point_nodes = Vec::new();
            let child_vertices = self.children[vertex].clone();

            for &child_vertex in child_vertices.iter() {
                if self.heavy_child[vertex] == Some(child_vertex) {
                    continue;
                }

                let child_path_node = self.build_path_cluster(child_vertex);
                let edge_point_node = self.create_add_edge(child_path_node, vertex, child_vertex);
                point_nodes.push(edge_point_node);
            }

            let path_unit_node = if point_nodes.is_empty() {
                self.create_vertex(vertex)
            } else {
                let rake_node = self.merge_point_nodes(&point_nodes);
                self.create_add_vertex(rake_node, vertex)
            };

            self.vertex_update_node[vertex] = Some(path_unit_node);
            path_unit_nodes.push(path_unit_node);
            current_vertex = self.heavy_child[vertex];
        }

        self.merge_path_nodes(&path_unit_nodes)
    }

    fn merge_path_nodes(&mut self, node_indices: &[usize]) -> usize {
        assert!(
            !node_indices.is_empty(),
            "path merge requires at least one node"
        );

        if node_indices.len() == 1 {
            return node_indices[0];
        }

        let split_index = self.split_nodes_by_size(node_indices);
        let left_node = self.merge_path_nodes(&node_indices[..split_index]);
        let right_node = self.merge_path_nodes(&node_indices[split_index..]);

        self.create_compress(left_node, right_node)
    }

    fn merge_point_nodes(&mut self, node_indices: &[usize]) -> usize {
        assert!(
            !node_indices.is_empty(),
            "point merge requires at least one node"
        );

        if node_indices.len() == 1 {
            return node_indices[0];
        }

        let split_index = self.split_nodes_by_size(node_indices);
        let left_node = self.merge_point_nodes(&node_indices[..split_index]);
        let right_node = self.merge_point_nodes(&node_indices[split_index..]);

        self.create_rake(left_node, right_node)
    }

    fn split_nodes_by_size(&self, node_indices: &[usize]) -> usize {
        let total_size = node_indices
            .iter()
            .map(|&node_index| self.cluster_nodes[node_index].size)
            .sum::<usize>();

        let mut prefix_size = 0usize;
        let mut best_split_index = 1usize;
        let mut best_difference = usize::MAX;

        for split_index in 1..node_indices.len() {
            prefix_size += self.cluster_nodes[node_indices[split_index - 1]].size;
            let suffix_size = total_size - prefix_size;
            let difference = prefix_size.abs_diff(suffix_size);

            if difference < best_difference {
                best_difference = difference;
                best_split_index = split_index;
            }
        }

        best_split_index
    }

    fn create_vertex(&mut self, vertex: usize) -> usize {
        let node_index = self.cluster_nodes.len();

        self.cluster_nodes.push(ClusterNode {
            kind: ClusterKind::Vertex { vertex },
            left: None,
            right: None,
            parent: None,
            size: 1,
        });

        node_index
    }

    fn create_add_vertex(&mut self, point_node: usize, vertex: usize) -> usize {
        let node_size = self.cluster_nodes[point_node].size + 1;

        self.create_unary_node(ClusterKind::AddVertex { vertex }, point_node, node_size)
    }

    fn create_add_edge(
        &mut self,
        path_node: usize,
        parent_vertex: usize,
        child_vertex: usize,
    ) -> usize {
        let node_size = self.cluster_nodes[path_node].size;

        self.create_unary_node(
            ClusterKind::AddEdge {
                parent_vertex,
                child_vertex,
            },
            path_node,
            node_size,
        )
    }

    fn create_compress(&mut self, left_node: usize, right_node: usize) -> usize {
        let node_size = self.cluster_nodes[left_node].size + self.cluster_nodes[right_node].size;

        self.create_binary_node(ClusterKind::Compress, left_node, right_node, node_size)
    }

    fn create_rake(&mut self, left_node: usize, right_node: usize) -> usize {
        let node_size = self.cluster_nodes[left_node].size + self.cluster_nodes[right_node].size;

        self.create_binary_node(ClusterKind::Rake, left_node, right_node, node_size)
    }

    fn create_unary_node(&mut self, kind: ClusterKind, child_node: usize, size: usize) -> usize {
        let node_index = self.cluster_nodes.len();

        self.cluster_nodes.push(ClusterNode {
            kind,
            left: Some(child_node),
            right: None,
            parent: None,
            size,
        });
        self.cluster_nodes[child_node].parent = Some(node_index);

        node_index
    }

    fn create_binary_node(
        &mut self,
        kind: ClusterKind,
        left_node: usize,
        right_node: usize,
        size: usize,
    ) -> usize {
        let node_index = self.cluster_nodes.len();

        self.cluster_nodes.push(ClusterNode {
            kind,
            left: Some(left_node),
            right: Some(right_node),
            parent: None,
            size,
        });
        self.cluster_nodes[left_node].parent = Some(node_index);
        self.cluster_nodes[right_node].parent = Some(node_index);

        node_index
    }
}

/// Static Top Tree 上で流す DP の演算。
///
/// `rake` は rooted tree にしたときの子の順序で評価される。
/// 順序依存の DP を流したい場合は、入力辺の順序をそのまま反映できるように木を与えること。
pub trait StaticTopTreeOperator {
    type Path;
    type Point;

    fn singleton(&self, vertex: usize) -> Self::Path;
    fn compress(&self, parent_path: &Self::Path, child_path: &Self::Path) -> Self::Path;
    fn rake(&self, left_point: &Self::Point, right_point: &Self::Point) -> Self::Point;
    fn add_edge(&self, path: &Self::Path, parent_vertex: usize, child_vertex: usize)
        -> Self::Point;
    fn add_vertex(&self, point: &Self::Point, vertex: usize) -> Self::Path;
}

enum ClusterValue<Path, Point> {
    Path(Path),
    Point(Point),
}

impl<Path, Point> ClusterValue<Path, Point> {
    fn as_path(&self) -> Option<&Path> {
        match self {
            Self::Path(value) => Some(value),
            Self::Point(_) => None,
        }
    }

    fn as_point(&self) -> Option<&Point> {
        match self {
            Self::Path(_) => None,
            Self::Point(value) => Some(value),
        }
    }
}

pub struct StaticTopTree<Op>
where
    Op: StaticTopTreeOperator,
{
    structure: StaticTopTreeStructure,
    operator: Op,
    values: Vec<ClusterValue<Op::Path, Op::Point>>,
}

impl<Op> StaticTopTree<Op>
where
    Op: StaticTopTreeOperator,
{
    pub fn new(structure: StaticTopTreeStructure, operator: Op) -> Self {
        let mut top_tree = Self {
            structure,
            operator,
            values: Vec::new(),
        };

        top_tree.rebuild();
        top_tree
    }

    pub fn from_edges(
        vertex_size: usize,
        edges: &[(usize, usize)],
        root: usize,
        operator: Op,
    ) -> Self {
        let structure = StaticTopTreeStructure::new(vertex_size, edges, root);
        Self::new(structure, operator)
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.structure.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.structure.is_empty()
    }

    #[inline]
    pub fn structure(&self) -> &StaticTopTreeStructure {
        &self.structure
    }

    #[inline]
    pub fn operator(&self) -> &Op {
        &self.operator
    }

    #[inline]
    pub fn operator_mut(&mut self) -> &mut Op {
        &mut self.operator
    }

    #[inline]
    pub fn all(&self) -> &Op::Path {
        self.values[self.structure.cluster_root]
            .as_path()
            .expect("the root cluster must be a path")
    }

    pub fn refresh_vertex(&mut self, vertex: usize) -> Option<()> {
        if !self.structure.contains(vertex) {
            return None;
        }

        let start_node = self.structure.vertex_update_node[vertex];
        self.refresh_from_node(start_node);

        Some(())
    }

    pub fn rebuild(&mut self) {
        let cluster_count = self.structure.cluster_nodes.len();
        let mut values = Vec::with_capacity(cluster_count);

        for node_index in 0..cluster_count {
            let value = Self::evaluate_node(&self.operator, &self.structure, &values, node_index);
            values.push(value);
        }

        self.values = values;
    }

    fn refresh_from_node(&mut self, start_node: usize) {
        let mut current_node = Some(start_node);

        while let Some(node_index) = current_node {
            let value =
                Self::evaluate_node(&self.operator, &self.structure, &self.values, node_index);
            self.values[node_index] = value;
            current_node = self.structure.cluster_nodes[node_index].parent;
        }
    }

    fn evaluate_node(
        operator: &Op,
        structure: &StaticTopTreeStructure,
        values: &[ClusterValue<Op::Path, Op::Point>],
        node_index: usize,
    ) -> ClusterValue<Op::Path, Op::Point> {
        let node = &structure.cluster_nodes[node_index];

        match node.kind {
            ClusterKind::Vertex { vertex } => ClusterValue::Path(operator.singleton(vertex)),
            ClusterKind::Compress => {
                let left_node = node.left.expect("compress node must have left child");
                let right_node = node.right.expect("compress node must have right child");
                let left_path = values[left_node]
                    .as_path()
                    .expect("compress left child must be a path");
                let right_path = values[right_node]
                    .as_path()
                    .expect("compress right child must be a path");

                ClusterValue::Path(operator.compress(left_path, right_path))
            }
            ClusterKind::Rake => {
                let left_node = node.left.expect("rake node must have left child");
                let right_node = node.right.expect("rake node must have right child");
                let left_point = values[left_node]
                    .as_point()
                    .expect("rake left child must be a point");
                let right_point = values[right_node]
                    .as_point()
                    .expect("rake right child must be a point");

                ClusterValue::Point(operator.rake(left_point, right_point))
            }
            ClusterKind::AddEdge {
                parent_vertex,
                child_vertex,
            } => {
                let child_node = node.left.expect("add_edge node must have one child");
                let child_path = values[child_node]
                    .as_path()
                    .expect("add_edge child must be a path");

                ClusterValue::Point(operator.add_edge(child_path, parent_vertex, child_vertex))
            }
            ClusterKind::AddVertex { vertex } => {
                let child_node = node.left.expect("add_vertex node must have one child");
                let child_point = values[child_node]
                    .as_point()
                    .expect("add_vertex child must be a point");

                ClusterValue::Path(operator.add_vertex(child_point, vertex))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{StaticTopTree, StaticTopTreeOperator, StaticTopTreeStructure};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    struct VertexSumOperator {
        vertex_values: Vec<i64>,
    }

    impl StaticTopTreeOperator for VertexSumOperator {
        type Path = i64;
        type Point = i64;

        fn singleton(&self, vertex: usize) -> Self::Path {
            self.vertex_values[vertex]
        }

        fn compress(&self, parent_path: &Self::Path, child_path: &Self::Path) -> Self::Path {
            *parent_path + *child_path
        }

        fn rake(&self, left_point: &Self::Point, right_point: &Self::Point) -> Self::Point {
            *left_point + *right_point
        }

        fn add_edge(
            &self,
            path: &Self::Path,
            _parent_vertex: usize,
            _child_vertex: usize,
        ) -> Self::Point {
            *path
        }

        fn add_vertex(&self, point: &Self::Point, vertex: usize) -> Self::Path {
            *point + self.vertex_values[vertex]
        }
    }

    struct TraversalTextOperator;

    impl StaticTopTreeOperator for TraversalTextOperator {
        type Path = String;
        type Point = String;

        fn singleton(&self, vertex: usize) -> Self::Path {
            vertex.to_string()
        }

        fn compress(&self, parent_path: &Self::Path, child_path: &Self::Path) -> Self::Path {
            format!("{parent_path}>{child_path}")
        }

        fn rake(&self, left_point: &Self::Point, right_point: &Self::Point) -> Self::Point {
            format!("{left_point}+{right_point}")
        }

        fn add_edge(
            &self,
            path: &Self::Path,
            parent_vertex: usize,
            child_vertex: usize,
        ) -> Self::Point {
            format!("{parent_vertex}-{child_vertex}[{path}]")
        }

        fn add_vertex(&self, point: &Self::Point, vertex: usize) -> Self::Path {
            format!("{vertex}({point})")
        }
    }

    #[test]
    fn from_edges_returns_single_vertex_value_for_single_vertex_tree() {
        // Arrange
        let vertex_size = 1usize;
        let edges = Vec::<(usize, usize)>::new();
        let root = 0usize;
        let operator = VertexSumOperator {
            vertex_values: vec![7i64],
        };
        let expected_total = 7i64;

        // Act
        let top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let actual_total = *top_tree.all();

        // Assert
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn structure_contains_distinguishes_last_valid_vertex_and_next_invalid_vertex() {
        // Arrange
        let vertex_size = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let root = 0usize;
        let structure = StaticTopTreeStructure::new(vertex_size, &edges, root);
        let lower_boundary_vertex = vertex_size - 2;
        let upper_valid_vertex = vertex_size - 1;
        let first_invalid_vertex = vertex_size;
        let expected_lower_boundary = true;
        let expected_upper_valid = true;
        let expected_first_invalid = false;

        // Act
        let actual_lower_boundary = structure.contains(lower_boundary_vertex);
        let actual_upper_valid = structure.contains(upper_valid_vertex);
        let actual_first_invalid = structure.contains(first_invalid_vertex);

        // Assert
        assert_eq!(actual_lower_boundary, expected_lower_boundary);
        assert_eq!(actual_upper_valid, expected_upper_valid);
        assert_eq!(actual_first_invalid, expected_first_invalid);
    }

    #[test]
    fn structure_heavy_child_returns_largest_subtree_child_for_root_vertex() {
        // Arrange
        let vertex_size = 5usize;
        let edges = vec![
            (0usize, 1usize),
            (1usize, 2usize),
            (1usize, 3usize),
            (0usize, 4usize),
        ];
        let root = 0usize;
        let structure = StaticTopTreeStructure::new(vertex_size, &edges, root);
        let expected_heavy_child = Some(1usize);

        // Act
        let actual_heavy_child = structure.heavy_child(root);

        // Assert
        assert_eq!(actual_heavy_child, expected_heavy_child);
    }

    #[test]
    fn structure_new_panics_when_vertex_size_is_zero() {
        // Arrange
        let vertex_size = 0usize;
        let edges = Vec::<(usize, usize)>::new();
        let root = 0usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_new_panics_when_root_is_out_of_range() {
        // Arrange
        let vertex_size = 1usize;
        let edges = Vec::<(usize, usize)>::new();
        let root = vertex_size;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_new_panics_when_edge_count_is_less_than_tree_size() {
        // Arrange
        let vertex_size = 3usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_new_panics_when_edge_count_is_greater_than_tree_size() {
        // Arrange
        let vertex_size = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize), (0usize, 2usize)];
        let root = 0usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_new_panics_when_self_loop_edge_exists() {
        // Arrange
        let vertex_size = 2usize;
        let edges = vec![(0usize, 0usize)];
        let root = 0usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_new_panics_when_graph_is_disconnected() {
        // Arrange
        let vertex_size = 4usize;
        let edges = vec![(0usize, 1usize), (2usize, 3usize), (2usize, 3usize)];
        let root = 0usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_new_panics_when_cycle_exists() {
        // Arrange
        let vertex_size = 3usize;
        let edges = vec![(0usize, 1usize), (0usize, 1usize)];
        let root = 0usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn structure_accessors_return_expected_values_for_valid_vertices() {
        // Arrange
        let vertex_size = 4usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize), (1usize, 3usize)];
        let root = 0usize;
        let structure = StaticTopTreeStructure::new(vertex_size, &edges, root);
        let expected_parent = Some(1usize);
        let expected_children = vec![2usize, 3usize];
        let expected_subtree_size = Some(3usize);
        let parent_vertex = 2usize;
        let children_vertex = 1usize;
        let subtree_vertex = 1usize;

        // Act
        let actual_parent = structure.parent_vertex(parent_vertex);
        let actual_children = structure
            .children_vertices(children_vertex)
            .unwrap()
            .to_vec();
        let actual_subtree_size = structure.subtree_size(subtree_vertex);

        // Assert
        assert_eq!(actual_parent, expected_parent);
        assert_eq!(actual_children, expected_children);
        assert_eq!(actual_subtree_size, expected_subtree_size);
    }

    #[test]
    fn structure_accessors_return_none_for_out_of_range_vertex() {
        // Arrange
        let vertex_size = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let invalid_vertex = vertex_size;
        let expected_value: Option<usize> = None;
        let expected_children: Option<&[usize]> = None;
        let structure = StaticTopTreeStructure::new(vertex_size, &edges, root);

        // Act
        let actual_parent = structure.parent_vertex(invalid_vertex);
        let actual_children = structure.children_vertices(invalid_vertex);
        let actual_heavy_child = structure.heavy_child(invalid_vertex);
        let actual_subtree_size = structure.subtree_size(invalid_vertex);

        // Assert
        assert_eq!(actual_parent, expected_value);
        assert_eq!(actual_children, expected_children);
        assert_eq!(actual_heavy_child, expected_value);
        assert_eq!(actual_subtree_size, expected_value);
    }

    #[test]
    fn structure_accepts_last_vertex_edge_and_nonzero_root() {
        // Arrange
        let vertex_size = 3;
        let last_vertex = vertex_size - 1;
        let edges = vec![(1, 0), (1, last_vertex)];
        let root = 1;
        let expected_parent = Some(root);
        let expected_subtree_size = Some(vertex_size);

        // Act
        let structure = StaticTopTreeStructure::new(vertex_size, &edges, root);
        let actual_parent = structure.parent_vertex(last_vertex);
        let actual_subtree_size = structure.subtree_size(root);

        // Assert
        assert_eq!(actual_parent, expected_parent);
        assert_eq!(actual_subtree_size, expected_subtree_size);
    }

    #[test]
    fn structure_new_panics_when_edge_endpoint_equals_vertex_size() {
        // Arrange
        let vertex_size = 2;
        let edges = vec![(0, vertex_size)];
        let root = 0;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| {
            StaticTopTreeStructure::new(vertex_size, &edges, root)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn structure_cluster_indices_are_valid_for_single_and_multiple_vertex_trees() {
        // Arrange
        let single_structure =
            StaticTopTreeStructure::new(1usize, &Vec::<(usize, usize)>::new(), 0usize);
        let multiple_edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let multiple_structure = StaticTopTreeStructure::new(3usize, &multiple_edges, 0usize);

        // Act
        let single_result = single_structure.cluster_root() < single_structure.cluster_count();
        let multiple_result =
            multiple_structure.cluster_root() < multiple_structure.cluster_count();

        // Assert
        assert!(single_result);
        assert!(multiple_result);
    }

    #[test]
    fn all_preserves_input_order_for_multiple_light_children_during_rake_merge() {
        // Arrange
        let vertex_size = 4usize;
        let edges = vec![(0usize, 1usize), (0usize, 2usize), (0usize, 3usize)];
        let root = 0usize;
        let operator = TraversalTextOperator;
        let expected_text = "0(0-2[2]+0-3[3])>1".to_string();

        // Act
        let top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let actual_text = top_tree.all().clone();

        // Assert
        assert_eq!(actual_text, expected_text);
    }

    #[test]
    fn refresh_vertex_updates_total_sum_for_root_vertex_on_chain_tree() {
        // Arrange
        let vertex_size = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let root = 0usize;
        let operator = VertexSumOperator {
            vertex_values: vec![1i64, 2i64, 3i64],
        };
        let mut top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let updated_root_value = 5i64;
        let expected_total = 10i64;
        let expected_refresh = Some(());

        // Act
        top_tree.operator_mut().vertex_values[root] = updated_root_value;
        let actual_refresh = top_tree.refresh_vertex(root);
        let actual_total = *top_tree.all();

        // Assert
        assert_eq!(actual_refresh, expected_refresh);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn refresh_vertex_updates_total_sum_for_light_leaf_vertex_on_branching_tree() {
        // Arrange
        let vertex_size = 5usize;
        let edges = vec![
            (0usize, 1usize),
            (1usize, 2usize),
            (1usize, 3usize),
            (0usize, 4usize),
        ];
        let root = 0usize;
        let light_leaf_vertex = 4usize;
        let operator = VertexSumOperator {
            vertex_values: vec![1i64, 2i64, 3i64, 4i64, 5i64],
        };
        let mut top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let updated_light_leaf_value = 11i64;
        let expected_total = 21i64;
        let expected_refresh = Some(());

        // Act
        top_tree.operator_mut().vertex_values[light_leaf_vertex] = updated_light_leaf_value;
        let actual_refresh = top_tree.refresh_vertex(light_leaf_vertex);
        let actual_total = *top_tree.all();

        // Assert
        assert_eq!(actual_refresh, expected_refresh);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn refresh_vertex_returns_none_for_first_invalid_vertex() {
        // Arrange
        let vertex_size = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let operator = VertexSumOperator {
            vertex_values: vec![10i64, 20i64],
        };
        let mut top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let first_invalid_vertex = vertex_size;
        let expected_refresh = None;

        // Act
        let actual_refresh = top_tree.refresh_vertex(first_invalid_vertex);

        // Assert
        assert_eq!(actual_refresh, expected_refresh);
    }

    #[test]
    fn refresh_vertex_updates_total_sum_for_heavy_path_non_root_vertex() {
        // Arrange
        let vertex_size = 3usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize)];
        let root = 0usize;
        let target_vertex = 2usize;
        let operator = VertexSumOperator {
            vertex_values: vec![1i64, 2i64, 3i64],
        };
        let mut top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let updated_value = 10i64;
        let expected_total = 13i64;
        let expected_refresh = Some(());

        // Act
        top_tree.operator_mut().vertex_values[target_vertex] = updated_value;
        let actual_refresh = top_tree.refresh_vertex(target_vertex);
        let actual_total = *top_tree.all();

        // Assert
        assert_eq!(actual_refresh, expected_refresh);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn rebuild_recalculates_total_sum_after_multiple_vertex_value_changes() {
        // Arrange
        let vertex_size = 4usize;
        let edges = vec![(0usize, 1usize), (1usize, 2usize), (1usize, 3usize)];
        let root = 0usize;
        let operator = VertexSumOperator {
            vertex_values: vec![1i64, 2i64, 3i64, 4i64],
        };
        let mut top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let replacement_values = vec![10i64, 20i64, 30i64, 40i64];
        let expected_total = 100i64;

        // Act
        top_tree.operator_mut().vertex_values = replacement_values;
        top_tree.rebuild();
        let actual_total = *top_tree.all();

        // Assert
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn top_tree_structure_and_operator_accessors_return_references() {
        // Arrange
        let vertex_size = 1usize;
        let edges = Vec::<(usize, usize)>::new();
        let root = 0usize;
        let operator = VertexSumOperator {
            vertex_values: vec![7i64],
        };
        let top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let expected_root = root;
        let expected_value = 7i64;

        // Act
        let actual_root = top_tree.structure().root();
        let actual_value = top_tree.operator().vertex_values[0];

        // Assert
        assert_eq!(actual_root, expected_root);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn top_tree_len_and_is_empty_match_structure() {
        // Arrange
        let vertex_size = 2usize;
        let edges = vec![(0usize, 1usize)];
        let root = 0usize;
        let operator = VertexSumOperator {
            vertex_values: vec![10i64, 20i64],
        };
        let top_tree = StaticTopTree::from_edges(vertex_size, &edges, root, operator);
        let expected_len = top_tree.structure().len();
        let expected_is_empty = top_tree.structure().is_empty();

        // Act
        let actual_len = top_tree.len();
        let actual_is_empty = top_tree.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }
}
