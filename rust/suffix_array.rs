use std::cmp::Ordering;

const THRESHOLD_NAIVE: usize = 10;
const THRESHOLD_DOUBLING: usize = 40;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SuffixArray {
    suffix_start_indices: Vec<usize>,
}

impl SuffixArray {
    pub fn new(text: &str) -> Self {
        let values = text
            .bytes()
            .map(|byte_value| byte_value as usize)
            .collect::<Vec<usize>>();

        Self {
            suffix_start_indices: Self::sa_is(&values, u8::MAX as usize),
        }
    }

    pub fn from_int_slice(values: &[usize], upper: usize) -> Self {
        for &value in values {
            assert!(value <= upper, "all values must be in [0, upper]");
        }

        Self {
            suffix_start_indices: Self::sa_is(values, upper),
        }
    }

    pub fn from_slice<T: Ord>(values: &[T]) -> Self {
        let (compressed_values, upper) = Self::compress_values(values);

        Self {
            suffix_start_indices: Self::sa_is(&compressed_values, upper),
        }
    }

    pub fn as_slice(&self) -> &[usize] {
        &self.suffix_start_indices
    }

    pub fn into_vec(self) -> Vec<usize> {
        self.suffix_start_indices
    }

    pub fn len(&self) -> usize {
        self.suffix_start_indices.len()
    }

    pub fn is_empty(&self) -> bool {
        self.suffix_start_indices.is_empty()
    }

    fn sa_naive(values: &[usize]) -> Vec<usize> {
        let length = values.len();
        let mut suffix_array = (0..length).collect::<Vec<usize>>();

        suffix_array.sort_by(|left_start, right_start| {
            if left_start == right_start {
                return Ordering::Equal;
            }

            let mut left_index = *left_start;
            let mut right_index = *right_start;

            while left_index < length && right_index < length {
                if values[left_index] < values[right_index] {
                    return Ordering::Less;
                }
                if values[right_index] < values[left_index] {
                    return Ordering::Greater;
                }
                left_index += 1;
                right_index += 1;
            }

            if left_index == length {
                Ordering::Less
            } else {
                Ordering::Greater
            }
        });

        suffix_array
    }

    fn compare_doubling_suffix(
        left_index: usize,
        right_index: usize,
        step: usize,
        ranks: &[usize],
    ) -> Ordering {
        if ranks[left_index] < ranks[right_index] {
            return Ordering::Less;
        }
        if ranks[right_index] < ranks[left_index] {
            return Ordering::Greater;
        }

        let left_next_rank = if left_index + step < ranks.len() {
            Some(ranks[left_index + step])
        } else {
            None
        };
        let right_next_rank = if right_index + step < ranks.len() {
            Some(ranks[right_index + step])
        } else {
            None
        };

        left_next_rank.cmp(&right_next_rank)
    }

    fn sa_doubling(values: &[usize]) -> Vec<usize> {
        let length = values.len();
        let mut suffix_array = (0..length).collect::<Vec<usize>>();
        let mut ranks = values.to_vec();
        let mut next_ranks = vec![0usize; length];

        let mut step = 1usize;
        while step < length {
            suffix_array
                .sort_by(|left, right| Self::compare_doubling_suffix(*left, *right, step, &ranks));

            next_ranks[suffix_array[0]] = 0;

            for suffix_order_index in 1..length {
                let previous_suffix_index = suffix_array[suffix_order_index - 1];
                let current_suffix_index = suffix_array[suffix_order_index];
                let previous_rank = next_ranks[previous_suffix_index];
                let order = Self::compare_doubling_suffix(
                    previous_suffix_index,
                    current_suffix_index,
                    step,
                    &ranks,
                );
                next_ranks[current_suffix_index] = if order == Ordering::Less {
                    previous_rank + 1
                } else {
                    previous_rank
                };
            }

            std::mem::swap(&mut ranks, &mut next_ranks);
            step *= 2;
        }

        suffix_array
    }

    fn build_ls_types(values: &[usize]) -> Vec<bool> {
        let length = values.len();
        let mut is_s_type = vec![false; length];

        for index in (0..length - 1).rev() {
            is_s_type[index] = if values[index] == values[index + 1] {
                is_s_type[index + 1]
            } else {
                values[index] < values[index + 1]
            };
        }

        is_s_type
    }

