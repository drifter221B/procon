use std::collections::{HashMap, VecDeque};

#[derive(Default)]
struct Node {
    children: HashMap<char, usize>,
    fail: usize,
    outputs: Vec<usize>,
    terminal_outputs: Vec<usize>,
}

pub struct AhoCorasick {
    nodes: Vec<Node>,
    pattern_lengths: Vec<usize>,
}

impl Default for AhoCorasick {
    fn default() -> Self {
        Self::new()
    }
}

impl AhoCorasick {

    pub fn new() -> Self {
        Self {
            nodes: vec![Node::default()],
            pattern_lengths: Vec::new(),
        }
    }

    pub fn insert(&mut self, words: &[String]) {

        for word in words {

            let id = self.pattern_lengths.len();
            let word_len = word.chars().count();
            self.pattern_lengths.push(word_len);

            if word_len == 0 {
                continue;
            }

            let mut state = 0;
            for ch in word.chars() {
                if !self.nodes[state].children.contains_key(&ch) {
                    let new_idx = self.nodes.len();
                    self.nodes.push(Node::default());
                    self.nodes[state].children.insert(ch, new_idx);
                }
                state = *self.nodes[state].children.get(&ch).expect("child just inserted or existed");
            }

            self.nodes[state].outputs.push(id);
            self.nodes[state].terminal_outputs.push(id);
        }
    }

    pub fn build_links(&mut self) {
        for node in &mut self.nodes {
            node.fail = 0;
            node.outputs = node.terminal_outputs.clone();
        }

        let mut queue = VecDeque::new();

        let root_children: Vec<usize> = self.nodes[0].children.values().copied().collect();
        for next_state in root_children {
            self.nodes[next_state].fail = 0;
            queue.push_back(next_state);
        }

        while let Some(state) = queue.pop_front() {
            let children: Vec<(char, usize)> = self.nodes[state]
                .children
                .iter()
                .map(|(&ch, &next)| (ch, next))
                .collect();

            for (ch, child_state) in children {
                let mut fail = self.nodes[state].fail;
                while fail != 0 && !self.nodes[fail].children.contains_key(&ch) {
                    fail = self.nodes[fail].fail;
                }

                let fail_state = if let Some(&next) = self.nodes[fail].children.get(&ch) {
                    next
                } else {
                    0
                };

                self.nodes[child_state].fail = fail_state;
                let outputs = self.nodes[fail_state].outputs.clone();
                self.nodes[child_state].outputs.extend(outputs);

                queue.push_back(child_state);
            }
        }
    }

    pub fn find_matches(&self, text: &str) -> Vec<(usize, usize)> {
        let mut state = 0;
        let mut results = Vec::new();

        for (i, ch) in text.chars().enumerate() {
            let mut current = state;
            while current != 0 && !self.nodes[current].children.contains_key(&ch) {
                current = self.nodes[current].fail;
            }

            if let Some(&next) = self.nodes[current].children.get(&ch) {
                state = next;
            } else {
                state = 0;
            }

            for &pattern_id in &self.nodes[state].outputs {
                let len = self.pattern_lengths[pattern_id];
                if i + 1 >= len {
                    results.push((i + 1 - len, pattern_id));
                }
            }
        }

        results
    }
}

#[cfg(test)]
mod tests {

    use super::AhoCorasick;

    fn create_pattern_strings(pattern_values: &[&str]) -> Vec<String> {
        pattern_values
            .iter()
            .map(|value| (*value).to_string())
            .collect()
    }

    fn build_automaton_with_patterns(pattern_values: &[&str]) -> AhoCorasick {
        let mut automaton = AhoCorasick::new();
        let pattern_strings = create_pattern_strings(pattern_values);
        automaton.insert(&pattern_strings);
        automaton.build_links();
        automaton
    }

    fn create_automaton_without_links(pattern_values: &[&str]) -> AhoCorasick {
        let mut automaton = AhoCorasick::new();
        let pattern_strings = create_pattern_strings(pattern_values);
        automaton.insert(&pattern_strings);
        automaton
    }

    fn build_automaton_from_string_patterns(pattern_values: &[String]) -> AhoCorasick {
        let mut automaton = AhoCorasick::new();
        automaton.insert(pattern_values);
        automaton.build_links();
        automaton
    }

