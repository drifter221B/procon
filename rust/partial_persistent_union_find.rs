use std::collections::HashMap;

#[derive(Debug, Clone)]
pub struct PartialPersistentUnionFind {
    parent: Vec<usize>,
    parent_updated_at: Vec<usize>,
    size_history: Vec<Vec<(usize, usize)>>,
    group_history: Vec<(usize, usize)>,
    node_count: usize,
    current_time: usize,
}

impl Default for PartialPersistentUnionFind {
    fn default() -> Self {
        Self::new(0)
    }
}

impl PartialPersistentUnionFind {

    const NO_UPDATE_TIME: usize = usize::MAX / 4;

    pub fn new(size: usize) -> Self {

        let parent = (0..size).collect::<Vec<usize>>();
        let parent_updated_at = vec![Self::NO_UPDATE_TIME; size];
        let size_history = vec![vec![(0, 1)]; size];
        let group_history = vec![(0, size)];

        Self {
            parent,
            parent_updated_at,
            size_history,
            group_history,
            node_count: size,
            current_time: 0,
        }
    }

    #[inline]
    pub fn time(&self) -> usize {
        self.current_time
    }

    #[inline]
    pub fn root(&self, vertex: usize) -> usize {
        self.root_at_time(vertex, self.current_time)
    }

    pub fn root_at_time(&self, vertex: usize, query_time: usize) -> usize {

        assert!(vertex + 1 <= self.node_count);

        let mut current_vertex = vertex;
        while self.parent[current_vertex] != current_vertex
            && self.parent_updated_at[current_vertex] <= query_time
        {
            current_vertex = self.parent[current_vertex];
        }

        current_vertex
    }

    #[inline]
    pub fn is_same_root(&self, vertex1: usize, vertex2: usize) -> bool {
        self.is_same_root_at_time(vertex1, vertex2, self.current_time)
    }

    #[inline]
    pub fn is_same_root_at_time(&self, vertex1: usize, vertex2: usize, query_time: usize) -> bool {
        self.root_at_time(vertex1, query_time) == self.root_at_time(vertex2, query_time)
    }

    pub fn unite(&mut self, reference_vertex: usize, vertex: usize) -> bool {

        assert!(reference_vertex + 1 <= self.node_count);
        assert!(vertex + 1 <= self.node_count);

        self.current_time += 1;

        let mut root_x = self.root_at_time(reference_vertex, self.current_time);
        let mut root_y = self.root_at_time(vertex, self.current_time);

        if root_x == root_y {
            return false;
        }

        let mut size_x = self.root_size_at_time(root_x, self.current_time);
        let mut size_y = self.root_size_at_time(root_y, self.current_time);

        if size_x <= size_y {
            std::mem::swap(&mut root_x, &mut root_y);
            std::mem::swap(&mut size_x, &mut size_y);
        }

        self.parent[root_y] = root_x;
        self.parent_updated_at[root_y] = self.current_time;

        self.size_history[root_x].push((self.current_time, size_x + size_y));
        self.record_group_count_if_needed(self.group_count() - 1);

        true
    }

    #[inline]
    pub fn root_size(&self, vertex: usize) -> usize {
        self.root_size_at_time(vertex, self.current_time)
    }

    pub fn root_size_at_time(&self, vertex: usize, query_time: usize) -> usize {
        let root_vertex = self.root_at_time(vertex, query_time);
        Self::history_value(&self.size_history[root_vertex], query_time)
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_count_at_time(self.current_time)
    }

    pub fn group_count_at_time(&self, query_time: usize) -> usize {
        Self::history_value(&self.group_history, query_time)
    }

    #[inline]
    pub fn same_group_members(&self, vertex: usize, index: usize) -> Vec<usize> {
        self.same_group_members_at_time(vertex, self.current_time, index)
    }

    pub fn same_group_members_at_time(&self, vertex: usize, query_time: usize, index: usize) -> Vec<usize> {

        let target_root = self.root_at_time(vertex, query_time);

        (0..self.node_count)
            .filter(|candidate| self.root_at_time(*candidate, query_time) == target_root)
            .map(|candidate| candidate + index)
            .collect::<Vec<usize>>()
    }

    #[inline]
    pub fn all_group_members(&self, index: usize) -> HashMap<usize, Vec<usize>> {
        self.all_group_members_at_time(self.current_time, index)
    }

    pub fn all_group_members_at_time(&self, query_time: usize, index: usize) -> HashMap<usize, Vec<usize>> {

        let mut members = HashMap::new();

        for vertex in 0..self.node_count {
            let root_vertex = self.root_at_time(vertex, query_time);
            members.entry(root_vertex).or_insert_with(Vec::new).push(vertex + index);
        }

        members
    }

    fn record_group_count_if_needed(&mut self, next_count: usize) {

        if let Some((_, current)) = self.group_history.last() {
            if *current == next_count {
                return;
            }
        }

        self.group_history.push((self.current_time, next_count));
    }