    fn build_bucket_sums(
        values: &[usize],
        upper: usize,
        is_s_type: &[bool],
    ) -> (Vec<usize>, Vec<usize>) {
        let mut sum_l = vec![0usize; upper + 1];
        let mut sum_s = vec![0usize; upper + 1];

        for index in 0..values.len() {
            if !is_s_type[index] {
                sum_s[values[index]] += 1;
            } else {
                sum_l[values[index] + 1] += 1;
            }
        }

        for value in 0..=upper {
            sum_s[value] += sum_l[value];
            if value < upper {
                sum_l[value + 1] += sum_s[value];
            }
        }

        (sum_l, sum_s)
    }

    fn induce_suffix_array(
        values: &[usize],
        upper: usize,
        is_s_type: &[bool],
        sum_l: &[usize],
        sum_s: &[usize],
        lms_positions: &[usize],
        suffix_array: &mut [usize],
    ) {
        const INVALID_INDEX: usize = usize::MAX;
        let length = values.len();

        suffix_array.fill(INVALID_INDEX);
        let mut buffer = sum_s.to_vec();

        for &lms_position in lms_positions {
            if lms_position == length {
                continue;
            }
            suffix_array[buffer[values[lms_position]]] = lms_position;
            buffer[values[lms_position]] += 1;
        }

        buffer.clone_from_slice(sum_l);
        let last_index = length - 1;
        suffix_array[buffer[values[last_index]]] = last_index;
        buffer[values[last_index]] += 1;

        for suffix_order_index in 0..length {
            let suffix_start_index = suffix_array[suffix_order_index];
            if suffix_start_index == INVALID_INDEX || suffix_start_index == 0 {
                continue;
            }

            let previous_suffix_start_index = suffix_start_index - 1;
            if !is_s_type[previous_suffix_start_index] {
                suffix_array[buffer[values[previous_suffix_start_index]]] =
                    previous_suffix_start_index;
                buffer[values[previous_suffix_start_index]] += 1;
            }
        }

        buffer.clone_from_slice(sum_l);
        for suffix_order_index in (0..length).rev() {
            let suffix_start_index = suffix_array[suffix_order_index];
            if suffix_start_index == INVALID_INDEX || suffix_start_index == 0 {
                continue;
            }

            let previous_suffix_start_index = suffix_start_index - 1;
            if is_s_type[previous_suffix_start_index] {
                let bucket_index = values[previous_suffix_start_index] + 1;
                debug_assert!(
                    bucket_index <= upper,
                    "S-type symbol must be smaller than upper"
                );
                buffer[bucket_index] -= 1;
                suffix_array[buffer[bucket_index]] = previous_suffix_start_index;
            }
        }
    }

