use std::ops::{Index, IndexMut, Neg};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FlowEdge<Capacity> {
    pub to: usize,
    pub rev: usize,
    pub capacity: Capacity,
}

impl<Capacity> FlowEdge<Capacity> {
    fn new(to: usize, rev: usize, capacity: Capacity) -> Self {
        Self { to, rev, capacity }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CostEdge<Capacity, Cost> {
    pub to: usize,
    pub rev: usize,
    pub capacity: Capacity,
    pub cost: Cost,
}

impl<Capacity, Cost> CostEdge<Capacity, Cost> {
    fn new(to: usize, rev: usize, capacity: Capacity, cost: Cost) -> Self {
        Self {
            to,
            rev,
            capacity,
            cost,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Graph<Edge> {
    adjacency_lists: Vec<Vec<Edge>>,
}

pub type FlowGraph<Capacity> = Graph<FlowEdge<Capacity>>;
pub type CostFlowGraph<Capacity, Cost> = Graph<CostEdge<Capacity, Cost>>;

impl<Edge> Graph<Edge> {
    pub fn new(vertex_count: usize) -> Self {
        let adjacency_lists: Vec<Vec<Edge>> = (0..vertex_count).map(|_| Vec::new()).collect();

        Self { adjacency_lists }
    }

    pub fn vertex_count(&self) -> usize {
        self.adjacency_lists.len()
    }

    pub fn validate_vertex_index(&self, vertex_index: usize) {
        assert!(
            vertex_index < self.vertex_count(),
            "vertex index is out of range"
        );
    }
}

impl<Edge> Index<usize> for Graph<Edge> {
    type Output = Vec<Edge>;

    fn index(&self, vertex_index: usize) -> &Self::Output {
        &self.adjacency_lists[vertex_index]
    }
}

impl<Edge> IndexMut<usize> for Graph<Edge> {
    fn index_mut(&mut self, vertex_index: usize) -> &mut Self::Output {
        &mut self.adjacency_lists[vertex_index]
    }
}

impl<Capacity> Graph<FlowEdge<Capacity>>
where
    Capacity: Copy + Default,
{
    pub fn add_edge(
        &mut self,
        from_vertex_index: usize,
        to_vertex_index: usize,
        capacity: Capacity,
    ) {
        self.validate_vertex_index(from_vertex_index);
        self.validate_vertex_index(to_vertex_index);

        let from_reverse_edge_index = self[to_vertex_index].len()
            + usize::from(from_vertex_index == to_vertex_index);
        let to_reverse_edge_index = self[from_vertex_index].len();

        self[from_vertex_index].push(FlowEdge::new(
            to_vertex_index,
            from_reverse_edge_index,
            capacity,
        ));
        self[to_vertex_index].push(FlowEdge::new(
            from_vertex_index,
            to_reverse_edge_index,
            Capacity::default(),
        ));
    }
}

impl<Capacity, Cost> Graph<CostEdge<Capacity, Cost>>
where
    Capacity: Copy + Default,
    Cost: Copy + Neg<Output = Cost>,
{
    pub fn add_edge(
        &mut self,
        from_vertex_index: usize,
        to_vertex_index: usize,
        capacity: Capacity,
        cost: Cost,
    ) {
        self.validate_vertex_index(from_vertex_index);
        self.validate_vertex_index(to_vertex_index);

        let from_reverse_edge_index = self[to_vertex_index].len()
            + usize::from(from_vertex_index == to_vertex_index);
        let to_reverse_edge_index = self[from_vertex_index].len();

        self[from_vertex_index].push(CostEdge::new(
            to_vertex_index,
            from_reverse_edge_index,
            capacity,
            cost,
        ));
        self[to_vertex_index].push(CostEdge::new(
            from_vertex_index,
            to_reverse_edge_index,
            Capacity::default(),
            -cost,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::{CostFlowGraph, FlowGraph};

    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn graph_new_with_zero_one_and_three_vertices_returns_requested_vertex_count() {
        // Arrange
        let empty_vertex_count = 0;
        let single_vertex_count = 1;
        let multiple_vertex_count = 3;
        let empty_graph = FlowGraph::<i64>::new(empty_vertex_count);
        let single_graph = FlowGraph::<i64>::new(single_vertex_count);
        let multiple_graph = FlowGraph::<i64>::new(multiple_vertex_count);

        // Act
        let empty_result = empty_graph.vertex_count();
        let single_result = single_graph.vertex_count();
        let multiple_result = multiple_graph.vertex_count();

        // Assert
        assert_eq!(empty_vertex_count, empty_result);
        assert_eq!(single_vertex_count, single_result);
        assert_eq!(multiple_vertex_count, multiple_result);
    }

    #[test]
    fn flow_graph_add_edge_with_single_edge_creates_forward_and_reverse_edges() {
        // Arrange
        let from_vertex_index = 0;
        let to_vertex_index = 1;
        let capacity = 2_i64;
        let mut flow_graph = FlowGraph::new(2);

        let expected_forward_index = 0;
        let expected_reverse_index = 0;
        let expected_reverse_capacity = 0;

        // Act
        flow_graph.add_edge(from_vertex_index, to_vertex_index, capacity);
        let forward_edge = flow_graph[from_vertex_index][0];
        let reverse_edge = flow_graph[to_vertex_index][0];

        // Assert
        assert_eq!(to_vertex_index, forward_edge.to);
        assert_eq!(expected_forward_index, forward_edge.rev);
        assert_eq!(capacity, forward_edge.capacity);
        assert_eq!(from_vertex_index, reverse_edge.to);
        assert_eq!(expected_reverse_index, reverse_edge.rev);
        assert_eq!(expected_reverse_capacity, reverse_edge.capacity);
    }

    #[test]
    fn flow_graph_add_edge_with_parallel_edges_keeps_reverse_indices_consistent() {
        // Arrange
        let from_vertex_index = 0;
        let to_vertex_index = 1;
        let first_capacity = 1_i64;
        let second_capacity = 2_i64;
        let mut flow_graph = FlowGraph::new(2);

        let expected_forward_index = 0;
        let expected_1_left = 1;
        let expected_reverse_index = 0;
        let expected_1_left_2 = 1;

        // Act
        flow_graph.add_edge(from_vertex_index, to_vertex_index, first_capacity);
        flow_graph.add_edge(from_vertex_index, to_vertex_index, second_capacity);
        let first_forward_edge = flow_graph[from_vertex_index][0];
        let second_forward_edge = flow_graph[from_vertex_index][1];
        let first_reverse_edge = flow_graph[to_vertex_index][0];
        let second_reverse_edge = flow_graph[to_vertex_index][1];

        // Assert
        assert_eq!(expected_forward_index, first_forward_edge.rev);
        assert_eq!(expected_1_left, second_forward_edge.rev);
        assert_eq!(expected_reverse_index, first_reverse_edge.rev);
        assert_eq!(expected_1_left_2, second_reverse_edge.rev);
        assert_eq!(first_capacity, first_forward_edge.capacity);
        assert_eq!(second_capacity, second_forward_edge.capacity);
    }

    #[test]
    fn cost_flow_graph_add_edge_with_positive_cost_creates_negative_reverse_cost() {
        // Arrange
        let from_vertex_index = 0;
        let to_vertex_index = 1;
        let capacity = 3_i64;
        let cost = 5_i64;
        let mut flow_graph = CostFlowGraph::new(2);

        let expected_reverse_capacity = 0;

        // Act
        flow_graph.add_edge(from_vertex_index, to_vertex_index, capacity, cost);
        let forward_edge = flow_graph[from_vertex_index][0];
        let reverse_edge = flow_graph[to_vertex_index][0];

        // Assert
        assert_eq!(to_vertex_index, forward_edge.to);
        assert_eq!(capacity, forward_edge.capacity);
        assert_eq!(cost, forward_edge.cost);
        assert_eq!(from_vertex_index, reverse_edge.to);
        assert_eq!(expected_reverse_capacity, reverse_edge.capacity);
        assert_eq!(-cost, reverse_edge.cost);
    }

    #[test]
    fn add_edge_panics_for_out_of_range_vertex() {
        // Arrange
        let mut graph = FlowGraph::<i64>::new(2);

        let add_edge_from_vertex = 0;
        let add_edge_to_vertex = 2;
        let add_edge_capacity = 1;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| {
            graph.add_edge(add_edge_from_vertex, add_edge_to_vertex, add_edge_capacity)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn negative_cost_creates_positive_reverse_cost() {
        // Arrange
        let mut graph = CostFlowGraph::<i64, i64>::new(2);
        let expected_reverse_cost = 7;

        let add_edge_from_vertex = 0;
        let add_edge_to_vertex = 1;
        let add_edge_capacity = 3;
        let add_edge_cost = -7;

        // Act
        graph.add_edge(
            add_edge_from_vertex,
            add_edge_to_vertex,
            add_edge_capacity,
            add_edge_cost,
        );
        let actual_reverse_cost = graph[1][0].cost;

        // Assert
        assert_eq!(actual_reverse_cost, expected_reverse_cost);
    }

    #[test]
    fn self_loop_keeps_consistent_reverse_indices() {
        // Arrange
        let vertex = 0usize;
        let forward_index = 0usize;
        let reverse_index = 1usize;
        let capacity = 5i64;
        let expected_reverse_capacity = 0i64;
        let expected_edge_count = 2usize;
        let mut graph = FlowGraph::new(1);

        // Act
        graph.add_edge(vertex, vertex, capacity);
        let actual_forward = graph[vertex][forward_index];
        let actual_reverse = graph[actual_forward.to][actual_forward.rev];
        let actual_round_trip = graph[actual_reverse.to][actual_reverse.rev];
        let actual_edge_count = graph[vertex].len();

        // Assert
        assert_eq!(actual_forward.to, vertex);
        assert_eq!(actual_forward.rev, reverse_index);
        assert_eq!(actual_forward.capacity, capacity);
        assert_eq!(actual_reverse.to, vertex);
        assert_eq!(actual_reverse.rev, forward_index);
        assert_eq!(actual_reverse.capacity, expected_reverse_capacity);
        assert_eq!(actual_round_trip, actual_forward);
        assert_eq!(actual_edge_count, expected_edge_count);
    }

    #[test]
    fn cost_flow_self_loop_keeps_consistent_reverse_indices() {
        // Arrange
        let vertex = 0usize;
        let forward_index = 0usize;
        let reverse_index = 1usize;
        let capacity = 5i64;
        let expected_reverse_capacity = 0i64;
        let expected_edge_count = 2usize;
        let cost = 7i64;
        let expected_reverse_cost = -cost;
        let mut graph = CostFlowGraph::new(1);

        // Act
        graph.add_edge(vertex, vertex, capacity, cost);
        let actual_forward = graph[vertex][forward_index];
        let actual_reverse = graph[actual_forward.to][actual_forward.rev];
        let actual_round_trip = graph[actual_reverse.to][actual_reverse.rev];
        let actual_edge_count = graph[vertex].len();

        // Assert
        assert_eq!(actual_forward.to, vertex);
        assert_eq!(actual_forward.rev, reverse_index);
        assert_eq!(actual_forward.capacity, capacity);
        assert_eq!(actual_reverse.to, vertex);
        assert_eq!(actual_reverse.rev, forward_index);
        assert_eq!(actual_reverse.capacity, expected_reverse_capacity);
        assert_eq!(actual_round_trip, actual_forward);
        assert_eq!(actual_edge_count, expected_edge_count);
        assert_eq!(actual_forward.cost, cost);
        assert_eq!(actual_reverse.cost, expected_reverse_cost);
    }

}