    fn history_value(history: &[(usize, usize)], query_time: usize) -> usize {

        let index = history.partition_point(|&(time, _)| time <= query_time);
        history[index.saturating_sub(1)].1
    }
}


#[cfg(test)]
mod tests {
    use super::PartialPersistentUnionFind;
    use std::panic::{AssertUnwindSafe, catch_unwind};

    #[test]
    fn default_matches_new_zero() {
        // Arrange
        let input_value_1 = 0;
        let default_value = PartialPersistentUnionFind::default();
        let new_value = PartialPersistentUnionFind::new(input_value_1);
        let expected_values = (0, 0);

        // Act
        let actual_default = (default_value.time(), default_value.group_count());
        let actual_new = (new_value.time(), new_value.group_count());

        // Assert
        assert_eq!(actual_default, expected_values);
        assert_eq!(actual_new, expected_values);
    }

    #[test]
    fn root_at_time_returns_previous_and_updated_root_around_unite_time() {
        // Arrange
        let size = 3;
        let (first, second) = (0, 1);
        let query_times = [0, 1, 2];
        let expected = [0, 1, 1];
        let mut uf = PartialPersistentUnionFind::new(size);
        uf.unite(first, second);

        // Act
        let actual = query_times.map(|time| uf.root_at_time(first, time));

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn root_at_time_accepts_maximum_query_time() {
        // Arrange
        let node_count = 2;
        let first_vertex = 0;
        let second_vertex = 1;
        let unmerged_union_find = PartialPersistentUnionFind::new(node_count);
        let mut merged_union_find = PartialPersistentUnionFind::new(node_count);
        let query_time = usize::MAX;
        let expected_before_merge = (first_vertex, second_vertex);
        merged_union_find.unite(first_vertex, second_vertex);
        let expected_after_merge = (second_vertex, second_vertex);

        // Act
        let actual_before_merge = (
            unmerged_union_find.root_at_time(first_vertex, query_time),
            unmerged_union_find.root_at_time(second_vertex, query_time),
        );
        let actual_after_merge = (
            merged_union_find.root_at_time(first_vertex, query_time),
            merged_union_find.root_at_time(second_vertex, query_time),
        );

        // Assert
        assert_eq!(actual_before_merge, expected_before_merge);
        assert_eq!(actual_after_merge, expected_after_merge);
    }

    #[test]
    fn root_size_at_time_updates_on_each_merge_boundary() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 0;
        let input_value_4 = 0;
        let input_value_5 = 1;
        let input_value_6 = 0;
        let input_value_7 = 1;
        let input_value_8 = 0;
        let input_value_9 = 2;
        let input_value_10 = 0;
        let input_value_11 = 1;
        let input_value_12 = 0;
        let input_value_13 = 2;
        let input_value_14 = 0;
        let input_value_15 = 3;

        let mut uf = PartialPersistentUnionFind::new(input_value_1);
        let expected_check_1 = 1;
        let expected_check_2 = 2;
        let expected_check_3 = 2;
        let expected_check_4 = 3;
        let expected_check_5 = 3;

        // Act
        let size_before_any = uf.root_size_at_time(input_value_2, input_value_3);
        uf.unite(input_value_4, input_value_5);
        let size_at_first_merge = uf.root_size_at_time(input_value_6, input_value_7);

        uf.unite(input_value_8, input_value_9);
        let size_before_second_merge = uf.root_size_at_time(input_value_10, input_value_11);
        let size_at_second_merge = uf.root_size_at_time(input_value_12, input_value_13);
        let size_after_second_merge = uf.root_size_at_time(input_value_14, input_value_15);

        // Assert
        assert_eq!(size_before_any, expected_check_1);
        assert_eq!(size_at_first_merge, expected_check_2);
        assert_eq!(size_before_second_merge, expected_check_3);
        assert_eq!(size_at_second_merge, expected_check_4);
        assert_eq!(size_after_second_merge, expected_check_5);
    }

    #[test]
    fn group_count_at_time_tracks_each_merge_step() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 0;
        let input_value_4 = 1;
        let input_value_5 = 1;
        let input_value_6 = 1;
        let input_value_7 = 2;
        let input_value_8 = 2;

        let mut uf = PartialPersistentUnionFind::new(input_value_1);
        let expected_check_1 = 3;
        let expected_check_2 = 2;
        let expected_check_3 = 1;
        let expected_check_4 = 1;

        // Act
        let initial_groups = uf.group_count_at_time(input_value_2);
        uf.unite(input_value_3, input_value_4);
        let after_first_merge = uf.group_count_at_time(input_value_5);
        uf.unite(input_value_6, input_value_7);
        let after_second_merge = uf.group_count_at_time(input_value_8);

        let actual_check_4 = uf.group_count();

