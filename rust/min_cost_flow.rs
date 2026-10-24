#[path = "flow_graph.rs"]
mod flow_graph;
#[path = "flow_value.rs"]
mod flow_value;

use self::flow_graph::CostFlowGraph;
use self::flow_value::{CapacityValue, CostValue};
use std::cmp::Reverse;
use std::collections::BinaryHeap;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinCostFlowResult<Capacity = i64, Cost = i64> {
    pub flow: Capacity,
    pub cost: Cost,
}

#[derive(Debug)]
struct ShortestPathState<Cost> {
    distances: Vec<Cost>,
    previous_vertex_indices: Vec<Option<usize>>,
    previous_edge_indices: Vec<Option<usize>>,
}

#[derive(Debug, Clone)]
pub struct MinCostFlow<Capacity = i64, Cost = i64> {
    graph: CostFlowGraph<Capacity, Cost>,
}

impl<Capacity, Cost> MinCostFlow<Capacity, Cost>
where
    Capacity: CapacityValue + Into<Cost>,
    Cost: CostValue,
{
    pub fn new(vertex_count: usize) -> Self {
        Self {
            graph: CostFlowGraph::new(vertex_count),
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.graph.vertex_count()
    }

    pub fn add_edge(
        &mut self,
        from_vertex_index: usize,
        to_vertex_index: usize,
        capacity: Capacity,
        cost: Cost,
    ) {
        self.graph.validate_vertex_index(from_vertex_index);
        self.graph.validate_vertex_index(to_vertex_index);
        assert!(
            Capacity::default() <= capacity,
            "capacity must be non-negative"
        );

        self.graph
            .add_edge(from_vertex_index, to_vertex_index, capacity, cost);
    }

    pub fn flow(
        &mut self,
        source_vertex_index: usize,
        sink_vertex_index: usize,
        flow_limit: Capacity,
    ) -> MinCostFlowResult<Capacity, Cost> {
        self.graph.validate_vertex_index(source_vertex_index);
        self.graph.validate_vertex_index(sink_vertex_index);
        assert!(
            Capacity::default() <= flow_limit,
            "flow_limit must be non-negative"
        );

        if flow_limit == Capacity::default() {
            return MinCostFlowResult {
                flow: Capacity::default(),
                cost: Cost::default(),
            };
        }

        if source_vertex_index == sink_vertex_index {
            return MinCostFlowResult {
                flow: flow_limit,
                cost: Cost::default(),
            };
        }

        let mut accumulated_flow = Capacity::default();
        let mut accumulated_cost = Cost::default();
        let mut potential = self.calculate_initial_potential();

        while accumulated_flow < flow_limit {
            let shortest_path_state = self.find_shortest_path(source_vertex_index, &potential);

            if shortest_path_state.distances[sink_vertex_index] == Cost::max_value() {
                break;
            }

            self.update_potential(&mut potential, &shortest_path_state.distances);

            let remaining_flow_limit = flow_limit - accumulated_flow;
            let additional_flow = self.calculate_additional_flow(
                source_vertex_index,
                sink_vertex_index,
                remaining_flow_limit,
                &shortest_path_state.previous_vertex_indices,
                &shortest_path_state.previous_edge_indices,
            );
            let path_cost = self.push_additional_flow(
                source_vertex_index,
                sink_vertex_index,
                additional_flow,
                &shortest_path_state.previous_vertex_indices,
                &shortest_path_state.previous_edge_indices,
            );

            accumulated_flow += additional_flow;
            accumulated_cost += path_cost * self.convert_capacity_to_cost(additional_flow);
        }

        MinCostFlowResult {
            flow: accumulated_flow,
            cost: accumulated_cost,
        }
    }

    pub fn min_cost_to_send(
        &mut self,
        source_vertex_index: usize,
        sink_vertex_index: usize,
        required_flow: Capacity,
    ) -> Option<Cost> {
        let flow_result = self.flow(source_vertex_index, sink_vertex_index, required_flow);

        if flow_result.flow == required_flow {
            Some(flow_result.cost)
        } else {
            None
        }
    }

    fn calculate_initial_potential(&self) -> Vec<Cost> {
        let vertex_count = self.graph.vertex_count();
        let mut potential = vec![Cost::default(); vertex_count];

        for _ in 0..vertex_count {
            let mut updated = false;

            for from_vertex_index in 0..vertex_count {
                let from_potential = potential[from_vertex_index];

                for edge in &self.graph[from_vertex_index] {
                    if edge.capacity == Capacity::default() {
                        continue;
                    }

                    let next_potential = from_potential + edge.cost;

                    if next_potential < potential[edge.to] {
                        potential[edge.to] = next_potential;
                        updated = true;
                    }
                }
            }

            if !updated {
                break;
            }
        }

        potential
    }

    fn find_shortest_path(
        &self,
        source_vertex_index: usize,
        potential: &[Cost],
    ) -> ShortestPathState<Cost> {
        let vertex_count = self.graph.vertex_count();
        let mut distances = vec![Cost::max_value(); vertex_count];
        let mut previous_vertex_indices = vec![None; vertex_count];
        let mut previous_edge_indices = vec![None; vertex_count];
        let mut priority_queue = BinaryHeap::new();

        distances[source_vertex_index] = Cost::default();
        priority_queue.push(Reverse((Cost::default(), source_vertex_index)));

        while let Some(Reverse((current_distance, current_vertex_index))) = priority_queue.pop() {
            if distances[current_vertex_index] < current_distance {
                continue;
            }

            for (edge_index, edge) in self.graph[current_vertex_index].iter().enumerate() {
                if edge.capacity == Capacity::default() {
                    continue;
                }

                let reduced_cost = edge.cost + potential[current_vertex_index] - potential[edge.to];
                debug_assert!(Cost::default() <= reduced_cost);

                let next_distance = current_distance + reduced_cost;

                if next_distance < distances[edge.to] {
                    distances[edge.to] = next_distance;
                    previous_vertex_indices[edge.to] = Some(current_vertex_index);
                    previous_edge_indices[edge.to] = Some(edge_index);
                    priority_queue.push(Reverse((next_distance, edge.to)));
                }
            }
        }

        ShortestPathState {
            distances,
            previous_vertex_indices,
            previous_edge_indices,
        }
    }

    fn update_potential(&self, potential: &mut [Cost], distances: &[Cost]) {
        for vertex_index in 0..self.graph.vertex_count() {
            if distances[vertex_index] == Cost::max_value() {
                continue;
            }

            potential[vertex_index] += distances[vertex_index];
        }
    }

    fn calculate_additional_flow(
        &self,
        source_vertex_index: usize,
        sink_vertex_index: usize,
        flow_limit: Capacity,
        previous_vertex_indices: &[Option<usize>],
        previous_edge_indices: &[Option<usize>],
    ) -> Capacity {
        let mut additional_flow = flow_limit;
        let mut current_vertex_index = sink_vertex_index;

        while current_vertex_index != source_vertex_index {
            let previous_vertex_index = previous_vertex_indices[current_vertex_index]
                .expect("sink must be reachable before calculating additional flow");
            let previous_edge_index = previous_edge_indices[current_vertex_index]
                .expect("sink must be reachable before calculating additional flow");
            let edge_capacity = self.graph[previous_vertex_index][previous_edge_index].capacity;

            additional_flow = additional_flow.min(edge_capacity);
            current_vertex_index = previous_vertex_index;
        }

        additional_flow
    }

    fn push_additional_flow(
        &mut self,
        source_vertex_index: usize,
        sink_vertex_index: usize,
        additional_flow: Capacity,
        previous_vertex_indices: &[Option<usize>],
        previous_edge_indices: &[Option<usize>],
    ) -> Cost {
        let mut current_vertex_index = sink_vertex_index;
        let mut path_cost = Cost::default();

        while current_vertex_index != source_vertex_index {
            let previous_vertex_index = previous_vertex_indices[current_vertex_index]
                .expect("sink must be reachable before pushing additional flow");
            let previous_edge_index = previous_edge_indices[current_vertex_index]
                .expect("sink must be reachable before pushing additional flow");
            let reverse_edge_index = self.graph[previous_vertex_index][previous_edge_index].rev;
            let edge_cost = self.graph[previous_vertex_index][previous_edge_index].cost;

            self.graph[previous_vertex_index][previous_edge_index].capacity -= additional_flow;
            self.graph[current_vertex_index][reverse_edge_index].capacity += additional_flow;
            path_cost += edge_cost;
            current_vertex_index = previous_vertex_index;
        }

        path_cost
    }

    fn convert_capacity_to_cost(&self, flow_value: Capacity) -> Cost {
        flow_value.into()
    }
}

#[cfg(test)]
mod tests {
    use super::{MinCostFlow, MinCostFlowResult};

    fn create_single_path_graph(path_capacity: i64, path_cost: i64) -> MinCostFlow {
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let mut flow_graph = MinCostFlow::new(2);
        flow_graph.add_edge(
            source_vertex_index,
            sink_vertex_index,
            path_capacity,
            path_cost,
        );
        flow_graph
    }

    #[test]
    fn vertex_count_with_zero_vertices_returns_requested_count() {
        // Arrange
        let empty_vertex_count = 0;
        let empty_flow_graph = MinCostFlow::<i64, i64>::new(empty_vertex_count);

        // Act
        let empty_result = empty_flow_graph.vertex_count();

        // Assert
        assert_eq!(empty_vertex_count, empty_result);
    }

    #[test]
    fn vertex_count_with_one_vertex_returns_requested_count() {
        // Arrange
        let single_vertex_count = 1;
        let single_flow_graph = MinCostFlow::<i64, i64>::new(single_vertex_count);

        // Act
        let single_result = single_flow_graph.vertex_count();

        // Assert
        assert_eq!(single_vertex_count, single_result);
    }

    #[test]
    fn vertex_count_with_three_vertices_returns_requested_count() {
        // Arrange
        let multiple_vertex_count = 3;
        let multiple_flow_graph = MinCostFlow::<i64, i64>::new(multiple_vertex_count);

        // Act
        let multiple_result = multiple_flow_graph.vertex_count();

        // Assert
        assert_eq!(multiple_vertex_count, multiple_result);
    }

    #[test]
    fn add_edge_with_from_vertex_index_equal_to_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let from_vertex_index = 2;
        let to_vertex_index = 1;
        let capacity = 1_i64;
        let cost = 1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flow_graph.add_edge(from_vertex_index, to_vertex_index, capacity, cost)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn add_edge_with_to_vertex_index_equal_to_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let from_vertex_index = 0;
        let to_vertex_index = 2;
        let capacity = 1_i64;
        let cost = 1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flow_graph.add_edge(from_vertex_index, to_vertex_index, capacity, cost)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn flow_with_source_vertex_index_equal_to_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let source_vertex_index = 2;
        let sink_vertex_index = 1;
        let flow_limit = 1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);
        flow_graph.add_edge(0, 1, 1_i64, 1_i64);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn flow_with_sink_vertex_index_equal_to_vertex_count_panics() {
        // Arrange
        let vertex_count = 2;
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let flow_limit = 1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);
        flow_graph.add_edge(0, 1, 1_i64, 1_i64);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn add_edge_with_negative_capacity_panics() {
        // Arrange
        let vertex_count = 2;
        let from_vertex_index = 0;
        let to_vertex_index = 1;
        let capacity = -1_i64;
        let cost = 1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flow_graph.add_edge(from_vertex_index, to_vertex_index, capacity, cost)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn flow_with_zero_flow_limit_returns_zero_flow_and_zero_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let path_capacity = 2;
        let path_cost = 5;
        let flow_limit = 0;
        let mut flow_graph = create_single_path_graph(path_capacity, path_cost);
        let expected_result = MinCostFlowResult { flow: 0, cost: 0 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_with_limit_one_on_capacity_two_path_returns_one_unit_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let path_capacity = 2;
        let path_cost = 5;
        let flow_limit = 1;
        let mut flow_graph = create_single_path_graph(path_capacity, path_cost);
        let expected_result = MinCostFlowResult { flow: 1, cost: 5 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_with_limit_two_on_capacity_two_path_returns_two_unit_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let path_capacity = 2;
        let path_cost = 5;
        let flow_limit = 2;
        let mut flow_graph = create_single_path_graph(path_capacity, path_cost);
        let expected_result = MinCostFlowResult { flow: 2, cost: 10 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_with_limit_three_on_capacity_two_path_stops_at_capacity() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let path_capacity = 2;
        let path_cost = 5;
        let flow_limit = 3;
        let mut flow_graph = create_single_path_graph(path_capacity, path_cost);
        let expected_result = MinCostFlowResult { flow: 2, cost: 10 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn min_cost_to_send_with_required_flow_above_capacity_returns_none() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let path_capacity = 2;
        let path_cost = 5;
        let required_flow = 3;
        let mut flow_graph = create_single_path_graph(path_capacity, path_cost);
        let expected_cost = None;

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(expected_cost, actual_cost);
    }

    #[test]
    fn flow_with_two_paths_of_different_costs_accumulates_total_minimum_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 3;
        let vertex_count = 4;
        let flow_limit = 2;
        let mut flow_graph = MinCostFlow::new(vertex_count);
        flow_graph.add_edge(0, 1, 1, 1);
        flow_graph.add_edge(1, 3, 1, 1);
        flow_graph.add_edge(0, 2, 1, 2);
        flow_graph.add_edge(2, 3, 1, 2);
        let expected_result = MinCostFlowResult { flow: 2, cost: 6 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_with_negative_flow_limit_panics() {
        // Arrange
        let vertex_count = 2;
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let flow_limit = -1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);
        flow_graph.add_edge(0, 1, 1_i64, 1_i64);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn flow_with_source_vertex_equal_to_sink_vertex_returns_requested_flow_and_zero_cost() {
        // Arrange
        let vertex_count = 3;
        let source_vertex_index = 1;
        let sink_vertex_index = 1;
        let flow_limit = 4_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(vertex_count);
        let expected_result = MinCostFlowResult {
            flow: flow_limit,
            cost: 0_i64,
        };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_with_zero_cost_paths_returns_zero_total_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let flow_limit = 2_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(3);
        flow_graph.add_edge(0, 1, 2_i64, 0_i64);
        flow_graph.add_edge(1, 2, 2_i64, 0_i64);
        let expected_result = MinCostFlowResult {
            flow: 2_i64,
            cost: 0_i64,
        };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_with_multiple_units_on_negative_cost_path_returns_negative_total_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let flow_limit = 2_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(3);
        flow_graph.add_edge(0, 1, 2_i64, -3_i64);
        flow_graph.add_edge(1, 2, 2_i64, 1_i64);
        let expected_result = MinCostFlowResult {
            flow: 2_i64,
            cost: -4_i64,
        };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(expected_result, actual_result);
    }

    #[test]
    fn flow_called_multiple_times_uses_remaining_residual_capacity() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let path_capacity = 2;
        let path_cost = 5;
        let unit_flow_limit = 1;
        let mut flow_graph = create_single_path_graph(path_capacity, path_cost);
        let expected_first_result = MinCostFlowResult { flow: 1, cost: 5 };
        let expected_second_result = MinCostFlowResult { flow: 1, cost: 5 };
        let expected_third_result = MinCostFlowResult { flow: 0, cost: 0 };

        // Act
        let actual_first_result =
            flow_graph.flow(source_vertex_index, sink_vertex_index, unit_flow_limit);
        let actual_second_result =
            flow_graph.flow(source_vertex_index, sink_vertex_index, unit_flow_limit);
        let actual_third_result =
            flow_graph.flow(source_vertex_index, sink_vertex_index, unit_flow_limit);

        // Assert
        assert_eq!(actual_first_result, expected_first_result);
        assert_eq!(actual_second_result, expected_second_result);
        assert_eq!(actual_third_result, expected_third_result);
    }

    #[test]
    fn flow_prefers_negative_cost_path_over_direct_expensive_edge() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(3);
        flow_graph.add_edge(0, 2, 1, 10);
        flow_graph.add_edge(0, 1, 1, -5);
        flow_graph.add_edge(1, 2, 1, 1);
        let expected_result = MinCostFlowResult { flow: 1, cost: -4 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, 1);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn flow_distributes_required_flow_across_negative_and_positive_cost_paths() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 3;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(4);
        flow_graph.add_edge(0, 1, 1, -2);
        flow_graph.add_edge(1, 3, 1, 1);
        flow_graph.add_edge(0, 2, 2, 1);
        flow_graph.add_edge(2, 3, 2, 2);
        let expected_result = MinCostFlowResult { flow: 3, cost: 5 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, 3);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn flow_ignores_zero_capacity_edge_as_augmenting_path() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(2);
        flow_graph.add_edge(0, 1, 0, -100);
        flow_graph.add_edge(0, 1, 1, 5);
        let expected_result = MinCostFlowResult { flow: 1, cost: 5 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, 1);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn flow_with_limit_above_reachable_capacity_sends_only_reachable_amount() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 3;
        let flow_limit = 5;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(4);
        flow_graph.add_edge(0, 1, 1, 1);
        flow_graph.add_edge(1, 3, 1, 1);
        flow_graph.add_edge(0, 2, 2, 2);
        flow_graph.add_edge(2, 3, 2, 2);
        let expected_result = MinCostFlowResult { flow: 3, cost: 10 };

        // Act
        let actual_result = flow_graph.flow(source_vertex_index, sink_vertex_index, flow_limit);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn min_cost_to_send_with_zero_required_flow_returns_zero_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 1;
        let required_flow = 0_i64;
        let mut flow_graph = create_single_path_graph(1, 5);
        let expected_cost = Some(0);

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(actual_cost, expected_cost);
    }

    #[test]
    fn min_cost_to_send_with_negative_cost_edge_returns_expected_total_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let vertex_count = 3;
        let required_flow = 1;
        let mut flow_graph = MinCostFlow::new(vertex_count);
        flow_graph.add_edge(0, 1, 1, -2);
        flow_graph.add_edge(1, 2, 1, 5);
        let expected_cost = Some(3);

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(expected_cost, actual_cost);
    }

    #[test]
    fn min_cost_to_send_with_i32_capacity_and_i64_cost_returns_expected_total_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let vertex_count = 3;
        let required_flow = 2_i32;
        let mut flow_graph = MinCostFlow::<i32, i64>::new(vertex_count);
        flow_graph.add_edge(0, 1, 2_i32, 4_i64);
        flow_graph.add_edge(1, 2, 2_i32, 5_i64);
        let expected_cost = Some(18_i64);

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(expected_cost, actual_cost);
    }

    #[test]
    fn min_cost_to_send_with_unreachable_sink_returns_none() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let required_flow = 1_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(3);
        flow_graph.add_edge(0, 1, 1_i64, 2_i64);
        let expected_cost = None;

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(expected_cost, actual_cost);
    }

    #[test]
    fn min_cost_to_send_with_i64_capacity_and_i128_cost_returns_expected_total_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 2;
        let vertex_count = 3;
        let required_flow = 2_i64;
        let mut flow_graph = MinCostFlow::<i64, i128>::new(vertex_count);
        flow_graph.add_edge(0, 1, 2_i64, 4_i128);
        flow_graph.add_edge(1, 2, 2_i64, 5_i128);
        let expected_cost = Some(18_i128);

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(expected_cost, actual_cost);
    }

    #[test]
    fn min_cost_to_send_with_same_source_and_sink_returns_zero_cost() {
        // Arrange
        let source_vertex_index = 0;
        let sink_vertex_index = 0;
        let required_flow = 3_i64;
        let mut flow_graph = MinCostFlow::<i64, i64>::new(1);
        let expected_cost = Some(0);

        // Act
        let actual_cost =
            flow_graph.min_cost_to_send(source_vertex_index, sink_vertex_index, required_flow);

        // Assert
        assert_eq!(actual_cost, expected_cost);
    }
}
