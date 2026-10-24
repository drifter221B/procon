/// 与えられた文字列の接尾辞のうち、辞書順で最大／最小となる開始位置を返す。
/// 空文字列の場合は 0 を返す。
use std::cmp::Ordering;

#[derive(Clone, Copy)]
enum SubstrOrder {
    Largest,
    Smallest,
}

fn substr_index(s: &str, order: SubstrOrder) -> usize {
    let bytes = s.as_bytes();
    if bytes.is_empty() {
        return 0;
    }

    let mut result_index = 0usize;

    for candidate_index in 1..bytes.len() {
        let ordering = bytes[candidate_index..].cmp(&bytes[result_index..]);
        let is_better = match order {
            SubstrOrder::Largest => ordering == Ordering::Greater,
            SubstrOrder::Smallest => ordering == Ordering::Less,
        };

        if is_better {
            result_index = candidate_index;
        }
    }

    result_index
}

pub fn largest_substr_index(s: &str) -> usize {
    substr_index(s, SubstrOrder::Largest)
}

pub fn smallest_substr_index(s: &str) -> usize {
    substr_index(s, SubstrOrder::Smallest)
}

#[cfg(test)]
mod tests {
    use super::{largest_substr_index, smallest_substr_index};

    #[test]
    fn largest_substring_of_same_characters_is_zero() {
        // Arrange
        let source = "aaaa";
        let expected_value = 0;

        // Act
        let actual_value = largest_substr_index(source);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn same_character_suffixes_return_first_largest_and_last_smallest_indices() {
        // Arrange
        let source = "aaaaa";
        let expected_values = (0, source.len() - 1);

        // Act
        let actual_values = (largest_substr_index(source), smallest_substr_index(source));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn empty_single_and_two_character_inputs_return_expected_indices() {
        // Arrange
        let sources = ["", "z", "az"];
        let expected_values = [(0, 0), (0, 0), (1, 0)];

        // Act
        let actual_values =
            sources.map(|source| (largest_substr_index(source), smallest_substr_index(source)));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn finds_expected_indices_for_banana_variation() {
        // Arrange
        let source = "banana";
        let expected_values = (2, 5);

        // Act
        let actual_values = (largest_substr_index(source), smallest_substr_index(source));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn prefers_later_suffix_when_prefixes_match() {
        // Arrange
        let source = "ababa";
        let expected_values = (1, 4);

        // Act
        let actual_values = (largest_substr_index(source), smallest_substr_index(source));

        // Assert
        // 接頭部分が一致しても、より大きい接尾辞を正しく選ぶ
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn utf8_input_returns_byte_index_with_current_behavior() {
        // Arrange
        let source = "aあ";
        let expected_values = (1, 0);

        // Act
        let actual_values = (largest_substr_index(source), smallest_substr_index(source));

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
