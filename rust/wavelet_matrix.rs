pub struct WaveletMatrix {
    matrix: Vec<Vec<bool>>,
    zeros: Vec<Vec<usize>>,
    mid: Vec<usize>,
    maxlog: usize,
    offset: i64,
    upper: i64,
    len: usize,
}

impl WaveletMatrix {
    pub fn new(data: &[i64]) -> Self {
        assert!(!data.is_empty(), "data must not be empty");

        let mut lower = data[0];
        let mut upper = data[0];

        for &v in data.iter().skip(1) {
            if v < lower {
                lower = v;
            }
            if upper < v {
                upper = v;
            }
        }

        Self::new_with_bounds(data, lower, upper)
    }

    pub fn new_with_bounds(data: &[i64], lower: i64, upper: i64) -> Self {
        assert!(!data.is_empty(), "data must not be empty");
        assert!(
            lower <= upper,
            "lower bound must be less than or equal to upper bound"
        );
        assert!(
            data.iter().all(|&value| lower <= value && value <= upper),
            "data values must be within provided bounds"
        );

        let offset = lower.saturating_sub(1);
        let shifted_upper = upper.wrapping_sub(offset) as u64;

        let maxlog = usize::max(1, Self::bit_length(shifted_upper));
        let len = data.len();

        let mut wm = Self {
            matrix: vec![vec![false; len]; maxlog],
            zeros: vec![vec![0; len + 1]; maxlog],
            mid: vec![0; maxlog],
            maxlog,
            offset,
            upper,
            len,
        };

        wm.build(data);
        wm
    }

    pub fn len(&self) -> usize {
        self.len
    }

    pub fn access(&self, pos: usize) -> i64 {
        assert!(pos < self.len, "index out of bounds");

        let mut idx = pos;
        let mut value = 0u64;

        for level in (0..self.maxlog).rev() {
            if self.matrix[level][idx] {
                idx = self.mid[level] + (idx - self.zeros[level][idx]);
                value |= 1u64 << level;
            } else {
                idx = self.zeros[level][idx];
            }
        }

        self.adjust(value)
    }

    pub fn kth_min(&self, l: usize, r: usize, k: usize) -> i64 {
        self.kth(l, r, k)
    }

    pub fn range_min(&self, l: usize, r: usize) -> i64 {
        assert!(l < r, "range must be non-empty");
        self.kth(l, r, 0)
    }

    pub fn range_max(&self, l: usize, r: usize) -> i64 {
        assert!(l < r, "range must be non-empty");
        self.kth(l, r, r - l - 1)
    }

    pub fn upper_bound(&self, l: usize, r: usize, x: i64) -> Option<i64> {
        assert!(l <= r && r <= self.len, "range out of bounds");

        let count = self.frequency_less_than(r, x) - self.frequency_less_than(l, x);

        if count == 0 {
            None
        } else {
            Some(self.kth(l, r, count - 1))
        }
    }

    pub fn lower_bound(&self, l: usize, r: usize, x: i64) -> Option<i64> {
        assert!(l <= r && r <= self.len, "range out of bounds");

        let count = self.frequency_less_than(r, x) - self.frequency_less_than(l, x);

        if r - l <= count {
            None
        } else {
            Some(self.kth(l, r, count))
        }
    }

    pub fn range_frequency(&self, l: usize, r: usize, lower: i64, upper: i64) -> usize {
        assert!(l <= r && r <= self.len, "range out of bounds");
        assert!(lower <= upper, "lower must be <= upper");

        let upper_count = self.frequency_less_than(r, upper) - self.frequency_less_than(l, upper);
        let lower_count = self.frequency_less_than(r, lower) - self.frequency_less_than(l, lower);

        upper_count - lower_count
    }

    fn build(&mut self, data: &[i64]) {
        let mut cur: Vec<u64> = data.iter().map(|&v| self.shift(v)).collect();
        let mut next = vec![0u64; self.len];

        for level in (0..self.maxlog).rev() {
            for i in 0..self.len {
                let bit = ((cur[i] >> level) & 1) == 1;
                self.matrix[level][i] = bit;
                self.zeros[level][i + 1] = self.zeros[level][i] + usize::from(!bit);
            }

            self.mid[level] = self.zeros[level][self.len];

            let mut p0 = 0;
            let mut p1 = self.mid[level];

            for i in 0..self.len {
                if self.matrix[level][i] {
                    next[p1] = cur[i];
                    p1 += 1;
                } else {
                    next[p0] = cur[i];
                    p0 += 1;
                }
            }

            std::mem::swap(&mut cur, &mut next);
        }
    }

