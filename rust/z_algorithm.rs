/// pattern match               -> kmp
/// prefix of one word          -> z
/// prefix of multiple words    -> trie
pub fn z_algorithm(s: &str) -> Vec<usize> {

    if s.is_empty() {
        return vec![];
    }

    let s = s.as_bytes();
    let n = s.len();

    let mut z = vec![0; n];
    z[0] = n;

    // [left, right)
    let mut left = 0;
    let mut right = 0;

    for i in 1..n {

        let mut match_length = if i < right {

            // s[left..right) = s[0..(right - left)]
            let tmp_left = i - left;
            let rest_length = right - i;

            std::cmp::min(z[tmp_left], rest_length)
        } else {
            0
        };

        while i + match_length < n && s[match_length] == s[i + match_length] {
            match_length += 1;
        }

        z[i] = match_length;

        if 0 < match_length {

            let new_right = i + match_length;
            if right < new_right {
                left = i;
                right = new_right;
            }

        }

    }

    z
}

#[cfg(test)]
mod tests {
    use super::z_algorithm;

    #[derive(Clone, Copy)]
    struct ZScanEvent {
        is_inside_window: bool,
        copied_match_length: usize,
        remaining_window_length: usize,
        match_length: usize,
        right_before_update: usize,
        right_after_update: usize,
    }

    fn naive_z_algorithm(input_text: &str) -> Vec<usize> {

        if input_text.is_empty() {
            return vec![];
        }

        let input_bytes = input_text.as_bytes();
        let byte_length = input_bytes.len();
        let mut naive_z_values = vec![0usize; byte_length];
        naive_z_values[0] = byte_length;

        for start_index in 1..byte_length {
            let mut matched_length = 0usize;
            while start_index + matched_length < byte_length
                && input_bytes[matched_length] == input_bytes[start_index + matched_length]
            {
                matched_length += 1;
            }
            naive_z_values[start_index] = matched_length;
        }

        naive_z_values
    }

    fn analyze_z_scan_events(input_text: &str) -> Vec<ZScanEvent> {

        if input_text.is_empty() {
            return vec![];
        }

        let input_bytes = input_text.as_bytes();
        let byte_length = input_bytes.len();
        let mut z_values = vec![0usize; byte_length];
        z_values[0] = byte_length;

        let mut left_index = 0usize;
        let mut right_index = 0usize;
        let mut scan_events = Vec::with_capacity(byte_length - 1);

        for position_index in 1..byte_length {
            let right_before_update = right_index;

            let mut is_inside_window = false;
            let mut copied_match_length = 0usize;
            let mut remaining_window_length = 0usize;
            let mut match_length = 0usize;

            if position_index < right_index {
                is_inside_window = true;
                let left_offset = position_index - left_index;
                remaining_window_length = right_index - position_index;
                copied_match_length = std::cmp::min(z_values[left_offset], remaining_window_length);
                match_length = copied_match_length;
            }

            while position_index + match_length < byte_length
                && input_bytes[match_length] == input_bytes[position_index + match_length]
            {
                match_length += 1;
            }

            z_values[position_index] = match_length;

            if 0 < match_length {
                let new_right_index = position_index + match_length;
                if right_index < new_right_index {
                    left_index = position_index;
                    right_index = new_right_index;
                }
            }

            let right_after_update = right_index;
            scan_events.push(ZScanEvent {
                is_inside_window,
                copied_match_length,
                remaining_window_length,
                match_length,
                right_before_update,
                right_after_update,
            });
        }

        scan_events
    }

    fn generate_deterministic_texts(max_length: usize, samples_per_length: usize) -> Vec<String> {
        let alphabet_bytes = [b'a', b'b', b'c', b'd'];
        let mut pseudo_random_state = 0x1234_5678_9abc_def0u64;
        let mut generated_texts = Vec::new();

        for text_length in 0..=max_length {
            for _sample_index in 0..samples_per_length {
                let mut text_bytes = Vec::with_capacity(text_length);
                for _byte_index in 0..text_length {
                    pseudo_random_state = pseudo_random_state
                        .wrapping_mul(6364136223846793005)
                        .wrapping_add(1);
                    let alphabet_index = (pseudo_random_state % alphabet_bytes.len() as u64) as usize;
                    let selected_byte = alphabet_bytes[alphabet_index];
                    text_bytes.push(selected_byte);
                }
                let generated_text = String::from_utf8(text_bytes).expect("ASCII bytes only");
                generated_texts.push(generated_text);
            }
        }

        generated_texts
    }