    fn naive_find_matches(pattern_values: &[String], text: &str) -> Vec<(usize, usize)> {
        let text_chars: Vec<char> = text.chars().collect();
        let mut naive_matches = Vec::new();

        for (pattern_id, pattern_value) in pattern_values.iter().enumerate() {
            let pattern_chars: Vec<char> = pattern_value.chars().collect();

            if pattern_chars.is_empty() {
                continue;
            }

            if text_chars.len() < pattern_chars.len() {
                continue;
            }

            let max_start_index = text_chars.len() - pattern_chars.len();

            for start_index in 0..=max_start_index {
                let end_index = start_index + pattern_chars.len();
                if text_chars[start_index..end_index] == pattern_chars[..] {
                    naive_matches.push((start_index, pattern_id));
                }
            }
        }

        naive_matches
    }

    fn sorted_matches(mut pattern_matches: Vec<(usize, usize)>) -> Vec<(usize, usize)> {
        pattern_matches.sort_unstable();
        pattern_matches
    }

    struct LinearCongruentialGenerator {
        state: u64,
    }

    impl LinearCongruentialGenerator {
        fn new(seed_value: u64) -> Self {
            Self { state: seed_value }
        }

        fn next_u64(&mut self) -> u64 {
            self.state = self
                .state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1442695040888963407);
            self.state
        }