    fn kth(&self, l: usize, r: usize, k: usize) -> i64 {
        assert!(l <= r && r <= self.len, "range out of bounds");
        assert!(k < r - l, "k must be within range length");

        let mut left = l;
        let mut right = r;
        let mut value = 0u64;
        let mut kk = k;

        for level in (0..self.maxlog).rev() {
            let cnt0 = self.zeros[level][right] - self.zeros[level][left];

            if kk < cnt0 {
                left = self.zeros[level][left];
                right = self.zeros[level][right];
            } else {
                value |= 1u64 << level;
                kk -= cnt0;
                left = self.mid[level] + (left - self.zeros[level][left]);
                right = self.mid[level] + (right - self.zeros[level][right]);
            }
        }

        self.adjust(value)
    }

    fn frequency_less_than(&self, pos: usize, c: i64) -> usize {
        assert!(pos <= self.len, "position out of bounds");

        if c <= self.offset {
            return 0;
        }
        if self.upper < c {
            return pos;
        }

        let mut l = 0;
        let mut r = pos;
        let mut cnt = 0usize;

        let shifted = self.shift(c);

        for level in (0..self.maxlog).rev() {
            if ((shifted >> level) & 1) == 1 {
                cnt += self.zeros[level][r] - self.zeros[level][l];
                l = self.mid[level] + (l - self.zeros[level][l]);
                r = self.mid[level] + (r - self.zeros[level][r]);
            } else {
                l = self.zeros[level][l];
                r = self.zeros[level][r];
            }
        }

        cnt
    }

    #[inline]
    fn bit_length(x: u64) -> usize {
        (64 - x.leading_zeros()) as usize
    }

    #[inline]
    fn adjust(&self, x: u64) -> i64 {
        (x as i64).wrapping_add(self.offset)
    }

    #[inline]
    fn shift(&self, x: i64) -> u64 {
        x.wrapping_sub(self.offset) as u64
    }
}

#[cfg(test)]
mod tests {
    use super::WaveletMatrix;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn capture_panic<F>(test_target: F) -> std::thread::Result<()>
    where
        F: FnOnce(),
    {
        catch_unwind(AssertUnwindSafe(test_target))
    }

    fn sorted_values_in_range(values: &[i64], left: usize, right: usize) -> Vec<i64> {
        let mut sorted_values = values[left..right].to_vec();
        sorted_values.sort_unstable();
        sorted_values
    }

    fn naive_kth_min(values: &[i64], left: usize, right: usize, kth_index: usize) -> i64 {
        let sorted_values = sorted_values_in_range(values, left, right);
        sorted_values[kth_index]
    }

    fn naive_range_min(values: &[i64], left: usize, right: usize) -> i64 {
        values[left..right]
            .iter()
            .copied()
            .min()
            .expect("range must be non-empty")
    }

    fn naive_range_max(values: &[i64], left: usize, right: usize) -> i64 {
        values[left..right]
            .iter()
            .copied()
            .max()
            .expect("range must be non-empty")
    }

    fn naive_lower_bound(values: &[i64], left: usize, right: usize, threshold: i64) -> Option<i64> {
        values[left..right]
            .iter()
            .copied()
            .filter(|&value| threshold <= value)
            .min()
    }

    fn naive_upper_bound(values: &[i64], left: usize, right: usize, threshold: i64) -> Option<i64> {
        values[left..right]
            .iter()
            .copied()
            .filter(|&value| value < threshold)
            .max()
    }

    fn naive_range_frequency(
        values: &[i64],
        left: usize,
        right: usize,
        lower_bound: i64,
        upper_bound: i64,
    ) -> usize {
        values[left..right]
            .iter()
            .copied()
            .filter(|&value| lower_bound <= value && value < upper_bound)
            .count()
    }

    fn collect_access_results(values_length: usize, wavelet_matrix: &WaveletMatrix) -> Vec<i64> {
        (0..values_length)
            .map(|index| wavelet_matrix.access(index))
            .collect()
    }

