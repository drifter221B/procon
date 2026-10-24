#[derive(Debug, Clone)]
pub struct Dsu<T, F>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{
    parent: Vec<usize>,
    rank: Vec<usize>,
    size: Vec<usize>,
    value: Vec<T>,
    group_count: usize,
    combine: F,
}

impl<T, F> Dsu<T, F>
where
    T: Clone,
    F: Fn(&T, &T) -> T,
{
    pub fn new(initial: Vec<T>, combine: F) -> Self {
        let n = initial.len();
        Self {
            parent: (0..n).collect(),
            rank: vec![0; n],
            size: vec![1; n],
            value: initial,
            group_count: n,
            combine,
        }
    }

    pub fn len(&self) -> usize {
        self.parent.len()
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn group_count(&self) -> usize {
        self.group_count
    }

    pub fn root(&mut self, x: usize) -> usize {
        if self.parent[x] == x {
            x
        } else {
            let root = self.root(self.parent[x]);
            self.parent[x] = root;
            root
        }
    }

    pub fn merge(&mut self, x: usize, y: usize) -> bool {
        let mut rx = self.root(x);
        let mut ry = self.root(y);
        if rx == ry {
            return false;
        }
        if self.rank[rx] < self.rank[ry] {
            std::mem::swap(&mut rx, &mut ry);
        }
        self.parent[ry] = rx;
        self.size[rx] += self.size[ry];
        let combined = (self.combine)(&self.value[rx], &self.value[ry]);
        self.value[rx] = combined;
        if self.rank[rx] == self.rank[ry] {
            self.rank[rx] += 1;
        }
        self.group_count -= 1;
        true
    }

    pub fn is_same_component(&mut self, x: usize, y: usize) -> bool {
        self.root(x) == self.root(y)
    }

    pub fn component_size(&mut self, x: usize) -> usize {
        let root = self.root(x);
        self.size[root]
    }

    pub fn component_value(&mut self, x: usize) -> T {
        let root = self.root(x);
        self.value[root].clone()
    }

    pub fn all_roots(&mut self) -> Vec<usize> {
        let mut roots = Vec::new();
        for i in 0..self.len() {
            if self.root(i) == i {
                roots.push(i);
            }
        }
        roots
    }
}

#[cfg(test)]
mod tests {
    use super::Dsu;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn new_with_two_values_returns_two_vertices_and_components() {
        // Arrange
        let values = vec![1, 2];
        let expected_lengths = (2, 2);

        // Act
        let dsu = Dsu::new(values, |left, _| *left);
        let actual_lengths = (dsu.len(), dsu.group_count());

        // Assert
        assert_eq!(actual_lengths, expected_lengths);
    }

    #[test]
    fn empty_initial_vector_has_zero_len_and_group_count() {
        // Arrange
        let dsu = Dsu::new(Vec::<i32>::new(), |left: &i32, right: &i32| left + right);
        let expected_values = (0, 0);

        // Act
        let actual_values = (dsu.len(), dsu.group_count());

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn singleton_component_returns_initial_size_and_value() {
        // Arrange
        let mut dsu = Dsu::new(vec![42], |left: &i32, right: &i32| left + right);
        let expected_values = (1, 42);

        let component_size_vertex = 0;
        let component_value_vertex = 0;

        // Act
        let actual_values = (
            dsu.component_size(component_size_vertex),
            dsu.component_value(component_value_vertex),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn root_with_out_of_range_vertex_panics() {
        // Arrange
        let mut dsu = Dsu::new(vec![1, 2], |left: &i32, right: &i32| left + right);
        let invalid_vertex = dsu.len();

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| dsu.root(invalid_vertex)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn non_commutative_combine_follows_root_selection_order() {
        // Arrange
        let mut dsu = Dsu::new(
            vec!["a".to_string(), "b".to_string()],
            |left: &String, right: &String| format!("{}>{}", left, right),
        );
        let expected_value = "b>a".to_string();

        let merge_left_vertex = 1;
        let merge_right_vertex = 0;
        let component_value_vertex = 0;

        // Act
        dsu.merge(merge_left_vertex, merge_right_vertex);
        let actual_value = dsu.component_value(component_value_vertex);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn merge_sequence_with_duplicate_preserves_component_sizes_counts_and_sums() {
        // Arrange
        let mut dsu = Dsu::new(vec![1, 2, 3], |left: &i32, right: &i32| left + right);
        let operations = [(0, 1), (0, 1), (1, 2)];
        let expected_snapshots = vec![(true, 2, 2, 3), (false, 2, 2, 3), (true, 1, 3, 6)];

        // Act
        let actual_snapshots = operations
            .into_iter()
            .map(|(left, right)| {
                let merged = dsu.merge(left, right);
                (
                    merged,
                    dsu.group_count(),
                    dsu.component_size(right),
                    dsu.component_value(right),
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_snapshots, expected_snapshots);
    }

    #[test]
    fn is_same_component_checks_connectivity() {
        // Arrange
        let mut dsu = Dsu::new(vec![0, 0, 0], |left: &i32, right: &i32| left + right);
        let left = 0;
        let right = 1;
        let separate = 2;
        let expected_connectivity = (false, true, false);

        // Act
        let before = dsu.is_same_component(left, right);
        dsu.merge(left, right);
        let actual_connectivity = (
            before,
            dsu.is_same_component(left, right),
            dsu.is_same_component(left, separate),
        );

        // Assert
        assert_eq!(actual_connectivity, expected_connectivity);
    }

    #[test]
    fn all_roots_collects_components() {
        // Arrange
        let mut dsu = Dsu::new(vec![1, 1, 1, 1], |left: &i32, right: &i32| left + right);
        let edges = [(0, 1), (2, 3)];
        for (left, right) in edges {
            dsu.merge(left, right);
        }
        let expected_roots = vec![0, 2];

        // Act
        let mut actual_roots = dsu.all_roots();
        actual_roots.sort_unstable();

        // Assert
        assert_eq!(actual_roots, expected_roots);
    }

    #[test]
    fn merge_with_out_of_range_vertex_panics() {
        // Arrange
        let mut dsu = Dsu::new(vec![1, 2], |left: &i32, right: &i32| left + right);
        let valid_vertex = 0;
        let invalid_vertex = dsu.len();

        // Act
        let actual_result =
            catch_unwind(AssertUnwindSafe(|| dsu.merge(valid_vertex, invalid_vertex)));

        // Assert
        assert!(actual_result.is_err());
    }

}
