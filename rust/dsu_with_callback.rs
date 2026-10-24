use std::collections::HashMap;
use std::fmt;

pub struct DsuWithCallback<Callback>
where
    Callback: FnMut(usize, usize),
{
    siz: Vec<usize>,
    rank: Vec<usize>,
    par: Vec<usize>,
    group_number: usize,
    callback: Callback,
}

impl<Callback> fmt::Debug for DsuWithCallback<Callback>
where
    Callback: FnMut(usize, usize),
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DsuWithCallback")
            .field("siz", &self.siz)
            .field("rank", &self.rank)
            .field("par", &self.par)
            .field("group_number", &self.group_number)
            .finish_non_exhaustive()
    }
}

impl<Callback> DsuWithCallback<Callback>
where
    Callback: FnMut(usize, usize),
{
    pub fn new(size: usize, callback: Callback) -> Self {
        Self {
            par: (0..size).collect::<Vec<usize>>(),
            rank: vec![0; size],
            siz: vec![1; size],
            group_number: size,
            callback,
        }
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.par.len()
    }

    #[inline]
    fn size(&self, vertex: usize) -> usize {
        self.siz[vertex]
    }

    #[inline]
    pub fn root(&mut self, x: usize) -> usize {
        if self.par[x] != x {
            self.par[x] = self.root(self.par[x]);
        }

        self.par[x]
    }

    pub fn all_roots(&mut self) -> Vec<usize> {
        (0..self.len())
            .filter(|&vertex| vertex == self.root(vertex))
            .collect::<Vec<usize>>()
    }

    #[inline]
    pub fn is_same_component(&mut self, x: usize, y: usize) -> bool {
        self.root(x) == self.root(y)
    }

    pub fn merge(&mut self, x: usize, y: usize) -> bool {
        let mut root_x = self.root(x);
        let mut root_y = self.root(y);

        if root_x == root_y {
            return false;
        }

        self.group_number -= 1;

        if self.rank[root_x] < self.rank[root_y] {
            std::mem::swap(&mut root_x, &mut root_y);
        }

        self.par[root_y] = root_x;
        self.siz[root_x] += self.siz[root_y];

        if self.rank[root_x] == self.rank[root_y] {
            self.rank[root_x] += 1;
        }

        (self.callback)(root_x, root_y);

        true
    }

    #[inline]
    pub fn component_size(&mut self, index: usize) -> usize {
        let root = self.root(index);
        self.size(root)
    }

    pub fn all_component_size(&mut self) -> Vec<usize> {
        self.all_roots()
            .into_iter()
            .map(|root| self.size(root))
            .collect()
    }

    #[inline]
    pub fn group_count(&self) -> usize {
        self.group_number
    }

    pub fn component_members(&mut self, x: usize) -> Vec<usize> {
        let root_x = self.root(x);
        let mut results = Vec::with_capacity(self.size(root_x));

        for vertex in 0..self.len() {
            if self.root(vertex) == root_x {
                results.push(vertex);
            }
        }

        results
    }

    pub fn components_by_root(&mut self) -> HashMap<usize, Vec<usize>> {
        let mut group_members = HashMap::new();

        for vertex in 0..self.len() {
            let root = self.root(vertex);
            let members = group_members
                .entry(root)
                .or_insert_with(|| Vec::with_capacity(self.size(root)));
            members.push(vertex);
        }

        group_members
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use std::collections::HashMap;

    use std::panic::{AssertUnwindSafe, catch_unwind};

    use std::rc::Rc;

    use super::*;

    fn build_dsu(
        size: usize,
        callback_log: Rc<RefCell<Vec<(usize, usize)>>>,
    ) -> DsuWithCallback<impl FnMut(usize, usize)> {
        DsuWithCallback::new(size, move |root_x, root_y| {
            callback_log.borrow_mut().push((root_x, root_y));
        })
    }

    fn build_dsu_without_log(size: usize) -> DsuWithCallback<impl FnMut(usize, usize)> {
        DsuWithCallback::new(size, |_, _| {})
    }

    fn build(size: usize) -> DsuWithCallback<impl FnMut(usize, usize)> {
        DsuWithCallback::new(size, |_, _| {})
    }

    #[test]
    fn new_with_size_zero_returns_empty_and_zero_group_count() {
        // Arrange
        let size = 0;
        let expected_is_empty = true;
        let expected_len = 0;
        let expected_group_count = 0;

        // Act
        let uf = build_dsu_without_log(size);
        let actual_is_empty = uf.is_empty();
        let actual_len = uf.len();
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_group_count, expected_group_count);
    }

    #[test]
    fn new_with_size_one_returns_single_group() {
        // Arrange
        let size = 1;
        let expected_is_empty = false;
        let expected_len = 1;
        let expected_group_count = 1;

        // Act
        let uf = build_dsu_without_log(size);
        let actual_is_empty = uf.is_empty();
        let actual_len = uf.len();
        let actual_group_count = uf.group_count();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_group_count, expected_group_count);
    }

    #[test]
    fn root_with_lower_bound_index_zero_returns_zero() {
        // Arrange
        let size = 3;
        let index = 0;
        let expected_root = 0;
        let mut uf = build_dsu_without_log(size);

        // Act
        let actual_root = uf.root(index);

        // Assert
        assert_eq!(actual_root, expected_root);
    }

    #[test]
    fn root_with_upper_bound_index_len_minus_one_returns_index() {
        // Arrange
        let size = 3;
        let mut uf = build_dsu_without_log(size);
        let index = uf.len() - 1;
        let expected_root = index;

        // Act
        let actual_root = uf.root(index);

        // Assert
        assert_eq!(actual_root, expected_root);
    }

    #[test]
    fn root_with_out_of_range_index_len_panics() {
        // Arrange
        let size = 3;
        let mut uf = build_dsu_without_log(size);
        let out_of_range_index = uf.len();

        let expected_boolean_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.root(out_of_range_index);
        }));

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
    }

    #[test]
    fn all_roots_returns_unique_roots_after_path_compression() {
        // Arrange
        let size = 4;
        let first_pair = (0, 1);
        let second_pair = (2, 3);
        let merge_pair = (0, 2);
        let compressed_vertex = 3;
        let mut dsu = build(size);
        dsu.merge(first_pair.0, first_pair.1);
        dsu.merge(second_pair.0, second_pair.1);
        dsu.merge(merge_pair.0, merge_pair.1);
        let expected_roots = vec![0];

        // Act
        dsu.root(compressed_vertex);
        let mut actual_roots = dsu.all_roots();
        actual_roots.sort_unstable();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn merge_with_distinct_groups_executes_callback_once_with_merged_roots() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let size = 3;
        let callback_log = Rc::new(RefCell::new(Vec::<(usize, usize)>::new()));
        let expected_merge_result = true;
        let expected_group_count = 2;
        let expected_callback_log = vec![(0, 1)];
        let mut uf = build_dsu(size, Rc::clone(&callback_log));

        // Act
        let actual_merge_result = uf.merge(input_value_1, input_value_2);
        let actual_group_count = uf.group_count();
        let actual_callback_log = callback_log.borrow().clone();

        // Assert
        assert_eq!(actual_merge_result, expected_merge_result);
        assert_eq!(actual_group_count, expected_group_count);
        assert_eq!(actual_callback_log, expected_callback_log);
    }

    #[test]
    fn merge_when_rank_swap_is_needed_executes_callback_with_swapped_roots() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 2;
        let input_value_4 = 0;
        let size = 4;
        let callback_log = Rc::new(RefCell::new(Vec::<(usize, usize)>::new()));
        let expected_callback_log = vec![(0, 1), (0, 2)];
        let mut uf = build_dsu(size, Rc::clone(&callback_log));
        uf.merge(input_value_1, input_value_2);

        let expected_boolean_1 = true;

        // Act
        let actual_merge_result = uf.merge(input_value_3, input_value_4);
        let actual_callback_log = callback_log.borrow().clone();

        // Assert
        assert_eq!(actual_merge_result, expected_boolean_1);
        assert_eq!(actual_callback_log, expected_callback_log);
    }

    #[test]
    fn merge_with_same_vertex_returns_false_and_does_not_execute_callback() {
        // Arrange
        let size = 4;
        let index = 2;
        let callback_log = Rc::new(RefCell::new(Vec::<(usize, usize)>::new()));
        let expected_merge_result = false;
        let expected_callback_log = Vec::<(usize, usize)>::new();
        let mut uf = build_dsu(size, Rc::clone(&callback_log));

        // Act
        let actual_merge_result = uf.merge(index, index);
        let actual_callback_log = callback_log.borrow().clone();

        // Assert
        assert_eq!(actual_merge_result, expected_merge_result);
        assert_eq!(actual_callback_log, expected_callback_log);
    }

    #[test]
    fn merge_with_already_connected_vertices_returns_false_and_does_not_execute_callback() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 0;
        let input_value_6 = 2;
        let size = 4;
        let callback_log = Rc::new(RefCell::new(Vec::<(usize, usize)>::new()));
        let expected_merge_result = false;
        let expected_callback_log = vec![(0, 1), (0, 2)];
        let mut uf = build_dsu(size, Rc::clone(&callback_log));
        uf.merge(input_value_1, input_value_2);
        uf.merge(input_value_3, input_value_4);

        // Act
        let actual_merge_result = uf.merge(input_value_5, input_value_6);
        let actual_callback_log = callback_log.borrow().clone();

        // Assert
        assert_eq!(actual_merge_result, expected_merge_result);
        assert_eq!(actual_callback_log, expected_callback_log);
    }

    #[test]
    fn merge_with_size_zero_panics_and_does_not_execute_callback() {
        // Arrange
        let size = 0;
        let reference_vertex = 0;
        let vertex = 0;
        let callback_log = Rc::new(RefCell::new(Vec::<(usize, usize)>::new()));
        let expected_callback_log = Vec::<(usize, usize)>::new();
        let mut uf = build_dsu(size, Rc::clone(&callback_log));

        let expected_boolean_1 = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = uf.merge(reference_vertex, vertex);
        }));
        let actual_callback_log = callback_log.borrow().clone();

        let actual_check_1 = panic_result.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_callback_log, expected_callback_log);
    }

    #[test]
    fn component_size_after_two_merges_returns_three() {
        // Arrange
        let size = 4;
        let first_reference_vertex = 0;
        let first_vertex = 1;
        let second_reference_vertex = 1;
        let second_vertex = 2;
        let target_vertex = 2;
        let expected_component_size = 3;
        let mut uf = build_dsu_without_log(size);
        uf.merge(first_reference_vertex, first_vertex);
        uf.merge(second_reference_vertex, second_vertex);

        // Act
        let actual_component_size = uf.component_size(target_vertex);

        // Assert
        assert_eq!(actual_component_size, expected_component_size);
    }

    #[test]
    fn all_component_size_returns_each_component_size() {
        // Arrange
        let size = 5;
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 3;
        let input_value_4 = 4;
        let mut dsu = build(size);
        dsu.merge(input_value_1, input_value_2);
        dsu.merge(input_value_3, input_value_4);
        let mut expected_sizes = vec![1, 2, 2];
        expected_sizes.sort_unstable();

        // Act
        let mut actual_sizes = dsu.all_component_size();
        actual_sizes.sort_unstable();

        // Assert
        assert_eq!(actual_sizes, expected_sizes);
    }

    #[test]
    fn group_views_count_always_matches_group_count_after_each_operation_step() {
        // Arrange
        let size = 6;
        let merge_operations = vec![(0, 1), (2, 3), (1, 2), (4, 5), (0, 5), (0, 5)];
        let mut uf = build_dsu_without_log(size);

        // Act
        for (left_vertex, right_vertex) in merge_operations {
            uf.merge(left_vertex, right_vertex);

            let actual_group_count = uf.group_count();
            let actual_roots_count = uf.all_roots().len();
            let actual_component_sizes_count = uf.all_component_size().len();
            let actual_group_members_count = uf.components_by_root().len();

            // Assert
            assert_eq!(actual_roots_count, actual_group_count);
            assert_eq!(actual_component_sizes_count, actual_group_count);
            assert_eq!(actual_group_members_count, actual_group_count);
        }
    }

    #[test]
    fn component_members_handles_singleton_and_merged_groups() {
        // Arrange
        let size = 4;
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 0;
        let input_value_4 = 3;
        let mut dsu = build(size);
        dsu.merge(input_value_1, input_value_2);
        let expected_merged = vec![0, 1];
        let expected_singleton = vec![3];

        // Act
        let actual_merged = dsu.component_members(input_value_3);
        let actual_singleton = dsu.component_members(input_value_4);

        // Assert
        assert_eq!(actual_merged, expected_merged);
        assert_eq!(actual_singleton, expected_singleton);
    }

    #[test]
    fn out_of_range_group_queries_panic() {
        // Arrange
        let size = 2;
        let input_value_1 = 2;
        let input_value_2 = 2;
        let mut dsu = build(size);

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        let actual_component_size =
            catch_unwind(AssertUnwindSafe(|| dsu.component_size(input_value_1)));
        let actual_members =
            catch_unwind(AssertUnwindSafe(|| dsu.component_members(input_value_2)));

        let actual_check_1 = actual_component_size.is_err();
        let actual_check_2 = actual_members.is_err();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_check_2, expected_boolean_2);
    }

    #[test]
    fn components_by_root_after_merging_all_vertices_returns_single_group() {
        // Arrange
        let input_value_1 = 0;
        let size = 5;
        let expected_group_members = HashMap::from([(0, (0..size).collect::<Vec<usize>>())]);
        let mut uf = build_dsu_without_log(size);
        for vertex in 1..size {
            uf.merge(input_value_1, vertex);
        }

        // Act
        let actual_group_members = uf.components_by_root();

        // Assert
        assert_eq!(actual_group_members, expected_group_members);
    }
}
