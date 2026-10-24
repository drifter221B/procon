/// Manacher 法により、各中心を持つ最長回文の半径を管理する。
///
/// `odd_radii()[i] == r` のとき、`i` を中心とする最長の奇数長回文は
/// 半開区間 `[i - r + 1, i + r)` であり、その長さは `2 * r - 1` となる。
///
/// `even_radii()[i] == r` のとき、`i - 1` と `i` の間を中心とする最長の
/// 偶数長回文は半開区間 `[i - r, i + r)` であり、その長さは `2 * r` となる。
/// 両方の配列長は入力要素数 `n` に等しい。非空入力では全ての奇数長半径が
/// 1 以上となり、先頭より前の境界を表す `even_radii()[0]` は 0 となる。
/// 末尾より後ろの境界 `n` は半径が自明に 0 であるため、配列には格納しない。
///
/// [`Manacher::new`] は文字列を Unicode scalar value (`char`) 単位で扱う。
/// 書記素クラスタ単位ではなく、Unicode 正規化も行わない。また、結果の添字は
/// 元の `str` のバイトオフセットではない。UTF-8 バイト単位で処理する場合は
/// `Manacher::from_slice(text.as_bytes())` を使用する。
///
/// 空入力では、奇数長と偶数長の半径配列はいずれも空になる。
///
/// 構築時間は `O(n)`、保持する領域は `O(n)` である。
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Manacher {
    odd_radii: Vec<usize>,
    even_radii: Vec<usize>,
}

impl Manacher {
    /// 文字列の回文半径を Unicode scalar value (`char`) 単位で構築する。
    ///
    /// 書記素クラスタへの分割と Unicode 正規化は行わない。結果の添字は
    /// `str` のバイトオフセットではない。文字数を `n` としたとき、構築時間と
    /// 一時領域はともに `O(n)` である。空文字列では両方の半径配列が空になる。
    pub fn new(text: &str) -> Self {
        let characters = text.chars().collect::<Vec<char>>();
        Self::from_slice(&characters)
    }

    /// スライスの各要素を一つの記号として回文半径を構築する。
    ///
    /// 要素数を `n` としたとき、構築時間は `O(n)`、結果以外の追加領域は
    /// `O(1)` である。空スライスでは両方の半径配列が空になる。
    pub fn from_slice<T: Eq>(values: &[T]) -> Self {
        Self {
            odd_radii: Self::calculate_odd_radii(values),
            even_radii: Self::calculate_even_radii(values),
        }
    }

    /// 各要素を中心とする最長奇数長回文の半径を返す。
    ///
    /// 配列長は入力要素数 `n` に等しい。`result[i] == r` が表す半開区間は
    /// `[i - r + 1, i + r)` であり、非空入力では全ての半径が 1 以上となる。
    pub fn odd_radii(&self) -> &[usize] {
        &self.odd_radii
    }

    /// 各要素の直前を中心とする最長偶数長回文の半径を返す。
    ///
    /// 配列長は入力要素数 `n` に等しい。`result[i] == r` が表す半開区間は
    /// `[i - r, i + r)` であり、`result[0]` は 0 となる。末尾より後ろの境界
    /// `n` は半径が自明に 0 であるため、配列には格納しない。
    pub fn even_radii(&self) -> &[usize] {
        &self.even_radii
    }

    /// 奇数長回文、偶数長回文の順で半径配列を返す。
    pub fn into_radii(self) -> (Vec<usize>, Vec<usize>) {
        (self.odd_radii, self.even_radii)
    }

    /// 入力の要素数を返す。
    ///
    /// [`Manacher::new`] で構築した場合は Unicode scalar value の個数であり、
    /// `str` のバイト数ではない。
    pub fn len(&self) -> usize {
        debug_assert_eq!(self.odd_radii.len(), self.even_radii.len());
        self.odd_radii.len()
    }

    /// 入力が空であるかを返す。
    pub fn is_empty(&self) -> bool {
        self.odd_radii.is_empty()
    }

    fn calculate_odd_radii<T: Eq>(values: &[T]) -> Vec<usize> {
        let length = values.len();
        let mut odd_radii = vec![0usize; length];
        let mut left = 0usize;
        let mut right = 0usize;

        for center in 0..length {
            let mut radius = if center < right {
                debug_assert!(left <= center);
                let mirrored_center = left + (right - 1 - center);
                std::cmp::min(odd_radii[mirrored_center], right - center)
            } else {
                1
            };

            while radius <= center
                && radius < length - center
                && values[center - radius] == values[center + radius]
            {
                radius += 1;
            }

            odd_radii[center] = radius;
            let candidate_right = center + radius;
            if right < candidate_right {
                left = center + 1 - radius;
                right = candidate_right;
            }
        }

        odd_radii
    }

