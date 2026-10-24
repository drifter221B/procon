use std::cmp::Ordering;

pub struct MinimalRotation {
    chars: Vec<char>,
    start_index: usize,
}

impl MinimalRotation {
    pub fn new(s: &str) -> Self {
        let chars: Vec<char> = s.chars().collect();
        let start_index = Self::find_start_index(&chars);

        Self { chars, start_index }
    }

    pub fn start_index(&self) -> usize {
        self.start_index
    }

    pub fn iter(&self) -> impl Iterator<Item = &char> {
        let (prefix, suffix) = self.chars.split_at(self.start_index());
        suffix.iter().chain(prefix.iter())
    }

    fn find_start_index(chars: &[char]) -> usize {
        let len = chars.len();
        if len < 2 {
            return 0;
        }

        let mut first = 0;
        let mut second = 1;
        let mut matched = 0;

        while first < len && second < len && matched < len {
            match chars[(first + matched) % len].cmp(&chars[(second + matched) % len]) {
                Ordering::Equal => matched += 1,
                Ordering::Greater => {
                    first += matched + 1;
                    if first == second {
                        first += 1;
                    }
                    matched = 0;
                }
                Ordering::Less => {
                    second += matched + 1;
                    if first == second {
                        second += 1;
                    }
                    matched = 0;
                }
            }
        }

        first.min(second)
    }
}

#[cfg(test)]
mod tests {
    use super::MinimalRotation;

    #[test]
    fn empty_string_returns_empty_rotation_at_zero() {
        // Arrange
        let input = "";
        let expected = (String::new(), 0);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn single_character_returns_same_rotation_at_zero() {
        // Arrange
        let input = "z";
        let expected = (String::from("z"), 0);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn ordinary_string_returns_smallest_rotation() {
        // Arrange
        let input = "caba";
        let expected = (String::from("abac"), 1);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn identical_characters_choose_first_start() {
        // Arrange
        let input = "aaaa";
        let expected = (String::from("aaaa"), 0);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn periodic_string_chooses_first_minimum_start() {
        // Arrange
        let input = "baba";
        let expected = (String::from("abab"), 1);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn alfa_finds_minimum_across_string_boundary() {
        // Arrange
        let input = "alfa";
        let expected = (String::from("aalf"), 3);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn aaba_preserves_candidate_after_repeated_prefix() {
        // Arrange
        let input = "aaba";
        let expected = (String::from("aaab"), 3);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn binary_string_finds_minimum_after_repeated_prefix() {
        // Arrange
        let input = "0010";
        let expected = (String::from("0001"), 3);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn unicode_string_uses_character_positions() {
        // Arrange
        let input = "😀いあ";
        let expected = (String::from("あ😀い"), 2);

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = (rotation.iter().collect::<String>(), rotation.start_index());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_as_str_is_accepted() {
        // Arrange
        let input = String::from("bca");
        let expected = String::from("abc");

        // Act
        let rotation = MinimalRotation::new(input.as_str());
        let actual = rotation.iter().collect::<String>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn borrowed_string_is_accepted() {
        // Arrange
        let input = String::from("bca");
        let expected = String::from("abc");

        // Act
        let rotation = MinimalRotation::new(&input);
        let actual = rotation.iter().collect::<String>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn string_slice_is_accepted() {
        // Arrange
        let input: &str = "bca";
        let expected = String::from("abc");

        // Act
        let rotation = MinimalRotation::new(input);
        let actual = rotation.iter().collect::<String>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn exhaustive_small_strings_match_all_rotations_and_first_start() {
        let alphabet = ['a', 'b', 'c'];
        let min_length = 1;
        let max_length = 8;

        for len in min_length..=max_length {
            let case_count = alphabet.len().pow(len);
            for mut encoded in 0..case_count {
                // Arrange
                let chars: Vec<char> = (0..len)
                    .map(|_| {
                        let character = alphabet[encoded % alphabet.len()];
                        encoded /= alphabet.len();
                        character
                    })
                    .collect();
                let input: String = chars.iter().collect();
                let expected = (0..chars.len())
                    .map(|start| {
                        let rotated: String =
                            chars[start..].iter().chain(chars[..start].iter()).collect();
                        (rotated, start)
                    })
                    .min()
                    .expect("all generated strings are nonempty");

                // Act
                let rotation = MinimalRotation::new(&input);
                let actual = (rotation.iter().collect::<String>(), rotation.start_index());

                // Assert
                assert_eq!(actual, expected, "input: {input:?}");
            }
        }
    }
}
