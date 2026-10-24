/// `source` を繰り返し連結した文字列から、`target` を部分列として取り出すのに
/// 必要な `source` の出現回数を返す。小文字 a-z のみを想定し、構築できない場合は -1 を返す。
pub fn generate_string(source: &str, target: &str) -> i64 {

    if source.is_empty() {
        return if target.is_empty() { 1 } else { -1 };
    }

    let source_bytes = source.as_bytes();
    let source_len = source_bytes.len();

    let mut dp = vec![vec![source_len; 26]; source_len + 1];
    let base = b'a';

    for i in (0..source_len).rev() {
        let (head, tail) = dp.split_at_mut(i + 1);
        head[i].copy_from_slice(&tail[0]);
        let index = (source_bytes[i] - base) as usize;
        dp[i][index] = i;
    }

    let mut index = 0usize;
    let mut source_counter = 1i64;

    for &c in target.as_bytes() {

        let next = (c - base) as usize;

        if dp[index][next] == source_len {
            index = 0;
            source_counter += 1;
        }

        if dp[index][next] == source_len {
            return -1;
        }

        index = dp[index][next] + 1;
    }

    source_counter
}

#[cfg(test)]
mod tests {
    use super::generate_string;

    use std::panic::{catch_unwind, AssertUnwindSafe};

    #[test]
    fn generate_string_with_empty_target_returns_one_for_empty_or_nonempty_source() {
        // Arrange
        let sources = ["", "abc"];
        let target = "";
        let expected_counts = [1, 1];
        // Act
        let actual_counts = sources.map(|source| generate_string(source, target));
        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn returns_minus_one_when_required_character_is_absent() {
        // Arrange
        let source = "abc";
        let target = "abd";
        let expected_count = -1;
        // Act
        let actual_count = generate_string(source, target);
        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn counts_repetitions_at_wrap_boundaries() {
        // Arrange
        let source = "ab";
        let targets = ["ab", "aba", "ababa"];
        let expected_counts = [1, 2, 3];
        // Act
        let actual_counts = targets.map(|target| generate_string(source, target));
        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn increments_counter_when_scanning_from_middle_of_source() {
        // Arrange
        let source = "abc";
        let target = "cabc";
        let expected_count = 2;
        // Act
        let actual_count = generate_string(source, target);
        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn repeating_last_character_increments_wrap_count() {
        // Arrange
        let source = "abc";
        let target = "ccc";
        let expected_value = 3;

        // Act
        let actual_value = generate_string(source, target);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn single_character_source_counts_repetitions() {
        // Arrange
        let source = "a";
        let target = "aaaa";
        let expected_value = 4;

        // Act
        let actual_value = generate_string(source, target);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn non_lowercase_input_panics_with_current_behavior() {
        // Arrange
        let source = "aA";
        let target = "A";

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| generate_string(source, target)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn generate_string_with_empty_source_and_nonempty_target_returns_minus_one() {
        // Arrange
        let source = "";
        let target = "a";
        let expected_count = -1;
        // Act
        let actual_count = generate_string(source, target);
        // Assert
        assert_eq!(actual_count, expected_count);
    }

}
