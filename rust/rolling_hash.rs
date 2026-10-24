/// 負数は非負の剰余へ正規化する。同じ `modulo` の剰余に写る整数は区別できない。
pub trait RollingHashInput {
    fn len(&self) -> usize;

    fn is_empty(&self) -> bool {
        self.len() == 0
    }

    fn value_at(&self, index: usize, modulo: u64) -> u64;
}

impl RollingHashInput for str {
    fn len(&self) -> usize {
        self.len()
    }

    fn value_at(&self, index: usize, modulo: u64) -> u64 {
        (1 + self.as_bytes()[index] as i128 - b'a' as i128).rem_euclid(modulo as i128) as u64
    }
}

impl RollingHashInput for String {
    fn len(&self) -> usize {
        self.len()
    }

    fn value_at(&self, index: usize, modulo: u64) -> u64 {
        self.as_str().value_at(index, modulo)
    }
}

macro_rules! impl_integer_input {
    ($wide:ty; $($integer:ty),+ $(,)?) => {
        $(
            impl RollingHashInput for [$integer] {
                fn len(&self) -> usize {
                    self.len()
                }

                fn value_at(&self, index: usize, modulo: u64) -> u64 {
                    (self[index] as $wide).rem_euclid(modulo as $wide) as u64
                }
            }
        )+
    };
}

impl_integer_input!(u128; u8, u16, u32, u64, u128, usize);
impl_integer_input!(i128; i8, i16, i32, i64, i128, isize);

impl<T, const N: usize> RollingHashInput for [T; N]
where
    [T]: RollingHashInput,
{
    fn len(&self) -> usize {
        N
    }

    fn value_at(&self, index: usize, modulo: u64) -> u64 {
        self.as_slice().value_at(index, modulo)
    }
}

impl<T> RollingHashInput for Vec<T>
where
    [T]: RollingHashInput,
{
    fn len(&self) -> usize {
        self.len()
    }

    fn value_at(&self, index: usize, modulo: u64) -> u64 {
        self.as_slice().value_at(index, modulo)
    }
}

impl<T: RollingHashInput + ?Sized> RollingHashInput for &T {
    fn len(&self) -> usize {
        T::len(*self)
    }

    fn value_at(&self, index: usize, modulo: u64) -> u64 {
        T::value_at(*self, index, modulo)
    }
}

pub struct RollingHash {
    size: usize,
    modulo: u64,
    base_mod: Vec<u64>,
    hash_list: Vec<u64>,
}

impl RollingHash {
    pub fn new<T: RollingHashInput + ?Sized>(source: &T, base: u64, modulo: u64) -> Self {
        let length = source.len();
        let mut base_mod = vec![1u64; length + 1];
        let mut hash_list = vec![1u64; length + 1];

        for i in 0..length {
            base_mod[i + 1] = Self::multiple(base_mod[i], base, modulo);
            hash_list[i + 1] = Self::add(
                Self::multiple(hash_list[i], base, modulo),
                source.value_at(i, modulo),
                modulo,
            );
        }

        RollingHash {
            size: length,
            modulo,
            base_mod,
            hash_list,
        }
    }

    const fn add(x: u64, y: u64, modulo: u64) -> u64 {
        ((x as u128 + y as u128) % modulo as u128) as u64
    }

    const fn subtract(x: u64, y: u64, modulo: u64) -> u64 {
        ((modulo as u128 + x as u128 - y as u128) % modulo as u128) as u64
    }

    const fn multiple(x: u64, y: u64, modulo: u64) -> u64 {
        (x as u128 * y as u128 % modulo as u128) as u64
    }

    #[inline(always)]
    pub fn len(&self) -> usize {
        self.size
    }

    #[inline(always)]
    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn hash(&self, left: usize, right: usize) -> u64 {
        Self::subtract(
            self.hash_list[right],
            Self::multiple(
                self.hash_list[left],
                self.base_mod[right - left],
                self.modulo,
            ),
            self.modulo,
        )
    }

