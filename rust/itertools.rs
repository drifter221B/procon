pub struct Itertools;

impl Itertools {

    pub fn combination<T: Clone>(vec: &Vec<T>, k: usize) -> Vec<Vec<T>> {

        fn dfs<T: Clone>(
            vec: &[T],
            k: usize,
            index: usize,
            current: &mut Vec<T>,
            results: &mut Vec<Vec<T>>,
        ) {
            if current.len() == k {
                results.push(current.clone());

                return;
            }

            for i in index..vec.len() {
                current.push(vec[i].clone());
                dfs(vec, k, i + 1, current, results);
                current.pop();
            }
        }

        let mut results = Vec::new();

        if vec.len() < k {
            return results;
        }

        dfs(vec, k, 0, &mut Vec::new(), &mut results);

        results
    }

    pub fn permutation<T: Clone>(vec: &Vec<T>, k: usize) -> Vec<Vec<T>> {

        fn dfs<T: Clone>(
            vec: &[T],
            k: usize,
            current: &mut Vec<usize>,
            results: &mut Vec<Vec<T>>,
        ) {
            if current.len() == k {

                let result: Vec<T> = current.iter()
                    .map(|&i| vec[i].clone())
                    .collect();

                results.push(result);

                return;
            }

            for i in 0..vec.len() {

                if !current.contains(&i) {
                    current.push(i);
                    dfs(vec, k, current, results);
                    current.pop();
                }
            }
        }

        let mut results = Vec::new();

        if vec.len() < k {
            return results;
        }

        dfs(vec, k, &mut Vec::new(), &mut results);

        results
    }

    pub fn next_permutation<T: Ord>(mut vec: Vec<T>, k: usize) -> Option<Vec<T>> {

        for _ in 0..k {
            if !Self::inner_next_permutation(&mut vec) {
                return None;
            }
        }
        Some(vec)
    }

    pub fn prev_permutation<T: Ord>(mut vec: Vec<T>, k: usize) -> Option<Vec<T>> {
        for _ in 0..k {
            if !Self::inner_prev_permutation(&mut vec) {
                return None;
            }
        }
        Some(vec)
    }

    fn inner_next_permutation<T: Ord>(seq: &mut [T]) -> bool {
        let n = seq.len();
        if n < 2 {
            return false;
        }

        let mut i = n - 2;
        while i != usize::MAX && seq[i + 1] <= seq[i]  {
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if i == 0 && seq[i + 1] <= seq[i] {
            seq.reverse();
            return false;
        }

        let mut j = n - 1;
        while seq[j] <= seq[i] {
            j -= 1;
        }

        seq.swap(i, j);
        seq[i + 1..].reverse();

        true
    }

    fn inner_prev_permutation<T: Ord>(seq: &mut [T]) -> bool {
        let n = seq.len();
        if n < 2 {
            return false;
        }

        let mut i = n - 2;
        while i != usize::MAX && seq[i] <= seq[i + 1] {
            if i == 0 {
                break;
            }
            i -= 1;
        }

        if i == 0 && seq[i] <= seq[i + 1] {
            seq.reverse();
            return false;
        }

        let mut j = n - 1;
        while seq[i] <= seq[j]  {
            j -= 1;
        }

        seq.swap(i, j);
        seq[i + 1..].reverse();

        true
    }

}

#[cfg(test)]
mod tests {
    use super::Itertools;

