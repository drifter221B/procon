use std::collections::HashMap;
use std::fmt;

#[derive(Debug, Clone)]
struct HistoryEntry {
    edge_index: usize,
    root_x: usize,
    root_y: usize,
    prev_rank_root_x: usize,
    prev_size_root_x: usize,
}

#[derive(Debug, Clone)]
pub struct RollbackDsu {
    parent: Vec<usize>,
    rank: Vec<usize>,
    component_size: Vec<usize>,
    element_count: usize,
    group_count: usize,
    history: Vec<HistoryEntry>,
    snapshots: HashMap<usize, Vec<usize>>,
}

impl Default for RollbackDsu {
    fn default() -> Self {
        Self::new(0)
    }
}

impl RollbackDsu {
    pub fn new(size: usize) -> Self {
        Self {
            parent: (0..size).collect(),
            rank: vec![0; size],
            component_size: vec![1; size],
            element_count: size,
            group_count: size,
            history: Vec::new(),
            snapshots: HashMap::new(),
        }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.element_count
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn version(&self) -> usize {
        self.history.len()
    }

    pub fn rollback_latest(&mut self) -> bool {
        if self.history.is_empty() {
            return false;
        }

        let target_version = self.version() - 1;
        self.rollback_to(target_version)
    }

    pub fn rollback_to(&mut self, version: usize) -> bool {
        if version > self.version() {
            return false;
        }

        let mut rolled_back = false;
        while version < self.version() {
            if let Some(entry) = self.history.pop() {
                self.rank[entry.root_x] = entry.prev_rank_root_x;
                self.component_size[entry.root_x] = entry.prev_size_root_x;
                self.parent[entry.root_y] = entry.root_y;
                self.group_count += 1;

                if let Some(versions) = self.snapshots.get_mut(&entry.edge_index) {
                    versions.pop();
                    if versions.is_empty() {
                        self.snapshots.remove(&entry.edge_index);
                    }
                }

                rolled_back = true;
            }
        }

        rolled_back
    }

    pub fn undo(&mut self, reference_vertex: usize, vertex: usize) -> bool {
        let idx = self.index(reference_vertex, vertex);
        match self.latest_snapshot_version(idx) {
            Some(prev_version) if self.version() > prev_version => self.rollback_to(prev_version),
            _ => false,
        }
    }

    pub fn undo_all(&mut self, reference_vertex: usize, vertex: usize) -> bool {
        let idx = self.index(reference_vertex, vertex);
        match self.oldest_snapshot_version(idx) {
            Some(prev_version) if self.version() > prev_version => self.rollback_to(prev_version),
            _ => false,
        }
    }

    pub fn last_activity(&self) -> Option<(usize, usize)> {
        self.history
            .last()
            .map(|entry| self.decode_index(entry.edge_index))
    }

    pub fn root(&self, mut vertex: usize) -> usize {
        assert!(vertex < self.len());

        while self.parent[vertex] != vertex {
            vertex = self.parent[vertex];
        }

        vertex
    }

    pub fn all_roots(&self) -> Vec<usize> {
        (0..self.len())
            .filter(|&vertex| self.root(vertex) == vertex)
            .collect()
    }

    #[inline]
    pub fn is_same_component(&self, vertex1: usize, vertex2: usize) -> bool {
        self.root(vertex1) == self.root(vertex2)
    }

    pub fn merge(&mut self, reference_vertex: usize, vertex: usize) -> bool {
        assert!(reference_vertex < self.len());
        assert!(vertex < self.len());

        let mut root_x = self.root(reference_vertex);
        let mut root_y = self.root(vertex);

        if root_x == root_y {
            return false;
        }

        let edge_index = self.index(reference_vertex, vertex);

        if self.rank[root_x] < self.rank[root_y] {
            std::mem::swap(&mut root_x, &mut root_y);
        }

        self.record(root_x, root_y, edge_index);

        self.group_count -= 1;
        self.parent[root_y] = root_x;
        self.component_size[root_x] += self.component_size[root_y];

        if self.rank[root_x] == self.rank[root_y] {
            self.rank[root_x] += 1;
        }

        true
    }

    #[inline]
    pub fn component_size(&self, vertex: usize) -> usize {
        let root_vertex = self.root(vertex);
        self.component_size[root_vertex]
    }

    pub fn all_component_size(&self) -> Vec<usize> {
        (0..self.len())
            .filter(|&vertex| self.root(vertex) == vertex)
            .map(|root_vertex| self.component_size[root_vertex])
            .collect()
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_count
    }

    pub fn component_members(&self, vertex: usize) -> Vec<usize> {
        assert!(vertex < self.len());

        let root_vertex = self.root(vertex);
        (0..self.len())
            .filter(|&candidate| self.root(candidate) == root_vertex)
            .collect()
    }

    pub fn components_by_root(&self) -> HashMap<usize, Vec<usize>> {
        let mut members = HashMap::new();

        for vertex in 0..self.len() {
            let root_vertex = self.root(vertex);
            members
                .entry(root_vertex)
                .or_insert_with(Vec::new)
                .push(vertex);
        }

        members
    }

    pub fn snapshot_version(&self, idx: usize, nth: usize) -> Option<usize> {
        self.snapshots
            .get(&idx)
            .and_then(|versions| versions.get(nth).copied())
    }

    fn index(&self, x: usize, y: usize) -> usize {
        assert!(x < self.len());
        assert!(y < self.len());
        x * self.len() + y
    }

    fn decode_index(&self, idx: usize) -> (usize, usize) {
        assert!(!self.is_empty());
        (idx / self.len(), idx % self.len())
    }

    fn record(&mut self, root_x: usize, root_y: usize, idx: usize) {
        let current_version = self.version();
        self.snapshots.entry(idx).or_default().push(current_version);
        let entry = HistoryEntry {
            edge_index: idx,
            root_x,
            root_y,
            prev_rank_root_x: self.rank[root_x],
            prev_size_root_x: self.component_size[root_x],
        };

        self.history.push(entry);
    }

    fn oldest_snapshot_version(&self, idx: usize) -> Option<usize> {
        self.snapshots
            .get(&idx)
            .and_then(|versions| versions.first().copied())
    }

    fn latest_snapshot_version(&self, idx: usize) -> Option<usize> {
        self.snapshots
            .get(&idx)
            .and_then(|versions| versions.last().copied())
    }
}

impl fmt::Display for RollbackDsu {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (root_vertex, group_members) in self.components_by_root() {
            write!(f, "root vertex:{root_vertex} - [")?;
            for (index, member) in group_members.iter().enumerate() {
                if index != 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{member}")?;
            }
            writeln!(f, "]")?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {

    use std::collections::{HashMap, HashSet};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::*;

    fn edge_index(element_count: usize, reference_vertex: usize, vertex: usize) -> usize {
        reference_vertex * element_count + vertex
    }

    fn sort_group_members(group_members: HashMap<usize, Vec<usize>>) -> Vec<(usize, Vec<usize>)> {
        let mut entries = group_members
            .into_iter()
            .map(|(root_vertex, mut members)| {
                members.sort_unstable();
                (root_vertex, members)
            })
            .collect::<Vec<(usize, Vec<usize>)>>();
        entries.sort_by_key(|(root_vertex, _)| *root_vertex);
        entries
    }

    #[derive(Debug)]
    struct NaiveDsuState {
        parent: Vec<usize>,
        rank: Vec<usize>,
        component_size: Vec<usize>,
    }

    impl NaiveDsuState {
        fn new(element_count: usize) -> Self {
            Self {
                parent: (0..element_count).collect(),
                rank: vec![0; element_count],
                component_size: vec![1; element_count],
            }
        }

        fn root(&self, mut vertex: usize) -> usize {
            while self.parent[vertex] != vertex {
                vertex = self.parent[vertex];
            }
            vertex
        }

        fn merge(&mut self, reference_vertex: usize, vertex: usize) -> bool {
            let mut root_x = self.root(reference_vertex);
            let mut root_y = self.root(vertex);

            if root_x == root_y {
                return false;
            }

            if self.rank[root_x] < self.rank[root_y] {
                std::mem::swap(&mut root_x, &mut root_y);
            }

            self.parent[root_y] = root_x;
            self.component_size[root_x] += self.component_size[root_y];
            if self.rank[root_x] == self.rank[root_y] {
                self.rank[root_x] += 1;
            }

            true
        }
    }

    #[derive(Debug)]
    struct NaiveRollbackDsu {
        element_count: usize,
        history: Vec<(usize, usize)>,
    }

    impl NaiveRollbackDsu {
        fn new(element_count: usize) -> Self {
            Self {
                element_count,
                history: Vec::new(),
            }
        }

        fn state(&self) -> NaiveDsuState {
            let mut state = NaiveDsuState::new(self.element_count);
            for &(reference_vertex, vertex) in &self.history {
                assert!(state.merge(reference_vertex, vertex));
            }
            state
        }

        fn version(&self) -> usize {
            self.history.len()
        }

        fn merge(&mut self, reference_vertex: usize, vertex: usize) -> bool {
            let state = self.state();
            if state.root(reference_vertex) == state.root(vertex) {
                return false;
            }

            self.history.push((reference_vertex, vertex));
            true
        }

        fn rollback_latest(&mut self) -> bool {
            if self.history.is_empty() {
                return false;
            }

            self.history.pop();
            true
        }

        fn rollback_to(&mut self, version: usize) -> bool {
            if version >= self.version() {
                return false;
            }

            self.history.truncate(version);
            true
        }

        fn undo(&mut self, reference_vertex: usize, vertex: usize) -> bool {
            let target_version = self
                .history
                .iter()
                .rposition(|&edge| edge == (reference_vertex, vertex));
            if let Some(version) = target_version {
                self.history.truncate(version);
                true
            } else {
                false
            }
        }

        fn undo_all(&mut self, reference_vertex: usize, vertex: usize) -> bool {
            let target_version = self
                .history
                .iter()
                .position(|&edge| edge == (reference_vertex, vertex));
            if let Some(version) = target_version {
                self.history.truncate(version);
                true
            } else {
                false
            }
        }
    }

    fn next_pseudo_random(seed: &mut u64) -> u64 {
        *seed = seed
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *seed
    }

    fn pseudo_random_or_recorded_edge(model: &NaiveRollbackDsu, seed: &mut u64) -> (usize, usize) {
        let selector = next_pseudo_random(seed);
        if !model.history.is_empty() && !selector.is_multiple_of(4) {
            model.history[(selector as usize) % model.history.len()]
        } else {
            let reference_vertex = (next_pseudo_random(seed) as usize) % model.element_count;
            let vertex = (next_pseudo_random(seed) as usize) % model.element_count;
            (reference_vertex, vertex)
        }
    }

    #[derive(Debug, PartialEq)]
    struct DsuObservation {
        len: usize,
        version: usize,
        group_count: usize,
        last_activity: Option<(usize, usize)>,
        roots: Vec<usize>,
        sizes: Vec<usize>,
        components: Vec<(usize, Vec<usize>)>,
        vertices: Vec<(usize, usize, Vec<usize>)>,
        connectivity: Vec<bool>,
    }

    fn observe_dsu(actual: &RollbackDsu, element_count: usize) -> DsuObservation {
        DsuObservation {
            len: actual.len(),
            version: actual.version(),
            group_count: actual.group_count(),
            last_activity: actual.last_activity(),
            roots: actual.all_roots(),
            sizes: actual.all_component_size(),
            components: sort_group_members(actual.components_by_root()),
            vertices: (0..element_count)
                .map(|vertex| {
                    (
                        actual.root(vertex),
                        actual.component_size(vertex),
                        actual.component_members(vertex),
                    )
                })
                .collect(),
            connectivity: (0..element_count)
                .flat_map(|first| {
                    (0..element_count).map(move |second| actual.is_same_component(first, second))
                })
                .collect(),
        }
    }

    fn expected_observation(model: &NaiveRollbackDsu) -> DsuObservation {
        let state = model.state();
        let roots: Vec<_> = (0..model.element_count)
            .filter(|&vertex| state.root(vertex) == vertex)
            .collect();
        let sizes = roots
            .iter()
            .map(|&root| state.component_size[root])
            .collect();
        let mut groups = HashMap::new();
        for vertex in 0..model.element_count {
            groups
                .entry(state.root(vertex))
                .or_insert_with(Vec::new)
                .push(vertex);
        }
        let vertices = (0..model.element_count)
            .map(|vertex| {
                let root = state.root(vertex);
                (root, state.component_size[root], groups[&root].clone())
            })
            .collect();
        let mut connectivity = Vec::new();
        for first in 0..model.element_count {
            for second in 0..model.element_count {
                connectivity.push(state.root(first) == state.root(second));
            }
        }
        DsuObservation {
            len: model.element_count,
            version: model.version(),
            group_count: roots.len(),
            last_activity: model.history.last().copied(),
            roots,
            sizes,
            components: sort_group_members(groups),
            vertices,
            connectivity,
        }
    }

    #[test]
    fn default_when_initialized_then_matches_new_zero_state() {
        // Arrange
        let default_dsu = RollbackDsu::default();
        let created_dsu = RollbackDsu::new(0);

        // Act
        let default_len = default_dsu.len();
        let created_len = created_dsu.len();
        let default_is_empty = default_dsu.is_empty();
        let created_is_empty = created_dsu.is_empty();
        let default_version = default_dsu.version();
        let created_version = created_dsu.version();
        let default_group_count = default_dsu.group_count();
        let created_group_count = created_dsu.group_count();
        let default_last_activity = default_dsu.last_activity();
        let created_last_activity = created_dsu.last_activity();

        // Assert
        assert_eq!(default_len, created_len);
        assert_eq!(default_is_empty, created_is_empty);
        assert_eq!(default_version, created_version);
        assert_eq!(default_group_count, created_group_count);
        assert_eq!(default_last_activity, created_last_activity);
    }

    #[test]
    fn new_when_size_is_one_then_returns_single_vertex_state() {
        // Arrange
        let element_count = 1;
        let uf = RollbackDsu::new(element_count);
        let target_vertex = 0;
        let expected_all_roots_3 = vec![0];
        let expected_all_component_size_4 = vec![1];

        let expected_root = 0;
        let expected_component_size = 1;

        // Act
        let actual_root = uf.root(target_vertex);
        let actual_component_size = uf.component_size(target_vertex);
        let actual_all_roots = uf.all_roots();
        let actual_all_component_size = uf.all_component_size();

        // Assert
        assert_eq!(actual_root, expected_root);
        assert_eq!(actual_component_size, expected_component_size);
        assert_eq!(actual_all_roots, expected_all_roots_3);
        assert_eq!(actual_all_component_size, expected_all_component_size_4);
    }

    #[test]
    fn components_by_root_when_new_with_three_vertices_then_each_vertex_is_singleton_group() {
        // Arrange
        let element_count = 3;
        let uf = RollbackDsu::new(element_count);
        let expected_group_members = vec![(0, vec![0]), (1, vec![1]), (2, vec![2])];

        // Act
        let actual_group_members = uf.components_by_root();
        let actual_sorted_group_members = sort_group_members(actual_group_members);

        // Assert
        assert_eq!(actual_sorted_group_members, expected_group_members);
    }

    #[test]
    fn new_when_size_is_zero_then_returns_known_empty_state() {
        // Arrange
        let uf = RollbackDsu::new(0);
        let snapshot_index_0 = 0;
        let nth_0 = 0;
        let expected_is_empty_2 = true;
        let expected_last_activity_5 = None;
        let expected_is_empty_6 = true;
        let expected_is_empty_7 = true;
        let expected_is_empty_8 = true;
        let expected_snapshot_version_9 = None;

        let expected_len = 0;
        let expected_version = 0;
        let expected_group_count = 0;

        // Act
        let len = uf.len();
        let is_empty = uf.is_empty();
        let version = uf.version();
        let group_count = uf.group_count();
        let last_activity = uf.last_activity();
        let roots = uf.all_roots();
        let component_sizes = uf.all_component_size();
        let components = uf.components_by_root();
        let snapshot_version = uf.snapshot_version(snapshot_index_0, nth_0);

        let actual_is_empty_6 = roots.is_empty();
        let actual_is_empty_7 = component_sizes.is_empty();
        let actual_is_empty_8 = components.is_empty();

        // Assert
        assert_eq!(len, expected_len);
        assert_eq!(is_empty, expected_is_empty_2);
        assert_eq!(version, expected_version);
        assert_eq!(group_count, expected_group_count);
        assert_eq!(last_activity, expected_last_activity_5);
        assert_eq!(actual_is_empty_6, expected_is_empty_6);
        assert_eq!(actual_is_empty_7, expected_is_empty_7);
        assert_eq!(actual_is_empty_8, expected_is_empty_8);
        assert_eq!(snapshot_version, expected_snapshot_version_9);
    }

    #[test]
    fn new_when_size_is_four_then_returns_known_initial_metadata() {
        // Arrange
        let element_count = 4;
        let uf = RollbackDsu::new(element_count);
        let expected_is_empty_2 = false;
        let expected_last_activity_5 = None;

        let expected_version = 0;

        // Act
        let len = uf.len();
        let is_empty = uf.is_empty();
        let version = uf.version();
        let group_count = uf.group_count();
        let last_activity = uf.last_activity();

        // Assert
        assert_eq!(len, element_count);
        assert_eq!(is_empty, expected_is_empty_2);
        assert_eq!(version, expected_version);
        assert_eq!(group_count, element_count);
        assert_eq!(last_activity, expected_last_activity_5);
    }

    #[test]
    fn merge_when_equal_rank_merge_is_rolled_back_and_retried_in_reverse_then_new_first_argument_becomes_root(
    ) {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let expected_first_merge_result_1 = true;
        let expected_rollback_result_2 = true;
        let expected_reversed_merge_result_5 = true;
        let expected_last_activity_11 = Some((1, 0));

        let expected_root_zero_after_rollback = 0;
        let expected_root_one_after_rollback = 1;
        let expected_root_6 = 1;
        let expected_root_7 = 1;
        let expected_component_size_8 = 2;
        let expected_group_count_9 = 1;
        let expected_version_10 = 1;

        // Act
        let first_merge_result = uf.merge(vertex_0, vertex_1);
        let rollback_result = uf.rollback_latest();
        let root_zero_after_rollback = uf.root(vertex_0);
        let root_one_after_rollback = uf.root(vertex_1);
        let reversed_merge_result = uf.merge(vertex_1, vertex_0);

        let actual_root_6 = uf.root(vertex_0);
        let actual_root_7 = uf.root(vertex_1);
        let actual_component_size_8 = uf.component_size(vertex_0);
        let actual_group_count_9 = uf.group_count();
        let actual_version_10 = uf.version();
        let actual_last_activity_11 = uf.last_activity();

        // Assert
        assert_eq!(first_merge_result, expected_first_merge_result_1);
        assert_eq!(rollback_result, expected_rollback_result_2);
        assert_eq!(root_zero_after_rollback, expected_root_zero_after_rollback);
        assert_eq!(root_one_after_rollback, expected_root_one_after_rollback);
        assert_eq!(reversed_merge_result, expected_reversed_merge_result_5);
        assert_eq!(actual_root_6, expected_root_6);
        assert_eq!(actual_root_7, expected_root_7);
        assert_eq!(actual_component_size_8, expected_component_size_8);
        assert_eq!(actual_group_count_9, expected_group_count_9);
        assert_eq!(actual_version_10, expected_version_10);
        assert_eq!(actual_last_activity_11, expected_last_activity_11);
    }

    #[test]
    fn snapshot_and_undo_when_same_edge_is_remerged_after_branch_then_only_new_version_is_recorded()
    {
        // Arrange
        let element_count = 5;
        let mut uf = RollbackDsu::new(element_count);
        let target_edge_index = edge_index(element_count, 2, 3);
        let branch_edge_index = edge_index(element_count, 3, 4);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let nth_0 = 0;
        let target_version_1 = 1;
        let vertex_4 = 4;
        let nth_1 = 1;
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_snapshot_version_3 = Some(1);
        let expected_rollback_to_4 = true;
        let expected_snapshot_version_5 = None;
        let expected_merge_6 = true;
        let expected_remerge_result_7 = true;
        let expected_new_snapshot_version_8 = Some(2);
        let expected_stale_snapshot_version_9 = None;
        let expected_undo_result_10 = true;
        let expected_is_same_component_12 = false;
        let expected_is_same_component_13 = true;
        let expected_snapshot_version_14 = None;
        let expected_snapshot_version_15 = Some(1);
        let expected_last_activity_16 = Some((3, 4));

        let expected_version_11 = 2;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_2, vertex_3);
        let actual_snapshot_version_3 = uf.snapshot_version(target_edge_index, nth_0);
        let actual_rollback_to_4 = uf.rollback_to(target_version_1);
        let actual_snapshot_version_5 = uf.snapshot_version(target_edge_index, nth_0);
        let actual_merge_6 = uf.merge(vertex_3, vertex_4);

        let remerge_result = uf.merge(vertex_2, vertex_3);
        let new_snapshot_version = uf.snapshot_version(target_edge_index, nth_0);
        let stale_snapshot_version = uf.snapshot_version(target_edge_index, nth_1);
        let undo_result = uf.undo(vertex_2, vertex_3);

        let actual_version_11 = uf.version();
        let actual_is_same_component_12 = uf.is_same_component(vertex_2, vertex_3);
        let actual_is_same_component_13 = uf.is_same_component(vertex_3, vertex_4);
        let actual_snapshot_version_14 = uf.snapshot_version(target_edge_index, nth_0);
        let actual_snapshot_version_15 = uf.snapshot_version(branch_edge_index, nth_0);
        let actual_last_activity_16 = uf.last_activity();

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(actual_snapshot_version_3, expected_snapshot_version_3);
        assert_eq!(actual_rollback_to_4, expected_rollback_to_4);
        assert_eq!(actual_snapshot_version_5, expected_snapshot_version_5);
        assert_eq!(actual_merge_6, expected_merge_6);
        assert_eq!(remerge_result, expected_remerge_result_7);
        assert_eq!(new_snapshot_version, expected_new_snapshot_version_8);
        assert_eq!(stale_snapshot_version, expected_stale_snapshot_version_9);
        assert_eq!(undo_result, expected_undo_result_10);
        assert_eq!(actual_version_11, expected_version_11);
        assert_eq!(actual_is_same_component_12, expected_is_same_component_12);
        assert_eq!(actual_is_same_component_13, expected_is_same_component_13);
        assert_eq!(actual_snapshot_version_14, expected_snapshot_version_14);
        assert_eq!(actual_snapshot_version_15, expected_snapshot_version_15);
        assert_eq!(actual_last_activity_16, expected_last_activity_16);
    }