    pub fn lcp(rh_a: &RollingHash, rh_b: &RollingHash, a_left: usize, b_left: usize) -> usize {
        let mut left = std::cmp::min(rh_a.len() - a_left, rh_b.len() - b_left) + 1;
        let mut right = 0;

        while 1 < left - right {
            let mid = right + (left - right) / 2;

            if rh_a.hash(a_left, a_left + mid) != rh_b.hash(b_left, b_left + mid) {
                left = mid;
            } else {
                right = mid;
            }
        }

        right
    }
}

#[cfg(test)]
mod tests {
    use super::RollingHash;

    const DEFAULT_BASE: u64 = 911_382_323;

    const DEFAULT_MODULO: u64 = 1_000_000_007;

    fn build_default_hash(source_text: &str) -> RollingHash {
        RollingHash::new(source_text, DEFAULT_BASE, DEFAULT_MODULO)
    }

    fn map_byte_to_value(byte_value: u8, modulo: u64) -> u64 {
        (modulo + 1 + byte_value as u64 - b'a' as u64) % modulo
    }

    fn naive_hash(source_text: &str, left: usize, right: usize, base: u64, modulo: u64) -> u64 {
        let source_bytes = source_text.as_bytes();
        let mut hash_value = 0u64;

        for &byte_value in &source_bytes[left..right] {
            let mapped_value = map_byte_to_value(byte_value, modulo);
            hash_value = ((hash_value as u128 * base as u128 + mapped_value as u128)
                % modulo as u128) as u64;
        }

        hash_value
    }

    fn naive_lcp(source_a: &str, source_b: &str, a_left: usize, b_left: usize) -> usize {
        let source_a_bytes = source_a.as_bytes();
        let source_b_bytes = source_b.as_bytes();
        let mut common_prefix_length = 0usize;

        while a_left + common_prefix_length < source_a_bytes.len()
            && b_left + common_prefix_length < source_b_bytes.len()
            && source_a_bytes[a_left + common_prefix_length]
                == source_b_bytes[b_left + common_prefix_length]
        {
            common_prefix_length += 1;
        }

        common_prefix_length
    }

    fn naive_integer_hash(source: &[i64], base: u64, modulo: u64) -> u64 {
        let mut hash_value = 0i128;
        for &value in source {
            hash_value = (hash_value * base as i128 + value as i128).rem_euclid(modulo as i128);
        }
        hash_value as u64
    }

    #[test]
    fn new_with_integer_array_slice_and_vec_returns_same_hash() {
        // Arrange
        let source_array = [10u64, 20, 30];
        let source_slice = source_array.as_slice();
        let source_vec = source_array.to_vec();
        let base = 100;
        let modulo = DEFAULT_MODULO;
        let expected_length = source_array.len();
        let expected_hash = 102_030;

        // Act
        let hashes = [
            RollingHash::new(&source_array, base, modulo),
            RollingHash::new(source_slice, base, modulo),
            RollingHash::new(&source_slice, base, modulo),
            RollingHash::new(&source_vec, base, modulo),
        ];
        let actual_results = hashes.map(|hash| (hash.len(), hash.hash(0, hash.len())));

        // Assert
        for (actual_length, actual_hash) in actual_results {
            assert_eq!(actual_length, expected_length);
            assert_eq!(actual_hash, expected_hash);
        }
    }

    #[test]
    fn new_with_string_and_borrowed_str_preserves_string_hash() {
        // Arrange
        let source_string = String::from("abc");
        let source_str = source_string.as_str();
        let base = 10;
        let modulo = DEFAULT_MODULO;
        let expected_hash = 123;
        let query_left = 0;
        let query_right = source_string.len();

        // Act
        let string_hash = RollingHash::new(&source_string, base, modulo);
        let borrowed_str_hash = RollingHash::new(&source_str, base, modulo);
        let actual_string_hash = string_hash.hash(query_left, query_right);
        let actual_borrowed_str_hash = borrowed_str_hash.hash(query_left, query_right);

        // Assert
        assert_eq!(actual_string_hash, expected_hash);
        assert_eq!(actual_borrowed_str_hash, expected_hash);
    }