    fn sa_is(values: &[usize], upper: usize) -> Vec<usize> {
        const INVALID_INDEX: usize = usize::MAX;
        let length = values.len();

        if length == 0 {
            return vec![];
        }
        if length == 1 {
            return vec![0];
        }
        if length == 2 {
            if values[0] < values[1] {
                return vec![0, 1];
            }
            return vec![1, 0];
        }
        if length < THRESHOLD_NAIVE {
            return Self::sa_naive(values);
        }
        if length < THRESHOLD_DOUBLING {
            return Self::sa_doubling(values);
        }

        let is_s_type = Self::build_ls_types(values);
        let (sum_l, sum_s) = Self::build_bucket_sums(values, upper, &is_s_type);
        let mut suffix_array = vec![INVALID_INDEX; length];

        let mut lms_index_map = vec![INVALID_INDEX; length + 1];
        let mut lms_count = 0usize;
        for suffix_start_index in 1..length {
            if !is_s_type[suffix_start_index - 1] && is_s_type[suffix_start_index] {
                lms_index_map[suffix_start_index] = lms_count;
                lms_count += 1;
            }
        }

        let mut lms_positions = Vec::with_capacity(lms_count);
        for suffix_start_index in 1..length {
            if !is_s_type[suffix_start_index - 1] && is_s_type[suffix_start_index] {
                lms_positions.push(suffix_start_index);
            }
        }

        Self::induce_suffix_array(
            values,
            upper,
            &is_s_type,
            &sum_l,
            &sum_s,
            &lms_positions,
            &mut suffix_array,
        );

        if lms_count != 0 {
            let mut sorted_lms_positions = Vec::with_capacity(lms_count);
            for &suffix_start_index in suffix_array.iter() {
                if suffix_start_index != INVALID_INDEX
                    && lms_index_map[suffix_start_index] != INVALID_INDEX
                {
                    sorted_lms_positions.push(suffix_start_index);
                }
            }

            let mut recursive_values = vec![0usize; lms_count];
            let mut recursive_upper = 0usize;
            recursive_values[lms_index_map[sorted_lms_positions[0]]] = 0;

            for sorted_index in 1..lms_count {
                let mut left_lms_start = sorted_lms_positions[sorted_index - 1];
                let mut right_lms_start = sorted_lms_positions[sorted_index];

                let left_lms_order = lms_index_map[left_lms_start];
                let right_lms_order = lms_index_map[right_lms_start];

                let left_lms_end = if left_lms_order + 1 < lms_count {
                    lms_positions[left_lms_order + 1]
                } else {
                    length
                };
                let right_lms_end = if right_lms_order + 1 < lms_count {
                    lms_positions[right_lms_order + 1]
                } else {
                    length
                };

                let mut is_same_substring = true;
                if left_lms_end - left_lms_start != right_lms_end - right_lms_start {
                    is_same_substring = false;
                } else {
                    while left_lms_start < left_lms_end
                        && values[left_lms_start] == values[right_lms_start]
                    {
                        left_lms_start += 1;
                        right_lms_start += 1;
                    }
                    if left_lms_start == length || values[left_lms_start] != values[right_lms_start]
                    {
                        is_same_substring = false;
                    }
                }

                if !is_same_substring {
                    recursive_upper += 1;
                }
                recursive_values[right_lms_order] = recursive_upper;
            }

            let recursive_suffix_array = Self::sa_is(&recursive_values, recursive_upper);
            for sorted_index in 0..lms_count {
                sorted_lms_positions[sorted_index] =
                    lms_positions[recursive_suffix_array[sorted_index]];
            }

            Self::induce_suffix_array(
                values,
                upper,
                &is_s_type,
                &sum_l,
                &sum_s,
                &sorted_lms_positions,
                &mut suffix_array,
            );
        }

        suffix_array
    }

    fn compress_values<T: Ord>(values: &[T]) -> (Vec<usize>, usize) {
        let length = values.len();
        if length == 0 {
            return (vec![], 0);
        }

        let mut sorted_indices = (0..length).collect::<Vec<usize>>();
        sorted_indices
            .sort_by(|left_index, right_index| values[*left_index].cmp(&values[*right_index]));

        let mut compressed_values = vec![0usize; length];
        let mut now_rank = 0usize;

        compressed_values[sorted_indices[0]] = 0;
        for sorted_order_index in 1..length {
            let previous_index = sorted_indices[sorted_order_index - 1];
            let current_index = sorted_indices[sorted_order_index];
            if values[previous_index] != values[current_index] {
                now_rank += 1;
            }
            compressed_values[current_index] = now_rank;
        }

        (compressed_values, now_rank)
    }
}

impl AsRef<[usize]> for SuffixArray {
    fn as_ref(&self) -> &[usize] {
        self.as_slice()
    }
}

#[cfg(test)]
mod tests {
    use super::{SuffixArray, THRESHOLD_DOUBLING, THRESHOLD_NAIVE};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn naive_suffix_array<T: Ord>(values: &[T]) -> Vec<usize> {
        let length = values.len();
        let mut suffix_array = (0..length).collect::<Vec<usize>>();
        suffix_array
            .sort_by(|left_start, right_start| values[*left_start..].cmp(&values[*right_start..]));
        suffix_array
    }

    fn make_periodic_values(length: usize, modulo: usize) -> Vec<usize> {
        (0..length)
            .map(|index| (index * 17usize + 3usize) % modulo)
            .collect::<Vec<usize>>()
    }

    fn build_single_lms_values(length: usize) -> Vec<usize> {
        assert!(2usize <= length);

        let mut values = vec![2usize; length];
        values[1] = 1usize;
        values
    }

