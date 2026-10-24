pub struct RollingHash {
    size: usize,
    modulo: u64,
    base_mod: Vec<u64>,
    hash_list: Vec<u64>,
}

impl RollingHash {

    pub fn new(s: &str, base: u64, modulo: u64) -> Self {

        let length = s.len();
        let mut base_mod = vec![1u64; length + 1];
        let mut hash_list = vec![1u64; length + 1];

        for (i, c) in s.bytes().enumerate() {
            base_mod[i + 1] = Self::multiple(base_mod[i], base, modulo);
            hash_list[i + 1] = Self::add(Self::multiple(hash_list[i], base, modulo), Self::subtract(1 + c as u64, b'a' as u64, modulo), modulo);
        }

        RollingHash {
            size: s.len(),
            modulo,
            base_mod,
            hash_list,
        }
    }

    const fn add(x: u64, y: u64, modulo: u64) -> u64 {
        (x + y) % modulo
    }

    const fn subtract(x: u64, y: u64, modulo: u64) -> u64 {
        (modulo + x - y) % modulo
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
        Self::subtract(self.hash_list[right], Self::multiple(self.hash_list[left], self.base_mod[right - left], self.modulo), self.modulo)
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
        let expected_length = source_text.as_bytes().len();

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

        // Assert
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

        // Assert
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