    fn calculate_even_radii<T: Eq>(values: &[T]) -> Vec<usize> {
        let length = values.len();
        let mut even_radii = vec![0usize; length];
        let mut left = 0usize;
        let mut right = 0usize;

        for center in 0..length {
            let mut radius = if center < right {
                debug_assert!(left <= center);
                let mirrored_center = left + (right - center);
                debug_assert!(mirrored_center < center);
                std::cmp::min(even_radii[mirrored_center], right - center)
            } else {
                0
            };

            while radius < center
                && radius < length - center
                && values[center - radius - 1] == values[center + radius]
            {
                radius += 1;
            }

            even_radii[center] = radius;
            let candidate_right = center + radius;
            if right < candidate_right {
                left = center - radius;
                right = candidate_right;
            }
        }

        even_radii
    }
}

#[cfg(test)]
mod tests {
    use super::Manacher;
    use std::cell::Cell;
    use std::rc::Rc;

    #[derive(Debug, Eq, PartialEq)]
    struct EqOnlyValue(u8);

    #[derive(Debug)]
    struct CountingEqValue {
        value: u8,
        comparison_count: Rc<Cell<usize>>,
    }

    impl PartialEq for CountingEqValue {
        fn eq(&self, other: &Self) -> bool {
            self.comparison_count.set(self.comparison_count.get() + 1);
            self.value == other.value
        }
    }

    impl Eq for CountingEqValue {}

    fn calculate_naive_radii<T: Eq>(values: &[T]) -> (Vec<usize>, Vec<usize>) {
        let length = values.len();
        let mut odd_radii = vec![0usize; length];
        let mut even_radii = vec![0usize; length];

        for center in 0..length {
            let mut odd_radius = 1usize;
            while odd_radius <= center
                && odd_radius < length - center
                && values[center - odd_radius] == values[center + odd_radius]
            {
                odd_radius += 1;
            }
            odd_radii[center] = odd_radius;

            let mut even_radius = 0usize;
            while even_radius < center
                && even_radius < length - center
                && values[center - even_radius - 1] == values[center + even_radius]
            {
                even_radius += 1;
            }
            even_radii[center] = even_radius;
        }

        (odd_radii, even_radii)
    }

    #[test]
    fn new_with_empty_text_returns_empty_radii() {
        // Arrange
        let text = "";

        let expected_manacher_odd_radii_right = &[];
        let expected_manacher_even_radii_right = &[];
        let expected_manacher_len_right = 0;

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();
        let actual_manacher_len = manacher.len();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
        assert_eq!(actual_manacher_len, expected_manacher_len_right);
        assert!(manacher.is_empty());
    }

    #[test]
    fn new_with_one_character_returns_unit_odd_radius() {
        // Arrange
        let text = "a";

        let expected_manacher_odd_radii_right = &[1];
        let expected_manacher_even_radii_right = &[0];
        let expected_manacher_len_right = 1;

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();
        let actual_manacher_len = manacher.len();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
        assert_eq!(actual_manacher_len, expected_manacher_len_right);
        assert!(!manacher.is_empty());
    }

