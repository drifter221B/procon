pub struct PersistentDsu {
    parent: Vec<Vec<usize>>,
    component_size: Vec<Vec<usize>>,
}

impl PersistentDsu {
    pub fn new(size: usize) -> Self {
        let parent = vec![(0..size).collect::<Vec<usize>>()];
        let component_size = vec![vec![1; size]];

        Self {
            parent,
            component_size,
        }
    }

    pub fn current_version(&self) -> usize {
        self.parent.len() - 1
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.parent[0].len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn root(&self, version: usize, vertex: usize) -> Option<usize> {
        if version >= self.parent.len() || vertex >= self.len() {
            return None;
        }

        let mut root_vertex = vertex;
        while self.parent[version][root_vertex] != root_vertex {
            root_vertex = self.parent[version][root_vertex];
        }

        Some(root_vertex)
    }

    pub fn component_size(&self, version: usize, vertex: usize) -> Option<usize> {
        let root_vertex = self.root(version, vertex)?;
        Some(self.component_size[version][root_vertex])
    }

    pub fn is_same_component(
        &self,
        version: usize,
        left_vertex: usize,
        right_vertex: usize,
    ) -> Option<bool> {
        Some(self.root(version, left_vertex)? == self.root(version, right_vertex)?)
    }

    pub fn merge(
        &mut self,
        version: usize,
        left_vertex: usize,
        right_vertex: usize,
    ) -> Option<usize> {
        if version >= self.parent.len() || left_vertex >= self.len() || right_vertex >= self.len() {
            return None;
        }

        let mut parent = self.parent[version].clone();
        let mut component_size = self.component_size[version].clone();

        let mut left_root = self.root(version, left_vertex)?;
        let mut right_root = self.root(version, right_vertex)?;
        if left_root == right_root {
            return Some(version);
        }

        if component_size[left_root] < component_size[right_root] {
            std::mem::swap(&mut left_root, &mut right_root);
        }

        parent[right_root] = left_root;
        component_size[left_root] += component_size[right_root];

        self.parent.push(parent);
        self.component_size.push(component_size);

        Some(self.current_version())
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::PersistentDsu;

    #[test]
    fn new_zero_creates_empty_version_zero_structure() {
        // Arrange
        let vertex_count = 0;
        let expected_state = (0, true, 0);

        // Act
        let dsu = PersistentDsu::new(vertex_count);
        let actual_state = (dsu.len(), dsu.is_empty(), dsu.current_version());

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn initializes_with_singleton_sets() {
        // Arrange
        let vertex_count = 3;
        let version = 0;
        let vertices = [0, 1, 2];
        let expected_state = (vertex_count, false, version);
        let expected_roots = [Some(0), Some(1), Some(2)];
        let expected_sizes = [Some(1); 3];

        // Act
        let dsu = PersistentDsu::new(vertex_count);
        let actual_state = (dsu.len(), dsu.is_empty(), dsu.current_version());
        let actual_roots = vertices.map(|vertex| dsu.root(version, vertex));
        let actual_sizes = vertices.map(|vertex| dsu.component_size(version, vertex));

        // Assert
        assert_eq!(actual_state, expected_state);
        assert_eq!(actual_roots, expected_roots);
        assert_eq!(actual_sizes, expected_sizes);
    }

    #[test]
    fn merge_accepts_non_root_vertices_from_different_components() {
        // Arrange
        let vertex_count = 4;
        let initial_version = 0;
        let first_edge = (0, 1);
        let second_edge = (2, 3);
        let merge_edge = (1, 3);
        let mut dsu = PersistentDsu::new(vertex_count);
        let version_one = dsu
            .merge(initial_version, first_edge.0, first_edge.1)
            .unwrap();
        let version_two = dsu
            .merge(version_one, second_edge.0, second_edge.1)
            .unwrap();
        let expected_states = vec![(Some(0), Some(vertex_count)); vertex_count];

        // Act
        let merged_version = dsu.merge(version_two, merge_edge.0, merge_edge.1).unwrap();
        let actual_states = (0..vertex_count)
            .map(|vertex| {
                (
                    dsu.root(merged_version, vertex),
                    dsu.component_size(merged_version, vertex),
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn root_returns_none_for_out_of_range_vertex() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 0;
        let vertex = 2;
        let expected_result = None;

        // Act
        let actual_result = dsu.root(version, vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn roots_are_idempotent_in_every_created_version() {
        // Arrange
        let vertex_count = 6;
        let initial_version = 0;
        let edges = [(0, 1), (1, 2), (3, 4), (4, 5)];
        let mut dsu = PersistentDsu::new(vertex_count);
        let version_one = dsu.merge(initial_version, edges[0].0, edges[0].1).unwrap();
        let version_two = dsu.merge(version_one, edges[1].0, edges[1].1).unwrap();
        let version_three = dsu.merge(version_one, edges[2].0, edges[2].1).unwrap();
        let version_four = dsu.merge(version_two, edges[3].0, edges[3].1).unwrap();
        let versions = [
            initial_version,
            version_one,
            version_two,
            version_three,
            version_four,
        ];

        // Act
        let actual_roots_and_second_roots = versions
            .into_iter()
            .flat_map(|version| {
                let dsu = &dsu;
                (0..vertex_count).map(move |vertex| {
                    let root = dsu.root(version, vertex).unwrap();
                    (Some(root), dsu.root(version, root))
                })
            })
            .collect::<Vec<_>>();

        // Assert
        for (root, second_root) in actual_roots_and_second_roots {
            assert_eq!(second_root, root);
        }
    }

    #[test]
    fn component_size_returns_none_for_nonexistent_version() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 1;
        let vertex = 0;
        let expected_result = None;

        // Act
        let actual_result = dsu.component_size(version, vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn component_sizes_match_membership_and_partition_total_in_every_version() {
        // Arrange
        let vertex_count = 6;
        let initial_version = 0;
        let edges = [(0, 1), (1, 2), (3, 4), (4, 5)];
        let mut dsu = PersistentDsu::new(vertex_count);
        let version_one = dsu.merge(initial_version, edges[0].0, edges[0].1).unwrap();
        let version_two = dsu.merge(version_one, edges[1].0, edges[1].1).unwrap();
        let version_three = dsu.merge(version_one, edges[2].0, edges[2].1).unwrap();
        let version_four = dsu.merge(version_two, edges[3].0, edges[3].1).unwrap();
        let versions = [
            initial_version,
            version_one,
            version_two,
            version_three,
            version_four,
        ];
        let expected_partition_total = vertex_count;

        // Act
        let mut actual_versions = Vec::new();
        for version in versions {
            let size_and_membership = (0..vertex_count)
                .map(|vertex| {
                    let membership_count = (0..vertex_count)
                        .filter(|&other| {
                            dsu.is_same_component(version, vertex, other) == Some(true)
                        })
                        .count();
                    (dsu.component_size(version, vertex), Some(membership_count))
                })
                .collect::<Vec<_>>();
            let roots = (0..vertex_count)
                .map(|vertex| dsu.root(version, vertex).unwrap())
                .collect::<HashSet<_>>();
            let partition_total = roots
                .into_iter()
                .map(|root| dsu.component_size(version, root).unwrap())
                .sum::<usize>();
            actual_versions.push((size_and_membership, partition_total));
        }

        // Assert
        for (size_and_membership, partition_total) in actual_versions {
            for (size, membership) in size_and_membership {
                assert_eq!(size, membership);
            }
            assert_eq!(partition_total, expected_partition_total);
        }
    }

    #[test]
    fn is_same_component_returns_none_when_only_left_vertex_is_out_of_range() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 0;
        let left_vertex = 2;
        let right_vertex = 0;
        let expected_result = None;

        // Act
        let actual_result = dsu.is_same_component(version, left_vertex, right_vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn is_same_component_returns_none_for_nonexistent_version() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 1;
        let left_vertex = 0;
        let right_vertex = 1;
        let expected_result = None;

        // Act
        let actual_result = dsu.is_same_component(version, left_vertex, right_vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn merge_creates_new_version_and_keeps_old() {
        // Arrange
        let mut dsu = PersistentDsu::new(4);
        let initial_version = 0;
        let edges = [(0, 1), (2, 3), (0, 2)];
        let query_pair = (0, 1);
        let observed_vertex = 3;
        let expected_first_version = 1;
        let expected_first_state = (Some(false), Some(true), Some(2));
        let expected_later_sizes = (Some(4), Some(2));

        // Act
        let first = dsu.merge(initial_version, edges[0].0, edges[0].1).unwrap();
        let actual_first_state = (
            dsu.is_same_component(initial_version, query_pair.0, query_pair.1),
            dsu.is_same_component(first, query_pair.0, query_pair.1),
            dsu.component_size(first, query_pair.0),
        );
        let second = dsu.merge(first, edges[1].0, edges[1].1).unwrap();
        let third = dsu.merge(second, edges[2].0, edges[2].1).unwrap();
        let actual_later_sizes = (
            dsu.component_size(third, observed_vertex),
            dsu.component_size(second, observed_vertex),
        );

        // Assert
        assert_eq!(first, expected_first_version);
        assert_eq!(actual_first_state, expected_first_state);
        assert_eq!(actual_later_sizes, expected_later_sizes);
    }

    #[test]
    fn merge_with_nonexistent_version_returns_none_without_changing_history() {
        // Arrange
        let mut dsu = PersistentDsu::new(2);
        let version = 1;
        let left_vertex = 0;
        let right_vertex = 1;
        let expected_result = None;
        let expected_latest_version = dsu.current_version();

        // Act
        let actual_result = dsu.merge(version, left_vertex, right_vertex);
        let actual_latest_version = dsu.current_version();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_latest_version, expected_latest_version);
    }

    #[test]
    fn merge_same_set_reuses_existing_version() {
        // Arrange
        let mut dsu = PersistentDsu::new(2);
        let initial_version = 0;
        let edge = (0, 1);
        let expected_version = dsu.merge(initial_version, edge.0, edge.1).unwrap();

        // Act
        let actual_version = dsu.merge(expected_version, edge.0, edge.1).unwrap();
        let actual_latest_version = dsu.current_version();

        // Assert
        assert_eq!(actual_version, expected_version);
        assert_eq!(actual_latest_version, expected_version);
    }

    #[test]
    fn merge_attaches_smaller_left_component_to_larger_right_component() {
        // Arrange
        let vertex_count = 3;
        let mut dsu = PersistentDsu::new(vertex_count);
        let initial_version = 0;
        let large_edge = (1, 2);
        let merge_edge = (0, 2);
        let version = dsu
            .merge(initial_version, large_edge.0, large_edge.1)
            .unwrap();
        let expected_states = vec![(Some(1), Some(vertex_count)); vertex_count];

        // Act
        let merged_version = dsu.merge(version, merge_edge.0, merge_edge.1).unwrap();
        let actual_states = (0..vertex_count)
            .map(|vertex| {
                (
                    dsu.root(merged_version, vertex),
                    dsu.component_size(merged_version, vertex),
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn merge_with_only_left_vertex_out_of_range_does_not_change_history() {
        // Arrange
        let mut dsu = PersistentDsu::new(3);
        let initial_version = 0;
        let edge = (0, 1);
        let isolated_vertex = 2;
        let invalid_vertex = dsu.len();
        let version = dsu.merge(initial_version, edge.0, edge.1).unwrap();
        let expected_state = (None, version, Some(false), Some(true), Some(false));

        // Act
        let result = dsu.merge(version, invalid_vertex, isolated_vertex);
        let actual_state = (
            result,
            dsu.current_version(),
            dsu.is_same_component(initial_version, edge.0, edge.1),
            dsu.is_same_component(version, edge.0, edge.1),
            dsu.is_same_component(version, edge.0, isolated_vertex),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn merge_on_empty_dsu_returns_none_without_creating_version() {
        // Arrange
        let mut dsu = PersistentDsu::new(0);
        let version = 0;
        let vertex = 0;
        let expected_state = (None, version);

        // Act
        let result = dsu.merge(version, vertex, vertex);
        let actual_state = (result, dsu.current_version());

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn branching_from_non_initial_version_keeps_sibling_histories_independent() {
        // Arrange
        let mut dsu = PersistentDsu::new(4);
        let initial_version = 0;
        let common_edge = (0, 1);
        let branches = [(1, 2), (1, 3)];
        let observed_vertex = 0;
        let query_vertices = [1, 2, 3];
        let base_version = dsu
            .merge(initial_version, common_edge.0, common_edge.1)
            .unwrap();
        let expected_states = vec![
            ([Some(true), Some(true), Some(false)], Some(3)),
            ([Some(true), Some(false), Some(true)], Some(3)),
        ];

        // Act
        let versions = branches.map(|(left, right)| dsu.merge(base_version, left, right).unwrap());
        let actual_states = versions
            .into_iter()
            .map(|version| {
                (
                    query_vertices
                        .map(|vertex| dsu.is_same_component(version, observed_vertex, vertex)),
                    dsu.component_size(version, observed_vertex),
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn branching_from_old_versions_keeps_histories_independent() {
        // Arrange
        let mut dsu = PersistentDsu::new(4);
        let initial_version = 0;
        let edges = [(0, 1), (2, 3)];
        let expected_connectivity = [[Some(true), Some(false)], [Some(false), Some(true)]];

        // Act
        let versions = edges.map(|(left, right)| dsu.merge(initial_version, left, right).unwrap());
        let actual_connectivity = versions
            .map(|version| edges.map(|(left, right)| dsu.is_same_component(version, left, right)));

        // Assert
        assert_eq!(actual_connectivity, expected_connectivity);
    }

    #[test]
    fn merging_same_component_in_historical_version_reuses_that_version() {
        // Arrange
        let mut dsu = PersistentDsu::new(4);
        let initial_version = 0;
        let first_edge = (0, 1);
        let second_edge = (2, 3);
        let historical_version = dsu
            .merge(initial_version, first_edge.0, first_edge.1)
            .unwrap();
        let latest_version = dsu
            .merge(historical_version, second_edge.0, second_edge.1)
            .unwrap();
        let expected_state = (
            Some(historical_version),
            latest_version,
            Some(false),
            Some(true),
        );

        // Act
        let returned_version = dsu.merge(historical_version, first_edge.1, first_edge.0);
        let actual_state = (
            returned_version,
            dsu.current_version(),
            dsu.is_same_component(historical_version, second_edge.0, second_edge.1),
            dsu.is_same_component(latest_version, second_edge.0, second_edge.1),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn root_returns_none_for_nonexistent_version() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 1;
        let vertex = 0;
        let expected_result = None;

        // Act
        let actual_result = dsu.root(version, vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }


    #[test]
    fn component_size_returns_none_for_out_of_range_vertex() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 0;
        let vertex = 2;
        let expected_result = None;

        // Act
        let actual_result = dsu.component_size(version, vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }


    #[test]
    fn is_same_component_returns_none_when_only_right_vertex_is_out_of_range() {
        // Arrange
        let dsu = PersistentDsu::new(2);
        let version = 0;
        let left_vertex = 0;
        let right_vertex = 2;
        let expected_result = None;

        // Act
        let actual_result = dsu.is_same_component(version, left_vertex, right_vertex);

        // Assert
        assert_eq!(actual_result, expected_result);
    }


    #[test]
    fn merge_with_right_vertex_out_of_range_returns_none() {
        // Arrange
        let mut dsu = PersistentDsu::new(2);
        let version = 0;
        let left_vertex = 0;
        let right_vertex = 2;
        let expected_result = None;
        let expected_latest_version = dsu.current_version();

        // Act
        let actual_result = dsu.merge(version, left_vertex, right_vertex);
        let actual_latest_version = dsu.current_version();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_latest_version, expected_latest_version);
    }

}
