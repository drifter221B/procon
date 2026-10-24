#[derive(Clone, Debug, Eq, PartialEq)]
pub struct LcpArray {
    lcp_values: Vec<usize>,
}

impl LcpArray {
    pub fn new(source_text: &str, suffix_array: &[usize]) -> Self {
        Self::from_slice(source_text.as_bytes(), suffix_array)
    }

    pub fn from_slice<T: Eq>(source_values: &[T], suffix_array: &[usize]) -> Self {
        Self {
            lcp_values: Self::calculate_lcp_array(source_values, suffix_array),
        }
    }

    pub fn as_slice(&self) -> &[usize] {
        &self.lcp_values
    }

    pub fn into_vec(self) -> Vec<usize> {
        self.lcp_values
    }

    pub fn len(&self) -> usize {
        self.lcp_values.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lcp_values.is_empty()
    }

    fn calculate_lcp_array<T: Eq>(source_values: &[T], suffix_array: &[usize]) -> Vec<usize> {
        let text_length = source_values.len();
        assert_eq!(suffix_array.len(), text_length);

        if text_length < 2 {
            return vec![];
        }

        let rank_by_start_index = Self::build_rank_by_start_index(text_length, suffix_array);
        Self::build_lcp_values(source_values, suffix_array, &rank_by_start_index)
    }

    fn build_rank_by_start_index(text_length: usize, suffix_array: &[usize]) -> Vec<usize> {
        let mut rank_by_start_index = vec![0usize; text_length];

        for (suffix_rank, &suffix_start_index) in suffix_array.iter().enumerate() {
            assert!(suffix_start_index < text_length);
            rank_by_start_index[suffix_start_index] = suffix_rank;
        }

        rank_by_start_index
    }

    fn build_lcp_values<T: Eq>(
        source_values: &[T],
        suffix_array: &[usize],
        rank_by_start_index: &[usize],
    ) -> Vec<usize> {
        let text_length = source_values.len();
        let mut lcp_values = vec![0usize; text_length - 1];
        let mut current_lcp_length = 0usize;

        for left_suffix_start_index in 0..text_length {
            if 0 < current_lcp_length {
                current_lcp_length -= 1;
            }

            let left_suffix_rank = rank_by_start_index[left_suffix_start_index];
            if left_suffix_rank == 0 {
                continue;
            }

            let right_suffix_start_index = suffix_array[left_suffix_rank - 1];

            while left_suffix_start_index + current_lcp_length < text_length
                && right_suffix_start_index + current_lcp_length < text_length
                && source_values[left_suffix_start_index + current_lcp_length]
                    == source_values[right_suffix_start_index + current_lcp_length]
            {
                current_lcp_length += 1;
            }

            lcp_values[left_suffix_rank - 1] = current_lcp_length;
        }

        lcp_values
    }
}

impl AsRef<[usize]> for LcpArray {
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::LcpArray;

    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    struct EqOnlyValue {
        value: i32,
    }

    fn build_naive_suffix_array(source_text: &str) -> Vec<usize> {
        build_naive_suffix_array_from_slice(source_text.as_bytes())
    }

    fn build_naive_suffix_array_from_slice<T: Ord>(source_values: &[T]) -> Vec<usize> {
        let text_length = source_values.len();
        let mut suffix_start_indices: Vec<usize> = (0..text_length).collect();
        suffix_start_indices.sort_by(|&left_suffix_start_index, &right_suffix_start_index| {
            source_values[left_suffix_start_index..].cmp(&source_values[right_suffix_start_index..])
        });
        suffix_start_indices
    }

    fn build_naive_lcp_array(source_text: &str, suffix_array: &[usize]) -> Vec<usize> {
        build_naive_lcp_array_from_slice(source_text.as_bytes(), suffix_array)
    }

    fn build_naive_lcp_array_from_slice<T: Eq>(
        source_values: &[T],
        suffix_array: &[usize],
    ) -> Vec<usize> {
        if suffix_array.len() < 2 {
            return vec![];
        }

        let text_length = source_values.len();
        let mut lcp_values = vec![0usize; suffix_array.len() - 1];

        for left_rank in 0..suffix_array.len() - 1 {
            let left_suffix_start_index = suffix_array[left_rank];
            let right_suffix_start_index = suffix_array[left_rank + 1];
            let mut common_prefix_length = 0usize;

            while left_suffix_start_index + common_prefix_length < text_length
                && right_suffix_start_index + common_prefix_length < text_length
                && source_values[left_suffix_start_index + common_prefix_length]
                    == source_values[right_suffix_start_index + common_prefix_length]
            {
                common_prefix_length += 1;
            }

            lcp_values[left_rank] = common_prefix_length;
        }

        lcp_values
    }