    #[test]
    fn z_algorithm_empty_input_returns_empty_vector() {
        // Arrange
        let input_text = "";
        let expected_z_values: Vec<usize> = vec![];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_single_character_input_returns_only_self_length() {
        // Arrange
        let input_text = "a";
        let expected_z_values = vec![1usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_two_equal_characters_returns_full_and_one() {
        // Arrange
        let input_text = "aa";
        let expected_z_values = vec![2usize, 1usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_two_different_characters_returns_full_and_zero() {
        // Arrange
        let input_text = "ab";
        let expected_z_values = vec![2usize, 0usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_all_equal_characters_returns_decreasing_sequence() {
        // Arrange
        let input_text = "aaaaa";
        let expected_z_values = vec![5usize, 4usize, 3usize, 2usize, 1usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_all_distinct_characters_returns_zeros_except_first() {
        // Arrange
        let input_text = "abcde";
        let expected_z_values = vec![5usize, 0usize, 0usize, 0usize, 0usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_overlapping_prefix_case_returns_expected_values() {
        // Arrange
        let input_text = "ababa";
        let expected_z_values = vec![5usize, 0usize, 3usize, 0usize, 1usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_non_overlapping_reoccurrence_case_returns_expected_values() {
        // Arrange
        let input_text = "abcab";
        let expected_z_values = vec![5usize, 0usize, 0usize, 2usize, 0usize];

        // Act
        let actual_z_values = z_algorithm(input_text);

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
    }

    #[test]
    fn z_algorithm_inside_window_reuse_case_detects_shorter_copied_match() {
        // Arrange
        let input_text = "aabaaab";
        let expected_z_values = vec![7usize, 1usize, 0usize, 2usize, 3usize, 1usize, 0usize];
        let expected_case_exists = true;
        let scan_events = analyze_z_scan_events(input_text);

        // Act
        let actual_z_values = z_algorithm(input_text);
        let actual_case_exists = scan_events.iter().any(|scan_event| {
            scan_event.is_inside_window
                && scan_event.copied_match_length < scan_event.remaining_window_length
        });

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
        assert_eq!(actual_case_exists, expected_case_exists);
    }

    #[test]
    fn z_algorithm_inside_window_extension_case_detects_equal_copied_and_remaining_lengths() {
        // Arrange
        let input_text = "aabaaab";
        let expected_z_values = vec![7usize, 1usize, 0usize, 2usize, 3usize, 1usize, 0usize];
        let expected_case_exists = true;
        let scan_events = analyze_z_scan_events(input_text);

        // Act
        let actual_z_values = z_algorithm(input_text);
        let actual_case_exists = scan_events.iter().any(|scan_event| {
            scan_event.is_inside_window
                && scan_event.copied_match_length == scan_event.remaining_window_length
                && scan_event.remaining_window_length < scan_event.match_length
        });

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
        assert_eq!(actual_case_exists, expected_case_exists);
    }

    #[test]
    fn z_algorithm_zero_match_case_keeps_next_position_result_consistent() {
        // Arrange
        let input_text = "abaaba";
        let expected_z_values = vec![6usize, 0usize, 1usize, 3usize, 0usize, 1usize];
        let expected_zero_match_without_update_exists = true;
        let expected_next_position_value = 1usize;
        let scan_events = analyze_z_scan_events(input_text);
        let next_position_index = 5usize;

        // Act
        let actual_z_values = z_algorithm(input_text);
        let actual_zero_match_without_update_exists = scan_events.iter().any(|scan_event| {
            scan_event.match_length == 0usize
                && scan_event.right_before_update == scan_event.right_after_update
        });
        let actual_next_position_value = actual_z_values[next_position_index];

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
        assert_eq!(
            actual_zero_match_without_update_exists,
            expected_zero_match_without_update_exists
        );
        assert_eq!(actual_next_position_value, expected_next_position_value);
    }

    #[test]
    fn z_algorithm_positive_match_updates_keep_farthest_right_boundary() {
        // Arrange
        let input_text = "aaaaaaaa";
        let expected_z_values = vec![8usize, 7usize, 6usize, 5usize, 4usize, 3usize, 2usize, 1usize];
        let expected_positive_update_exists = true;
        let expected_positive_no_update_exists = true;
        let scan_events = analyze_z_scan_events(input_text);

        // Act
        let actual_z_values = z_algorithm(input_text);
        let actual_positive_update_exists = scan_events.iter().any(|scan_event| {
            0usize < scan_event.match_length
                && scan_event.right_before_update < scan_event.right_after_update
        });
        let actual_positive_no_update_exists = scan_events.iter().any(|scan_event| {
            0usize < scan_event.match_length
                && scan_event.right_before_update == scan_event.right_after_update
        });

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
        assert_eq!(actual_positive_update_exists, expected_positive_update_exists);
        assert_eq!(
            actual_positive_no_update_exists,
            expected_positive_no_update_exists
        );
    }

    #[test]
    fn z_algorithm_non_empty_inputs_have_first_value_equal_to_byte_length() {
        // Arrange
        let input_texts = vec!["a", "ab", "ababa", "aabaaab", "ああ"];
        let expected_first_values: Vec<usize> = input_texts.iter().map(|input_text| input_text.len()).collect();

        // Act
        let actual_first_values: Vec<usize> = input_texts
            .iter()
            .map(|input_text| {
                let actual_z_values = z_algorithm(input_text);
                actual_z_values[0]
            })
            .collect();

        // Assert
        assert_eq!(actual_first_values, expected_first_values);
    }

    #[test]
    fn z_algorithm_each_position_value_is_bounded_by_remaining_suffix_length() {
        // Arrange
        let input_texts = vec!["a", "ab", "aaaaa", "ababa", "aabaaab", "あああ"];
        let expected_all_positions_within_bound = true;

        // Act
        let actual_all_positions_within_bound = input_texts.iter().all(|input_text| {
            let input_byte_length = input_text.len();
            let actual_z_values = z_algorithm(input_text);
            actual_z_values
                .iter()
                .enumerate()
                .all(|(position_index, &match_length)| match_length <= input_byte_length - position_index)
        });

        // Assert
        assert_eq!(
            actual_all_positions_within_bound,
            expected_all_positions_within_bound
        );
    }

    #[test]
    fn z_algorithm_utf8_input_is_processed_by_byte_length() {
        // Arrange
        let input_text = "あああ";
        let expected_z_values = vec![9usize, 0usize, 0usize, 6usize, 0usize, 0usize, 3usize, 0usize, 0usize];
        let expected_byte_length = input_text.len();
        let expected_character_count = input_text.chars().count();
        let expected_is_byte_length_based = true;

        // Act
        let actual_z_values = z_algorithm(input_text);
        let actual_result_length = actual_z_values.len();
        let actual_first_value = actual_z_values[0];
        let actual_is_byte_length_based = actual_result_length == expected_byte_length
            && actual_result_length != expected_character_count
            && actual_first_value == expected_byte_length;

        // Assert
        assert_eq!(actual_z_values, expected_z_values);
        assert_eq!(actual_is_byte_length_based, expected_is_byte_length_based);
    }

    #[test]
    fn z_algorithm_matches_naive_implementation_for_deterministic_random_inputs() {
        // Arrange
        let max_length = 32usize;
        let samples_per_length = 8usize;
        let generated_texts = generate_deterministic_texts(max_length, samples_per_length);
        let expected_values = generated_texts
            .iter()
            .map(|input_text| (input_text, naive_z_algorithm(input_text)))
            .collect::<Vec<_>>();

        // Act
        let actual_values = generated_texts
            .iter()
            .map(|input_text| (input_text, z_algorithm(input_text)))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