    #[test]
    fn new_with_two_equal_characters_finds_even_palindrome() {
        // Arrange
        let text = "aa";

        let expected_manacher_odd_radii_right = &[1, 1];
        let expected_manacher_even_radii_right = &[0, 1];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_two_different_characters_returns_zero_even_radii() {
        // Arrange
        let text = "ab";

        let expected_manacher_odd_radii_right = &[1, 1];
        let expected_manacher_even_radii_right = &[0, 0];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_aba_finds_centered_odd_palindrome() {
        // Arrange
        let text = "aba";

        let expected_manacher_odd_radii_right = &[1, 2, 1];
        let expected_manacher_even_radii_right = &[0, 0, 0];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_abba_finds_centered_even_palindrome() {
        // Arrange
        let text = "abba";

        let expected_manacher_odd_radii_right = &[1, 1, 1, 1];
        let expected_manacher_even_radii_right = &[0, 0, 2, 0];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_abacaba_finds_nested_odd_palindromes() {
        // Arrange
        let text = "abacaba";

        let expected_manacher_odd_radii_right = &[1, 2, 1, 4, 1, 2, 1];
        let expected_manacher_even_radii_right = &[0, 0, 0, 0, 0, 0, 0];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_all_equal_characters_finds_boundary_limited_palindromes() {
        // Arrange
        let text = "aaaa";

        let expected_manacher_odd_radii_right = &[1, 2, 2, 1];
        let expected_manacher_even_radii_right = &[0, 1, 2, 1];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_unicode_text_uses_unicode_scalar_values() {
        // Arrange
        let text = "あいいあ";

        let expected_manacher_len_right = 4;
        let expected_manacher_odd_radii_right = &[1, 1, 1, 1];
        let expected_manacher_even_radii_right = &[0, 0, 2, 0];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_len = manacher.len();
        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_len, expected_manacher_len_right);
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_with_japanese_odd_palindrome_finds_unicode_scalar_radius() {
        // Arrange
        let text = "あいういあ";

        let expected_manacher_odd_radii_right = &[1, 1, 3, 1, 1];
        let expected_manacher_even_radii_right = &[0, 0, 0, 0, 0];

        // Act
        let manacher = Manacher::new(text);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn new_matches_from_slice_of_unicode_scalar_values() {
        // Arrange
        let text = "aあいいあa";
        let characters = text.chars().collect::<Vec<char>>();

        // Act
        let from_text = Manacher::new(text);
        let from_characters = Manacher::from_slice(&characters);

        // Assert
        assert_eq!(from_text, from_characters);
    }

    #[test]
    fn from_slice_with_utf8_bytes_uses_different_radius_units_from_new() {
        // Arrange
        let text = "あいいあ";

        // Act
        let character_manacher = Manacher::new(text);
        let byte_manacher = Manacher::from_slice(text.as_bytes());

        let actual_character_manacher_len = character_manacher.len();
        let actual_character_manacher_len_right = text.chars().count();
        let actual_byte_manacher_len = byte_manacher.len();
        let actual_byte_manacher_len_right = text.len();
        let actual_character_manacher_len_2 = character_manacher.len();
        let actual_character_manacher_len_right_2 = byte_manacher.len();

        // Assert
        assert_eq!(
            actual_character_manacher_len,
            actual_character_manacher_len_right
        );
        assert_eq!(actual_byte_manacher_len, actual_byte_manacher_len_right);
        assert_ne!(
            actual_character_manacher_len_2,
            actual_character_manacher_len_right_2
        );
    }

    #[test]
    fn from_slice_accepts_values_that_are_eq_but_not_clone() {
        // Arrange
        let values = [EqOnlyValue(1), EqOnlyValue(2), EqOnlyValue(1)];

        let expected_manacher_odd_radii_right = &[1, 2, 1];
        let expected_manacher_even_radii_right = &[0, 0, 0];

        // Act
        let manacher = Manacher::from_slice(&values);

        let actual_manacher_odd_radii = manacher.odd_radii();
        let actual_manacher_even_radii = manacher.even_radii();

        // Assert
        assert_eq!(actual_manacher_odd_radii, expected_manacher_odd_radii_right);
        assert_eq!(
            actual_manacher_even_radii,
            expected_manacher_even_radii_right
        );
    }

    #[test]
    fn from_slice_uses_a_linear_number_of_comparisons_for_long_all_equal_sequence() {
        // Arrange
        let length = 10_000usize;
        let comparison_count = Rc::new(Cell::new(0usize));
        let values = (0..length)
            .map(|_| CountingEqValue {
                value: 1,
                comparison_count: Rc::clone(&comparison_count),
            })
            .collect::<Vec<CountingEqValue>>();
        let comparison_upper_bound = 8 * length;

        // Act
        let manacher = Manacher::from_slice(&values);
        let actual_comparison_count = comparison_count.get();

        let actual_manacher_len = manacher.len();

        // Assert
        assert_eq!(actual_manacher_len, length);
        assert!(actual_comparison_count <= comparison_upper_bound);
    }

    #[test]
    fn from_slice_matches_naive_radii_for_all_ternary_sequences_up_to_length_eight() {
        // Arrange
        let maximum_length = 8usize;
        let alphabet_size = 3usize;
        let expected_input_count = 9_841usize;
        let mut inputs = Vec::new();
        for length in 0..=maximum_length {
            let sequence_count = alphabet_size.pow(length as u32);
            for sequence_code in 0..sequence_count {
                let mut remaining_code = sequence_code;
                let mut values = vec![0u8; length];
                for value in &mut values {
                    *value = (remaining_code % alphabet_size) as u8;
                    remaining_code /= alphabet_size;
                }
                inputs.push(values);
            }
        }
        assert_eq!(inputs.len(), expected_input_count);
        let expected_radii = inputs
            .iter()
            .map(|values| calculate_naive_radii(values))
            .collect::<Vec<(Vec<usize>, Vec<usize>)>>();

        // Act
        let actual_radii = inputs
            .iter()
            .map(|values| Manacher::from_slice(values).into_radii())
            .collect::<Vec<(Vec<usize>, Vec<usize>)>>();

        // Assert
        for ((input, actual), expected) in inputs.iter().zip(&actual_radii).zip(&expected_radii) {
            assert_eq!(actual, expected, "input: {:?}", input);
        }
    }

    #[test]
    fn into_radii_returns_odd_then_even_radii_without_changing_values() {
        // Arrange
        let manacher = Manacher::new("abba");

        let expected_odd_radii_right = vec![1, 1, 1, 1];
        let expected_even_radii_right = vec![0, 0, 2, 0];

        // Act
        let (odd_radii, even_radii) = manacher.into_radii();

        // Assert
        assert_eq!(odd_radii, expected_odd_radii_right);
        assert_eq!(even_radii, expected_even_radii_right);
    }

    #[test]
    fn clone_preserves_radii_and_equals_original() {
        // Arrange
        let text = "aba";
        let manacher = Manacher::new(text);

        // Act
        let cloned_manacher = manacher.clone();

        // Assert
        assert_eq!(cloned_manacher, manacher);
    }

    #[test]
    fn debug_of_nonempty_manacher_contains_type_name() {
        // Arrange
        let text = "aba";
        let manacher = Manacher::new(text);
        let expected_type_name = "Manacher";

        // Act
        let debug_text = format!("{manacher:?}");

        // Assert
        assert!(debug_text.contains(expected_type_name));
    }

}