    #[test]
    fn lcp_array_with_empty_text_and_empty_suffix_array_returns_empty_vector() {
        // Arrange
        let source_text = "";
        let suffix_array: Vec<usize> = vec![];
        let expected_lcp_values: Vec<usize> = vec![];

        // Act
        let actual_lcp_values = LcpArray::new(source_text, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_with_single_character_text_returns_empty_vector() {
        // Arrange
        let source_text = "a";
        let suffix_array = vec![0usize];
        let expected_lcp_values: Vec<usize> = vec![];

        // Act
        let actual_lcp_values = LcpArray::new(source_text, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_with_banana_suffix_array_returns_expected_values() {
        // Arrange
        let source_text = "banana";
        let suffix_array = vec![5usize, 3usize, 1usize, 0usize, 4usize, 2usize];
        let expected_lcp_values = vec![1usize, 3usize, 0usize, 0usize, 2usize];

        // Act
        let actual_lcp_array = LcpArray::new(source_text, &suffix_array);
        let actual_lcp_values = actual_lcp_array.as_slice();
        let actual_len = actual_lcp_array.len();
        let actual_is_empty = actual_lcp_array.is_empty();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
        assert_eq!(actual_len, expected_lcp_values.len());
        assert!(!actual_is_empty);
    }

    #[test]
    fn lcp_array_for_all_same_character_lengths_x_minus_1_x_x_plus_1_returns_increasing_values() {
        // Arrange
        let center_length = 3usize;
        let left_length = center_length - 1;
        let right_length = center_length + 1;

        let left_text = "a".repeat(left_length);
        let center_text = "a".repeat(center_length);
        let right_text = "a".repeat(right_length);

        let left_suffix_array = build_naive_suffix_array(&left_text);
        let center_suffix_array = build_naive_suffix_array(&center_text);
        let right_suffix_array = build_naive_suffix_array(&right_text);

        let expected_left_lcp_values = vec![1usize];
        let expected_center_lcp_values = vec![1usize, 2usize];
        let expected_right_lcp_values = vec![1usize, 2usize, 3usize];

        // Act
        let actual_left_lcp_values = LcpArray::new(&left_text, &left_suffix_array).into_vec();
        let actual_center_lcp_values = LcpArray::new(&center_text, &center_suffix_array).into_vec();
        let actual_right_lcp_values = LcpArray::new(&right_text, &right_suffix_array).into_vec();

        // Assert
        assert_eq!(actual_left_lcp_values, expected_left_lcp_values);
        assert_eq!(actual_center_lcp_values, expected_center_lcp_values);
        assert_eq!(actual_right_lcp_values, expected_right_lcp_values);
    }

    #[test]
    fn lcp_array_matches_naive_results_for_empty_single_and_multiple_texts() {
        // Arrange
        let sample_texts = ["", "a", "ab", "aba", "mississippi"];
        let suffix_arrays: Vec<_> = sample_texts
            .iter()
            .map(|text| build_naive_suffix_array(text))
            .collect();
        let expected_values: Vec<_> = sample_texts
            .iter()
            .zip(&suffix_arrays)
            .map(|(text, suffix_array)| build_naive_lcp_array(text, suffix_array))
            .collect();

        // Act
        let actual_values: Vec<_> = sample_texts
            .iter()
            .zip(&suffix_arrays)
            .map(|(text, suffix_array)| LcpArray::new(text, suffix_array).into_vec())
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn lcp_array_with_utf8_multibyte_text_matches_naive_results() {
        // Arrange
        let source_text = "あいうあい";
        let suffix_array = build_naive_suffix_array(source_text);
        let expected_lcp_values = build_naive_lcp_array(source_text, &suffix_array);

        // Act
        let actual_lcp_values = LcpArray::new(source_text, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_with_unsorted_but_valid_suffix_array_returns_current_behavior_values() {
        // Arrange
        let source_text = "banana";
        let suffix_array = vec![0usize, 1usize, 2usize, 3usize, 4usize, 5usize];
        let expected_lcp_values = vec![0usize, 0usize, 0usize, 0usize, 0usize];

        // Act
        let actual_lcp_values = LcpArray::new(source_text, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_panics_when_suffix_array_length_is_different_from_text_length() {
        // Arrange
        let source_text = "abc";
        let suffix_array = vec![0usize, 1usize];

        // Act
        let actual_result = std::panic::catch_unwind(|| LcpArray::new(source_text, &suffix_array));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_array_panics_when_suffix_array_contains_out_of_range_index() {
        // Arrange
        let source_text = "abc";
        let suffix_array = vec![0usize, 1usize, 3usize];

        // Act
        let actual_result = std::panic::catch_unwind(|| LcpArray::new(source_text, &suffix_array));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_array_from_slice_with_integer_values_matches_naive_results() {
        // Arrange
        let source_values = vec![1i32, 2i32, 1i32, 2i32, 1i32];
        let suffix_array = build_naive_suffix_array_from_slice(&source_values);
        let expected_lcp_values = build_naive_lcp_array_from_slice(&source_values, &suffix_array);

        // Act
        let actual_lcp_values = LcpArray::from_slice(&source_values, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_from_slice_with_empty_values_and_empty_suffix_array_returns_empty_vector() {
        // Arrange
        let source_values: Vec<i32> = vec![];
        let suffix_array: Vec<usize> = vec![];
        let expected_lcp_values: Vec<usize> = vec![];

        // Act
        let actual_lcp_array = LcpArray::from_slice(&source_values, &suffix_array);
        let actual_lcp_values = actual_lcp_array.as_slice();
        let actual_is_empty = actual_lcp_array.is_empty();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
        assert!(actual_is_empty);
    }

    #[test]
    fn lcp_array_from_slice_with_single_value_and_single_suffix_array_returns_empty_vector() {
        // Arrange
        let source_values = vec![42i32];
        let suffix_array = vec![0usize];
        let expected_lcp_values: Vec<usize> = vec![];

        // Act
        let actual_lcp_values = LcpArray::from_slice(&source_values, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_from_slice_for_all_same_value_lengths_x_minus_1_x_x_plus_1_returns_increasing_values(
    ) {
        // Arrange
        let center_length = 3usize;
        let left_length = center_length - 1;
        let right_length = center_length + 1;

        let left_values = vec![7i32; left_length];
        let center_values = vec![7i32; center_length];
        let right_values = vec![7i32; right_length];

        let left_suffix_array = build_naive_suffix_array_from_slice(&left_values);
        let center_suffix_array = build_naive_suffix_array_from_slice(&center_values);
        let right_suffix_array = build_naive_suffix_array_from_slice(&right_values);

        let expected_left_lcp_values = vec![1usize];
        let expected_center_lcp_values = vec![1usize, 2usize];
        let expected_right_lcp_values = vec![1usize, 2usize, 3usize];

        // Act
        let actual_left_lcp_values =
            LcpArray::from_slice(&left_values, &left_suffix_array).into_vec();
        let actual_center_lcp_values =
            LcpArray::from_slice(&center_values, &center_suffix_array).into_vec();
        let actual_right_lcp_values =
            LcpArray::from_slice(&right_values, &right_suffix_array).into_vec();

        // Assert
        assert_eq!(actual_left_lcp_values, expected_left_lcp_values);
        assert_eq!(actual_center_lcp_values, expected_center_lcp_values);
        assert_eq!(actual_right_lcp_values, expected_right_lcp_values);
    }

    #[test]
    fn lcp_array_from_slice_with_eq_only_values_and_manual_suffix_array_returns_expected_values() {
        // Arrange
        let source_values = vec![
            EqOnlyValue { value: 1 },
            EqOnlyValue { value: 2 },
            EqOnlyValue { value: 1 },
            EqOnlyValue { value: 2 },
        ];
        let suffix_array = vec![2usize, 0usize, 3usize, 1usize];
        let expected_lcp_values = vec![2usize, 0usize, 1usize];

        // Act
        let actual_lcp_values = LcpArray::from_slice(&source_values, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_from_slice_with_min_and_max_values_matches_naive_results() {
        // Arrange
        let source_values = vec![i32::MIN, 0i32, i32::MAX, 0i32];
        let suffix_array = build_naive_suffix_array_from_slice(&source_values);
        let expected_lcp_values = build_naive_lcp_array_from_slice(&source_values, &suffix_array);

        // Act
        let actual_lcp_values = LcpArray::from_slice(&source_values, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_from_slice_panics_when_suffix_array_length_is_different_from_values_length() {
        // Arrange
        let source_values = vec![1i32, 2i32, 3i32];
        let suffix_array = vec![0usize, 1usize];

        // Act
        let actual_result =
            std::panic::catch_unwind(|| LcpArray::from_slice(&source_values, &suffix_array));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_array_from_slice_panics_when_suffix_array_contains_out_of_range_index() {
        // Arrange
        let source_values = vec![1i32, 2i32, 3i32];
        let suffix_array = vec![0usize, 1usize, 3usize];

        // Act
        let actual_result =
            std::panic::catch_unwind(|| LcpArray::from_slice(&source_values, &suffix_array));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_array_from_slice_with_duplicate_indices_in_suffix_array_returns_current_behavior_values()
    {
        // Arrange
        let source_values = vec![1i32, 1i32, 1i32, 1i32];
        let suffix_array = vec![0usize, 1usize, 1usize, 3usize];
        let expected_lcp_values = vec![0usize, 3usize, 1usize];

        // Act
        let actual_lcp_values = LcpArray::from_slice(&source_values, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }

    #[test]
    fn lcp_array_from_slice_with_missing_index_in_suffix_array_returns_current_behavior_values() {
        // Arrange
        let source_values = vec![10i32, 20i32, 30i32, 40i32];
        let suffix_array = vec![0usize, 1usize, 2usize, 2usize];
        let expected_lcp_values = vec![0usize, 0usize, 2usize];

        // Act
        let actual_lcp_values = LcpArray::from_slice(&source_values, &suffix_array).into_vec();

        // Assert
        assert_eq!(actual_lcp_values, expected_lcp_values);
    }
}
