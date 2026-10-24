pub struct Permutation<T> {
    v: Vec<T>,
}

impl<T: Ord + Clone> Permutation<T> {
    pub fn new(v: Vec<T>) -> Self {
        Self { v }
    }

    pub fn as_slice(&self) -> &Vec<T> {
        &self.v
    }

    pub fn list(&self) -> Vec<T> {
        self.v.clone()
    }

    pub fn permutate_iter(&mut self) -> impl Iterator<Item = Vec<T>> + '_ {
        std::iter::successors(Some(self.list()), move |_| {
            if self.next_permutation() {
                Some(self.list())
            } else {
                None
            }
        })
    }

    pub fn rev_permutate_iter(&mut self) -> impl Iterator<Item = Vec<T>> + '_ {
        std::iter::successors(Some(self.list()), move |_| {
            if self.prev_permutation() {
                Some(self.list())
            } else {
                None
            }
        })
    }

    pub fn permutate(&mut self) -> Vec<Vec<T>> {
        std::iter::once(self.list())
            .chain(std::iter::from_fn(|| match self.next_permutation() {
                true => Some(self.list()),
                false => None,
            }))
            .collect()
    }

    pub fn rev_permutate(&mut self) -> Vec<Vec<T>> {
        std::iter::once(self.list())
            .chain(std::iter::from_fn(|| match self.prev_permutation() {
                true => Some(self.list()),
                false => None,
            }))
            .collect()
    }

    pub fn prev_permutation(&mut self) -> bool {
        match self.prev_pivot() {
            Some(index) => {
                self.range_reverse(index);
                self.swap_prev_pivot(index);

                true
            }
            None => false,
        }
    }

    pub fn next_permutation(&mut self) -> bool {
        match self.next_pivot() {
            Some(index) => {
                self.range_reverse(index);
                self.swap_next_pivot(index);

                true
            }
            None => false,
        }
    }

    fn range_reverse(&mut self, index: usize) {
        let n = self.v.len();

        let half_len = (n - (index + 1)) / 2;

        for i in 0..half_len {
            let a = index + 1 + i;
            let b = n - 1 - i;

            self.v.swap(a, b);
        }
    }

    fn prev_pivot(&self) -> Option<usize> {
        if self.v.len() < 2 {
            return None;
        }

        let mut i = self.v.len() - 1;

        while let Some(index) = i.checked_sub(1) {
            if self.v[index] <= self.v[index + 1] {
                i = index;
            } else {
                return Some(index);
            }
        }

        None
    }

    fn next_pivot(&self) -> Option<usize> {
        if self.v.len() < 2 {
            return None;
        }

        let mut i = self.v.len() - 1;

        while let Some(index) = i.checked_sub(1) {
            if self.v[index + 1] <= self.v[index] {
                i = index;
            } else {
                return Some(index);
            }
        }

        None
    }

    fn swap_prev_pivot(&mut self, index: usize) {
        let n = self.v.len();

        for i in index..n {
            if self.v[i] < self.v[index] {
                self.v.swap(i, index);
                return;
            }
        }

        unreachable!()
    }

    fn swap_next_pivot(&mut self, index: usize) {
        let n = self.v.len();
        for i in index..n {
            if self.v[index] < self.v[i] {
                self.v.swap(i, index);
                return;
            }
        }

        unreachable!()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn as_slice_on_empty_input_returns_empty_sequence() {
        // Arrange
        let v: Vec<i32> = vec![];

        let expected = v.clone();
        let p = Permutation::new(v);

        // Act
        let actual = p.as_slice().clone();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn as_slice_on_single_input_returns_original_sequence() {
        // Arrange
        let v = vec![42];

        let expected = v.clone();
        let p = Permutation::new(v);

        // Act
        let actual = p.as_slice().clone();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn as_slice_on_multiple_inputs_returns_original_sequence() {
        // Arrange
        let v = vec![0, 1, 2];

        let expected = v.clone();
        let p = Permutation::new(v);

        // Act
        let actual = p.as_slice().clone();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn list_on_empty_input_returns_empty_sequence() {
        // Arrange
        let v: Vec<i32> = vec![];

        let expected = v.clone();
        let p = Permutation::new(v);

        // Act
        let actual = p.list();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn list_on_single_input_returns_original_sequence() {
        // Arrange
        let v = vec![42];

        let expected = v.clone();
        let p = Permutation::new(v);

        // Act
        let actual = p.list();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn list_on_multiple_inputs_returns_original_sequence() {
        // Arrange
        let v = vec![0, 1, 2];

        let expected = v.clone();
        let p = Permutation::new(v);

        // Act
        let actual = p.list();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn permutate_iter_returns_same_sequence_as_permutate() {
        // Arrange
        let v = vec![0, 1, 2];
        let mut iterator_permutation = Permutation::new(v.clone());
        let mut vector_permutation = Permutation::new(v);

        let expected = vector_permutation.permutate();

        // Act
        let actual = iterator_permutation.permutate_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn rev_permutate_iter_returns_same_sequence_as_rev_permutate() {
        // Arrange
        let v = vec![2, 1, 0];
        let mut iterator_permutation = Permutation::new(v.clone());
        let mut vector_permutation = Permutation::new(v);

        let expected = vector_permutation.rev_permutate();

        // Act
        let actual = iterator_permutation
            .rev_permutate_iter()
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn permutate_on_empty_input_returns_one_empty_permutation() {
        // Arrange
        let v: Vec<i32> = vec![];
        let mut p = Permutation::new(v);

        // The iterator advances only after the initial permutation.
        let expected: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn permutate_on_single_input_returns_original_permutation() {
        // Arrange
        let v = vec![42];
        let mut p = Permutation::new(v);
        let expected = vec![vec![42]];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn permutate_from_smallest_returns_all_permutations_in_ascending_order() {
        // Arrange
        let v = vec![0, 1, 2];
        let mut p = Permutation::new(v);
        let expected = vec![
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn permutate_with_duplicates_returns_unique_ascending_permutations() {
        // Arrange
        let v = vec![1, 1, 2];
        let mut p = Permutation::new(v);
        let expected = vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn permutate_from_largest_returns_only_current_permutation() {
        // Arrange
        let v = vec![3, 2, 2, 1];
        let mut p = Permutation::new(v);
        let expected = vec![vec![3, 2, 2, 1]];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn permutate_from_middle_state_returns_remaining_suffix_sequence() {
        // Arrange
        let v = vec![0, 1, 2];

        let mut p = Permutation::new(v);

        p.next_permutation();
        let expected = vec![
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn permutate_works_for_string_values() {
        // Arrange
        let v = vec![String::from("a"), String::from("b")];

        let mut p = Permutation::new(v);

        let expected = vec![
            vec![String::from("a"), String::from("b")],
            vec![String::from("b"), String::from("a")],
        ];

        // Act
        let actual = p.permutate();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn rev_permutate_on_empty_input_returns_one_empty_permutation() {
        // Arrange
        let v: Vec<i32> = vec![];
        let mut p = Permutation::new(v);

        // The iterator advances only after the initial permutation.
        let expected: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual = p.rev_permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn rev_permutate_on_single_input_returns_original_permutation() {
        // Arrange
        let v = vec![42];
        let mut p = Permutation::new(v);
        let expected = vec![vec![42]];

        // Act
        let actual = p.rev_permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn rev_permutate_from_largest_returns_all_permutations_in_descending_order() {
        // Arrange
        let v = vec![2, 1, 0];
        let mut p = Permutation::new(v);
        let expected = vec![
            vec![2, 1, 0],
            vec![2, 0, 1],
            vec![1, 2, 0],
            vec![1, 0, 2],
            vec![0, 2, 1],
            vec![0, 1, 2],
        ];

        // Act
        let actual = p.rev_permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn rev_permutate_with_duplicates_returns_unique_descending_permutations() {
        // Arrange
        let v = vec![2, 2, 1];
        let mut p = Permutation::new(v);
        let expected = vec![vec![2, 2, 1], vec![2, 1, 2], vec![1, 2, 2]];

        // Act
        let actual = p.rev_permutate();

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn rev_permutate_from_smallest_returns_only_current_permutation() {
        // Arrange
        let v = vec![0, 1, 1, 3];

        let expected = vec![v.clone()];
        let mut p = Permutation::new(v);

        // Act
        let actual = p.rev_permutate();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn prev_permutation_on_empty_input_returns_false_and_keeps_empty() {
        // Arrange
        let v: Vec<i32> = vec![];
        let mut p = Permutation::new(v);

        let expected_boolean_1 = false;
        let expected_boolean_2 = true;

        // Act
        let actual_check_1 = p.prev_permutation();
        let actual_check_2 = p.list().is_empty();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual_check_2, expected_boolean_2);
    }

    #[test]
    fn prev_permutation_on_single_input_returns_false_and_preserves_value() {
        // Arrange
        let v = vec![42];

        let expected = v.clone();
        let mut p = Permutation::new(v);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = p.prev_permutation();

        let actual = p.list();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn prev_permutation_from_descending_order_returns_true_and_previous_order() {
        // Arrange
        let v = vec![2, 1, 0];

        let mut p = Permutation::new(v);

        let expected = vec![2, 0, 1];

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = p.prev_permutation();

        let actual = p.list();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn prev_permutation_with_duplicates_returns_true_and_previous_distinct_order() {
        // Arrange
        let v = vec![2, 2, 1];

        let mut p = Permutation::new(v);

        let expected = vec![2, 1, 2];

        let expected_boolean_1 = true;

        // Act
        let actual_check_1 = p.prev_permutation();

        let actual = p.list();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn prev_permutation_from_smallest_returns_false_and_preserves_order() {
        // Arrange
        let v = vec![0, 1, 1, 3];

        let expected = v.clone();
        let mut p = Permutation::new(v);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = p.prev_permutation();

        let actual = p.as_slice().clone();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn prev_permutation_once_updates_internal_state_to_previous_order() {
        // Arrange
        let v = vec![2, 1, 0];

        let mut p = Permutation::new(v);

        let expected = vec![2, 0, 1];

        let expected_boolean_1 = true;

        // Act
        let changed = p.prev_permutation();
        let actual = p.list();

        // Assert
        assert_eq!(changed, expected_boolean_1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn next_permutation_on_empty_input_returns_no_successors() {
        // Arrange
        let v: Vec<i32> = vec![];
        let mut p = Permutation::new(v);

        // The iterator advances only after the initial permutation.
        let expected: Vec<Vec<i32>> = vec![];

        // Act
        let mut actual = vec![];
        while p.next_permutation() {
            actual.push(p.as_slice().clone());
        }

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn next_permutation_on_single_input_keeps_only_initial_permutation() {
        // Arrange
        let v = vec![42];
        let mut p = Permutation::new(v);
        let expected = vec![vec![42]];

        // Act
        let mut actual = vec![p.as_slice().clone()];
        while p.next_permutation() {
            actual.push(p.as_slice().clone());
        }

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn next_permutation_enumerates_all_ascending_permutations() {
        // Arrange
        let v = vec![0, 1, 2];
        let mut p = Permutation::new(v);
        let expected = vec![
            vec![0, 1, 2],
            vec![0, 2, 1],
            vec![1, 0, 2],
            vec![1, 2, 0],
            vec![2, 0, 1],
            vec![2, 1, 0],
        ];

        // Act
        let mut actual = vec![p.as_slice().clone()];
        while p.next_permutation() {
            actual.push(p.as_slice().clone());
        }

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn next_permutation_with_duplicates_enumerates_unique_ascending_permutations() {
        // Arrange
        let v = vec![1, 1, 2];
        let mut p = Permutation::new(v);
        let expected = vec![vec![1, 1, 2], vec![1, 2, 1], vec![2, 1, 1]];

        // Act
        let mut actual = vec![p.as_slice().clone()];
        while p.next_permutation() {
            actual.push(p.as_slice().clone());
        }

        // Assert
        assert_eq!(actual, expected, "lexicographical order should match");
    }

    #[test]
    fn next_permutation_from_largest_returns_false_and_preserves_order() {
        // Arrange
        let v = vec![3, 2, 2, 1];

        let expected = v.clone();
        let mut p = Permutation::new(v);

        let expected_boolean_1 = false;

        // Act
        let actual_check_1 = p.next_permutation();
        let actual = p.as_slice().clone();

        // Assert
        assert_eq!(actual_check_1, expected_boolean_1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn next_permutation_once_updates_internal_state_to_next_order() {
        // Arrange
        let v = vec![0, 1, 2];

        let mut p = Permutation::new(v);

        let expected = vec![0, 2, 1];

        let expected_boolean_1 = true;

        // Act
        let changed = p.next_permutation();
        let actual = p.list();

        // Assert
        assert_eq!(changed, expected_boolean_1);
        assert_eq!(actual, expected);
    }

    #[test]
    fn next_then_prev_through_all_permutations_restores_original_order() {
        // Arrange
        let v = vec![0, 1, 2, 3];

        let expected = v.clone();
        let mut p = Permutation::new(v);

        // Act
        while p.next_permutation() {}
        while p.prev_permutation() {}

        let actual_check_1 = p.as_slice().clone();

        // Assert
        assert_eq!(actual_check_1, expected);
    }
}