        // Assert
        assert_eq!(initial_groups, expected_check_1);
        assert_eq!(after_first_merge, expected_check_2);
        assert_eq!(after_second_merge, expected_check_3);
        assert_eq!(actual_check_4, expected_check_4);
    }

    #[test]
    fn same_group_members_at_time_respects_time_and_index_offset() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 0;
        let input_value_4 = 1;
        let input_value_5 = 0;
        let input_value_6 = 1;
        let input_value_7 = 0;
        let input_value_8 = 1;
        let input_value_9 = 1;
        let input_value_10 = 1;
        let input_value_11 = 2;
        let input_value_12 = 0;
        let input_value_13 = 2;
        let input_value_14 = 1;

        let mut uf = PartialPersistentUnionFind::new(input_value_1);
        let expected_check_1 = vec![1];
        let expected_check_2 = vec![1, 2];
        let expected_check_3 = vec![1, 2, 3];

        // Act
        let before_merge =
            uf.same_group_members_at_time(input_value_2, input_value_3, input_value_4);
        uf.unite(input_value_5, input_value_6);
        let after_first_merge =
            uf.same_group_members_at_time(input_value_7, input_value_8, input_value_9);
        uf.unite(input_value_10, input_value_11);
        let after_second_merge =
            uf.same_group_members_at_time(input_value_12, input_value_13, input_value_14);

        // Assert
        assert_eq!(before_merge, expected_check_1);
        assert_eq!(after_first_merge, expected_check_2);
        assert_eq!(after_second_merge, expected_check_3);
    }

    fn normalize_members(mut lists: Vec<Vec<usize>>) -> Vec<Vec<usize>> {
        for list in lists.iter_mut() {
            list.sort_unstable();
        }
        lists.sort_unstable();
        lists
    }

    #[test]
    fn all_group_members_at_time_gathers_each_component_boundary() {
        // Arrange
        let node_count = 4;
        let merges = [(0, 1), (2, 3)];
        let initial_time = 0;
        let first_merge_time = 1;
        let second_merge_time = 2;
        let index = 0;
        let expected_initial = normalize_members(vec![vec![0], vec![1], vec![2], vec![3]]);
        let expected_after_first_merge = normalize_members(vec![vec![0, 1], vec![2], vec![3]]);
        let expected_after_second_merge = normalize_members(vec![vec![0, 1], vec![2, 3]]);
        let mut union_find = PartialPersistentUnionFind::new(node_count);
        for (first_vertex, second_vertex) in merges {
            union_find.unite(first_vertex, second_vertex);
        }

        // Act
        let actual_initial = normalize_members(
            union_find
                .all_group_members_at_time(initial_time, index)
                .into_values()
                .collect(),
        );
        let actual_after_first_merge = normalize_members(
            union_find
                .all_group_members_at_time(first_merge_time, index)
                .into_values()
                .collect(),
        );
        let actual_after_second_merge = normalize_members(
            union_find
                .all_group_members_at_time(second_merge_time, index)
                .into_values()
                .collect(),
        );

        // Assert
        assert_eq!(actual_initial, expected_initial);
        assert_eq!(actual_after_first_merge, expected_after_first_merge);
        assert_eq!(actual_after_second_merge, expected_after_second_merge);
    }

    #[test]
    fn unite_same_component_still_advances_time_with_current_behavior() {
        // Arrange
        let input_value_1 = 2;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 0;
        let input_value_5 = 1;
        let mut union_find = PartialPersistentUnionFind::new(input_value_1);
        union_find.unite(input_value_2, input_value_3);
        let expected_values = (false, 2);

        // Act
        let actual_values = (
            union_find.unite(input_value_4, input_value_5),
            union_find.time(),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn latest_accessors_match_time_based_variants() {
        // Arrange
        let input_value_1 = 3;
        let input_value_2 = 0;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let input_value_5 = 1;
        let input_value_6 = 1;
        let input_value_7 = 0;
        let input_value_8 = 1;
        let input_value_9 = 1;
        let input_value_10 = 1;
        let input_value_11 = 0;
        let mut union_find = PartialPersistentUnionFind::new(input_value_1);
        union_find.unite(input_value_2, input_value_3);
        let current_time = union_find.time();
        let expected_values = (
            union_find.root_at_time(input_value_4, current_time),
            union_find.root_size_at_time(input_value_5, current_time),
            union_find.same_group_members_at_time(input_value_6, current_time, input_value_7),
        );

        // Act
        let actual_values = (
            union_find.root(input_value_8),
            union_find.root_size(input_value_9),
            union_find.same_group_members(input_value_10, input_value_11),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn out_of_range_queries_panic() {
        // Arrange
        let input_value_1 = 2;
        let input_value_2 = 2;
        let input_value_3 = 0;
        let input_value_4 = 2;
        let input_value_5 = 0;
        let input_value_6 = 0;
        let union_find = PartialPersistentUnionFind::new(input_value_1);

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        let actual_root = catch_unwind(AssertUnwindSafe(|| {
            union_find.root_at_time(input_value_2, input_value_3)
        }));
        let actual_members = catch_unwind(AssertUnwindSafe(|| {
            union_find.same_group_members_at_time(input_value_4, input_value_5, input_value_6)
        }));

        let actual_check_1 = actual_root.is_err();
        let actual_check_2 = actual_members.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_check_2, expected_boolean_2);
    }
}