    #[test]
    fn new_accepts_all_builtin_integer_types() {
        // Arrange
        let source_u8 = [1u8, 2, 3];
        let source_u16 = [1u16, 2, 3];
        let source_u32 = [1u32, 2, 3];
        let source_u64 = [1u64, 2, 3];
        let source_u128 = [1u128, 2, 3];
        let source_usize = [1usize, 2, 3];
        let source_i8 = [1i8, 2, 3];
        let source_i16 = [1i16, 2, 3];
        let source_i32 = [1i32, 2, 3];
        let source_i64 = [1i64, 2, 3];
        let source_i128 = [1i128, 2, 3];
        let source_isize = [1isize, 2, 3];
        let base = 10;
        let modulo = DEFAULT_MODULO;
        let query_left = 0;
        let query_right = 3;
        let expected_hash = 123;

        // Act
        let hashes = [
            RollingHash::new(&source_u8, base, modulo),
            RollingHash::new(&source_u16, base, modulo),
            RollingHash::new(&source_u32, base, modulo),
            RollingHash::new(&source_u64, base, modulo),
            RollingHash::new(&source_u128, base, modulo),
            RollingHash::new(&source_usize, base, modulo),
            RollingHash::new(&source_i8, base, modulo),
            RollingHash::new(&source_i16, base, modulo),
            RollingHash::new(&source_i32, base, modulo),
            RollingHash::new(&source_i64, base, modulo),
            RollingHash::new(&source_i128, base, modulo),
            RollingHash::new(&source_isize, base, modulo),
        ];
        let actual_hashes = hashes.map(|hash| hash.hash(query_left, query_right));

        // Assert
        for actual_hash in actual_hashes {
            assert_eq!(actual_hash, expected_hash);
        }
    }

    #[test]
    fn hash_for_empty_single_and_multiple_integer_inputs_matches_naive_hash_for_all_ranges() {
        // Arrange
        let source_cases: &[&[i64]] = &[&[], &[0], &[-1], &[0, -1, 7, -1, 0], &[7, 0, -1]];
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;

        // Act
        let mut actual_states = Vec::new();
        let mut actual_results = Vec::new();
        for &source in source_cases {
            let rolling_hash = RollingHash::new(source, base, modulo);
            actual_states.push((
                (rolling_hash.len(), rolling_hash.is_empty()),
                (source.len(), source.is_empty()),
            ));
            for left in 0..=source.len() {
                for right in left..=source.len() {
                    let actual_hash = rolling_hash.hash(left, right);
                    let expected_hash = naive_integer_hash(&source[left..right], base, modulo);
                    actual_results.push((actual_hash, expected_hash));
                }
            }
        }

        // Assert
        for (actual_state, expected_state) in actual_states {
            assert_eq!(actual_state, expected_state);
        }
        for (actual_result, expected_result) in actual_results {
            assert_eq!(actual_result, expected_result);
        }
    }

