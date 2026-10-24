struct Kmp {
    dp: Vec<usize>,
    pattern: String,
}

impl Kmp {
    pub fn new(pattern: String) -> Self {
        let dp = Self::build_table(pattern.as_str());

        Self { dp, pattern }
    }

    fn build_table(s: &str) -> Vec<usize> {
        let s = s.as_bytes();
        let n = s.len();

        let mut dp = vec![0; n];

        for i in 1..n {
            let mut pos = dp[i - 1];

            while 0 < pos && s[i] != s[pos] {
                pos = dp[pos - 1];
            }

            if s[i] == s[pos] {
                pos += 1;
            }

            dp[i] = pos;
        }

        dp
    }

    fn table(&self) -> &Vec<usize> {
        self.dp.as_ref()
    }

    pub fn search(&self, target: String) -> Vec<usize> {
        let dp = self.table();

        let pattern = self.pattern.as_bytes();
        let target = target.as_bytes();

        let mut pattern_index = 0;
        let mut target_index = 0;
        let mut results = vec![];

        while let Some(&c) = target.get(target_index) {
            if c == pattern[pattern_index] {
                pattern_index += 1;
                target_index += 1;

                if pattern_index == pattern.len() {
                    results.push(target_index - pattern_index);
                    pattern_index = dp[pattern_index - 1];
                }
            } else {
                if 0 < pattern_index {
                    pattern_index = dp[pattern_index - 1];
                } else {
                    target_index += 1;
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {
    use super::Kmp;

    fn create_kmp(pattern: &str) -> Kmp {
        Kmp::new(pattern.to_string())
    }

    fn generate_binary_strings(max_length: usize) -> Vec<String> {
        let mut strings = vec![String::new()];

        for length in 1..=max_length {
            let number_of_strings = 1usize << length;

            for value in 0..number_of_strings {
                let string = (0..length)
                    .rev()
                    .map(|shift| {
                        if value & (1usize << shift) == 0 {
                            'a'
                        } else {
                            'b'
                        }
                    })
                    .collect();

                strings.push(string);
            }
        }

        strings
    }

    fn build_table_naively(pattern: &str) -> Vec<usize> {
        let pattern = pattern.as_bytes();

        (0..pattern.len())
            .map(|end| {
                let prefix_length = end + 1;

                (1..prefix_length)
                    .rev()
                    .find(|&candidate_length| {
                        pattern[..candidate_length]
                            == pattern[prefix_length - candidate_length..prefix_length]
                    })
                    .unwrap_or(0)
            })
            .collect()
    }

    fn search_naively(pattern: &str, target: &str) -> Vec<usize> {
        let pattern = pattern.as_bytes();
        let target = target.as_bytes();

        assert!(!pattern.is_empty());

        target
            .windows(pattern.len())
            .enumerate()
            .filter_map(|(index, window)| (window == pattern).then_some(index))
            .collect()
    }

    #[test]
    fn new_with_empty_pattern_creates_instance_with_empty_pattern_and_table() {
        // Arrange
        let pattern = "";
        let expected_pattern = pattern.to_string();
        let expected_table: Vec<usize> = vec![];

        // Act
        let kmp = create_kmp(pattern);
        let actual_pattern = kmp.pattern.clone();
        let actual_table = kmp.table().clone();

        // Assert
        assert_eq!(actual_pattern, expected_pattern);
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_with_empty_string_returns_empty_vector() {
        // Arrange
        let pattern = "";
        let expected_table: Vec<usize> = vec![];

        // Act
        let actual_table = Kmp::build_table(pattern);

        // Assert
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_with_single_character_returns_zero_table() {
        // Arrange
        let pattern = "a";
        let expected_table = vec![0usize];

        // Act
        let actual_table = Kmp::build_table(pattern);

        // Assert
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_with_all_distinct_characters_returns_all_zeroes() {
        // Arrange
        let pattern = "abcde";
        let expected_table = vec![0usize, 0, 0, 0, 0];

        // Act
        let actual_table = Kmp::build_table(pattern);

        // Assert
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_with_same_character_repetition_returns_growing_prefix_lengths() {
        // Arrange
        let pattern = "aaaaa";
        let expected_table = vec![0usize, 1, 2, 3, 4];

        // Act
        let actual_table = Kmp::build_table(pattern);

        // Assert
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_with_ababaca_returns_expected_partial_match_lengths() {
        // Arrange
        let pattern = "ababaca";
        let expected_table = vec![0usize, 0, 1, 2, 3, 0, 1];

        // Act
        let actual_table = Kmp::build_table(pattern);

        // Assert
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_with_fallback_and_shorter_border_match_returns_expected_lengths() {
        // Arrange
        let pattern = "aabaaab";
        let expected_table = vec![0usize, 1, 0, 1, 2, 2, 3];

        // Act
        let actual_table = Kmp::build_table(pattern);

        // Assert
        assert_eq!(actual_table, expected_table);
    }

    #[test]
    fn build_table_for_all_small_binary_patterns_matches_naive_implementation() {
        // Arrange
        let max_pattern_length = 8usize;
        let patterns = generate_binary_strings(max_pattern_length);
        let expected_tables = patterns
            .iter()
            .map(|pattern| build_table_naively(pattern))
            .collect::<Vec<_>>();

        // Act
        let actual_tables = patterns
            .iter()
            .map(|pattern| Kmp::build_table(pattern))
            .collect::<Vec<_>>();

        // Assert
        for ((pattern, actual_table), expected_table) in patterns
            .iter()
            .zip(actual_tables.iter())
            .zip(expected_tables.iter())
        {
            assert_eq!(
                actual_table, expected_table,
                "table differs for pattern {pattern:?}"
            );
        }
    }

    #[test]
    fn search_with_single_char_pattern_and_empty_target_returns_empty_positions() {
        // Arrange
        let pattern = "a";
        let target = "".to_string();
        let expected_positions: Vec<usize> = vec![];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_single_char_pattern_and_exact_match_returns_zero() {
        // Arrange
        let pattern = "a";
        let target = "a".to_string();
        let expected_positions = vec![0usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_single_char_pattern_and_suffix_match_returns_one() {
        // Arrange
        let pattern = "a";
        let target = "ba".to_string();
        let expected_positions = vec![1usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_single_char_pattern_and_no_match_returns_empty_positions() {
        // Arrange
        let pattern = "a";
        let target = "bbb".to_string();
        let expected_positions: Vec<usize> = vec![];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_single_char_pattern_and_repeated_target_returns_all_positions() {
        // Arrange
        let pattern = "a";
        let target = "aaaa".to_string();
        let expected_positions = vec![0usize, 1, 2, 3];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_non_matching_strings_returns_empty_positions() {
        // Arrange
        let pattern = "abc";
        let target = "def".to_string();
        let expected_positions: Vec<usize> = vec![];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_when_target_length_is_pattern_length_minus_one_returns_empty_positions() {
        // Arrange
        let pattern = "abc";
        let target = "ab".to_string();
        let expected_positions: Vec<usize> = vec![];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_when_target_length_equals_pattern_length_returns_single_match_at_zero() {
        // Arrange
        let pattern = "abc";
        let target = "abc".to_string();
        let expected_positions = vec![0usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_when_target_length_is_pattern_length_plus_one_returns_single_match_at_one() {
        // Arrange
        let pattern = "abc";
        let target = "xabc".to_string();
        let expected_positions = vec![1usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_finds_single_occurrences_at_start_middle_and_end_positions() {
        // Arrange
        let pattern = "abc";
        let target = "abcxabcxabc".to_string();
        let expected_positions = vec![0usize, 4, 8];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_multiple_non_overlapping_matches_returns_each_start_position() {
        // Arrange
        let pattern = "abc";
        let target = "abcxxabc".to_string();
        let expected_positions = vec![0usize, 5];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_overlapping_matches_returns_all_overlapping_start_positions() {
        // Arrange
        let pattern = "ana";
        let target = "banana".to_string();
        let expected_positions = vec![1usize, 3];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_prefix_suffix_overlap_pattern_keeps_match_after_failure_transition() {
        // Arrange
        let pattern = "ababac";
        let target = "ababababac".to_string();
        let expected_positions = vec![4usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_empty_pattern_and_empty_target_returns_empty_positions() {
        // Arrange
        let pattern = "";
        let target = "".to_string();
        let expected_positions: Vec<usize> = vec![];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_empty_pattern_and_non_empty_target_panics() {
        // Arrange
        let pattern = "";
        let target = "a".to_string();
        let kmp = create_kmp(pattern);

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| kmp.search(target)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn search_with_unicode_pattern_returns_byte_index_position() {
        // Arrange
        let pattern = "い";
        let target = "あいあ".to_string();
        let expected_positions = vec![3usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_is_case_sensitive_for_upper_and_lower_letters() {
        // Arrange
        let pattern = "abc";
        let target = "Abcabc".to_string();
        let expected_positions = vec![3usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_results_are_independent_across_multiple_calls_on_same_instance() {
        // Arrange
        let pattern = "aba";
        let first_target = "ababa".to_string();
        let second_target = "xxxx".to_string();
        let expected_first_positions = vec![0usize, 2];
        let expected_second_positions: Vec<usize> = vec![];
        let kmp = create_kmp(pattern);

        // Act
        let actual_first_positions = kmp.search(first_target);
        let actual_second_positions = kmp.search(second_target);

        // Assert
        assert_eq!(actual_first_positions, expected_first_positions);
        assert_eq!(actual_second_positions, expected_second_positions);
    }

    #[test]
    fn search_with_multiple_failure_transitions_reuses_current_target_byte() {
        // Arrange
        let pattern = "ababaca";
        let target = "ababaababaca".to_string();
        let expected_positions = vec![5usize];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_dense_overlapping_matches_returns_every_start_position() {
        // Arrange
        let pattern = "aaa";
        let target = "aaaaa".to_string();
        let expected_positions = vec![0usize, 1, 2];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_overlapping_multibyte_pattern_returns_byte_positions() {
        // Arrange
        let pattern = "あaあ";
        let target = "あaあaあ".to_string();
        let expected_positions = vec![0usize, 4];
        let kmp = create_kmp(pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_with_long_degenerate_input_returns_last_possible_position() {
        // Arrange
        let pattern_repetition = 10_000usize;
        let target_repetition = 100_000usize;
        let pattern = "a".repeat(pattern_repetition) + "b";
        let target = "a".repeat(target_repetition) + "b";
        let expected_positions = vec![90_000usize];
        let kmp = create_kmp(&pattern);

        // Act
        let actual_positions = kmp.search(target);

        // Assert
        assert_eq!(actual_positions, expected_positions);
    }

    #[test]
    fn search_for_all_small_binary_inputs_matches_naive_implementation() {
        // Arrange
        let max_pattern_length = 6usize;
        let max_target_length = 8usize;
        let patterns = generate_binary_strings(max_pattern_length)
            .into_iter()
            .filter(|pattern| !pattern.is_empty())
            .collect::<Vec<_>>();
        let targets = generate_binary_strings(max_target_length);
        let cases = patterns
            .iter()
            .flat_map(|pattern| {
                targets
                    .iter()
                    .map(move |target| (pattern.clone(), target.clone()))
            })
            .collect::<Vec<_>>();
        let expected_positions = cases
            .iter()
            .map(|(pattern, target)| search_naively(pattern, target))
            .collect::<Vec<_>>();

        // Act
        let actual_positions = cases
            .iter()
            .map(|(pattern, target)| create_kmp(pattern).search(target.clone()))
            .collect::<Vec<_>>();

        // Assert
        for (((pattern, target), actual), expected) in cases
            .iter()
            .zip(actual_positions.iter())
            .zip(expected_positions.iter())
        {
            assert_eq!(
                actual, expected,
                "positions differ for pattern {pattern:?} and target {target:?}"
            );
        }
    }
}