        fn next_usize_below(&mut self, upper_exclusive: usize) -> usize {
            if upper_exclusive == 0 {
                0
            } else {
                (self.next_u64() as usize) % upper_exclusive
            }
        }
    }

    fn generate_random_text(
        random_generator: &mut LinearCongruentialGenerator,
        text_len: usize,
        alphabet_values: &[char],
    ) -> String {
        (0..text_len)
            .map(|_| {
                let alphabet_index = random_generator.next_usize_below(alphabet_values.len());
                alphabet_values[alphabet_index]
            })
            .collect()
    }

    #[test]
    fn new_find_matches_for_empty_text_returns_empty_vector() {
        // Arrange
        let text = "";
        let expected_matches: Vec<(usize, usize)> = Vec::new();
        let automaton = AhoCorasick::new();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn rebuild_after_adding_suffix_updates_existing_node_outputs() {
        // Arrange
        let initial_pattern_values = vec!["she".to_string()];
        let additional_pattern_values = vec!["he".to_string()];
        let text = "she";
        let expected_matches = vec![(0usize, 0usize), (1usize, 1usize)];
        let mut automaton = AhoCorasick::new();
        automaton.insert(&initial_pattern_values);
        automaton.build_links();
        automaton.insert(&additional_pattern_values);
        automaton.build_links();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn insert_with_empty_patterns_then_build_links_returns_no_matches() {
        // Arrange
        let pattern_values: Vec<String> = Vec::new();
        let text = "abc";
        let expected_matches: Vec<(usize, usize)> = Vec::new();
        let mut automaton = AhoCorasick::new();
        automaton.insert(&pattern_values);
        automaton.build_links();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn single_pattern_after_build_links_returns_one_expected_match() {
        // Arrange
        let pattern_values = ["abc"];
        let text = "abc";
        let expected_matches = vec![(0usize, 0usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn find_matches_without_build_links_fixes_spec_as_direct_outputs_only() {
        // Arrange
        let pattern_values = ["he", "she"];
        let text = "she";
        let expected_matches = vec![(0usize, 1usize)];
        let automaton = create_automaton_without_links(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn build_links_twice_keeps_results_without_duplicate_matches() {
        // Arrange
        let pattern_values = ["ab", "b"];
        let text = "ab";
        let expected_matches = vec![(0usize, 0usize), (1usize, 1usize)];
        let mut automaton = AhoCorasick::new();
        let pattern_strings = create_pattern_strings(&pattern_values);
        automaton.insert(&pattern_strings);
        automaton.build_links();
        automaton.build_links();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn insert_after_build_links_changes_behavior_before_and_after_rebuild() {
        // Arrange
        let first_pattern_values = vec!["he".to_string()];
        let second_pattern_values = vec!["she".to_string()];
        let text = "she";
        let expected_before_rebuild = vec![(0usize, 1usize)];
        let expected_after_rebuild = vec![(0usize, 1usize), (1usize, 0usize)];
        let mut automaton = AhoCorasick::new();
        automaton.insert(&first_pattern_values);
        automaton.build_links();
        automaton.insert(&second_pattern_values);

        // Act
        let actual_before_rebuild = automaton.find_matches(text);
        automaton.build_links();
        let actual_after_rebuild = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_before_rebuild, expected_before_rebuild);
        assert_eq!(actual_after_rebuild, expected_after_rebuild);
    }

    #[test]
    fn default_instance_can_insert_build_and_find_match() {
        // Arrange
        let pattern_values = vec!["a".to_string()];
        let text = "a";
        let expected_matches = vec![(0usize, 0usize)];
        let mut automaton = AhoCorasick::default();
        automaton.insert(&pattern_values);
        automaton.build_links();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn rebuild_after_reinserting_existing_terminal_keeps_each_output_once() {
        // Arrange
        let initial_pattern_values = vec!["b".to_string(), "ab".to_string()];
        let additional_pattern_values = vec!["ab".to_string()];
        let text = "ab";
        let expected_matches = vec![(0usize, 1usize), (0usize, 2usize), (1usize, 0usize)];
        let mut automaton = AhoCorasick::new();
        automaton.insert(&initial_pattern_values);
        automaton.build_links();
        automaton.insert(&additional_pattern_values);
        automaton.build_links();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn build_links_follows_multiple_failures_to_root_child() {
        // Arrange
        let pattern_values = ["aaax", "x"];
        let text = "aaax";
        let expected_matches = vec![(0usize, 0usize), (3usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn find_matches_consumes_same_character_after_multiple_failures() {
        // Arrange
        let pattern_values = ["abcd", "bcd", "cdx"];
        let text = "abcdx";
        let expected_matches = vec![(0usize, 0usize), (1usize, 1usize), (2usize, 2usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn find_matches_is_case_sensitive() {
        // Arrange
        let pattern_values = ["abc", "ABC"];
        let text = "AbcABCabc";
        let expected_matches = vec![(3usize, 1usize), (6usize, 0usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn consecutive_find_matches_calls_do_not_share_search_state() {
        // Arrange
        let pattern_values = ["ab", "bc"];
        let first_text = "ab";
        let second_text = "bc";
        let expected_first_matches = vec![(0usize, 0usize)];
        let expected_second_matches = vec![(0usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_first_matches = automaton.find_matches(first_text);
        let actual_second_matches = automaton.find_matches(second_text);

        // Assert
        assert_eq!(actual_first_matches, expected_first_matches);
        assert_eq!(actual_second_matches, expected_second_matches);
    }

    #[test]
    fn find_matches_matches_naive_for_empty_single_and_multiple_pattern_sets() {
        // Arrange
        let empty_pattern_values: Vec<String> = Vec::new();
        let empty_text = "abc";
        let single_pattern_values = create_pattern_strings(&["ab"]);
        let single_text = "ababa";
        let multiple_pattern_values = create_pattern_strings(&["he", "she", "his", "hers"]);
        let multiple_text = "ahishers";
        let expected_empty = sorted_matches(naive_find_matches(&empty_pattern_values, empty_text));
        let expected_single =
            sorted_matches(naive_find_matches(&single_pattern_values, single_text));
        let expected_multiple =
            sorted_matches(naive_find_matches(&multiple_pattern_values, multiple_text));
        let empty_automaton = build_automaton_from_string_patterns(&empty_pattern_values);
        let single_automaton = build_automaton_from_string_patterns(&single_pattern_values);
        let multiple_automaton = build_automaton_from_string_patterns(&multiple_pattern_values);

        // Act
        let actual_empty = sorted_matches(empty_automaton.find_matches(empty_text));
        let actual_single = sorted_matches(single_automaton.find_matches(single_text));
        let actual_multiple = sorted_matches(multiple_automaton.find_matches(multiple_text));

        // Assert
        assert_eq!(actual_empty, expected_empty);
        assert_eq!(actual_single, expected_single);
        assert_eq!(actual_multiple, expected_multiple);
    }

    #[test]
    fn find_matches_matches_naive_for_deterministic_random_patterns_and_texts() {
        // Arrange
        let seed_value = 0x0123_4567_89ab_cdefu64;
        let trial_count = 80usize;
        let max_pattern_count = 6usize;
        let max_pattern_len = 4usize;
        let max_text_len = 12usize;
        let alphabet_values = ['a', 'b', 'c'];
        let expected_all_trials_match = true;
        let mut random_generator = LinearCongruentialGenerator::new(seed_value);
        let mut cases = Vec::with_capacity(trial_count);
        for _trial_index in 0..trial_count {
            let pattern_count = 1usize + random_generator.next_usize_below(max_pattern_count);
            let mut pattern_values: Vec<String> = Vec::with_capacity(pattern_count);

            for _pattern_index in 0..pattern_count {
                let pattern_len = 1usize + random_generator.next_usize_below(max_pattern_len);
                let pattern_value =
                    generate_random_text(&mut random_generator, pattern_len, &alphabet_values);
                pattern_values.push(pattern_value);
            }

            let text_len = random_generator.next_usize_below(max_text_len + 1usize);
            let text = generate_random_text(&mut random_generator, text_len, &alphabet_values);
            let expected_matches = sorted_matches(naive_find_matches(&pattern_values, &text));
            cases.push((pattern_values, text, expected_matches));
        }
        let expected_matches_by_case: Vec<Vec<(usize, usize)>> = cases
            .iter()
            .map(|(_, _, expected_matches)| expected_matches.clone())
            .collect();

        // Act
        let actual_matches_by_case: Vec<Vec<(usize, usize)>> = cases
            .iter()
            .map(|(pattern_values, text, _)| {
                let automaton = build_automaton_from_string_patterns(&pattern_values);
                sorted_matches(automaton.find_matches(text))
            })
            .collect();

        // Assert
        assert_eq!(
            actual_matches_by_case == expected_matches_by_case,
            expected_all_trials_match
        );
    }

    #[test]
    fn insert_empty_string_pattern_fixes_spec_as_ignored_for_matching() {
        // Arrange
        let pattern_values = [""];
        let text = "abc";
        let expected_matches: Vec<(usize, usize)> = Vec::new();
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn patterns_with_shared_prefix_match_without_breaking_fail_transitions() {
        // Arrange
        let pattern_values = ["ab", "abc"];
        let text = "xabc";
        let expected_matches = vec![(1usize, 0usize), (1usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn after_mismatch_fallback_to_root_and_resume_matching_next_characters() {
        // Arrange
        let pattern_values = ["ab", "bc"];
        let text = "abxbc";
        let expected_matches = vec![(0usize, 0usize), (3usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn empty_text_returns_no_matches_even_when_patterns_are_registered() {
        // Arrange
        let pattern_values = ["a", "ab", "abc"];
        let text = "";
        let expected_matches: Vec<(usize, usize)> = Vec::new();
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn overlapping_matches_are_not_missed_in_repeated_text_region() {
        // Arrange
        let pattern_values = ["ana"];
        let text = "banana";
        let expected_matches = vec![(1usize, 0usize), (3usize, 0usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn prefix_related_patterns_return_multiple_matches_at_same_end_position() {
        // Arrange
        let pattern_values = ["a", "aa", "aaa"];
        let text = "aaa";
        let expected_matches = vec![
            (0usize, 0usize),
            (0usize, 1usize),
            (1usize, 0usize),
            (0usize, 2usize),
            (1usize, 1usize),
            (2usize, 0usize),
        ];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn non_matching_text_returns_empty_vector() {
        // Arrange
        let pattern_values = ["ab", "bc"];
        let text = "dddd";
        let expected_matches: Vec<(usize, usize)> = Vec::new();
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn duplicate_pattern_registration_returns_two_distinct_pattern_ids() {
        // Arrange
        let pattern_values = ["aba", "aba"];
        let text = "aba";
        let expected_matches = vec![(0usize, 0usize), (0usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn mix_of_empty_and_non_empty_patterns_keeps_non_empty_pattern_id_alignment() {
        // Arrange
        let pattern_values = ["", "ab", ""];
        let text = "zab";
        let expected_matches = vec![(1usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn multiple_insert_calls_accumulate_registered_patterns() {
        // Arrange
        let first_pattern_values = vec!["ab".to_string()];
        let second_pattern_values = vec!["bc".to_string()];
        let text = "abc";
        let expected_matches = vec![(0usize, 0usize), (1usize, 1usize)];
        let mut automaton = AhoCorasick::new();
        automaton.insert(&first_pattern_values);
        automaton.insert(&second_pattern_values);
        automaton.build_links();

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn suffix_related_patterns_inherit_outputs_through_fail_links() {
        // Arrange
        let pattern_values = ["he", "she", "hers"];
        let text = "ushers";
        let expected_matches = vec![(1usize, 1usize), (2usize, 0usize), (2usize, 2usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn pattern_length_boundary_with_text_minus_one_equal_and_plus_one_behaves_correctly() {
        // Arrange
        let pattern_values = ["abc", "abcd", "abcde"];
        let text = "abcd";
        let expected_matches = vec![(0usize, 0usize), (0usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn single_pattern_occurring_multiple_times_returns_all_start_indices() {
        // Arrange
        let pattern_values = ["abc"];
        let text = "abcxabcabc";
        let expected_matches = vec![(0usize, 0usize), (4usize, 0usize), (7usize, 0usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn unicode_text_returns_character_based_start_index() {
        // Arrange
        let pattern_values = ["いあ"];
        let text = "あいあ";
        let expected_matches = vec![(1usize, 0usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }

    #[test]
    fn same_end_position_pattern_ids_are_returned_in_defined_order() {
        // Arrange
        let pattern_values = ["ab", "b"];
        let text = "ab";
        let expected_matches = vec![(0usize, 0usize), (1usize, 1usize)];
        let automaton = build_automaton_with_patterns(&pattern_values);

        // Act
        let actual_matches = automaton.find_matches(text);

        // Assert
        assert_eq!(actual_matches, expected_matches);
    }
}
