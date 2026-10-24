use std::vec::Vec;

#[derive(Debug)]
pub struct SccGraph {
    vertex_count: usize,
    edges: Vec<(usize, usize)>,
}

impl SccGraph {
    pub fn new(vertex_count: usize) -> Self {
        Self {
            vertex_count,
            edges: Vec::new(),
        }
    }

    pub fn add_edge(&mut self, from: usize, to: usize) {
        assert!(from < self.vertex_count);
        assert!(to < self.vertex_count);

        self.edges.push((from, to));
    }

    pub fn scc(&self) -> Vec<Vec<usize>> {

        let n = self.vertex_count;
        let mut graph = vec![Vec::new(); n];
        let mut rev_graph = vec![Vec::new(); n];
        for &(u, v) in &self.edges {
            graph[u].push(v);
            rev_graph[v].push(u);
        }

        let mut order = Vec::with_capacity(n);
        let mut visited = vec![false; n];
        for v in 0..n {

            if visited[v] {
                continue;
            }

            dfs_forward(v, &graph, &mut visited, &mut order);
        }

        let mut comp_id = vec![None; n];
        let mut components = Vec::new();
        for &v in order.iter().rev() {

            if comp_id[v].is_some() {
                continue;
            }

            let mut comp = Vec::new();
            dfs_backward(v, components.len(), &rev_graph, &mut comp_id, &mut comp);
            components.push(comp);
        }

        components
    }
}

fn dfs_forward(
    start: usize,
    graph: &[Vec<usize>],
    visited: &mut [bool],
    order: &mut Vec<usize>,
) {
    let mut stack = vec![(start, false)];
    while let Some((v, processed)) = stack.pop() {

        if processed {
            order.push(v);
            continue;
        }

        if visited[v] {
            continue;
        }

        visited[v] = true;
        stack.push((v, true));

        for &to in &graph[v] {

            if !visited[to] {
                stack.push((to, false));
            }
        }
    }
}

fn dfs_backward(
    start: usize,
    comp_idx: usize,
    graph: &[Vec<usize>],
    comp_id: &mut [Option<usize>],
    comp: &mut Vec<usize>,
) {
    let mut stack = vec![start];
    while let Some(v) = stack.pop() {

        if comp_id[v].is_some() {
            continue;
        }

        comp_id[v] = Some(comp_idx);
        comp.push(v);

        for &to in &graph[v] {

            if comp_id[to].is_none() {
                stack.push(to);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::SccGraph;

    #[test]
    fn empty_graph_returns_empty_components() {
        // Arrange
        let vertex_count = 0;
        let expected_components: Vec<Vec<usize>> = vec![];
        let graph = SccGraph::new(vertex_count);

        // Act
        let actual_components = graph.scc();

        // Assert
        assert_eq!(actual_components, expected_components);
    }

    #[test]
    fn multi_edges_do_not_change_scc_result() {
        // Arrange
        let vertex_count = 2;
        let edges: [(usize, usize); 3] = [(0, 1), (0, 1), (1, 0)];
        let expected_components = vec![vec![0, 1]];
        let mut graph = SccGraph::new(vertex_count);
        for (from, to) in edges {
            graph.add_edge(from, to);
        }
        // Act
        let actual_components = graph.scc();
        // Assert
        assert_eq!(actual_components, expected_components);
    }

    #[test]
    fn single_vertex_self_loop_returns_one_component() {
        // Arrange
        let vertex_count = 1;
        let self_vertex = 0;
        let expected_components = vec![vec![0]];
        let mut graph = SccGraph::new(vertex_count);
        graph.add_edge(self_vertex, self_vertex);

        // Act
        let actual_components = graph.scc();

        // Assert
        assert_eq!(actual_components, expected_components);
    }

    #[test]
    fn graph_with_one_pair_and_two_singletons_returns_three_components() {
        // Arrange
        let vertex_count = 4;
        let edges = [(0, 1), (1, 0), (2, 3)];
        let expected_component_count = 3;
        let expected_components = [vec![0, 1], vec![2], vec![3]];
        let mut graph = SccGraph::new(vertex_count);
        for (from, to) in edges {
            graph.add_edge(from, to);
        }

        // Act
        let actual_components = graph.scc();

        // Assert
        assert_eq!(actual_components.len(), expected_component_count);
        for expected_component in expected_components {
            assert!(actual_components.contains(&expected_component));
        }
    }

    #[test]
    fn linear_chain_results_in_singletons() {
        // Arrange
        let vertex_count = 3;
        let edges = [(0, 1), (1, 2)];
        let expected_component_count = 3;
        let expected_components = [vec![0], vec![1], vec![2]];
        let mut graph = SccGraph::new(vertex_count);
        for (from, to) in edges {
            graph.add_edge(from, to);
        }

        // Act
        let actual_components = graph.scc();

        // Assert
        assert_eq!(actual_components.len(), expected_component_count);
        for expected_component in expected_components {
            assert!(actual_components.contains(&expected_component));
        }
    }

    #[test]
    fn isolated_vertices_form_singleton_components() {
        // Arrange
        let vertex_count = 3;
        let edges: [(usize, usize); 0] = [];
        let expected_components = vec![vec![2], vec![1], vec![0]];
        let mut graph = SccGraph::new(vertex_count);
        for (from, to) in edges {
            graph.add_edge(from, to);
        }
        // Act
        let actual_components = graph.scc();
        // Assert
        assert_eq!(actual_components, expected_components);
    }

    #[test]
    fn self_loop_vertex_stays_single_component() {
        // Arrange
        let vertex_count = 2;
        let edges: [(usize, usize); 2] = [(0, 0), (0, 1)];
        let expected_components = vec![vec![0], vec![1]];
        let mut graph = SccGraph::new(vertex_count);
        for (from, to) in edges {
            graph.add_edge(from, to);
        }
        // Act
        let actual_components = graph.scc();
        // Assert
        assert_eq!(actual_components, expected_components);
    }

    #[test]
    fn add_edge_panics_when_from_equals_vertex_count() {
        // Arrange
        let vertex_count = 2;
        let from = vertex_count;
        let to = vertex_count - 1;
        let mut graph = SccGraph::new(vertex_count);

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| graph.add_edge(from, to)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn add_edge_panics_when_to_equals_vertex_count() {
        // Arrange
        let vertex_count = 2;
        let from = vertex_count - 1;
        let to = vertex_count;
        let mut graph = SccGraph::new(vertex_count);

        // Act
        let actual_result =
            std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| graph.add_edge(from, to)));

        // Assert
        assert!(actual_result.is_err());
    }
}
