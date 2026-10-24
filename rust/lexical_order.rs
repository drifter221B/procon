fn lexical_order(limit: usize) -> Vec<usize> {

    let mut cur = 1;
    let mut results = vec![];

    for _ in 0..limit {

        results.push(cur);

        if cur * 10 <= limit {
            cur *= 10;
        } else {

            while cur % 10 == 9 || limit <= cur {
                cur /= 10;
            }

            cur += 1;
        }
    }

    results
}

#[cfg(test)]
mod tests {
    use super::lexical_order;

    #[test]
    fn generates_sequence_in_lexicographical_order() {
        // Arrange
        let single_limit = 1;
        let multi_limit = 13;
        let expected_values = (vec![1], vec![1, 10, 11, 12, 13, 2, 3, 4, 5, 6, 7, 8, 9]);

        // Act
        let actual_values = (lexical_order(single_limit), lexical_order(multi_limit));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn limit_zero_returns_empty_vector() {
        // Arrange
        let limit = 0;
        let expected_values: Vec<usize> = vec![];

        // Act
        let actual_values = lexical_order(limit);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn boundaries_nine_ten_eleven_follow_lexicographical_order() {
        // Arrange
        let limits = (9, 10, 11);
        let expected_values = (
            vec![1, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![1, 10, 2, 3, 4, 5, 6, 7, 8, 9],
            vec![1, 10, 11, 2, 3, 4, 5, 6, 7, 8, 9],
        );

        // Act
        let actual_values = (
            lexical_order(limits.0),
            lexical_order(limits.1),
            lexical_order(limits.2),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn values_ending_with_nine_roll_over_correctly() {
        // Arrange
        let limits = (19, 109);
        let expected_19 = vec![
            1, 10, 11, 12, 13, 14, 15, 16, 17, 18, 19, 2, 3, 4, 5, 6, 7, 8, 9,
        ];
        let expected_109_prefix = vec![1, 10, 100, 101, 102, 103, 104, 105, 106, 107, 108, 109, 11];

        // Act
        let actual_19 = lexical_order(limits.0);
        let actual_109 = lexical_order(limits.1);

        // Assert
        assert_eq!(actual_19, expected_19);
        assert_eq!(actual_109[..expected_109_prefix.len()], expected_109_prefix);
    }
}