    #[test]
    fn lcp_for_integer_inputs_counts_elements_from_each_start() {
        // Arrange
        let source_a = [99i64, 0, -1, -1, 7];
        let source_b = [0i64, -1, -1, 8, 7];
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;
        let a_left = 1;
        let b_left = 0;
        let expected_lcp = 3;
        let expected_lcp_at_end = 0;
        let rolling_hash_a = RollingHash::new(&source_a, base, modulo);
        let rolling_hash_b = RollingHash::new(&source_b, base, modulo);

        // Act
        let actual_lcp = RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);
        let actual_lcp_at_end =
            RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, source_a.len(), b_left);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
        assert_eq!(actual_lcp_at_end, expected_lcp_at_end);
    }

    #[test]
    fn lcp_for_integer_inputs_matches_direct_comparison_for_all_start_indices() {
        // Arrange
        let source_cases: &[&[i64]] = &[&[], &[0], &[-1, -1, 0, 7], &[7, -1, -1, 0]];
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;

        // Act
        let mut collected_results = Vec::new();
        for &source_a in source_cases {
            for &source_b in source_cases {
                let rolling_hash_a = RollingHash::new(source_a, base, modulo);
                let rolling_hash_b = RollingHash::new(source_b, base, modulo);
                for a_left in 0..=source_a.len() {
                    for b_left in 0..=source_b.len() {
                        let actual_lcp =
                            RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);
                        let expected_lcp = source_a[a_left..]
                            .iter()
                            .zip(&source_b[b_left..])
                            .take_while(|(value_a, value_b)| value_a == value_b)
                            .count();
                        collected_results.push((actual_lcp, expected_lcp));
                    }
                }
            }
        }

        // Assert
        for (actual_lcp, expected_lcp) in collected_results {
            assert_eq!(actual_lcp, expected_lcp);
        }
    }

    #[test]
    fn hash_for_u128_values_preserves_bits_above_u64_before_normalizing() {
        // Arrange
        let source = [u128::MAX, u64::MAX as u128, 1u128 << 64];
        let base = DEFAULT_BASE;
        let modulo = u64::MAX - 1;
        let expected_hashes = [3, 1, 2];

        // Act
        let rolling_hash = RollingHash::new(&source, base, modulo);
        let actual_hashes = std::array::from_fn(|index| rolling_hash.hash(index, index + 1));

        // Assert
        assert_eq!(actual_hashes, expected_hashes);
    }

    #[test]
    fn hash_for_i128_boundaries_normalizes_negative_values_without_overflow() {
        // Arrange
        let source = [i128::MIN, i128::MIN + 1, i128::MAX];
        let base = DEFAULT_BASE;
        let modulo = u64::MAX;
        let expected_hashes = [(1u64 << 63) - 1, 1u64 << 63, (1u64 << 63) - 1];

        // Act
        let rolling_hash = RollingHash::new(&source, base, modulo);
        let actual_hashes = std::array::from_fn(|index| rolling_hash.hash(index, index + 1));

        // Assert
        assert_eq!(actual_hashes, expected_hashes);
    }

    #[test]
    fn hash_for_integer_input_with_large_modulo_matches_alternating_sum_for_all_ranges() {
        // Arrange
        let modulo = u64::MAX;
        let base = modulo - 1;
        let source = [modulo - 1, modulo - 2, 0, modulo - 1];

        // Act
        let rolling_hash = RollingHash::new(&source, base, modulo);
        let mut actual_results = Vec::new();
        for left in 0..=source.len() {
            for right in left..=source.len() {
                // base ≡ -1 のため、積を使わない交代和で期待値を計算できる。
                let expected_hash = source[left..right].iter().fold(0i128, |hash, &value| {
                    (value as i128 - hash).rem_euclid(modulo as i128)
                }) as u64;
                let actual_hash = rolling_hash.hash(left, right);
                actual_results.push((actual_hash, expected_hash));
            }
        }

        // Assert
        for (actual_hash, expected_hash) in actual_results {
            assert_eq!(actual_hash, expected_hash);
        }
    }

    #[test]
    fn hash_for_integer_input_with_modulo_one_returns_zero_for_all_ranges() {
        // Arrange
        let source = [i128::MIN, 0, -1, i128::MAX];
        let base = u64::MAX;
        let modulo = 1;
        let expected_hash = 0;

        // Act
        let rolling_hash = RollingHash::new(&source, base, modulo);
        let mut actual_hashes = Vec::new();
        for left in 0..=source.len() {
            for right in left..=source.len() {
                actual_hashes.push(rolling_hash.hash(left, right));
            }
        }

        // Assert
        for actual_hash in actual_hashes {
            assert_eq!(actual_hash, expected_hash);
        }
    }

    #[test]
    fn hash_for_integers_with_equal_residues_returns_same_hash() {
        // Arrange
        let source = [-1i64, 100, 201];
        let base = DEFAULT_BASE;
        let modulo = 101;
        let expected_hash = 100;

        // Act
        let rolling_hash = RollingHash::new(&source, base, modulo);
        let actual_hashes =
            std::array::from_fn::<_, 3, _>(|index| rolling_hash.hash(index, index + 1));

        // Assert
        for actual_hash in actual_hashes {
            assert_eq!(actual_hash, expected_hash);
        }
    }

    #[test]
    fn new_with_empty_string_returns_zero_length_and_empty_state_and_zero_hash_for_empty_range() {
        // Arrange
        let source_text = "";
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;
        let expected_length = 0usize;
        let expected_is_empty = true;
        let query_left = 0usize;
        let query_right = 0usize;
        let expected_hash = 0u64;

        // Act
        let rolling_hash = RollingHash::new(source_text, base, modulo);
        let actual_length = rolling_hash.len();
        let actual_is_empty = rolling_hash.is_empty();
        let actual_hash = rolling_hash.hash(query_left, query_right);

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn new_with_single_character_returns_non_empty_length_one() {
        // Arrange
        let source_text = "a";
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;
        let expected_length = 1usize;
        let expected_is_empty = false;

        // Act
        let rolling_hash = RollingHash::new(source_text, base, modulo);
        let actual_length = rolling_hash.len();
        let actual_is_empty = rolling_hash.is_empty();

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn new_with_three_characters_returns_length_three() {
        // Arrange
        let source_text = "abc";
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;
        let expected_length = 3usize;

        // Act
        let rolling_hash = RollingHash::new(source_text, base, modulo);
        let actual_length = rolling_hash.len();

        // Assert
        assert_eq!(actual_length, expected_length);
    }

    #[test]
    fn new_with_zero_base_can_still_calculate_hash_without_panic() {
        // Arrange
        let source_text = "abc";
        let base = 0u64;
        let modulo = DEFAULT_MODULO;
        let query_left = 0usize;
        let query_right = source_text.len();
        let expected_hash = naive_hash(source_text, query_left, query_right, base, modulo);

        // Act
        let rolling_hash = RollingHash::new(source_text, base, modulo);
        let actual_hash = rolling_hash.hash(query_left, query_right);

        // Assert
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn new_with_modulo_one_returns_zero_for_all_hash_ranges() {
        // Arrange
        let source_text = "abc";
        let base = DEFAULT_BASE;
        let modulo = 1u64;
        let expected_hash_0_0 = 0u64;
        let expected_hash_0_1 = 0u64;
        let expected_hash_1_2 = 0u64;
        let expected_hash_0_3 = 0u64;

        // Act
        let rolling_hash = RollingHash::new(source_text, base, modulo);
        let actual_hash_0_0 = rolling_hash.hash(0, 0);
        let actual_hash_0_1 = rolling_hash.hash(0, 1);
        let actual_hash_1_2 = rolling_hash.hash(1, 2);
        let actual_hash_0_3 = rolling_hash.hash(0, 3);

        // Assert
        assert_eq!(actual_hash_0_0, expected_hash_0_0);
        assert_eq!(actual_hash_0_1, expected_hash_0_1);
        assert_eq!(actual_hash_1_2, expected_hash_1_2);
        assert_eq!(actual_hash_0_3, expected_hash_0_3);
    }

    #[test]
    fn new_with_modulo_zero_panics_by_division_by_zero() {
        // Arrange
        let source_text = "abc";
        let base = DEFAULT_BASE;
        let modulo = 0u64;

        // Act
        let actual_result =
            std::panic::catch_unwind(|| RollingHash::new(source_text, base, modulo));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn new_with_utf8_string_uses_byte_length_not_character_length() {
        // Arrange
        let source_text = "あa";
        let base = DEFAULT_BASE;
        let modulo = DEFAULT_MODULO;
        let expected_length = source_text.len();

        // Act
        let rolling_hash = RollingHash::new(source_text, base, modulo);
        let actual_length = rolling_hash.len();

        // Assert
        assert_eq!(actual_length, expected_length);
    }

    #[test]
    fn hash_matches_naive_hash_for_all_ranges_in_empty_single_and_multiple_cases() {
        // Arrange
        let source_cases = vec!["", "a", "abca"];

        // Act
        let mut collected_results = Vec::new();
        for source_text in &source_cases {
            let rolling_hash = build_default_hash(source_text);
            let source_length = source_text.len();
            for left in 0..=source_length {
                for right in left..=source_length {
                    let actual_hash = rolling_hash.hash(left, right);
                    let expected_hash =
                        naive_hash(source_text, left, right, DEFAULT_BASE, DEFAULT_MODULO);
                    collected_results.push((actual_hash, expected_hash));
                }
            }
        }

        // Assert
        for (actual_hash, expected_hash) in collected_results {
            assert_eq!(actual_hash, expected_hash);
        }
    }

    #[test]
    fn lcp_matches_naive_lcp_for_all_start_indices_in_empty_single_and_multiple_cases() {
        // Arrange
        let source_cases = vec!["", "a", "abca"];

        // Act
        let mut collected_results = Vec::new();
        for source_a in &source_cases {
            for source_b in &source_cases {
                let rolling_hash_a = build_default_hash(source_a);
                let rolling_hash_b = build_default_hash(source_b);
                let source_a_length = source_a.len();
                let source_b_length = source_b.len();
                for a_left in 0..=source_a_length {
                    for b_left in 0..=source_b_length {
                        let actual_lcp =
                            RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);
                        let expected_lcp = naive_lcp(source_a, source_b, a_left, b_left);
                        collected_results.push((actual_lcp, expected_lcp));
                    }
                }
            }
        }

        // Assert
        for (actual_lcp, expected_lcp) in collected_results {
            assert_eq!(actual_lcp, expected_lcp);
        }
    }

    #[test]
    fn hash_matches_naive_hash_for_boundary_lengths_x_minus_1_x_x_plus_1() {
        // Arrange
        let source_text = "abcde";
        let boundary_center = 2usize;
        let range_length_x_minus_1 = boundary_center - 1;
        let range_length_x = boundary_center;
        let range_length_x_plus_1 = boundary_center + 1;
        let expected_hash_x_minus_1 = naive_hash(
            source_text,
            0,
            range_length_x_minus_1,
            DEFAULT_BASE,
            DEFAULT_MODULO,
        );
        let expected_hash_x =
            naive_hash(source_text, 0, range_length_x, DEFAULT_BASE, DEFAULT_MODULO);
        let expected_hash_x_plus_1 = naive_hash(
            source_text,
            0,
            range_length_x_plus_1,
            DEFAULT_BASE,
            DEFAULT_MODULO,
        );

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash_x_minus_1 = rolling_hash.hash(0, range_length_x_minus_1);
        let actual_hash_x = rolling_hash.hash(0, range_length_x);
        let actual_hash_x_plus_1 = rolling_hash.hash(0, range_length_x_plus_1);

        // Assert
        assert_eq!(actual_hash_x_minus_1, expected_hash_x_minus_1);
        assert_eq!(actual_hash_x, expected_hash_x);
        assert_eq!(actual_hash_x_plus_1, expected_hash_x_plus_1);
    }

    #[test]
    fn hash_panics_when_right_exceeds_length() {
        // Arrange
        let source_text = "abc";
        let left = 0usize;
        let right = source_text.len() + 1;
        let rolling_hash = build_default_hash(source_text);

        // Act
        let actual_result = std::panic::catch_unwind(|| rolling_hash.hash(left, right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn hash_panics_when_left_exceeds_length() {
        // Arrange
        let source_text = "abc";
        let left = source_text.len() + 1;
        let right = source_text.len() + 1;
        let rolling_hash = build_default_hash(source_text);

        // Act
        let actual_result = std::panic::catch_unwind(|| rolling_hash.hash(left, right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_for_same_string_and_same_start_returns_whole_length() {
        // Arrange
        let source_text = "abac";
        let a_left = 0usize;
        let b_left = 0usize;
        let expected_lcp = source_text.len();
        let rolling_hash_a = build_default_hash(source_text);
        let rolling_hash_b = build_default_hash(source_text);

        // Act
        let actual_lcp = RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn lcp_returns_shorter_suffix_length_when_other_side_is_longer() {
        // Arrange
        let source_a = "abcde";
        let source_b = "cdefgh";
        let a_left = 2usize;
        let b_left = 0usize;
        let expected_lcp = 3usize;
        let rolling_hash_a = build_default_hash(source_a);
        let rolling_hash_b = build_default_hash(source_b);

        // Act
        let actual_lcp = RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn lcp_returns_zero_when_either_start_index_is_equal_to_length() {
        // Arrange
        let source_a = "abc";
        let source_b = "abc";
        let a_left_at_end = source_a.len();
        let b_left_start = 0usize;
        let a_left_start = 0usize;
        let b_left_at_end = source_b.len();
        let expected_lcp = 0usize;
        let rolling_hash_a = build_default_hash(source_a);
        let rolling_hash_b = build_default_hash(source_b);

        // Act
        let actual_lcp_when_a_is_end = RollingHash::lcp(
            &rolling_hash_a,
            &rolling_hash_b,
            a_left_at_end,
            b_left_start,
        );
        let actual_lcp_when_b_is_end = RollingHash::lcp(
            &rolling_hash_a,
            &rolling_hash_b,
            a_left_start,
            b_left_at_end,
        );

        // Assert
        assert_eq!(actual_lcp_when_a_is_end, expected_lcp);
        assert_eq!(actual_lcp_when_b_is_end, expected_lcp);
    }

    #[test]
    fn lcp_panics_when_a_left_exceeds_length() {
        // Arrange
        let source_a = "abc";
        let source_b = "abc";
        let a_left = source_a.len() + 1;
        let b_left = 0usize;
        let rolling_hash_a = build_default_hash(source_a);
        let rolling_hash_b = build_default_hash(source_b);

        // Act
        let actual_result = std::panic::catch_unwind(|| {
            RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left)
        });

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_panics_when_b_left_exceeds_length() {
        // Arrange
        let source_a = "abc";
        let source_b = "abc";
        let a_left = 0usize;
        let b_left = source_b.len() + 1;
        let rolling_hash_a = build_default_hash(source_a);
        let rolling_hash_b = build_default_hash(source_b);

        // Act
        let actual_result = std::panic::catch_unwind(|| {
            RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left)
        });

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_for_utf8_input_returns_byte_based_common_prefix_length() {
        // Arrange
        let source_a = "あい";
        let source_b = "あう";
        let a_left = 0usize;
        let b_left = 0usize;
        let expected_lcp = 5usize;
        let rolling_hash_a = build_default_hash(source_a);
        let rolling_hash_b = build_default_hash(source_b);

        // Act
        let actual_lcp = RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn hash_for_full_range_matches_naive_hash_of_entire_string() {
        // Arrange
        let source_text = "abacaba";
        let query_left = 0usize;
        let query_right = source_text.len();
        let expected_hash = naive_hash(
            source_text,
            query_left,
            query_right,
            DEFAULT_BASE,
            DEFAULT_MODULO,
        );

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash = rolling_hash.hash(query_left, query_right);

        // Assert
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn hash_returns_zero_for_empty_ranges_at_start_middle_and_end() {
        // Arrange
        let source_text = "abc";
        let expected_hash_start = 0u64;
        let expected_hash_middle = 0u64;
        let expected_hash_end = 0u64;

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash_start = rolling_hash.hash(0, 0);
        let actual_hash_middle = rolling_hash.hash(2, 2);
        let actual_hash_end = rolling_hash.hash(source_text.len(), source_text.len());

        // Assert
        assert_eq!(actual_hash_start, expected_hash_start);
        assert_eq!(actual_hash_middle, expected_hash_middle);
        assert_eq!(actual_hash_end, expected_hash_end);
    }

    #[test]
    fn hash_for_first_single_byte_range_matches_naive_hash() {
        // Arrange
        let source_text = "abc";
        let query_left = 0usize;
        let query_right = 1usize;
        let expected_hash = naive_hash(
            source_text,
            query_left,
            query_right,
            DEFAULT_BASE,
            DEFAULT_MODULO,
        );

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash = rolling_hash.hash(query_left, query_right);

        // Assert
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn hash_for_last_single_byte_range_matches_naive_hash() {
        // Arrange
        let source_text = "abc";
        let query_left = source_text.len() - 1;
        let query_right = source_text.len();
        let expected_hash = naive_hash(
            source_text,
            query_left,
            query_right,
            DEFAULT_BASE,
            DEFAULT_MODULO,
        );

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash = rolling_hash.hash(query_left, query_right);

        // Assert
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn hash_matches_for_equal_substrings_in_same_string() {
        // Arrange
        let source_text = "ababa";
        let left_a = 0usize;
        let right_a = 2usize;
        let left_b = 2usize;
        let right_b = 4usize;

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash_a = rolling_hash.hash(left_a, right_a);
        let actual_hash_b = rolling_hash.hash(left_b, right_b);

        // Assert
        assert_eq!(actual_hash_a, actual_hash_b);
    }

    #[test]
    fn hash_differs_for_different_substrings_in_same_string() {
        // Arrange
        let source_text = "ababa";
        let left_a = 0usize;
        let right_a = 2usize;
        let left_b = 1usize;
        let right_b = 3usize;

        // Act
        let rolling_hash = build_default_hash(source_text);
        let actual_hash_a = rolling_hash.hash(left_a, right_a);
        let actual_hash_b = rolling_hash.hash(left_b, right_b);

        // Assert
        assert_ne!(actual_hash_a, actual_hash_b);
    }

    #[test]
    fn hash_panics_when_left_is_greater_than_right() {
        // Arrange
        let source_text = "abc";
        let left = 2usize;
        let right = 1usize;
        let rolling_hash = build_default_hash(source_text);

        // Act
        let actual_result = std::panic::catch_unwind(|| rolling_hash.hash(left, right));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn lcp_returns_zero_when_first_byte_mismatches() {
        // Arrange
        let source_a = "abc";
        let source_b = "xbc";
        let a_left = 0usize;
        let b_left = 0usize;
        let expected_lcp = 0usize;
        let rolling_hash_a = build_default_hash(source_a);
        let rolling_hash_b = build_default_hash(source_b);

        // Act
        let actual_lcp = RollingHash::lcp(&rolling_hash_a, &rolling_hash_b, a_left, b_left);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn lcp_handles_mismatch_positions_x_minus_1_x_x_plus_1() {
        // Arrange
        let mismatch_center = 2usize;
        let expected_lcp_x_minus_1 = mismatch_center - 1;
        let expected_lcp_x = mismatch_center;
        let expected_lcp_x_plus_1 = mismatch_center + 1;

        let source_a_x_minus_1 = "abzz";
        let source_b_x_minus_1 = "aczz";
        let source_a_x = "abcz";
        let source_b_x = "abdz";
        let source_a_x_plus_1 = "abcd";
        let source_b_x_plus_1 = "abce";

        let rolling_hash_a_x_minus_1 = build_default_hash(source_a_x_minus_1);
        let rolling_hash_b_x_minus_1 = build_default_hash(source_b_x_minus_1);
        let rolling_hash_a_x = build_default_hash(source_a_x);
        let rolling_hash_b_x = build_default_hash(source_b_x);
        let rolling_hash_a_x_plus_1 = build_default_hash(source_a_x_plus_1);
        let rolling_hash_b_x_plus_1 = build_default_hash(source_b_x_plus_1);

        // Act
        let actual_lcp_x_minus_1 =
            RollingHash::lcp(&rolling_hash_a_x_minus_1, &rolling_hash_b_x_minus_1, 0, 0);
        let actual_lcp_x = RollingHash::lcp(&rolling_hash_a_x, &rolling_hash_b_x, 0, 0);
        let actual_lcp_x_plus_1 =
            RollingHash::lcp(&rolling_hash_a_x_plus_1, &rolling_hash_b_x_plus_1, 0, 0);

        // Assert
        assert_eq!(actual_lcp_x_minus_1, expected_lcp_x_minus_1);
        assert_eq!(actual_lcp_x, expected_lcp_x);
        assert_eq!(actual_lcp_x_plus_1, expected_lcp_x_plus_1);
    }
}