    #[test]
    fn root_when_vertex_is_len_then_panics() {
        // Arrange
        let uf = RollbackDsu::new(2);
        let invalid_vertex = uf.len();
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| uf.root(invalid_vertex)));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn merge_when_reference_vertex_is_len_then_panics() {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let invalid_reference_vertex = uf.len();
        let vertex = 0;
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            uf.merge(invalid_reference_vertex, vertex)
        }));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn merge_when_vertex_is_len_then_panics() {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let reference_vertex = 0;
        let invalid_vertex = uf.len();
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            uf.merge(reference_vertex, invalid_vertex)
        }));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn component_size_when_vertex_is_len_then_panics() {
        // Arrange
        let uf = RollbackDsu::new(2);
        let invalid_vertex = uf.len();
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| uf.component_size(invalid_vertex)));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn component_members_when_vertex_is_len_then_panics() {
        // Arrange
        let uf = RollbackDsu::new(2);
        let invalid_vertex = uf.len();
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| uf.component_members(invalid_vertex)));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn is_same_component_when_first_vertex_is_len_then_panics() {
        // Arrange
        let uf = RollbackDsu::new(2);
        let invalid_vertex = uf.len();
        let vertex_0 = 0;
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            uf.is_same_component(invalid_vertex, vertex_0)
        }));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn is_same_component_when_second_vertex_is_len_then_panics() {
        // Arrange
        let uf = RollbackDsu::new(2);
        let invalid_vertex = uf.len();
        let vertex_0 = 0;
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            uf.is_same_component(vertex_0, invalid_vertex)
        }));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn undo_when_reference_vertex_is_len_then_panics_without_changing_history() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let existing_edge_index = edge_index(element_count, 0, 1);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_is_err_2 = true;
        let expected_last_activity_5 = Some((0, 1));
        let expected_is_same_component_6 = true;
        let expected_snapshot_version_7 = Some(0);

        let expected_version_3 = 1;
        let expected_group_count_4 = 2;
        let invalid_reference_vertex = element_count;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.undo(invalid_reference_vertex, vertex_0);
        }));

        let actual_is_err_2 = panic_result.is_err();
        let actual_version_3 = uf.version();
        let actual_group_count_4 = uf.group_count();
        let actual_last_activity_5 = uf.last_activity();
        let actual_is_same_component_6 = uf.is_same_component(vertex_0, vertex_1);
        let actual_snapshot_version_7 = uf.snapshot_version(existing_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_is_err_2, expected_is_err_2);
        assert_eq!(actual_version_3, expected_version_3);
        assert_eq!(actual_group_count_4, expected_group_count_4);
        assert_eq!(actual_last_activity_5, expected_last_activity_5);
        assert_eq!(actual_is_same_component_6, expected_is_same_component_6);
        assert_eq!(actual_snapshot_version_7, expected_snapshot_version_7);
    }

    #[test]
    fn undo_when_vertex_is_len_and_index_collides_then_panics_without_rolling_back() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let existing_edge_index = edge_index(element_count, 1, 0);
        let vertex_1 = 1;
        let vertex_0 = 0;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_is_err_2 = true;
        let expected_last_activity_5 = Some((1, 0));
        let expected_is_same_component_6 = true;
        let expected_snapshot_version_7 = Some(0);

        let expected_version_3 = 1;
        let expected_group_count_4 = 2;
        let invalid_vertex = element_count;

        // Act
        let actual_merge_1 = uf.merge(vertex_1, vertex_0);

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.undo(vertex_0, invalid_vertex);
        }));

        let actual_is_err_2 = panic_result.is_err();
        let actual_version_3 = uf.version();
        let actual_group_count_4 = uf.group_count();
        let actual_last_activity_5 = uf.last_activity();
        let actual_is_same_component_6 = uf.is_same_component(vertex_1, vertex_0);
        let actual_snapshot_version_7 = uf.snapshot_version(existing_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_is_err_2, expected_is_err_2);
        assert_eq!(actual_version_3, expected_version_3);
        assert_eq!(actual_group_count_4, expected_group_count_4);
        assert_eq!(actual_last_activity_5, expected_last_activity_5);
        assert_eq!(actual_is_same_component_6, expected_is_same_component_6);
        assert_eq!(actual_snapshot_version_7, expected_snapshot_version_7);
    }

    #[test]
    fn undo_all_when_reference_vertex_is_len_then_panics_without_changing_history() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let existing_edge_index = edge_index(element_count, 0, 1);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_is_err_2 = true;
        let expected_last_activity_5 = Some((0, 1));
        let expected_is_same_component_6 = true;
        let expected_snapshot_version_7 = Some(0);

        let expected_version_3 = 1;
        let expected_group_count_4 = 2;
        let invalid_reference_vertex = element_count;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.undo_all(invalid_reference_vertex, vertex_0);
        }));

        let actual_is_err_2 = panic_result.is_err();
        let actual_version_3 = uf.version();
        let actual_group_count_4 = uf.group_count();
        let actual_last_activity_5 = uf.last_activity();
        let actual_is_same_component_6 = uf.is_same_component(vertex_0, vertex_1);
        let actual_snapshot_version_7 = uf.snapshot_version(existing_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_is_err_2, expected_is_err_2);
        assert_eq!(actual_version_3, expected_version_3);
        assert_eq!(actual_group_count_4, expected_group_count_4);
        assert_eq!(actual_last_activity_5, expected_last_activity_5);
        assert_eq!(actual_is_same_component_6, expected_is_same_component_6);
        assert_eq!(actual_snapshot_version_7, expected_snapshot_version_7);
    }

    #[test]
    fn undo_all_when_vertex_is_len_and_index_collides_then_panics_without_rolling_back() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let existing_edge_index = edge_index(element_count, 1, 0);
        let vertex_1 = 1;
        let vertex_0 = 0;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_is_err_2 = true;
        let expected_last_activity_5 = Some((1, 0));
        let expected_is_same_component_6 = true;
        let expected_snapshot_version_7 = Some(0);

        let expected_version_3 = 1;
        let expected_group_count_4 = 2;
        let invalid_vertex = element_count;

        // Act
        let actual_merge_1 = uf.merge(vertex_1, vertex_0);

        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.undo_all(vertex_0, invalid_vertex);
        }));

        let actual_is_err_2 = panic_result.is_err();
        let actual_version_3 = uf.version();
        let actual_group_count_4 = uf.group_count();
        let actual_last_activity_5 = uf.last_activity();
        let actual_is_same_component_6 = uf.is_same_component(vertex_1, vertex_0);
        let actual_snapshot_version_7 = uf.snapshot_version(existing_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_is_err_2, expected_is_err_2);
        assert_eq!(actual_version_3, expected_version_3);
        assert_eq!(actual_group_count_4, expected_group_count_4);
        assert_eq!(actual_last_activity_5, expected_last_activity_5);
        assert_eq!(actual_is_same_component_6, expected_is_same_component_6);
        assert_eq!(actual_snapshot_version_7, expected_snapshot_version_7);
    }

    #[test]
    fn rollback_latest_when_history_is_empty_then_returns_false_and_state_is_unchanged() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let previous_version = uf.version();
        let previous_group_count = uf.group_count();
        let expected_rollback_result_1 = false;

        // Act
        let rollback_result = uf.rollback_latest();
        let actual_version = uf.version();
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(rollback_result, expected_rollback_result_1);
        assert_eq!(actual_version, previous_version);
        assert_eq!(actual_group_count, previous_group_count);
    }

    #[test]
    fn display_when_instance_is_empty_then_returns_empty_string() {
        // Arrange
        let uf = RollbackDsu::new(0);
        let expected_rendered_1 = "";

        // Act
        let rendered = format!("{uf}");

        // Assert
        assert_eq!(rendered, expected_rendered_1);
    }

    #[test]
    fn component_members_when_instance_is_empty_then_panics() {
        // Arrange
        let uf = RollbackDsu::new(0);
        let vertex_0 = 0;
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| uf.component_members(vertex_0)));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn rollback_latest_when_repeated_until_history_is_empty_then_clears_activity_and_returns_false()
    {
        // Arrange
        let mut uf = RollbackDsu::new(4);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_merge_3 = true;
        let expected_first_result_4 = true;
        let expected_first_state_5 = (2, Some((2, 3)));
        let expected_second_result_6 = true;
        let expected_second_state_7 = (1, Some((0, 1)));
        let expected_third_result_8 = true;
        let expected_third_state_9 = (0, None);
        let expected_fourth_result_10 = false;
        let expected_fourth_state_11 = (0, None);
        let expected_all_roots_13 = vec![0, 1, 2, 3];

        let expected_group_count_12 = 4;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_2, vertex_3);
        let actual_merge_3 = uf.merge(vertex_1, vertex_2);

        let first_result = uf.rollback_latest();
        let first_state = (uf.version(), uf.last_activity());
        let second_result = uf.rollback_latest();
        let second_state = (uf.version(), uf.last_activity());
        let third_result = uf.rollback_latest();
        let third_state = (uf.version(), uf.last_activity());
        let fourth_result = uf.rollback_latest();
        let fourth_state = (uf.version(), uf.last_activity());

        let actual_group_count_12 = uf.group_count();
        let actual_all_roots_13 = uf.all_roots();

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(actual_merge_3, expected_merge_3);
        assert_eq!(first_result, expected_first_result_4);
        assert_eq!(first_state, expected_first_state_5);
        assert_eq!(second_result, expected_second_result_6);
        assert_eq!(second_state, expected_second_state_7);
        assert_eq!(third_result, expected_third_result_8);
        assert_eq!(third_state, expected_third_state_9);
        assert_eq!(fourth_result, expected_fourth_result_10);
        assert_eq!(fourth_state, expected_fourth_state_11);
        assert_eq!(actual_group_count_12, expected_group_count_12);
        assert_eq!(actual_all_roots_13, expected_all_roots_13);
    }

    #[test]
    fn merge_when_vertices_are_disconnected_then_returns_true_and_updates_count_and_version() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let reference_vertex = 0;
        let vertex = 1;
        let previous_group_count = uf.group_count();
        let previous_version = uf.version();
        let expected_merge_result_1 = true;
        let expected_group_count_2 = previous_group_count - 1;
        let expected_version_3 = previous_version + 1;

        // Act
        let merge_result = uf.merge(reference_vertex, vertex);
        let actual_group_count = uf.group_count();
        let actual_version = uf.version();

        // Assert
        assert_eq!(merge_result, expected_merge_result_1);
        assert_eq!(actual_group_count, expected_group_count_2);
        assert_eq!(actual_version, expected_version_3);
    }

    #[test]
    fn snapshot_version_when_edge_exists_then_returns_version_before_target_merge() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let reference_vertex = 0;
        let vertex = 1;
        let snapshot_index = edge_index(element_count, reference_vertex, vertex);
        let _ = uf.merge(reference_vertex, vertex);
        let nth_0 = 0;
        let expected_snapshot_version_1 = Some(0);

        // Act
        let actual_snapshot_version = uf.snapshot_version(snapshot_index, nth_0);

        // Assert
        assert_eq!(actual_snapshot_version, expected_snapshot_version_1);
    }

    #[test]
    fn snapshot_version_when_edge_does_not_exist_then_returns_none() {
        // Arrange
        let element_count = 3;
        let uf = RollbackDsu::new(element_count);
        let reference_vertex = 1;
        let vertex = 2;
        let snapshot_index = edge_index(element_count, reference_vertex, vertex);
        let nth_0 = 0;
        let expected_snapshot_version_1 = None;

        // Act
        let actual_snapshot_version = uf.snapshot_version(snapshot_index, nth_0);

        // Assert
        assert_eq!(actual_snapshot_version, expected_snapshot_version_1);
    }

    #[test]
    fn snapshot_version_when_nth_is_out_of_bounds_then_returns_none() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let reference_vertex = 0;
        let vertex = 1;
        let snapshot_index = edge_index(element_count, reference_vertex, vertex);
        let _ = uf.merge(reference_vertex, vertex);
        let out_of_bounds_nth = 1;
        let expected_snapshot_version_1 = None;

        // Act
        let actual_snapshot_version = uf.snapshot_version(snapshot_index, out_of_bounds_nth);

        // Assert
        assert_eq!(actual_snapshot_version, expected_snapshot_version_1);
    }

    #[test]
    fn rollback_to_when_target_equals_current_version_then_returns_false_and_state_is_unchanged() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let _ = uf.merge(0, 1);
        let current_version = uf.version();
        let previous_group_count = uf.group_count();
        let expected_rollback_result_1 = false;

        // Act
        let rollback_result = uf.rollback_to(current_version);
        let actual_group_count = uf.group_count();
        let actual_version = uf.version();

        // Assert
        assert_eq!(rollback_result, expected_rollback_result_1);
        assert_eq!(actual_group_count, previous_group_count);
        assert_eq!(actual_version, current_version);
    }

    #[test]
    fn rollback_to_when_target_is_future_version_then_returns_false_and_state_is_unchanged() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let _ = uf.merge(0, 1);
        let current_version = uf.version();
        let future_version = current_version + 1;
        let previous_group_count = uf.group_count();
        let expected_rollback_result_1 = false;

        // Act
        let rollback_result = uf.rollback_to(future_version);
        let actual_group_count = uf.group_count();
        let actual_version = uf.version();

        // Assert
        assert_eq!(rollback_result, expected_rollback_result_1);
        assert_eq!(actual_group_count, previous_group_count);
        assert_eq!(actual_version, current_version);
    }

    #[test]
    fn rollback_to_when_history_removed_then_related_snapshot_version_becomes_none() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let reference_vertex = 0;
        let vertex = 1;
        let snapshot_index = edge_index(element_count, reference_vertex, vertex);
        let _ = uf.merge(reference_vertex, vertex);
        let nth_0 = 0;
        let target_version_0 = 0;
        let expected_snapshot_before_rollback_1 = Some(0);
        let expected_snapshot_after_rollback_2 = None;

        // Act
        let snapshot_before_rollback = uf.snapshot_version(snapshot_index, nth_0);
        let _ = uf.rollback_to(target_version_0);
        let snapshot_after_rollback = uf.snapshot_version(snapshot_index, nth_0);

        // Assert
        assert_eq!(
            snapshot_before_rollback,
            expected_snapshot_before_rollback_1
        );
        assert_eq!(snapshot_after_rollback, expected_snapshot_after_rollback_2);
    }

    #[test]
    fn merge_when_reexecuted_after_undo_then_version_advances_again() {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let reference_vertex = 0;
        let vertex = 1;
        let expected_first_merge_result_1 = true;
        let expected_undo_result_2 = true;
        let expected_second_merge_result_3 = true;

        let expected_final_version = 1;

        // Act
        let first_merge_result = uf.merge(reference_vertex, vertex);
        let undo_result = uf.undo(reference_vertex, vertex);
        let second_merge_result = uf.merge(reference_vertex, vertex);
        let final_version = uf.version();

        // Assert
        assert_eq!(first_merge_result, expected_first_merge_result_1);
        assert_eq!(undo_result, expected_undo_result_2);
        assert_eq!(second_merge_result, expected_second_merge_result_3);
        assert_eq!(final_version, expected_final_version);
    }

    #[test]
    fn snapshot_version_when_target_merge_follows_previous_history_then_returns_nonzero_version() {
        // Arrange
        let element_count = 4;
        let mut uf = RollbackDsu::new(element_count);
        let target_edge_index = edge_index(element_count, 2, 3);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_merge_result_2 = true;
        let expected_snapshot_version_4 = Some(1);

        let expected_version_before_target = 1;
        let expected_version_5 = 2;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let version_before_target = uf.version();

        let merge_result = uf.merge(vertex_2, vertex_3);
        let snapshot_version = uf.snapshot_version(target_edge_index, nth_0);

        let actual_version_5 = uf.version();

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(merge_result, expected_merge_result_2);
        assert_eq!(version_before_target, expected_version_before_target);
        assert_eq!(snapshot_version, expected_snapshot_version_4);
        assert_eq!(actual_version_5, expected_version_5);
    }

    #[test]
    fn rollback_to_when_target_is_middle_version_then_preserves_prefix_snapshot_and_removes_suffix_snapshots(
    ) {
        // Arrange
        let element_count = 6;
        let mut uf = RollbackDsu::new(element_count);
        let prefix_edge_index = edge_index(element_count, 0, 1);
        let first_suffix_edge_index = edge_index(element_count, 2, 3);
        let second_suffix_edge_index = edge_index(element_count, 4, 5);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let vertex_4 = 4;
        let vertex_5 = 5;
        let target_version_1 = 1;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_merge_3 = true;
        let expected_rollback_result_4 = true;
        let expected_last_activity_7 = Some((0, 1));
        let expected_snapshot_version_8 = Some(0);
        let expected_snapshot_version_9 = None;
        let expected_snapshot_version_10 = None;
        let expected_is_same_component_11 = true;
        let expected_is_same_component_12 = false;
        let expected_is_same_component_13 = false;

        let expected_version_5 = 1;
        let expected_group_count_6 = 5;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_2, vertex_3);
        let actual_merge_3 = uf.merge(vertex_4, vertex_5);

        let rollback_result = uf.rollback_to(target_version_1);

        let actual_version_5 = uf.version();
        let actual_group_count_6 = uf.group_count();
        let actual_last_activity_7 = uf.last_activity();
        let actual_snapshot_version_8 = uf.snapshot_version(prefix_edge_index, nth_0);
        let actual_snapshot_version_9 = uf.snapshot_version(first_suffix_edge_index, nth_0);
        let actual_snapshot_version_10 = uf.snapshot_version(second_suffix_edge_index, nth_0);
        let actual_is_same_component_11 = uf.is_same_component(vertex_0, vertex_1);
        let actual_is_same_component_12 = uf.is_same_component(vertex_2, vertex_3);
        let actual_is_same_component_13 = uf.is_same_component(vertex_4, vertex_5);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(actual_merge_3, expected_merge_3);
        assert_eq!(rollback_result, expected_rollback_result_4);
        assert_eq!(actual_version_5, expected_version_5);
        assert_eq!(actual_group_count_6, expected_group_count_6);
        assert_eq!(actual_last_activity_7, expected_last_activity_7);
        assert_eq!(actual_snapshot_version_8, expected_snapshot_version_8);
        assert_eq!(actual_snapshot_version_9, expected_snapshot_version_9);
        assert_eq!(actual_snapshot_version_10, expected_snapshot_version_10);
        assert_eq!(actual_is_same_component_11, expected_is_same_component_11);
        assert_eq!(actual_is_same_component_12, expected_is_same_component_12);
        assert_eq!(actual_is_same_component_13, expected_is_same_component_13);
    }

    #[test]
    fn rollback_latest_when_history_exists_then_reverts_last_merge() {
        // Arrange
        let element_count = 2;
        let mut uf = RollbackDsu::new(element_count);
        let _ = uf.merge(0, 1);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let expected_rollback_result_1 = true;
        let expected_is_same_component_2 = false;

        let expected_version = 0;

        // Act
        let rollback_result = uf.rollback_latest();
        let actual_is_same_component = uf.is_same_component(vertex_0, vertex_1);
        let actual_group_count = uf.group_count();
        let actual_version = uf.version();

        // Assert
        assert_eq!(rollback_result, expected_rollback_result_1);
        assert_eq!(actual_is_same_component, expected_is_same_component_2);
        assert_eq!(actual_group_count, element_count);
        assert_eq!(actual_version, expected_version);
    }

    #[test]
    fn rollback_latest_when_lower_rank_first_argument_causes_root_swap_then_restores_pre_merge_state(
    ) {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let prefix_edge_index = edge_index(element_count, 0, 1);
        let target_edge_index = edge_index(element_count, 2, 0);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_rollback_result_3 = true;
        let expected_is_same_component_7 = true;
        let expected_is_same_component_8 = false;
        let expected_all_roots_12 = vec![0, 2];
        let expected_all_component_size_13 = vec![2, 1];
        let expected_last_activity_16 = Some((0, 1));
        let expected_snapshot_version_17 = Some(0);
        let expected_snapshot_version_18 = None;

        let expected_root_4 = 0;
        let expected_root_5 = 0;
        let expected_root_6 = 2;
        let expected_component_size_9 = 2;
        let expected_component_size_10 = 2;
        let expected_component_size_11 = 1;
        let expected_group_count_14 = 2;
        let expected_version_15 = 1;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_2, vertex_0);

        let rollback_result = uf.rollback_latest();

        let actual_root_4 = uf.root(vertex_0);
        let actual_root_5 = uf.root(vertex_1);
        let actual_root_6 = uf.root(vertex_2);
        let actual_is_same_component_7 = uf.is_same_component(vertex_0, vertex_1);
        let actual_is_same_component_8 = uf.is_same_component(vertex_0, vertex_2);
        let actual_component_size_9 = uf.component_size(vertex_0);
        let actual_component_size_10 = uf.component_size(vertex_1);
        let actual_component_size_11 = uf.component_size(vertex_2);
        let actual_all_roots_12 = uf.all_roots();
        let actual_all_component_size_13 = uf.all_component_size();
        let actual_group_count_14 = uf.group_count();
        let actual_version_15 = uf.version();
        let actual_last_activity_16 = uf.last_activity();
        let actual_snapshot_version_17 = uf.snapshot_version(prefix_edge_index, nth_0);
        let actual_snapshot_version_18 = uf.snapshot_version(target_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(rollback_result, expected_rollback_result_3);
        assert_eq!(actual_root_4, expected_root_4);
        assert_eq!(actual_root_5, expected_root_5);
        assert_eq!(actual_root_6, expected_root_6);
        assert_eq!(actual_is_same_component_7, expected_is_same_component_7);
        assert_eq!(actual_is_same_component_8, expected_is_same_component_8);
        assert_eq!(actual_component_size_9, expected_component_size_9);
        assert_eq!(actual_component_size_10, expected_component_size_10);
        assert_eq!(actual_component_size_11, expected_component_size_11);
        assert_eq!(actual_all_roots_12, expected_all_roots_12);
        assert_eq!(actual_all_component_size_13, expected_all_component_size_13);
        assert_eq!(actual_group_count_14, expected_group_count_14);
        assert_eq!(actual_version_15, expected_version_15);
        assert_eq!(actual_last_activity_16, expected_last_activity_16);
        assert_eq!(actual_snapshot_version_17, expected_snapshot_version_17);
        assert_eq!(actual_snapshot_version_18, expected_snapshot_version_18);
    }

    #[test]
    fn rollback_to_when_three_merges_and_target_is_one_then_rolls_back_two_latest_histories() {
        // Arrange
        let mut uf = RollbackDsu::new(4);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(2, 3);
        let _ = uf.merge(1, 2);
        let rollback_target_version = 1;
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let expected_rollback_result_1 = true;
        let expected_is_same_component_0_1_4 = true;
        let expected_is_same_component_2_3_5 = false;
        let expected_is_same_component_0_2_6 = false;

        let expected_group_count = 3;

        // Act
        let rollback_result = uf.rollback_to(rollback_target_version);
        let actual_version = uf.version();
        let actual_group_count = uf.group_count();
        let is_same_component_0_1 = uf.is_same_component(vertex_0, vertex_1);
        let is_same_component_2_3 = uf.is_same_component(vertex_2, vertex_3);
        let is_same_component_0_2 = uf.is_same_component(vertex_0, vertex_2);

        // Assert
        assert_eq!(rollback_result, expected_rollback_result_1);
        assert_eq!(actual_version, rollback_target_version);
        assert_eq!(actual_group_count, expected_group_count);
        assert_eq!(is_same_component_0_1, expected_is_same_component_0_1_4);
        assert_eq!(is_same_component_2_3, expected_is_same_component_2_3_5);
        assert_eq!(is_same_component_0_2, expected_is_same_component_0_2_6);
    }

    #[test]
    fn rollback_to_when_target_is_zero_after_three_merges_then_all_vertices_become_singletons() {
        // Arrange
        let mut uf = RollbackDsu::new(4);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(2, 3);
        let _ = uf.merge(1, 2);
        let rollback_target_version = 0;
        let expected_rollback_result_1 = true;
        let expected_all_roots_4 = vec![0, 1, 2, 3];

        let expected_group_count = 4;

        // Act
        let rollback_result = uf.rollback_to(rollback_target_version);
        let actual_version = uf.version();
        let actual_group_count = uf.group_count();
        let actual_all_roots = uf.all_roots();

        // Assert
        assert_eq!(rollback_result, expected_rollback_result_1);
        assert_eq!(actual_version, rollback_target_version);
        assert_eq!(actual_group_count, expected_group_count);
        assert_eq!(actual_all_roots, expected_all_roots_4);
    }

    #[test]
    fn snapshot_and_undo_when_merge_direction_changes_then_treated_as_separate_indices() {
        // Arrange
        let element_count = 2;
        let mut uf = RollbackDsu::new(element_count);
        let index_01 = edge_index(element_count, 0, 1);
        let index_10 = edge_index(element_count, 1, 0);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let nth_0 = 0;
        let target_version_0 = 0;
        let expected_first_merge_result_1 = true;
        let expected_snapshot_01_before_2 = Some(0);
        let expected_snapshot_10_before_3 = None;
        let expected_rollback_result_4 = true;
        let expected_second_merge_result_5 = true;
        let expected_snapshot_01_after_6 = None;
        let expected_snapshot_10_after_7 = Some(0);
        let expected_undo_original_direction_result_8 = false;
        let expected_undo_reversed_direction_result_9 = true;

        let expected_final_version = 0;

        // Act
        let first_merge_result = uf.merge(vertex_0, vertex_1);
        let snapshot_01_before = uf.snapshot_version(index_01, nth_0);
        let snapshot_10_before = uf.snapshot_version(index_10, nth_0);
        let rollback_result = uf.rollback_to(target_version_0);
        let second_merge_result = uf.merge(vertex_1, vertex_0);
        let snapshot_01_after = uf.snapshot_version(index_01, nth_0);
        let snapshot_10_after = uf.snapshot_version(index_10, nth_0);
        let undo_original_direction_result = uf.undo(vertex_0, vertex_1);
        let undo_reversed_direction_result = uf.undo(vertex_1, vertex_0);
        let final_version = uf.version();

        // Assert
        assert_eq!(first_merge_result, expected_first_merge_result_1);
        assert_eq!(snapshot_01_before, expected_snapshot_01_before_2);
        assert_eq!(snapshot_10_before, expected_snapshot_10_before_3);
        assert_eq!(rollback_result, expected_rollback_result_4);
        assert_eq!(second_merge_result, expected_second_merge_result_5);
        assert_eq!(snapshot_01_after, expected_snapshot_01_after_6);
        assert_eq!(snapshot_10_after, expected_snapshot_10_after_7);
        assert_eq!(
            undo_original_direction_result,
            expected_undo_original_direction_result_8
        );
        assert_eq!(
            undo_reversed_direction_result,
            expected_undo_reversed_direction_result_9
        );
        assert_eq!(final_version, expected_final_version);
    }

    #[test]
    fn undo_when_edge_does_not_exist_in_history_then_returns_false() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let _ = uf.merge(0, 1);
        let reference_vertex = 1;
        let vertex = 2;
        let previous_version = uf.version();
        let expected_undo_result_1 = false;

        // Act
        let undo_result = uf.undo(reference_vertex, vertex);
        let actual_version = uf.version();

        // Assert
        assert_eq!(undo_result, expected_undo_result_1);
        assert_eq!(actual_version, previous_version);
    }

    #[test]
    fn undo_all_when_edge_does_not_exist_in_history_then_returns_false() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let _ = uf.merge(0, 1);
        let reference_vertex = 1;
        let vertex = 2;
        let previous_version = uf.version();
        let expected_undo_all_result_1 = false;

        // Act
        let undo_all_result = uf.undo_all(reference_vertex, vertex);
        let actual_version = uf.version();

        // Assert
        assert_eq!(undo_all_result, expected_undo_all_result_1);
        assert_eq!(actual_version, previous_version);
    }

    #[test]
    fn undo_when_edge_is_in_middle_history_then_rolls_back_suffix_from_that_edge() {
        // Arrange
        let mut uf = RollbackDsu::new(4);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(2, 3);
        let _ = uf.merge(1, 2);
        let reference_vertex = 2;
        let vertex = 3;
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let expected_undo_result_1 = true;
        let expected_is_same_component_0_1_4 = true;
        let expected_is_same_component_2_3_5 = false;
        let expected_is_same_component_0_2_6 = false;

        let expected_version = 1;
        let expected_group_count = 3;

        // Act
        let undo_result = uf.undo(reference_vertex, vertex);
        let actual_version = uf.version();
        let actual_group_count = uf.group_count();
        let is_same_component_0_1 = uf.is_same_component(vertex_0, vertex_1);
        let is_same_component_2_3 = uf.is_same_component(vertex_2, vertex_3);
        let is_same_component_0_2 = uf.is_same_component(vertex_0, vertex_2);

        // Assert
        assert_eq!(undo_result, expected_undo_result_1);
        assert_eq!(actual_version, expected_version);
        assert_eq!(actual_group_count, expected_group_count);
        assert_eq!(is_same_component_0_1, expected_is_same_component_0_1_4);
        assert_eq!(is_same_component_2_3, expected_is_same_component_2_3_5);
        assert_eq!(is_same_component_0_2, expected_is_same_component_0_2_6);
    }

    #[test]
    fn undo_all_when_edge_is_in_middle_history_then_rolls_back_suffix_from_that_edge() {
        // Arrange
        let mut uf = RollbackDsu::new(4);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(2, 3);
        let _ = uf.merge(1, 2);
        let reference_vertex = 2;
        let vertex = 3;
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let expected_undo_all_result_1 = true;
        let expected_is_same_component_0_1_4 = true;
        let expected_is_same_component_2_3_5 = false;
        let expected_is_same_component_0_2_6 = false;

        let expected_version = 1;
        let expected_group_count = 3;

        // Act
        let undo_all_result = uf.undo_all(reference_vertex, vertex);
        let actual_version = uf.version();
        let actual_group_count = uf.group_count();
        let is_same_component_0_1 = uf.is_same_component(vertex_0, vertex_1);
        let is_same_component_2_3 = uf.is_same_component(vertex_2, vertex_3);
        let is_same_component_0_2 = uf.is_same_component(vertex_0, vertex_2);

        // Assert
        assert_eq!(undo_all_result, expected_undo_all_result_1);
        assert_eq!(actual_version, expected_version);
        assert_eq!(actual_group_count, expected_group_count);
        assert_eq!(is_same_component_0_1, expected_is_same_component_0_1_4);
        assert_eq!(is_same_component_2_3, expected_is_same_component_2_3_5);
        assert_eq!(is_same_component_0_2, expected_is_same_component_0_2_6);
    }

    #[test]
    fn undo_when_called_twice_for_same_edge_then_second_call_returns_false() {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let _ = uf.merge(0, 1);
        let reference_vertex = 0;
        let vertex = 1;
        let expected_first_undo_result_1 = true;
        let expected_second_undo_result_2 = false;

        let expected_final_version = 0;

        // Act
        let first_undo_result = uf.undo(reference_vertex, vertex);
        let second_undo_result = uf.undo(reference_vertex, vertex);
        let final_version = uf.version();

        // Assert
        assert_eq!(first_undo_result, expected_first_undo_result_1);
        assert_eq!(second_undo_result, expected_second_undo_result_2);
        assert_eq!(final_version, expected_final_version);
    }

    #[test]
    fn undo_when_direction_is_reversed_then_returns_false_and_state_is_unchanged() {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let _ = uf.merge(0, 1);
        let previous_version = uf.version();
        let vertex_1 = 1;
        let vertex_0 = 0;
        let expected_undo_result_1 = false;
        let expected_is_same_component_0_1_3 = true;

        // Act
        let undo_result = uf.undo(vertex_1, vertex_0);
        let actual_version = uf.version();
        let is_same_component_0_1 = uf.is_same_component(vertex_0, vertex_1);

        // Assert
        assert_eq!(undo_result, expected_undo_result_1);
        assert_eq!(actual_version, previous_version);
        assert_eq!(is_same_component_0_1, expected_is_same_component_0_1_3);
    }

    #[test]
    fn undo_when_instance_size_is_zero_then_panics() {
        // Arrange
        let mut uf = RollbackDsu::new(0);
        let reference_vertex = 0;
        let vertex = 0;
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| uf.undo(reference_vertex, vertex)));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn undo_all_when_instance_size_is_zero_then_panics() {
        // Arrange
        let mut uf = RollbackDsu::new(0);
        let reference_vertex = 0;
        let vertex = 0;
        let expected_is_err_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| uf.undo_all(reference_vertex, vertex)));

        let actual_is_err_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_is_err_1, expected_is_err_1);
    }

    #[test]
    fn undo_when_successful_merge_used_non_root_vertices_then_reverts_that_merge() {
        // Arrange
        let element_count = 4;
        let mut uf = RollbackDsu::new(element_count);
        let target_edge_index = edge_index(element_count, 1, 3);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let nth_0 = 0;
        let queried_edge_index_5 = edge_index(element_count, 0, 1);
        let queried_edge_index_6 = edge_index(element_count, 2, 3);
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_merge_3 = true;
        let expected_undo_result_4 = true;
        let expected_last_activity_7 = Some((2, 3));
        let expected_component_members_8 = vec![0, 1];
        let expected_component_members_9 = vec![2, 3];
        let expected_is_same_component_10 = false;
        let expected_snapshot_version_11 = None;
        let expected_snapshot_version_12 = Some(0);
        let expected_snapshot_version_13 = Some(1);

        let expected_version_5 = 2;
        let expected_group_count_6 = 2;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_2, vertex_3);
        let actual_merge_3 = uf.merge(vertex_1, vertex_3);

        let undo_result = uf.undo(vertex_1, vertex_3);

        let actual_version_5 = uf.version();
        let actual_group_count_6 = uf.group_count();
        let actual_last_activity_7 = uf.last_activity();
        let actual_component_members_8 = uf.component_members(vertex_0);
        let actual_component_members_9 = uf.component_members(vertex_2);
        let actual_is_same_component_10 = uf.is_same_component(vertex_1, vertex_3);
        let actual_snapshot_version_11 = uf.snapshot_version(target_edge_index, nth_0);
        let actual_snapshot_version_12 = uf.snapshot_version(queried_edge_index_5, nth_0);
        let actual_snapshot_version_13 = uf.snapshot_version(queried_edge_index_6, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(actual_merge_3, expected_merge_3);
        assert_eq!(undo_result, expected_undo_result_4);
        assert_eq!(actual_version_5, expected_version_5);
        assert_eq!(actual_group_count_6, expected_group_count_6);
        assert_eq!(actual_last_activity_7, expected_last_activity_7);
        assert_eq!(actual_component_members_8, expected_component_members_8);
        assert_eq!(actual_component_members_9, expected_component_members_9);
        assert_eq!(actual_is_same_component_10, expected_is_same_component_10);
        assert_eq!(actual_snapshot_version_11, expected_snapshot_version_11);
        assert_eq!(actual_snapshot_version_12, expected_snapshot_version_12);
        assert_eq!(actual_snapshot_version_13, expected_snapshot_version_13);
    }

    #[test]
    fn last_activity_when_initial_and_after_merge_then_changes_from_none_to_latest_pair() {
        // Arrange
        let mut uf = RollbackDsu::new(2);
        let reference_vertex = 0;
        let vertex = 1;
        let expected_initial_last_activity_1 = None;
        let expected_final_last_activity_2 = Some((reference_vertex, vertex));

        // Act
        let initial_last_activity = uf.last_activity();
        let _ = uf.merge(reference_vertex, vertex);
        let final_last_activity = uf.last_activity();

        // Assert
        assert_eq!(initial_last_activity, expected_initial_last_activity_1);
        assert_eq!(final_last_activity, expected_final_last_activity_2);
    }

    #[test]
    fn last_activity_when_multiple_merges_then_returns_the_latest_merge_pair() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let _ = uf.merge(0, 1);
        let latest_reference_vertex = 1;
        let latest_vertex = 2;
        let _ = uf.merge(latest_reference_vertex, latest_vertex);
        let expected_last_activity_1 = Some((latest_reference_vertex, latest_vertex));

        // Act
        let actual_last_activity = uf.last_activity();

        // Assert
        assert_eq!(actual_last_activity, expected_last_activity_1);
    }

    #[test]
    fn merge_when_rank_is_different_then_lower_rank_root_is_attached_to_higher_rank_root() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let _ = uf.merge(1, 0);
        let _ = uf.merge(2, 0);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;

        let expected_root_of_vertex_zero = 1;
        let expected_root_of_vertex_one = 1;
        let expected_root_of_vertex_two = 1;

        // Act
        let root_of_vertex_zero = uf.root(vertex_0);
        let root_of_vertex_one = uf.root(vertex_1);
        let root_of_vertex_two = uf.root(vertex_2);

        // Assert
        assert_eq!(root_of_vertex_zero, expected_root_of_vertex_zero);
        assert_eq!(root_of_vertex_one, expected_root_of_vertex_one);
        assert_eq!(root_of_vertex_two, expected_root_of_vertex_two);
    }

    #[test]
    fn all_roots_and_all_component_size_when_multiple_merges_then_root_and_size_correspondence_is_consistent(
    ) {
        // Arrange
        let element_count = 4;
        let mut uf = RollbackDsu::new(element_count);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(1, 2);
        let expected_component_size_pairs_1 = vec![(0, 3), (3, 1)];

        // Act
        let actual_roots = uf.all_roots();
        let actual_component_sizes = uf.all_component_size();
        let actual_component_size_pairs = actual_roots
            .iter()
            .copied()
            .zip(actual_component_sizes.iter().copied())
            .collect::<Vec<(usize, usize)>>();
        let actual_total_component_size = actual_component_sizes.iter().sum::<usize>();

        // Assert
        assert_eq!(actual_component_size_pairs, expected_component_size_pairs_1);
        assert_eq!(actual_total_component_size, element_count);
    }

    #[test]
    fn components_by_root_when_multiple_merges_then_union_of_groups_matches_all_vertices() {
        // Arrange
        let element_count = 5;
        let mut uf = RollbackDsu::new(element_count);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(2, 3);
        let expected_union = (0..element_count).collect::<HashSet<usize>>();

        // Act
        let actual_group_members = uf.components_by_root();
        let actual_union = actual_group_members
            .values()
            .flat_map(|members| members.iter().copied())
            .collect::<HashSet<usize>>();

        // Assert
        assert_eq!(actual_union, expected_union);
    }

    #[test]
    fn is_same_component_when_checked_before_and_after_merge_then_reflects_connectivity() {
        // Arrange
        let mut uf = RollbackDsu::new(3);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let expected_same_vertex_before_merge_1 = true;
        let expected_distinct_vertices_before_merge_2 = false;
        let expected_merge_result_3 = true;
        let expected_distinct_vertices_after_merge_4 = true;

        // Act
        let same_vertex_before_merge = uf.is_same_component(vertex_0, vertex_0);
        let distinct_vertices_before_merge = uf.is_same_component(vertex_0, vertex_1);
        let merge_result = uf.merge(vertex_0, vertex_1);
        let distinct_vertices_after_merge = uf.is_same_component(vertex_0, vertex_1);

        // Assert
        assert_eq!(
            same_vertex_before_merge,
            expected_same_vertex_before_merge_1
        );
        assert_eq!(
            distinct_vertices_before_merge,
            expected_distinct_vertices_before_merge_2
        );
        assert_eq!(merge_result, expected_merge_result_3);
        assert_eq!(
            distinct_vertices_after_merge,
            expected_distinct_vertices_after_merge_4
        );
    }

    #[test]
    fn merge_when_vertices_are_already_connected_then_returns_false_and_state_is_unchanged() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let reference_vertex = 0;
        let vertex = 1;
        let _ = uf.merge(reference_vertex, vertex);
        let previous_group_count = uf.group_count();
        let previous_version = uf.version();
        let expected_merge_result_1 = false;

        // Act
        let merge_result = uf.merge(reference_vertex, vertex);
        let actual_group_count = uf.group_count();
        let actual_version = uf.version();

        // Assert
        assert_eq!(merge_result, expected_merge_result_1);
        assert_eq!(actual_group_count, previous_group_count);
        assert_eq!(actual_version, previous_version);
    }

    #[test]
    fn merge_when_both_arguments_are_same_vertex_then_state_is_unchanged() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let vertex = 1;
        let self_edge_index = edge_index(element_count, vertex, vertex);
        let nth_0 = 0;
        let expected_merge_result_1 = false;
        let expected_last_activity_4 = None;
        let expected_snapshot_version_5 = None;

        let expected_version_3 = 0;

        // Act
        let merge_result = uf.merge(vertex, vertex);

        let actual_group_count_2 = uf.group_count();
        let actual_version_3 = uf.version();
        let actual_last_activity_4 = uf.last_activity();
        let actual_snapshot_version_5 = uf.snapshot_version(self_edge_index, nth_0);

        // Assert
        assert_eq!(merge_result, expected_merge_result_1);
        assert_eq!(actual_group_count_2, element_count);
        assert_eq!(actual_version_3, expected_version_3);
        assert_eq!(actual_last_activity_4, expected_last_activity_4);
        assert_eq!(actual_snapshot_version_5, expected_snapshot_version_5);
    }

    #[test]
    fn merge_when_vertices_are_transitively_connected_then_does_not_record_history() {
        // Arrange
        let element_count = 3;
        let mut uf = RollbackDsu::new(element_count);
        let redundant_edge_index = edge_index(element_count, 0, 2);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_merge_result_3 = false;
        let expected_last_activity_6 = Some((1, 2));
        let expected_snapshot_version_7 = None;

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_1, vertex_2);
        let previous_version = uf.version();
        let previous_group_count = uf.group_count();

        let merge_result = uf.merge(vertex_0, vertex_2);

        let actual_version_4 = uf.version();
        let actual_group_count_5 = uf.group_count();
        let actual_last_activity_6 = uf.last_activity();
        let actual_snapshot_version_7 = uf.snapshot_version(redundant_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(merge_result, expected_merge_result_3);
        assert_eq!(actual_version_4, previous_version);
        assert_eq!(actual_group_count_5, previous_group_count);
        assert_eq!(actual_last_activity_6, expected_last_activity_6);
        assert_eq!(actual_snapshot_version_7, expected_snapshot_version_7);
    }

    #[test]
    fn component_members_when_multiple_merges_then_returns_only_vertices_in_same_component() {
        // Arrange
        let mut uf = RollbackDsu::new(5);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(1, 2);
        let target_vertex = 1;
        let expected_members = vec![0, 1, 2];

        // Act
        let actual_members = uf.component_members(target_vertex);

        // Assert
        assert_eq!(actual_members, expected_members);
    }

    #[test]
    fn component_members_when_vertex_is_isolated_among_multiple_components_then_returns_only_itself(
    ) {
        // Arrange
        let mut uf = RollbackDsu::new(5);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let vertex_4 = 4;
        let expected_merge_1 = true;
        let expected_merge_2 = true;
        let expected_members_3 = vec![4];

        // Act
        let actual_merge_1 = uf.merge(vertex_0, vertex_1);
        let actual_merge_2 = uf.merge(vertex_2, vertex_3);

        let members = uf.component_members(vertex_4);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(actual_merge_2, expected_merge_2);
        assert_eq!(members, expected_members_3);
    }

    #[test]
    fn display_when_multiple_groups_exist_then_contains_one_line_per_group_without_order_dependency(
    ) {
        // Arrange
        let mut uf = RollbackDsu::new(4);
        let _ = uf.merge(0, 1);
        let _ = uf.merge(2, 3);
        let expected_lines = ["root vertex:0 - [0, 1]", "root vertex:2 - [2, 3]"]
            .iter()
            .map(|line| (*line).to_string())
            .collect::<HashSet<String>>();

        // Act
        let rendered = format!("{uf}");
        let actual_lines = rendered
            .lines()
            .map(|line| line.to_string())
            .collect::<HashSet<String>>();

        // Assert
        assert_eq!(actual_lines, expected_lines);
    }

    #[test]
    fn clone_when_clone_is_mutated_and_rolled_back_then_original_is_unchanged() {
        // Arrange
        let element_count = 4;
        let mut original = RollbackDsu::new(element_count);
        let original_edge_index = edge_index(element_count, 0, 1);
        let vertex_0 = 0;
        let vertex_1 = 1;
        let vertex_2 = 2;
        let vertex_3 = 3;
        let target_version_0 = 0;
        let nth_0 = 0;
        let expected_merge_1 = true;
        let expected_clone_merge_result_2 = true;
        let expected_clone_rollback_result_3 = true;
        let expected_last_activity_6 = Some((0, 1));
        let expected_snapshot_version_7 = Some(0);
        let expected_is_same_component_8 = true;
        let expected_is_same_component_9 = false;
        let expected_last_activity_12 = None;
        let expected_snapshot_version_13 = None;

        let expected_version_4 = 1;
        let expected_group_count_5 = 3;
        let expected_version_10 = 0;

        // Act
        let actual_merge_1 = original.merge(vertex_0, vertex_1);
        let mut cloned = original.clone();

        let clone_merge_result = cloned.merge(vertex_2, vertex_3);
        let clone_rollback_result = cloned.rollback_to(target_version_0);

        let actual_version_4 = original.version();
        let actual_group_count_5 = original.group_count();
        let actual_last_activity_6 = original.last_activity();
        let actual_snapshot_version_7 = original.snapshot_version(original_edge_index, nth_0);
        let actual_is_same_component_8 = original.is_same_component(vertex_0, vertex_1);
        let actual_is_same_component_9 = original.is_same_component(vertex_2, vertex_3);
        let actual_version_10 = cloned.version();
        let actual_group_count_11 = cloned.group_count();
        let actual_last_activity_12 = cloned.last_activity();
        let actual_snapshot_version_13 = cloned.snapshot_version(original_edge_index, nth_0);

        // Assert
        assert_eq!(actual_merge_1, expected_merge_1);
        assert_eq!(clone_merge_result, expected_clone_merge_result_2);
        assert_eq!(clone_rollback_result, expected_clone_rollback_result_3);
        assert_eq!(actual_version_4, expected_version_4);
        assert_eq!(actual_group_count_5, expected_group_count_5);
        assert_eq!(actual_last_activity_6, expected_last_activity_6);
        assert_eq!(actual_snapshot_version_7, expected_snapshot_version_7);
        assert_eq!(actual_is_same_component_8, expected_is_same_component_8);
        assert_eq!(actual_is_same_component_9, expected_is_same_component_9);
        assert_eq!(actual_version_10, expected_version_10);
        assert_eq!(actual_group_count_11, element_count);
        assert_eq!(actual_last_activity_12, expected_last_activity_12);
        assert_eq!(actual_snapshot_version_13, expected_snapshot_version_13);
    }

    #[test]
    fn display_when_instance_has_single_vertex_then_matches_exact_format() {
        // Arrange
        let uf = RollbackDsu::new(1);
        let expected_rendered_1 = "root vertex:0 - [0]\n";

        // Act
        let rendered = format!("{uf}");

        // Assert
        assert_eq!(rendered, expected_rendered_1);
    }

    #[test]
    fn mixed_operations_with_fixed_seed_then_matches_naive_replay_model_after_every_step() {
        // Arrange
        #[derive(Debug, Clone, Copy)]
        enum Operation {
            Merge(usize, usize),
            RollbackLatest,
            RollbackTo(usize),
            Undo(usize, usize),
            UndoAll(usize, usize),
        }
        let element_count = 8;
        let operation_count = 512;
        let mut model = NaiveRollbackDsu::new(element_count);
        let mut seed = 0xD15E_A5E5_CAFE_BABEu64;
        let mut operations = Vec::new();
        let mut expected_steps = Vec::new();
        let mut operations_seen = [false; 5];
        let expected_coverage = [true; 5];
        for _ in 0..operation_count {
            let (kind, operation, result) = match next_pseudo_random(&mut seed) % 10 {
                0..=4 => {
                    let first = next_pseudo_random(&mut seed) as usize % element_count;
                    let second = next_pseudo_random(&mut seed) as usize % element_count;
                    (
                        0,
                        Operation::Merge(first, second),
                        model.merge(first, second),
                    )
                }
                5 => (1, Operation::RollbackLatest, model.rollback_latest()),
                6 => {
                    let version = next_pseudo_random(&mut seed) as usize % (model.version() + 2);
                    (
                        2,
                        Operation::RollbackTo(version),
                        model.rollback_to(version),
                    )
                }
                7..=8 => {
                    let (first, second) = pseudo_random_or_recorded_edge(&model, &mut seed);
                    (3, Operation::Undo(first, second), model.undo(first, second))
                }
                _ => {
                    let (first, second) = pseudo_random_or_recorded_edge(&model, &mut seed);
                    (
                        4,
                        Operation::UndoAll(first, second),
                        model.undo_all(first, second),
                    )
                }
            };
            operations_seen[kind] = true;
            operations.push((kind, operation));
            expected_steps.push((result, expected_observation(&model)));
        }

        // Act
        let mut actual = RollbackDsu::new(element_count);
        let mut successful_operations = [false; 5];
        let actual_steps: Vec<_> = operations
            .iter()
            .map(|&(kind, operation)| {
                let result = match operation {
                    Operation::Merge(first, second) => actual.merge(first, second),
                    Operation::RollbackLatest => actual.rollback_latest(),
                    Operation::RollbackTo(version) => actual.rollback_to(version),
                    Operation::Undo(first, second) => actual.undo(first, second),
                    Operation::UndoAll(first, second) => actual.undo_all(first, second),
                };
                successful_operations[kind] |= result;
                (result, observe_dsu(&actual, element_count))
            })
            .collect();

        // Assert
        assert_eq!(actual_steps, expected_steps);
        assert_eq!(operations_seen, expected_coverage);
        assert_eq!(successful_operations, expected_coverage);
    }
}