    fn build_pseudorandom_values(length: usize, upper: usize, seed: u64) -> Vec<usize> {
        let mut state = seed;
        let value_modulo = upper + 1usize;
        let mut values = Vec::with_capacity(length);

        for _ in 0..length {
            state = state
                .wrapping_mul(6364136223846793005u64)
                .wrapping_add(1442695040888963407u64);
            values.push((state as usize) % value_modulo);
        }

        values
    }

    fn assert_suffix_array_is_permutation(input_length: usize, suffix_array: &[usize]) {
        assert_eq!(suffix_array.len(), input_length);

        let mut seen_suffix_start_indices = vec![false; input_length];
        for &suffix_start_index in suffix_array.iter() {
            assert!(suffix_start_index < input_length);
            assert!(!seen_suffix_start_indices[suffix_start_index]);
            seen_suffix_start_indices[suffix_start_index] = true;
        }

        assert!(seen_suffix_start_indices.into_iter().all(|is_seen| is_seen));
    }

    #[test]
    fn new_when_empty_string_creates_empty_suffix_array() {
        // Arrange
        let input_text = "";
        let expected_suffix_array: Vec<usize> = vec![];

        let expected_length_right = 0usize;

        // Act
        let suffix_array = SuffixArray::new(input_text);
        let actual_suffix_array = suffix_array.as_slice();
        let actual_length = suffix_array.len();
        let actual_is_empty = suffix_array.is_empty();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array.as_slice());
        assert_eq!(actual_length, expected_length_right);
        assert!(actual_is_empty);
    }

    #[test]
    fn new_when_single_character_creates_suffix_array_containing_zero() {
        // Arrange
        let input_text = "a";
        let expected_suffix_array = vec![0usize];

        let expected_length_right = 1usize;

        // Act
        let suffix_array = SuffixArray::new(input_text);
        let actual_suffix_array: &[usize] = suffix_array.as_ref();
        let actual_length = suffix_array.len();
        let actual_is_empty = suffix_array.is_empty();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array.as_slice());
        assert_eq!(actual_length, expected_length_right);
        assert!(!actual_is_empty);
    }

    #[test]
    fn new_when_missisippi_returns_acl_example_order() {
        // Arrange
        let input_text = "missisippi";
        let expected_suffix_array = vec![9usize, 6, 4, 1, 0, 8, 7, 5, 3, 2];

        // Act
        let actual_suffix_array = SuffixArray::new(input_text).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn new_when_two_characters_ascending_returns_zero_one() {
        // Arrange
        let input_text = "ab";
        let expected_suffix_array = vec![0usize, 1usize];

        // Act
        let actual_suffix_array = SuffixArray::new(input_text).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn new_when_two_characters_descending_returns_one_zero() {
        // Arrange
        let input_text = "ba";
        let expected_suffix_array = vec![1usize, 0usize];

        // Act
        let actual_suffix_array = SuffixArray::new(input_text).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn new_when_two_identical_characters_returns_shorter_suffix_first() {
        // Arrange
        let input_text = "aa";
        let expected_suffix_array = vec![1usize, 0usize];

        // Act
        let actual_suffix_array = SuffixArray::new(input_text).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn new_when_utf8_multibyte_text_matches_naive_byte_suffix_array() {
        // Arrange
        let input_text = "あいあ";
        let expected_suffix_array = naive_suffix_array(input_text.as_bytes());

        // Act
        let actual_suffix_array = SuffixArray::new(input_text).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_empty_set_returns_empty_suffix_array() {
        // Arrange
        let input_values: Vec<usize> = vec![];
        let upper_value = 0usize;
        let expected_suffix_array: Vec<usize> = vec![];

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_singleton_returns_zero() {
        // Arrange
        let input_values = vec![7usize];
        let upper_value = 10usize;
        let expected_suffix_array = vec![0usize];

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_multiple_elements_matches_naive_result() {
        // Arrange
        let input_values = vec![1usize, 0, 2, 1, 0];
        let upper_value = 2usize;
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_values_include_lower_and_upper_bounds_matches_naive_result() {
        // Arrange
        let upper_value = 5usize;
        let input_values = vec![0usize, upper_value, 0usize, upper_value, 1usize];
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_value_exceeds_upper_panics() {
        // Arrange
        let input_values = vec![0usize, 3usize, 1usize];
        let upper_value = 2usize;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = SuffixArray::from_int_slice(&input_values, upper_value).into_vec();
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn from_int_slice_when_length_is_threshold_naive_x_minus_1_matches_naive_result() {
        // Arrange
        let threshold_length = THRESHOLD_NAIVE;
        let input_length = threshold_length - 1usize;
        let upper_value = 6usize;
        let input_values = make_periodic_values(input_length, upper_value + 1usize);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_length_is_threshold_naive_x_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_NAIVE;
        let upper_value = 6usize;
        let input_values = make_periodic_values(input_length, upper_value + 1usize);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_length_is_threshold_naive_x_plus_1_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_NAIVE + 1usize;
        let upper_value = 6usize;
        let input_values = make_periodic_values(input_length, upper_value + 1usize);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_length_is_threshold_doubling_x_minus_1_matches_naive_result() {
        // Arrange
        let threshold_length = THRESHOLD_DOUBLING;
        let input_length = threshold_length - 1usize;
        let upper_value = 7usize;
        let input_values = make_periodic_values(input_length, upper_value + 1usize);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_length_is_threshold_doubling_x_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_DOUBLING;
        let upper_value = 7usize;
        let input_values = make_periodic_values(input_length, upper_value + 1usize);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_length_is_threshold_doubling_x_plus_1_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_DOUBLING + 1usize;
        let upper_value = 7usize;
        let input_values = make_periodic_values(input_length, upper_value + 1usize);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_upper_is_zero_and_all_values_equal_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_DOUBLING + 1usize;
        let upper_value = 0usize;
        let input_values = vec![0usize; input_length];
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_monotonic_increasing_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_DOUBLING + 1usize;
        let upper_value = input_length - 1usize;
        let input_values = (0..input_length).collect::<Vec<usize>>();
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_monotonic_decreasing_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_DOUBLING + 1usize;
        let upper_value = input_length - 1usize;
        let input_values = (0..input_length).rev().collect::<Vec<usize>>();
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_single_lms_position_matches_naive_result() {
        // Arrange
        let input_length = THRESHOLD_DOUBLING + 1usize;
        let upper_value = 2usize;
        let input_values = build_single_lms_values(input_length);
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_upper_is_sparse_large_value_matches_naive_result() {
        // Arrange
        let input_values = vec![0usize, 2usize, 5usize, 0usize, 5usize, 2usize, 0usize];
        let upper_value = 100usize;
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_empty_input_and_upper_100_returns_empty_vector() {
        // Arrange
        let input_values: Vec<usize> = vec![];
        let upper_value = 100usize;
        let expected_suffix_array: Vec<usize> = vec![];

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_two_elements_ascending_returns_zero_one() {
        // Arrange
        let input_values = vec![0usize, 1usize];
        let upper_value = 1usize;
        let expected_suffix_array = vec![0usize, 1usize];

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_two_elements_descending_returns_one_zero() {
        // Arrange
        let input_values = vec![1usize, 0usize];
        let upper_value = 1usize;
        let expected_suffix_array = vec![1usize, 0usize];

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_when_two_equal_elements_returns_one_zero() {
        // Arrange
        let input_values = vec![1usize, 1usize];
        let upper_value = 1usize;
        let expected_suffix_array = vec![1usize, 0usize];

        // Act
        let actual_suffix_array =
            SuffixArray::from_int_slice(&input_values, upper_value).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_int_slice_result_length_matches_input_length_on_boundary_and_extreme_cases() {
        // Arrange
        let input_lengths = vec![
            0usize,
            1usize,
            2usize,
            THRESHOLD_NAIVE - 1usize,
            THRESHOLD_NAIVE,
            THRESHOLD_NAIVE + 1usize,
            THRESHOLD_DOUBLING - 1usize,
            THRESHOLD_DOUBLING,
            THRESHOLD_DOUBLING + 1usize,
        ];
        let upper_value = 4usize;

        // Act
        let mut actual_suffix_array_lengths = Vec::with_capacity(input_lengths.len());
        for &input_length in input_lengths.iter() {
            let input_values = if input_length == 0usize {
                vec![]
            } else {
                make_periodic_values(input_length, upper_value + 1usize)
            };
            let actual_suffix_array =
                SuffixArray::from_int_slice(&input_values, upper_value).into_vec();
            actual_suffix_array_lengths.push(actual_suffix_array.len());
        }

        // Assert
        assert_eq!(actual_suffix_array_lengths, input_lengths);
    }

    #[test]
    fn from_int_slice_when_multiple_pseudorandom_cases_matches_naive_result() {
        // Arrange
        let upper_value = 11usize;
        let case_count = 50usize;
        let cases = (0..case_count)
            .map(|case_index| {
                let input_length = (case_index * 7usize) % (THRESHOLD_DOUBLING + 17usize);
                let input_values =
                    build_pseudorandom_values(input_length, upper_value, case_index as u64 + 1u64);
                let expected_suffix_array = naive_suffix_array(&input_values);
                (input_values, expected_suffix_array)
            })
            .collect::<Vec<_>>();
        let expected_arrays = cases
            .iter()
            .map(|(_, expected)| expected.clone())
            .collect::<Vec<_>>();

        // Act
        let actual_arrays = cases
            .iter()
            .map(|(input, _)| SuffixArray::from_int_slice(input, upper_value).into_vec())
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_arrays, expected_arrays);
    }

    #[test]
    fn from_slice_when_empty_set_returns_empty_suffix_array() {
        // Arrange
        let input_values: Vec<i64> = vec![];
        let expected_suffix_array: Vec<usize> = vec![];

        // Act
        let actual_suffix_array = SuffixArray::from_slice(&input_values).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_slice_when_singleton_returns_zero() {
        // Arrange
        let input_values = vec![42_i64];
        let expected_suffix_array = vec![0usize];

        // Act
        let actual_suffix_array = SuffixArray::from_slice(&input_values).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_slice_when_multiple_elements_matches_naive_result() {
        // Arrange
        let input_values = vec![-10_i64, 100_i64, -10_i64, 50_i64];
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array = SuffixArray::from_slice(&input_values).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_slice_when_all_elements_equal_returns_shorter_suffix_first() {
        // Arrange
        let input_values = vec![7i64, 7i64, 7i64, 7i64];
        let expected_suffix_array = vec![3usize, 2usize, 1usize, 0usize];

        // Act
        let actual_suffix_array = SuffixArray::from_slice(&input_values).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_slice_when_input_is_vec_string_matches_naive_result() {
        // Arrange
        let input_values = vec![
            String::from("ba"),
            String::from("aa"),
            String::from("ba"),
            String::from("ab"),
        ];
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array = SuffixArray::from_slice(&input_values).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn from_slice_when_values_are_sparse_large_range_matches_naive_result() {
        // Arrange
        let input_values = vec![-1_000_000_i64, 5_i64, -1_000_000_i64, 9_999_999_i64];
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array = SuffixArray::from_slice(&input_values).into_vec();

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn sa_naive_when_input_has_duplicates_returns_lexicographic_suffix_order() {
        // Arrange
        let input_values = vec![2usize, 2usize, 1usize, 2usize];
        let expected_suffix_array = naive_suffix_array(&input_values);

        // Act
        let actual_suffix_array = SuffixArray::sa_naive(&input_values);

        // Assert
        assert_eq!(actual_suffix_array, expected_suffix_array);
    }

    #[test]
    fn suffix_array_all_public_apis_return_permutation_indices() {
        // Arrange
        let input_text = "abracadabra";
        let int_values = vec![2usize, 0usize, 1usize, 2usize, 0usize, 1usize, 0usize];
        let int_upper = 2usize;
        let generic_values = vec![3i64, 1i64, 3i64, 0i64, 2i64];

        // Act
        let actual_text_suffix_array = SuffixArray::new(input_text).into_vec();
        let actual_int_suffix_array =
            SuffixArray::from_int_slice(&int_values, int_upper).into_vec();
        let actual_generic_suffix_array = SuffixArray::from_slice(&generic_values).into_vec();

        // Assert
        assert_suffix_array_is_permutation(input_text.len(), &actual_text_suffix_array);
        assert_suffix_array_is_permutation(int_values.len(), &actual_int_suffix_array);
        assert_suffix_array_is_permutation(generic_values.len(), &actual_generic_suffix_array);
    }
}