    fn find_first_query_mismatch_with_query_values(
        values: &[i64],
        wavelet_matrix: &WaveletMatrix,
        query_values: &[i64],
    ) -> Option<String> {
        let values_length = values.len();
        let actual_length = wavelet_matrix.len();

        if actual_length != values_length {
            return Some(format!(
                "len mismatch: expected={values_length}, actual={actual_length}"
            ));
        }

        let actual_values = collect_access_results(values_length, wavelet_matrix);
        if actual_values != values {
            return Some(format!(
                "access mismatch: expected={values:?}, actual={actual_values:?}"
            ));
        }

        for left in 0..=values_length {
            for right in left..=values_length {
                let range_length = right - left;

                if left < right {
                    for kth_index in 0..range_length {
                        let expected_kth = naive_kth_min(values, left, right, kth_index);
                        let actual_kth = wavelet_matrix.kth_min(left, right, kth_index);
                        if actual_kth != expected_kth {
                            return Some(format!(
                                "kth_min mismatch: left={left}, right={right}, kth={kth_index}, expected={expected_kth}, actual={actual_kth}"
                            ));
                        }
                    }

                    let expected_range_min = naive_range_min(values, left, right);
                    let actual_range_min = wavelet_matrix.range_min(left, right);
                    if actual_range_min != expected_range_min {
                        return Some(format!(
                            "range_min mismatch: left={left}, right={right}, expected={expected_range_min}, actual={actual_range_min}"
                        ));
                    }

                    let expected_range_max = naive_range_max(values, left, right);
                    let actual_range_max = wavelet_matrix.range_max(left, right);
                    if actual_range_max != expected_range_max {
                        return Some(format!(
                            "range_max mismatch: left={left}, right={right}, expected={expected_range_max}, actual={actual_range_max}"
                        ));
                    }
                }

                for &query_value in query_values {
                    let expected_lower_bound = naive_lower_bound(values, left, right, query_value);
                    let actual_lower_bound = wavelet_matrix.lower_bound(left, right, query_value);
                    if actual_lower_bound != expected_lower_bound {
                        return Some(format!(
                            "lower_bound mismatch: left={left}, right={right}, x={query_value}, expected={expected_lower_bound:?}, actual={actual_lower_bound:?}"
                        ));
                    }

                    let expected_upper_bound = naive_upper_bound(values, left, right, query_value);
                    let actual_upper_bound = wavelet_matrix.upper_bound(left, right, query_value);
                    if actual_upper_bound != expected_upper_bound {
                        return Some(format!(
                            "upper_bound mismatch: left={left}, right={right}, x={query_value}, expected={expected_upper_bound:?}, actual={actual_upper_bound:?}"
                        ));
                    }
                }

                for &lower_bound in query_values {
                    for &upper_bound in query_values {
                        if lower_bound <= upper_bound {
                            let expected_frequency = naive_range_frequency(
                                values,
                                left,
                                right,
                                lower_bound,
                                upper_bound,
                            );
                            let actual_frequency = wavelet_matrix.range_frequency(
                                left,
                                right,
                                lower_bound,
                                upper_bound,
                            );
                            if actual_frequency != expected_frequency {
                                return Some(format!(
                                    "range_frequency mismatch: left={left}, right={right}, lower={lower_bound}, upper={upper_bound}, expected={expected_frequency}, actual={actual_frequency}"
                                ));
                            }
                        }
                    }
                }
            }
        }

        None
    }

    fn find_first_query_mismatch(values: &[i64], wavelet_matrix: &WaveletMatrix) -> Option<String> {
        let min_value = values
            .iter()
            .copied()
            .min()
            .expect("values must be non-empty");
        let max_value = values
            .iter()
            .copied()
            .max()
            .expect("values must be non-empty");

        let mut query_values = vec![
            min_value.saturating_sub(1),
            min_value,
            min_value.saturating_add(1),
            max_value.saturating_sub(1),
            max_value,
            max_value.saturating_add(1),
            0,
        ];
        query_values.sort_unstable();
        query_values.dedup();

        find_first_query_mismatch_with_query_values(values, wavelet_matrix, &query_values)
    }

    fn enumerate_arrays(alphabet: &[i64], max_length: usize) -> Vec<Vec<i64>> {
        let mut arrays = Vec::new();

        for length in 1..=max_length {
            let case_count = alphabet.len().pow(length as u32);

            for encoded_value in 0..case_count {
                let mut remaining_value = encoded_value;
                let mut values = Vec::with_capacity(length);

                for _ in 0..length {
                    values.push(alphabet[remaining_value % alphabet.len()]);
                    remaining_value /= alphabet.len();
                }

                arrays.push(values);
            }
        }

        arrays
    }

