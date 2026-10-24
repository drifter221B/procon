pub fn int_to_roman(num: i32) -> String {

    let mut result = String::new();
    let mut tmp = num;

    let values = vec![1000, 900, 500, 400, 100, 90, 50, 40, 10, 9, 5, 4, 1];
    let symbols = vec!["M",  "CM", "D",  "CD", "C",  "XC", "L", "XL", "X",  "IX", "V",  "IV", "I"];
    let m = values.len();
    let n = symbols.len();

    assert_eq!(n, m);

    for i in 0..n {

        while values[i] <= tmp {
            tmp -= values[i];
            result.push_str(symbols[i]);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::int_to_roman;

    #[test]
    fn converts_basic_values() {
        // Arrange
        let cases = [(1, "I"), (4, "IV"), (9, "IX"), (58, "LVIII")];
        let expected_values = ["I", "IV", "IX", "LVIII"];

        // Act
        let actual_values = cases.map(|(input, _)| int_to_roman(input));

        // Assert
        assert_eq!(actual_values, expected_values.map(String::from));
    }

    #[test]
    fn one_thousand_returns_single_thousands_symbol() {
        // Arrange
        let input = 1000;
        let expected_value = "M";

        // Act
        let actual_value = int_to_roman(input);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn nineteen_ninety_four_uses_multiple_subtractive_pairs() {
        // Arrange
        let input = 1994;
        let expected_value = "MCMXCIV";

        // Act
        let actual_value = int_to_roman(input);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn subtractive_boundaries_are_converted() {
        // Arrange
        let inputs = [
            3, 4, 5, 8, 9, 10, 39, 40, 41, 89, 90, 91, 399, 400, 401, 899, 900, 901,
        ];
        let expected_values = [
            "III", "IV", "V", "VIII", "IX", "X", "XXXIX", "XL", "XLI", "LXXXIX", "XC", "XCI",
            "CCCXCIX", "CD", "CDI", "DCCCXCIX", "CM", "CMI",
        ];

        // Act
        let actual_values = inputs.map(int_to_roman);

        // Assert
        assert_eq!(actual_values, expected_values.map(String::from));
    }

    #[test]
    fn upper_bound_example_3999_is_converted() {
        // Arrange
        let input = 3999;
        let expected_value = "MMMCMXCIX".to_string();

        // Act
        let actual_value = int_to_roman(input);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn non_positive_inputs_return_empty_string_with_current_behavior() {
        // Arrange
        let inputs = (0, -1);
        let expected_values = (String::new(), String::new());

        // Act
        let actual_values = (int_to_roman(inputs.0), int_to_roman(inputs.1));

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