    #[test]
    fn combination_when_vector_is_empty_and_k_is_zero_returns_single_empty_set() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let combination_size = 0usize;
        let expected_combinations: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_vector_is_empty_and_k_is_one_returns_empty_results() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let combination_size = 1usize;
        let expected_combinations: Vec<Vec<i32>> = vec![];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_vector_has_single_element_and_k_is_zero_returns_single_empty_set() {
        // Arrange
        let input_values = vec![42];
        let combination_size = 0usize;
        let expected_combinations: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_vector_has_single_element_and_k_is_one_returns_single_element_set() {
        // Arrange
        let input_values = vec![42];
        let combination_size = 1usize;
        let expected_combinations = vec![vec![42]];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_vector_has_single_element_and_k_is_two_returns_empty_results() {
        // Arrange
        let input_values = vec![42];
        let combination_size = 2usize;
        let expected_combinations: Vec<Vec<i32>> = vec![];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_vector_has_three_elements_and_k_is_two_returns_all_pairs() {
        // Arrange
        let input_values = vec![1, 2, 3];
        let combination_size = 2usize;
        let expected_combinations = vec![vec![1, 2], vec![1, 3], vec![2, 3]];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_k_is_length_minus_one_length_length_plus_one_returns_expected_counts() {
        // Arrange
        let input_values = vec![1, 2, 3];
        let x = input_values.len();
        let k_minus_one = x - 1usize;
        let k_equal = x;
        let k_plus_one = x + 1usize;
        let expected_count_minus_one = 3usize;
        let expected_count_equal = 1usize;
        let expected_count_plus_one = 0usize;

        // Act
        let actual_combinations_minus_one = Itertools::combination(&input_values, k_minus_one);
        let actual_combinations_equal = Itertools::combination(&input_values, k_equal);
        let actual_combinations_plus_one = Itertools::combination(&input_values, k_plus_one);

        // Assert
        assert_eq!(expected_count_minus_one, actual_combinations_minus_one.len());
        assert_eq!(expected_count_equal, actual_combinations_equal.len());
        assert_eq!(expected_count_plus_one, actual_combinations_plus_one.len());
    }

    #[test]
    fn combination_when_vector_has_duplicates_returns_index_based_results_with_duplicates() {
        // Arrange
        let input_values = vec![1, 1, 2];
        let combination_size = 2usize;
        let expected_combinations = vec![vec![1, 1], vec![1, 2], vec![1, 2]];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn combination_when_type_is_string_generates_expected_combinations() {
        // Arrange
        let input_values = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let combination_size = 2usize;
        let expected_combinations = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["a".to_string(), "c".to_string()],
            vec!["b".to_string(), "c".to_string()],
        ];

        // Act
        let actual_combinations = Itertools::combination(&input_values, combination_size);

        // Assert
        assert_eq!(expected_combinations, actual_combinations);
    }

    #[test]
    fn permutation_when_vector_is_empty_and_k_is_zero_returns_single_empty_permutation() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let permutation_size = 0usize;
        let expected_permutations: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_vector_is_empty_and_k_is_one_returns_empty_results() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let permutation_size = 1usize;
        let expected_permutations: Vec<Vec<i32>> = vec![];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_vector_has_single_element_and_k_is_zero_returns_single_empty_permutation() {
        // Arrange
        let input_values = vec![42];
        let permutation_size = 0usize;
        let expected_permutations: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_vector_has_single_element_and_k_is_one_returns_single_element_permutation() {
        // Arrange
        let input_values = vec![42];
        let permutation_size = 1usize;
        let expected_permutations = vec![vec![42]];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_vector_has_single_element_and_k_is_two_returns_empty_results() {
        // Arrange
        let input_values = vec![42];
        let permutation_size = 2usize;
        let expected_permutations: Vec<Vec<i32>> = vec![];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_vector_has_three_elements_and_k_is_zero_returns_single_empty_permutation() {
        // Arrange
        let input_values = vec![1, 2, 3];
        let permutation_size = 0usize;
        let expected_permutations: Vec<Vec<i32>> = vec![vec![]];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_vector_has_three_elements_and_k_is_two_returns_all_two_length_permutations()
    {
        // Arrange
        let input_values = vec![1, 2, 3];
        let permutation_size = 2usize;
        let expected_permutations = vec![
            vec![1, 2],
            vec![1, 3],
            vec![2, 1],
            vec![2, 3],
            vec![3, 1],
            vec![3, 2],
        ];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_k_is_length_minus_one_length_length_plus_one_returns_expected_counts() {
        // Arrange
        let input_values = vec![1, 2, 3];
        let x = input_values.len();
        let k_minus_one = x - 1usize;
        let k_equal = x;
        let k_plus_one = x + 1usize;
        let expected_count_minus_one = 6usize;
        let expected_count_equal = 6usize;
        let expected_count_plus_one = 0usize;

        // Act
        let actual_permutations_minus_one = Itertools::permutation(&input_values, k_minus_one);
        let actual_permutations_equal = Itertools::permutation(&input_values, k_equal);
        let actual_permutations_plus_one = Itertools::permutation(&input_values, k_plus_one);

        // Assert
        assert_eq!(expected_count_minus_one, actual_permutations_minus_one.len());
        assert_eq!(expected_count_equal, actual_permutations_equal.len());
        assert_eq!(expected_count_plus_one, actual_permutations_plus_one.len());
    }

    #[test]
    fn permutation_when_vector_has_duplicates_returns_index_based_duplicate_permutations() {
        // Arrange
        let input_values = vec![1, 1, 2];
        let permutation_size = 2usize;
        let expected_permutations = vec![
            vec![1, 1],
            vec![1, 2],
            vec![1, 1],
            vec![1, 2],
            vec![2, 1],
            vec![2, 1],
        ];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn permutation_when_type_is_string_generates_expected_permutations() {
        // Arrange
        let input_values = vec!["a".to_string(), "b".to_string(), "c".to_string()];
        let permutation_size = 2usize;
        let expected_permutations = vec![
            vec!["a".to_string(), "b".to_string()],
            vec!["a".to_string(), "c".to_string()],
            vec!["b".to_string(), "a".to_string()],
            vec!["b".to_string(), "c".to_string()],
            vec!["c".to_string(), "a".to_string()],
            vec!["c".to_string(), "b".to_string()],
        ];

        // Act
        let actual_permutations = Itertools::permutation(&input_values, permutation_size);

        // Assert
        assert_eq!(expected_permutations, actual_permutations);
    }

    #[test]
    fn next_permutation_when_vector_is_empty_and_k_is_zero_returns_some_empty_vector() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let step_count = 0usize;
        let expected_next = Some(vec![]);

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_is_empty_and_k_is_one_returns_none() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let step_count = 1usize;
        let expected_next: Option<Vec<i32>> = None;

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_has_single_element_and_k_is_zero_returns_same_vector() {
        // Arrange
        let input_values = vec![42];
        let step_count = 0usize;
        let expected_next = Some(vec![42]);

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_has_single_element_and_k_is_one_returns_none() {
        // Arrange
        let input_values = vec![42];
        let step_count = 1usize;
        let expected_next: Option<Vec<i32>> = None;

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_has_multiple_elements_and_k_is_zero_returns_same_vector() {
        // Arrange
        let input_values = vec![2, 1, 3];
        let step_count = 0usize;
        let expected_next = Some(vec![2, 1, 3]);

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_is_ascending_and_k_is_one_returns_next_lexicographical_order()
    {
        // Arrange
        let input_values = vec![1, 2, 3];
        let step_count = 1usize;
        let expected_next = Some(vec![1, 3, 2]);

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_is_descending_and_k_is_one_returns_none() {
        // Arrange
        let input_values = vec![3, 2, 1];
        let step_count = 1usize;
        let expected_next: Option<Vec<i32>> = None;

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_has_duplicates_and_has_next_returns_expected_vector() {
        // Arrange
        let input_values = vec![1, 1, 2];
        let step_count = 1usize;
        let expected_next = Some(vec![1, 2, 1]);

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_vector_has_duplicates_and_no_next_returns_none() {
        // Arrange
        let input_values = vec![2, 1, 1];
        let step_count = 1usize;
        let expected_next: Option<Vec<i32>> = None;

        // Act
        let actual_next = Itertools::next_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_next, actual_next);
    }

    #[test]
    fn next_permutation_when_k_is_x_minus_one_x_x_plus_one_returns_expected_options() {
        // Arrange
        let input_values_for_x_minus_one = vec![1, 2, 3];
        let input_values_for_x = vec![1, 2, 3];
        let input_values_for_x_plus_one = vec![1, 2, 3];
        let x = 5usize;
        let step_count_minus_one = x - 1usize;
        let step_count_equal = x;
        let step_count_plus_one = x + 1usize;
        let expected_next_minus_one = Some(vec![3, 1, 2]);
        let expected_next_equal = Some(vec![3, 2, 1]);
        let expected_next_plus_one: Option<Vec<i32>> = None;

        // Act
        let actual_next_minus_one =
            Itertools::next_permutation(input_values_for_x_minus_one, step_count_minus_one);
        let actual_next_equal = Itertools::next_permutation(input_values_for_x, step_count_equal);
        let actual_next_plus_one =
            Itertools::next_permutation(input_values_for_x_plus_one, step_count_plus_one);

        // Assert
        assert_eq!(expected_next_minus_one, actual_next_minus_one);
        assert_eq!(expected_next_equal, actual_next_equal);
        assert_eq!(expected_next_plus_one, actual_next_plus_one);
    }

    #[test]
    fn prev_permutation_when_vector_is_empty_and_k_is_zero_returns_some_empty_vector() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let step_count = 0usize;
        let expected_prev = Some(vec![]);

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_is_empty_and_k_is_one_returns_none() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let step_count = 1usize;
        let expected_prev: Option<Vec<i32>> = None;

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_has_single_element_and_k_is_zero_returns_same_vector() {
        // Arrange
        let input_values = vec![42];
        let step_count = 0usize;
        let expected_prev = Some(vec![42]);

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_has_single_element_and_k_is_one_returns_none() {
        // Arrange
        let input_values = vec![42];
        let step_count = 1usize;
        let expected_prev: Option<Vec<i32>> = None;

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_has_multiple_elements_and_k_is_zero_returns_same_vector() {
        // Arrange
        let input_values = vec![2, 1, 3];
        let step_count = 0usize;
        let expected_prev = Some(vec![2, 1, 3]);

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_is_descending_and_k_is_one_returns_previous_lexicographical_order()
    {
        // Arrange
        let input_values = vec![3, 2, 1];
        let step_count = 1usize;
        let expected_prev = Some(vec![3, 1, 2]);

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_is_ascending_and_k_is_one_returns_none() {
        // Arrange
        let input_values = vec![1, 2, 3];
        let step_count = 1usize;
        let expected_prev: Option<Vec<i32>> = None;

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_has_duplicates_and_has_prev_returns_expected_vector() {
        // Arrange
        let input_values = vec![2, 1, 1];
        let step_count = 1usize;
        let expected_prev = Some(vec![1, 2, 1]);

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_vector_has_duplicates_and_no_prev_returns_none() {
        // Arrange
        let input_values = vec![1, 1, 2];
        let step_count = 1usize;
        let expected_prev: Option<Vec<i32>> = None;

        // Act
        let actual_prev = Itertools::prev_permutation(input_values, step_count);

        // Assert
        assert_eq!(expected_prev, actual_prev);
    }

    #[test]
    fn prev_permutation_when_k_is_x_minus_one_x_x_plus_one_returns_expected_options() {
        // Arrange
        let input_values_for_x_minus_one = vec![3, 2, 1];
        let input_values_for_x = vec![3, 2, 1];
        let input_values_for_x_plus_one = vec![3, 2, 1];
        let x = 5usize;
        let step_count_minus_one = x - 1usize;
        let step_count_equal = x;
        let step_count_plus_one = x + 1usize;
        let expected_prev_minus_one = Some(vec![1, 3, 2]);
        let expected_prev_equal = Some(vec![1, 2, 3]);
        let expected_prev_plus_one: Option<Vec<i32>> = None;

        // Act
        let actual_prev_minus_one =
            Itertools::prev_permutation(input_values_for_x_minus_one, step_count_minus_one);
        let actual_prev_equal = Itertools::prev_permutation(input_values_for_x, step_count_equal);
        let actual_prev_plus_one =
            Itertools::prev_permutation(input_values_for_x_plus_one, step_count_plus_one);

        // Assert
        assert_eq!(expected_prev_minus_one, actual_prev_minus_one);
        assert_eq!(expected_prev_equal, actual_prev_equal);
        assert_eq!(expected_prev_plus_one, actual_prev_plus_one);
    }
}