    #[test]
    fn new_panics_when_data_is_empty() {
        // Arrange
        let input_values: Vec<i64> = vec![];

        // Act
        let panic_result = capture_panic(|| {
            let _wavelet_matrix = WaveletMatrix::new(&input_values);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_single_value_returns_length_one_and_access_matches() {
        // Arrange
        let input_values = vec![7i64];
        let query_index = 0usize;
        let expected_length = 1usize;
        let expected_value = 7i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_length = wavelet_matrix.len();
        let actual_value = wavelet_matrix.access(query_index);

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn new_with_mixed_signed_values_preserves_access_for_all_indices() {
        // Arrange
        let input_values = vec![-8i64, -1, 0, 4, 9];
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_values = collect_access_results(input_values.len(), &wavelet_matrix);

        // Assert
        assert_eq!(actual_values, input_values);
    }

    #[test]
    fn new_with_duplicate_values_preserves_access_for_all_indices() {
        // Arrange
        let input_values = vec![5i64, 5, -3, -3, 5, 0];
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_values = collect_access_results(input_values.len(), &wavelet_matrix);

        // Assert
        assert_eq!(actual_values, input_values);
    }

    #[test]
    fn new_with_extreme_values_preserves_access() {
        // Arrange
        let input_values = vec![i64::MIN, 0, i64::MAX];
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_values = collect_access_results(input_values.len(), &wavelet_matrix);

        // Assert
        assert_eq!(actual_values, input_values);
    }

    #[test]
    fn new_with_i64_minimum_and_zero_matches_reference_at_64_bit_width() {
        // Arrange
        let input_values = vec![i64::MIN, 0];
        let expected_mismatch: Option<String> = None;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_mismatch = find_first_query_mismatch(&input_values, &wavelet_matrix);

        // Assert
        assert_eq!(actual_mismatch, expected_mismatch);
    }

    #[test]
    fn new_with_zero_and_i64_maximum_matches_reference_at_64_bit_width() {
        // Arrange
        let input_values = vec![0i64, i64::MAX];
        let expected_mismatch: Option<String> = None;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_mismatch = find_first_query_mismatch(&input_values, &wavelet_matrix);

        // Assert
        assert_eq!(actual_mismatch, expected_mismatch);
    }

    #[test]
    fn new_with_bounds_panics_when_data_is_empty() {
        // Arrange
        let input_values: Vec<i64> = vec![];
        let lower_bound = 0i64;
        let upper_bound = 0i64;

        // Act
        let panic_result = capture_panic(|| {
            let _wavelet_matrix =
                WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_bounds_panics_when_upper_is_less_than_lower() {
        // Arrange
        let input_values = vec![1i64, 2, 3];
        let lower_bound = 5i64;
        let upper_bound = 4i64;

        // Act
        let panic_result = capture_panic(|| {
            let _wavelet_matrix =
                WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_bounds_exact_range_preserves_access() {
        // Arrange
        let input_values = vec![-3i64, 0, 5, 9, -1];
        let lower_bound = -3i64;
        let upper_bound = 9i64;
        let wavelet_matrix =
            WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);

        // Act
        let actual_values = collect_access_results(input_values.len(), &wavelet_matrix);

        // Assert
        assert_eq!(actual_values, input_values);
    }

    #[test]
    fn new_with_bounds_wider_range_preserves_access() {
        // Arrange
        let input_values = vec![-3i64, 0, 5, 9, -1];
        let lower_bound = -10i64;
        let upper_bound = 20i64;
        let wavelet_matrix =
            WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);

        // Act
        let actual_values = collect_access_results(input_values.len(), &wavelet_matrix);

        // Assert
        assert_eq!(actual_values, input_values);
    }

    #[test]
    fn new_with_bounds_panics_when_data_contains_out_of_range_value() {
        // Arrange
        let input_values = vec![-1i64, 0, 1];
        let lower_bound = 0i64;
        let upper_bound = 1i64;

        // Act
        let panic_result = capture_panic(|| {
            let _wavelet_matrix =
                WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_extreme_bounds_matches_reference_results() {
        // Arrange
        let input_values = vec![i64::MIN, i64::MIN + 1, -1024, -1];
        let lower_bound = i64::MIN;
        let upper_bound = i64::MAX;
        let expected_mismatch: Option<String> = None;
        let wavelet_matrix =
            WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);

        // Act
        let mismatch = find_first_query_mismatch(&input_values, &wavelet_matrix);

        // Assert
        assert_eq!(mismatch, expected_mismatch);
    }

    #[test]
    fn new_with_bounds_on_multiple_values_returns_input_length() {
        // Arrange
        let input_values = vec![-3i64, 0, 5, 9];
        let lower_bound = -10i64;
        let upper_bound = 10i64;
        let expected_length = input_values.len();
        let wavelet_matrix =
            WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);

        // Act
        let actual_length = wavelet_matrix.len();

        // Assert
        assert_eq!(actual_length, expected_length);
    }

    #[test]
    fn new_with_bounds_panics_when_data_contains_value_above_upper_bound() {
        // Arrange
        let input_values = vec![0i64, 1, 2];
        let lower_bound = 0i64;
        let upper_bound = 1i64;

        // Act
        let panic_result = capture_panic(|| {
            let _wavelet_matrix =
                WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn new_with_bounds_at_i64_minimum_matches_reference_queries() {
        // Arrange
        let input_values = vec![i64::MIN, i64::MIN, i64::MIN];
        let lower_bound = i64::MIN;
        let upper_bound = i64::MIN;
        let expected_mismatch: Option<String> = None;
        let wavelet_matrix =
            WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);

        // Act
        let actual_mismatch = find_first_query_mismatch(&input_values, &wavelet_matrix);

        // Assert
        assert_eq!(actual_mismatch, expected_mismatch);
    }

    #[test]
    fn new_with_bounds_wider_than_data_matches_reference_outside_actual_range() {
        // Arrange
        let input_values = vec![10i64, 20];
        let constructor_lower_bound = 0i64;
        let constructor_upper_bound = 30i64;
        let left = 0usize;
        let right = input_values.len();
        let query_values = vec![0i64, 9, 10, 20, 21, 30];
        let frequency_ranges = vec![(0i64, 10i64), (0, 21), (9, 21), (21, 30)];
        let expected_bound_results = query_values
            .iter()
            .map(|&query_value| {
                (
                    query_value,
                    naive_lower_bound(&input_values, left, right, query_value),
                    naive_upper_bound(&input_values, left, right, query_value),
                )
            })
            .collect::<Vec<_>>();
        let expected_frequency_results = frequency_ranges
            .iter()
            .map(|&(lower_bound, upper_bound)| {
                (
                    lower_bound,
                    upper_bound,
                    naive_range_frequency(&input_values, left, right, lower_bound, upper_bound),
                )
            })
            .collect::<Vec<_>>();
        let wavelet_matrix = WaveletMatrix::new_with_bounds(
            &input_values,
            constructor_lower_bound,
            constructor_upper_bound,
        );

        // Act
        let actual_bound_results = query_values
            .iter()
            .map(|&query_value| {
                (
                    query_value,
                    wavelet_matrix.lower_bound(left, right, query_value),
                    wavelet_matrix.upper_bound(left, right, query_value),
                )
            })
            .collect::<Vec<_>>();
        let actual_frequency_results = frequency_ranges
            .iter()
            .map(|&(lower_bound, upper_bound)| {
                (
                    lower_bound,
                    upper_bound,
                    wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound),
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_bound_results, expected_bound_results);
        assert_eq!(actual_frequency_results, expected_frequency_results);
    }

    #[test]
    fn new_with_bounds_matches_reference_around_bit_width_transition() {
        // Arrange
        let lower_bound = 0i64;
        let upper_bounds = [6i64, 7, 8];
        let test_cases = upper_bounds
            .iter()
            .copied()
            .map(|upper_bound| {
                let input_values = vec![lower_bound, upper_bound];
                let wavelet_matrix =
                    WaveletMatrix::new_with_bounds(&input_values, lower_bound, upper_bound);
                (input_values, wavelet_matrix)
            })
            .collect::<Vec<_>>();
        let expected_mismatches: Vec<Option<String>> = vec![None, None, None];

        // Act
        let actual_mismatches = test_cases
            .iter()
            .map(|(input_values, wavelet_matrix)| {
                find_first_query_mismatch(input_values, wavelet_matrix)
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_mismatches, expected_mismatches);
    }

    #[test]
    fn extreme_duplicate_values_order_queries_match_reference_for_all_non_empty_ranges() {
        // Arrange
        let input_values = vec![i64::MIN, i64::MIN, -1, 0, i64::MAX, i64::MAX];
        let wavelet_matrix = WaveletMatrix::new(&input_values);
        let mut expected_results = Vec::new();

        for left in 0..input_values.len() {
            for right in left + 1..=input_values.len() {
                expected_results.push((
                    left,
                    right,
                    sorted_values_in_range(&input_values, left, right),
                    naive_range_min(&input_values, left, right),
                    naive_range_max(&input_values, left, right),
                ));
            }
        }

        // Act
        let mut actual_results = Vec::new();

        for left in 0..input_values.len() {
            for right in left + 1..=input_values.len() {
                let actual_kth_values = (0..right - left)
                    .map(|kth_index| wavelet_matrix.kth_min(left, right, kth_index))
                    .collect::<Vec<_>>();

                actual_results.push((
                    left,
                    right,
                    actual_kth_values,
                    wavelet_matrix.range_min(left, right),
                    wavelet_matrix.range_max(left, right),
                ));
            }
        }

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn extreme_threshold_bounds_match_reference_for_all_non_empty_ranges() {
        // Arrange
        let input_values = vec![i64::MIN, i64::MIN, -1, 0, i64::MAX, i64::MAX];
        let query_values = [i64::MIN, i64::MAX];
        let wavelet_matrix = WaveletMatrix::new(&input_values);
        let mut expected_results = Vec::new();

        for left in 0..input_values.len() {
            for right in left + 1..=input_values.len() {
                for &query_value in &query_values {
                    expected_results.push((
                        left,
                        right,
                        query_value,
                        naive_lower_bound(&input_values, left, right, query_value),
                        naive_upper_bound(&input_values, left, right, query_value),
                    ));
                }
            }
        }

        // Act
        let mut actual_results = Vec::new();

        for left in 0..input_values.len() {
            for right in left + 1..=input_values.len() {
                for &query_value in &query_values {
                    actual_results.push((
                        left,
                        right,
                        query_value,
                        wavelet_matrix.lower_bound(left, right, query_value),
                        wavelet_matrix.upper_bound(left, right, query_value),
                    ));
                }
            }
        }

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn access_panics_when_index_equals_length() {
        // Arrange
        let input_values = vec![1i64, 2, 3];
        let out_of_bounds_index = input_values.len();
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.access(out_of_bounds_index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn kth_min_returns_global_minimum_for_full_range() {
        // Arrange
        let input_values = vec![7i64, -2, 4, 1];
        let left = 0usize;
        let right = input_values.len();
        let kth_index = 0usize;
        let expected_value = -2i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.kth_min(left, right, kth_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_returns_global_maximum_for_full_range() {
        // Arrange
        let input_values = vec![7i64, -2, 4, 1];
        let left = 0usize;
        let right = input_values.len();
        let kth_index = input_values.len() - 1;
        let expected_value = 7i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.kth_min(left, right, kth_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_returns_sorted_kth_value_in_subrange() {
        // Arrange
        let input_values = vec![9i64, 1, 5, 3, 8, 2];
        let left = 1usize;
        let right = 5usize;
        let kth_index = 2usize;
        let expected_value = 5i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.kth_min(left, right, kth_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_counts_duplicate_values_as_separate_positions() {
        // Arrange
        let input_values = vec![4i64, 2, 2, 2, 9];
        let left = 0usize;
        let right = input_values.len();
        let kth_index = 2usize;
        let expected_value = 2i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.kth_min(left, right, kth_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_on_single_element_range_returns_the_element() {
        // Arrange
        let input_values = vec![6i64, -1, 3];
        let left = 1usize;
        let right = 2usize;
        let kth_index = 0usize;
        let expected_value = -1i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.kth_min(left, right, kth_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_panics_when_k_equals_range_length() {
        // Arrange
        let input_values = vec![1i64, 4, 7];
        let left = 0usize;
        let right = input_values.len();
        let out_of_range_kth_index = right - left;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.kth_min(left, right, out_of_range_kth_index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn kth_min_panics_on_empty_range() {
        // Arrange
        let input_values = vec![1i64, 4, 7];
        let left = 1usize;
        let right = 1usize;
        let kth_index = 0usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.kth_min(left, right, kth_index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn kth_min_panics_when_right_exceeds_length() {
        // Arrange
        let input_values = vec![1i64, 4, 7];
        let left = 0usize;
        let right = input_values.len() + 1;
        let kth_index = 0usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.kth_min(left, right, kth_index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn kth_min_panics_when_right_is_less_than_left() {
        // Arrange
        let input_values = vec![1i64, 4, 7];
        let left = 2usize;
        let right = 1usize;
        let kth_index = 0usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.kth_min(left, right, kth_index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_min_returns_minimum_in_range() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 0usize;
        let right = input_values.len();
        let expected_value = -2i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_min(left, right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_min_on_single_element_range_returns_the_element() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 2usize;
        let right = 3usize;
        let expected_value = -2i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_min(left, right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_min_panics_on_empty_range() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 1usize;
        let right = 1usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.range_min(left, right);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_min_panics_when_right_exceeds_length() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 0usize;
        let right = input_values.len() + 1;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.range_min(left, right);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_min_panics_when_right_is_less_than_left() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 2usize;
        let right = 1usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.range_min(left, right);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_max_returns_maximum_in_range() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 0usize;
        let right = input_values.len();
        let expected_value = 9i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_max(left, right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_max_on_single_element_range_returns_the_element() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 2usize;
        let right = 3usize;
        let expected_value = -2i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_max(left, right);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_max_panics_on_empty_range() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 1usize;
        let right = 1usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.range_max(left, right);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_max_panics_when_right_exceeds_length() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 0usize;
        let right = input_values.len() + 1;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.range_max(left, right);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_max_panics_when_right_is_less_than_left() {
        // Arrange
        let input_values = vec![9i64, 4, -2, 6];
        let left = 2usize;
        let right = 1usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.range_max(left, right);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn upper_bound_on_empty_range_returns_none() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 1usize;
        let right = 1usize;
        let query_value = 4i64;
        let expected_value = None;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.upper_bound(left, right, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_handles_x_minus_one_x_x_plus_one_boundaries() {
        // Arrange
        let input_values = vec![1i64, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let x = 3i64;
        let query_value_x_minus_one = x - 1;
        let query_value_x = x;
        let query_value_x_plus_one = x + 1;
        let expected_x_minus_one = Some(1i64);
        let expected_x = Some(1i64);
        let expected_x_plus_one = Some(3i64);
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_x_minus_one = wavelet_matrix.upper_bound(left, right, query_value_x_minus_one);
        let actual_x = wavelet_matrix.upper_bound(left, right, query_value_x);
        let actual_x_plus_one = wavelet_matrix.upper_bound(left, right, query_value_x_plus_one);

        // Assert
        assert_eq!(actual_x_minus_one, expected_x_minus_one);
        assert_eq!(actual_x, expected_x);
        assert_eq!(actual_x_plus_one, expected_x_plus_one);
    }

    #[test]
    fn upper_bound_returns_none_when_no_value_is_less_than_x() {
        // Arrange
        let input_values = vec![1i64, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let query_value = 1i64;
        let expected_value = None;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.upper_bound(left, right, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_with_duplicates_returns_maximum_less_than_x() {
        // Arrange
        let input_values = vec![1i64, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let query_value = 3i64;
        let expected_value = Some(1i64);
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.upper_bound(left, right, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_panics_when_right_is_less_than_left() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 2usize;
        let right = 1usize;
        let query_value = 4i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.upper_bound(left, right, query_value);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn upper_bound_panics_when_right_exceeds_length() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 0usize;
        let right = input_values.len() + 1;
        let query_value = 4i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.upper_bound(left, right, query_value);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn upper_bound_panics_when_reversed_range_has_equal_prefix_counts() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 2usize;
        let right = 1usize;
        let query_value = 0i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.upper_bound(left, right, query_value);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn lower_bound_on_empty_range_returns_none() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 1usize;
        let right = 1usize;
        let query_value = 4i64;
        let expected_value = None;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.lower_bound(left, right, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_handles_x_minus_one_x_x_plus_one_boundaries() {
        // Arrange
        let input_values = vec![1i64, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let x = 3i64;
        let query_value_x_minus_one = x - 1;
        let query_value_x = x;
        let query_value_x_plus_one = x + 1;
        let expected_x_minus_one = Some(3i64);
        let expected_x = Some(3i64);
        let expected_x_plus_one = Some(5i64);
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_x_minus_one = wavelet_matrix.lower_bound(left, right, query_value_x_minus_one);
        let actual_x = wavelet_matrix.lower_bound(left, right, query_value_x);
        let actual_x_plus_one = wavelet_matrix.lower_bound(left, right, query_value_x_plus_one);

        // Assert
        assert_eq!(actual_x_minus_one, expected_x_minus_one);
        assert_eq!(actual_x, expected_x);
        assert_eq!(actual_x_plus_one, expected_x_plus_one);
    }

    #[test]
    fn lower_bound_returns_none_when_no_value_is_greater_or_equal_x() {
        // Arrange
        let input_values = vec![1i64, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let query_value = 6i64;
        let expected_value = None;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.lower_bound(left, right, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_with_duplicates_returns_x_when_x_exists() {
        // Arrange
        let input_values = vec![1i64, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let query_value = 3i64;
        let expected_value = Some(3i64);
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.lower_bound(left, right, query_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_panics_when_right_is_less_than_left() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 2usize;
        let right = 1usize;
        let query_value = 0i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.lower_bound(left, right, query_value);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn lower_bound_panics_when_right_exceeds_length() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 0usize;
        let right = input_values.len() + 1;
        let query_value = 4i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.lower_bound(left, right, query_value);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn lower_bound_panics_on_reversed_range_regardless_of_overflow_checks() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 2usize;
        let right = 1usize;
        let query_value = 4i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value = wavelet_matrix.lower_bound(left, right, query_value);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_frequency_counts_values_in_half_open_interval() {
        // Arrange
        let input_values = vec![1i64, 2, 3, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let lower_bound = 2i64;
        let upper_bound = 4i64;
        let expected_value = 3usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_frequency_uses_inclusive_lower_and_exclusive_upper_boundaries() {
        // Arrange
        let input_values = vec![2i64, 3, 3, 4];
        let left = 0usize;
        let right = input_values.len();
        let x = 3i64;
        let lower_x_minus_one = x - 1;
        let upper_x = x;
        let lower_x = x;
        let upper_x_plus_one = x + 1;
        let lower_x_plus_one = x + 1;
        let upper_x_plus_two = x + 2;
        let expected_x_minus_one_to_x = 1usize;
        let expected_x_to_x_plus_one = 2usize;
        let expected_x_plus_one_to_x_plus_two = 1usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_x_minus_one_to_x =
            wavelet_matrix.range_frequency(left, right, lower_x_minus_one, upper_x);
        let actual_x_to_x_plus_one =
            wavelet_matrix.range_frequency(left, right, lower_x, upper_x_plus_one);
        let actual_x_plus_one_to_x_plus_two =
            wavelet_matrix.range_frequency(left, right, lower_x_plus_one, upper_x_plus_two);

        // Assert
        assert_eq!(actual_x_minus_one_to_x, expected_x_minus_one_to_x);
        assert_eq!(actual_x_to_x_plus_one, expected_x_to_x_plus_one);
        assert_eq!(
            actual_x_plus_one_to_x_plus_two,
            expected_x_plus_one_to_x_plus_two
        );
    }

    #[test]
    fn range_frequency_on_empty_range_returns_zero() {
        // Arrange
        let input_values = vec![2i64, 3, 3, 4];
        let left = 2usize;
        let right = 2usize;
        let lower_bound = 1i64;
        let upper_bound = 5i64;
        let expected_value = 0usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_frequency_with_equal_bounds_returns_zero() {
        // Arrange
        let input_values = vec![2i64, 3, 3, 4];
        let left = 0usize;
        let right = input_values.len();
        let lower_bound = 3i64;
        let upper_bound = 3i64;
        let expected_value = 0usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_frequency_counts_duplicate_values() {
        // Arrange
        let input_values = vec![2i64, 3, 3, 3, 4];
        let left = 0usize;
        let right = input_values.len();
        let lower_bound = 3i64;
        let upper_bound = 4i64;
        let expected_value = 3usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_frequency_outside_data_value_range_returns_zero() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let lower_bound = 10i64;
        let upper_bound = 20i64;
        let expected_value = 0usize;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_value = wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn range_frequency_panics_when_upper_is_less_than_lower() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 0usize;
        let right = input_values.len();
        let lower_bound = 5i64;
        let upper_bound = 4i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value =
                wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_frequency_panics_when_right_exceeds_length() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 0usize;
        let right = input_values.len() + 1;
        let lower_bound = 1i64;
        let upper_bound = 6i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value =
                wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_frequency_panics_when_right_is_less_than_left() {
        // Arrange
        let input_values = vec![1i64, 3, 5];
        let left = 2usize;
        let right = 1usize;
        let lower_bound = 1i64;
        let upper_bound = 6i64;
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let panic_result = capture_panic(|| {
            let _actual_value =
                wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn range_frequency_between_i64_extremes_includes_minimum_and_excludes_maximum() {
        // Arrange
        let input_values = vec![i64::MIN, i64::MIN, -1, 0, i64::MAX, i64::MAX];
        let left = 0usize;
        let right = input_values.len();
        let lower_bound = i64::MIN;
        let upper_bound = i64::MAX;
        let expected_frequency =
            naive_range_frequency(&input_values, left, right, lower_bound, upper_bound);
        let wavelet_matrix = WaveletMatrix::new(&input_values);

        // Act
        let actual_frequency =
            wavelet_matrix.range_frequency(left, right, lower_bound, upper_bound);

        // Assert
        assert_eq!(actual_frequency, expected_frequency);
    }

    #[test]
    fn query_results_match_reference_for_singleton_and_multi_element_sets() {
        // Arrange
        let singleton_values = vec![42i64];
        let multi_values = vec![-5i64, 3, 3, 0, 8];
        let expected_mismatch: Option<String> = None;
        let singleton_wavelet_matrix = WaveletMatrix::new(&singleton_values);
        let multi_wavelet_matrix = WaveletMatrix::new(&multi_values);

        // Act
        let singleton_mismatch =
            find_first_query_mismatch(&singleton_values, &singleton_wavelet_matrix);
        let multi_mismatch = find_first_query_mismatch(&multi_values, &multi_wavelet_matrix);

        // Assert
        assert_eq!(singleton_mismatch, expected_mismatch);
        assert_eq!(multi_mismatch, expected_mismatch);
    }

    #[test]
    fn exhaustive_small_alphabet_queries_match_reference() {
        // Arrange
        let alphabet = [-1i64, 0, 1];
        let max_length = 5usize;
        let query_values = [-2i64, -1, 0, 1, 2];
        let input_cases = enumerate_arrays(&alphabet, max_length);
        let expected_mismatch: Option<String> = None;

        // Act
        let actual_mismatch = input_cases.iter().find_map(|input_values| {
            let wavelet_matrix = WaveletMatrix::new(input_values);
            let new_mismatch = find_first_query_mismatch_with_query_values(
                input_values,
                &wavelet_matrix,
                &query_values,
            )
            .map(|details| format!("constructor=new, values={input_values:?}, {details}"));

            new_mismatch.or_else(|| {
                let bounded_wavelet_matrix =
                    WaveletMatrix::new_with_bounds(input_values, alphabet[0], alphabet[2]);
                find_first_query_mismatch_with_query_values(
                    input_values,
                    &bounded_wavelet_matrix,
                    &query_values,
                )
                .map(|details| {
                    format!("constructor=new_with_bounds, values={input_values:?}, {details}")
                })
            })
        });

        // Assert
        assert_eq!(actual_mismatch, expected_mismatch);
    }
}
