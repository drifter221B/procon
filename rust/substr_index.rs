#[derive(Clone, Copy)]
enum SubstringKind {
    Smallest,
    Largest,
}

fn substr_index(s: &str, kind: SubstringKind) -> usize {
    let bytes = s.as_bytes();

    (0..bytes.len())
        .reduce(|best_index, candidate_index| {
            if is_better_suffix(bytes, candidate_index, best_index, kind) {
                candidate_index
            } else {
                best_index
            }
        })
        .unwrap_or(0)
}

fn is_better_suffix(
    bytes: &[u8],
    candidate_index: usize,
    best_index: usize,
    kind: SubstringKind,
) -> bool {
    match kind {
        SubstringKind::Smallest => bytes[candidate_index..] < bytes[best_index..],
        SubstringKind::Largest => bytes[best_index..] < bytes[candidate_index..],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn brute_force_substr_index(s: &str, kind: SubstringKind) -> usize {
        let bytes = s.as_bytes();

        if bytes.is_empty() {
            return 0;
        }

        let mut best_index = 0usize;
        for index in 1..bytes.len() {
            let ordering = bytes[index..].cmp(&bytes[best_index..]);
            let should_update = match kind {
                SubstringKind::Smallest => ordering == std::cmp::Ordering::Less,
                SubstringKind::Largest => ordering == std::cmp::Ordering::Greater,
            };

            if should_update {
                best_index = index;
            }
        }

        best_index
    }

    // “banana” のすべての接尾辞を列挙すると、
    // 0: "banana"
    // 1: "anana"
    // 2: "nana"
    // 3: "ana"
    // 4: "na"
    // 5: "a"
    // これらを辞書順で見ると、
    //   Smallest: "a"      → index = 5
    //   Largest:  "nana"   → index = 2
    #[test]
    fn substr_index_banana_smallest_suffix_returns_index_five() {
        // Arrange
        let s = "banana";
        let kind = SubstringKind::Smallest;
        let expected_index = 5;

        // Act
        let actual_index = substr_index(s, kind);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn substr_index_banana_largest_suffix_returns_index_two() {
        // Arrange
        let s = "banana";
        let kind = SubstringKind::Largest;
        let expected_index = 2;

        // Act
        let actual_index = substr_index(s, kind);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    // すべて同一文字からなる文字列では、どの接尾辞をとっても同じ文字列なので
    // 返り値は常に 0（最初の接尾辞）になる
    #[test]
    fn substr_index_all_equal_chars_smallest_returns_final_index() {
        // Arrange
        let s = "aaaaa";
        let kind = SubstringKind::Smallest;
        let expected_index = s.len() - 1;

        // Act
        let actual_index = substr_index(s, kind);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    #[test]
    fn substr_index_all_equal_chars_largest_returns_zero() {
        // Arrange
        let s = "aaaaa";
        let kind = SubstringKind::Largest;
        let expected_index = 0;

        // Act
        let actual_index = substr_index(s, kind);

        // Assert
        assert_eq!(actual_index, expected_index);
    }

    // 文字列長が 1 の場合、常に index=0
    #[test]
    fn substr_index_single_char_returns_zero_for_both_kinds() {
        // Arrange
        let s = "Z";
        let kinds = [SubstringKind::Smallest, SubstringKind::Largest];
        let expected_indices = [0, 0];

        // Act
        let actual_indices = kinds.map(|kind| substr_index(s, kind));

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    // 空文字列の場合、while ループに入らず result=0 が返る
    #[test]
    fn substr_index_empty_string_returns_zero_for_both_kinds() {
        // Arrange
        let s = "";
        let kinds = [SubstringKind::Smallest, SubstringKind::Largest];
        let expected_indices = [0, 0];

        // Act
        let actual_indices = kinds.map(|kind| substr_index(s, kind));

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    // “abcd” の接尾辞は ["abcd","bcd","cd","d"]
    // 小さい辞書順 (Smallest): "abcd" → index=0
    // 大きい辞書順 (Largest):   "d"    → index=3
    #[test]
    fn substr_index_ascending_chars_returns_first_and_final_indices() {
        // Arrange
        let s = "abcd";
        let kinds = [SubstringKind::Smallest, SubstringKind::Largest];
        let expected_indices = [0, 3];

        // Act
        let actual_indices = kinds.map(|kind| substr_index(s, kind));

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    // “cba” の接尾辞は ["cba","ba","a"]
    // 小さい辞書順: "a"   → index=2
    // 大きい辞書順: "cba" → index=0
    #[test]
    fn substr_index_descending_chars_returns_final_and_first_indices() {
        // Arrange
        let s = "cba";
        let kinds = [SubstringKind::Smallest, SubstringKind::Largest];
        let expected_indices = [2, 0];

        // Act
        let actual_indices = kinds.map(|kind| substr_index(s, kind));

        // Assert
        assert_eq!(actual_indices, expected_indices);
    }

    #[test]
    fn abab_prefix_related_suffixes_match_bruteforce() {
        // Arrange
        let s = "abab";
        let smallest_kind = SubstringKind::Smallest;
        let largest_kind = SubstringKind::Largest;
        let expected_smallest = brute_force_substr_index(s, smallest_kind);
        let expected_largest = brute_force_substr_index(s, largest_kind);

        // Act
        let actual_smallest = substr_index(s, smallest_kind);
        let actual_largest = substr_index(s, largest_kind);

        // Assert
        assert_eq!(actual_smallest, expected_smallest);
        assert_eq!(actual_largest, expected_largest);
    }

    #[test]
    fn aaabaaa_repeated_character_run_matches_bruteforce() {
        // Arrange
        let s = "aaabaaa";
        let smallest_kind = SubstringKind::Smallest;
        let largest_kind = SubstringKind::Largest;
        let expected_smallest = brute_force_substr_index(s, smallest_kind);
        let expected_largest = brute_force_substr_index(s, largest_kind);

        // Act
        let actual_smallest = substr_index(s, smallest_kind);
        let actual_largest = substr_index(s, largest_kind);

        // Assert
        assert_eq!(actual_smallest, expected_smallest);
        assert_eq!(actual_largest, expected_largest);
    }

    #[test]
    fn ascii_uppercase_and_lowercase_follow_byte_order() {
        // Arrange
        let s = "aA";
        let smallest_kind = SubstringKind::Smallest;
        let largest_kind = SubstringKind::Largest;
        let expected_smallest = brute_force_substr_index(s, smallest_kind);
        let expected_largest = brute_force_substr_index(s, largest_kind);

        // Act
        let actual_smallest = substr_index(s, smallest_kind);
        let actual_largest = substr_index(s, largest_kind);

        // Assert
        assert_eq!(actual_smallest, expected_smallest);
        assert_eq!(actual_largest, expected_largest);
    }

    #[test]
    fn utf8_string_uses_byte_suffix_order() {
        // Arrange
        let s = "あいあ";
        let smallest_kind = SubstringKind::Smallest;
        let largest_kind = SubstringKind::Largest;
        let expected_smallest = brute_force_substr_index(s, smallest_kind);
        let expected_largest = brute_force_substr_index(s, largest_kind);

        // Act
        let actual_smallest = substr_index(s, smallest_kind);
        let actual_largest = substr_index(s, largest_kind);

        // Assert
        assert_eq!(actual_smallest, expected_smallest);
        assert_eq!(actual_largest, expected_largest);
    }

    #[test]
    fn multiple_cases_match_bruteforce_suffix_comparison() {
        // Arrange
        let cases = vec!["banana", "ababa", "zzza", "mississippi"];
        let smallest_kind = SubstringKind::Smallest;
        let largest_kind = SubstringKind::Largest;
        let expected_results = cases
            .iter()
            .map(|s| {
                (
                    brute_force_substr_index(s, smallest_kind),
                    brute_force_substr_index(s, largest_kind),
                )
            })
            .collect::<Vec<_>>();

        // Act
        let actual_results = cases
            .iter()
            .map(|s| {
                (
                    substr_index(s, smallest_kind),
                    substr_index(s, largest_kind),
                )
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_results, expected_results);
    }
}
