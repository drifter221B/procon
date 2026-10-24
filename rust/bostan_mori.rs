mod modint;
use modint::ModInt;

fn multiply_polynomials(first: &[ModInt], second: &[ModInt]) -> Vec<ModInt> {
    assert!(!first.is_empty(), "first polynomial must not be empty");
    assert!(!second.is_empty(), "second polynomial must not be empty");

    let modulus = first[0].modulus();
    let mut result = vec![ModInt::new(0, modulus); first.len() + second.len() - 1];

    for (i, &first_value) in first.iter().enumerate() {
        for (j, &second_value) in second.iter().enumerate() {
            let index = i + j;
            result[index] += first_value * second_value;
        }
    }

    result
}

fn create_alternating_sign_denominator(denominator: &[ModInt]) -> Vec<ModInt> {
    denominator
        .iter()
        .enumerate()
        .map(|(index, coefficient)| {
            if index % 2 == 0 {
                *coefficient
            } else {
                -*coefficient
            }
        })
        .collect()
}

fn pick_stride_coefficients(polynomial: &[ModInt], take_even: bool) -> Vec<ModInt> {
    let start_index = if take_even { 0 } else { 1 };
    polynomial.iter().skip(start_index).step_by(2).copied().collect()
}

fn pick_stride_coefficients_or_zero(polynomial: &[ModInt], take_even: bool) -> Vec<ModInt> {
    let stride_coefficients = pick_stride_coefficients(polynomial, take_even);

    if !stride_coefficients.is_empty() {
        return stride_coefficients;
    }

    vec![ModInt::new(0, polynomial[0].modulus())]
}

pub fn bostan_mori(mut numerator: Vec<ModInt>, mut denominator: Vec<ModInt>, mut index: usize) -> ModInt {
    assert!(!numerator.is_empty(), "numerator must not be empty");
    assert!(!denominator.is_empty(), "denominator must not be empty");
    assert!(denominator[0].value() != 0, "denominator constant term must be non-zero");

    while 0 < index {
        let alternating_denominator = create_alternating_sign_denominator(&denominator);
        let numerator_convolved = multiply_polynomials(&numerator, &alternating_denominator);
        let denominator_convolved = multiply_polynomials(&denominator, &alternating_denominator);

        let take_even = index % 2 == 0;
        numerator = pick_stride_coefficients_or_zero(&numerator_convolved, take_even);
        denominator = pick_stride_coefficients(&denominator_convolved, true);
        index >>= 1;
    }

    numerator[0] / denominator[0]
}

#[cfg(test)]
mod tests {
    use std::panic;

    use super::{bostan_mori, create_alternating_sign_denominator, multiply_polynomials, pick_stride_coefficients, ModInt};

    fn create_modint_vector(values: &[i64], modulus: u32) -> Vec<ModInt> {
        values
            .iter()
            .copied()
            .map(|value| ModInt::new(value, modulus))
            .collect()
    }

    fn extract_modint_values(values: &[ModInt]) -> Vec<u32> {
        values.iter().map(|value| value.value()).collect()
    }

    fn calculate_naive_rational_coefficient(numerator: &[ModInt], denominator: &[ModInt], target_index: usize) -> ModInt {
        let modulus = numerator[0].modulus();
        let mut coefficient_values = vec![ModInt::new(0, modulus); target_index + 1];

        for index in 0..=target_index {
            let mut current_numerator_coefficient = if index < numerator.len() {
                numerator[index]
            } else {
                ModInt::new(0, modulus)
            };

            let denominator_end = denominator.len().min(index + 1);
            for denominator_index in 1..denominator_end {
                current_numerator_coefficient -= denominator[denominator_index] * coefficient_values[index - denominator_index];
            }

            coefficient_values[index] = current_numerator_coefficient / denominator[0];
        }

        coefficient_values[target_index]
    }

    fn create_deterministic_random_coefficients(mut random_state: u64, coefficient_count: usize) -> Vec<i64> {
        let mut coefficient_values = Vec::with_capacity(coefficient_count);

        for _ in 0..coefficient_count {
            random_state = random_state
                .wrapping_mul(6364136223846793005)
                .wrapping_add(1);
            let coefficient_value = ((random_state >> 32) % 41) as i64 - 20;
            coefficient_values.push(coefficient_value);
        }

        coefficient_values
    }

    #[test]
    fn multiply_polynomials_panics_when_first_polynomial_is_empty() {
        // Arrange
        let first_polynomial: Vec<ModInt> = vec![];
        let second_polynomial = create_modint_vector(&[1], 998244353);

        // Act
        let panic_result = panic::catch_unwind(|| {
            let _result_polynomial = multiply_polynomials(&first_polynomial, &second_polynomial);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn multiply_polynomials_panics_when_second_polynomial_is_empty() {
        // Arrange
        let first_polynomial = create_modint_vector(&[1], 998244353);
        let second_polynomial: Vec<ModInt> = vec![];

        // Act
        let panic_result = panic::catch_unwind(|| {
            let _result_polynomial = multiply_polynomials(&first_polynomial, &second_polynomial);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn multiply_polynomials_returns_degree_zero_product_for_singleton_polynomials() {
        // Arrange
        let modulus = 11;
        let first_polynomial = create_modint_vector(&[2], modulus);
        let second_polynomial = create_modint_vector(&[3], modulus);
        let expected_values = vec![6];

        // Act
        let actual_polynomial = multiply_polynomials(&first_polynomial, &second_polynomial);
        let actual_values = extract_modint_values(&actual_polynomial);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn multiply_polynomials_scales_all_coefficients_when_first_polynomial_is_singleton() {
        // Arrange
        let modulus = 11;
        let first_polynomial = create_modint_vector(&[2], modulus);
        let second_polynomial = create_modint_vector(&[1, 4, 7], modulus);
        let expected_values = vec![2, 8, 3];

        // Act
        let actual_polynomial = multiply_polynomials(&first_polynomial, &second_polynomial);
        let actual_values = extract_modint_values(&actual_polynomial);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn multiply_polynomials_returns_expected_convolution_for_multi_element_inputs() {
        // Arrange
        let modulus = 11;
        let first_polynomial = create_modint_vector(&[1, 2, 3], modulus);
        let second_polynomial = create_modint_vector(&[4, 5], modulus);
        let expected_values = vec![4, 2, 0, 4];

        // Act
        let actual_polynomial = multiply_polynomials(&first_polynomial, &second_polynomial);
        let actual_values = extract_modint_values(&actual_polynomial);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn multiply_polynomials_accepts_different_modulus_inputs_and_uses_first_modulus() {
        // Arrange
        let first_polynomial = create_modint_vector(&[2, 3], 11);
        let second_polynomial = create_modint_vector(&[10, 5], 13);
        let expected_values = vec![9, 7, 4];
        let expected_modulus = 11;

        // Act
        let actual_polynomial = multiply_polynomials(&first_polynomial, &second_polynomial);
        let actual_values = extract_modint_values(&actual_polynomial);
        let actual_modulus = actual_polynomial[0].modulus();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_modulus, expected_modulus);
    }

    #[test]
    fn create_alternating_sign_denominator_returns_empty_vector_for_empty_input() {
        // Arrange
        let denominator: Vec<ModInt> = vec![];
        let expected_denominator: Vec<ModInt> = vec![];

        // Act
        let actual_denominator = create_alternating_sign_denominator(&denominator);

        // Assert
        assert_eq!(actual_denominator, expected_denominator);
    }

    #[test]
    fn create_alternating_sign_denominator_keeps_sign_for_singleton_input() {
        // Arrange
        let denominator = create_modint_vector(&[5], 11);
        let expected_values = vec![5];

        // Act
        let actual_denominator = create_alternating_sign_denominator(&denominator);
        let actual_values = extract_modint_values(&actual_denominator);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn create_alternating_sign_denominator_flips_only_odd_indices_for_multi_element_input() {
        // Arrange
        let denominator = create_modint_vector(&[2, 3, 4, 5], 11);
        let expected_values = vec![2, 8, 4, 6];

        // Act
        let actual_denominator = create_alternating_sign_denominator(&denominator);
        let actual_values = extract_modint_values(&actual_denominator);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pick_stride_coefficients_returns_empty_vector_when_input_is_empty_and_take_even_is_true() {
        // Arrange
        let polynomial: Vec<ModInt> = vec![];
        let take_even = true;
        let expected_polynomial: Vec<ModInt> = vec![];

        // Act
        let actual_polynomial = pick_stride_coefficients(&polynomial, take_even);

        // Assert
        assert_eq!(actual_polynomial, expected_polynomial);
    }

    #[test]
    fn pick_stride_coefficients_returns_empty_vector_when_input_is_empty_and_take_even_is_false() {
        // Arrange
        let polynomial: Vec<ModInt> = vec![];
        let take_even = false;
        let expected_polynomial: Vec<ModInt> = vec![];

        // Act
        let actual_polynomial = pick_stride_coefficients(&polynomial, take_even);

        // Assert
        assert_eq!(actual_polynomial, expected_polynomial);
    }

    #[test]
    fn pick_stride_coefficients_returns_even_indices_for_length_zero_one_and_two_inputs() {
        // Arrange
        let modulus = 998244353;
        let polynomial_length_zero: Vec<ModInt> = vec![];
        let polynomial_length_one = create_modint_vector(&[7], modulus);
        let polynomial_length_two = create_modint_vector(&[7, 8], modulus);
        let expected_length_zero: Vec<ModInt> = vec![];
        let expected_length_one = create_modint_vector(&[7], modulus);
        let expected_length_two = create_modint_vector(&[7], modulus);

        let pick_stride_coefficients_take_even = true;

        // Act
        let actual_length_zero =
            pick_stride_coefficients(&polynomial_length_zero, pick_stride_coefficients_take_even);
        let actual_length_one =
            pick_stride_coefficients(&polynomial_length_one, pick_stride_coefficients_take_even);
        let actual_length_two =
            pick_stride_coefficients(&polynomial_length_two, pick_stride_coefficients_take_even);

        // Assert
        assert_eq!(actual_length_zero, expected_length_zero);
        assert_eq!(actual_length_one, expected_length_one);
        assert_eq!(actual_length_two, expected_length_two);
    }

    #[test]
    fn pick_stride_coefficients_returns_odd_indices_for_length_zero_one_and_two_inputs() {
        // Arrange
        let modulus = 998244353;
        let polynomial_length_zero: Vec<ModInt> = vec![];
        let polynomial_length_one = create_modint_vector(&[7], modulus);
        let polynomial_length_two = create_modint_vector(&[7, 8], modulus);
        let expected_length_zero: Vec<ModInt> = vec![];
        let expected_length_one: Vec<ModInt> = vec![];
        let expected_length_two = create_modint_vector(&[8], modulus);

        let pick_stride_coefficients_take_even = false;

        // Act
        let actual_length_zero =
            pick_stride_coefficients(&polynomial_length_zero, pick_stride_coefficients_take_even);
        let actual_length_one =
            pick_stride_coefficients(&polynomial_length_one, pick_stride_coefficients_take_even);
        let actual_length_two =
            pick_stride_coefficients(&polynomial_length_two, pick_stride_coefficients_take_even);

        // Assert
        assert_eq!(actual_length_zero, expected_length_zero);
        assert_eq!(actual_length_one, expected_length_one);
        assert_eq!(actual_length_two, expected_length_two);
    }

    #[test]
    fn bostan_mori_panics_when_numerator_is_empty() {
        // Arrange
        let numerator: Vec<ModInt> = vec![];
        let denominator = create_modint_vector(&[1], 998244353);
        let index = 0;

        // Act
        let panic_result = panic::catch_unwind(|| {
            let _coefficient = bostan_mori(numerator.clone(), denominator.clone(), index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn bostan_mori_panics_when_denominator_is_empty() {
        // Arrange
        let numerator = create_modint_vector(&[1], 998244353);
        let denominator: Vec<ModInt> = vec![];
        let index = 0;

        // Act
        let panic_result = panic::catch_unwind(|| {
            let _coefficient = bostan_mori(numerator.clone(), denominator.clone(), index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn bostan_mori_panics_when_denominator_constant_term_is_zero() {
        // Arrange
        let numerator = create_modint_vector(&[1], 998244353);
        let denominator = create_modint_vector(&[0, 1], 998244353);
        let index = 0;

        // Act
        let panic_result = panic::catch_unwind(|| {
            let _coefficient = bostan_mori(numerator.clone(), denominator.clone(), index);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn bostan_mori_returns_numerator_zero_divided_by_denominator_zero_when_index_is_zero() {
        // Arrange
        let modulus = 998244353;
        let numerator = create_modint_vector(&[6, 4], modulus);
        let denominator = create_modint_vector(&[3, 5], modulus);
        let index = 0;
        let expected_coefficient = ModInt::new(2, modulus);

        // Act
        let actual_coefficient = bostan_mori(numerator, denominator, index);

        // Assert
        assert_eq!(actual_coefficient, expected_coefficient);
    }

    #[test]
    fn bostan_mori_returns_one_for_indices_zero_one_and_two_for_one_over_one_minus_x() {
        // Arrange
        let modulus = 998244353;
        let numerator = create_modint_vector(&[1], modulus);
        let denominator = create_modint_vector(&[1, -1], modulus);
        let expected_values = vec![1, 1, 1];

        let bostan_mori_index = 0;
        let bostan_mori_index_2 = 1;
        let bostan_mori_index_3 = 2;

        // Act
        let actual_values = vec![
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index).value(),
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index_2).value(),
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index_3).value(),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bostan_mori_returns_one_for_index_zero_and_usize_max_for_one_over_one_minus_x() {
        // Arrange
        let modulus = 998244353;
        let numerator = create_modint_vector(&[1], modulus);
        let denominator = create_modint_vector(&[1, -1], modulus);
        let expected_values = vec![1, 1];

        let bostan_mori_index = 0;
        let bostan_mori_index_2 = usize::MAX;

        // Act
        let actual_values = vec![
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index).value(),
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index_2).value(),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bostan_mori_returns_zero_one_one_for_indices_zero_one_and_two_for_x_over_one_minus_x() {
        // Arrange
        let modulus = 998244353;
        let numerator = create_modint_vector(&[0, 1], modulus);
        let denominator = create_modint_vector(&[1, -1], modulus);
        let expected_values = vec![0, 1, 1];

        let bostan_mori_index = 0;
        let bostan_mori_index_2 = 1;
        let bostan_mori_index_3 = 2;

        // Act
        let actual_values = vec![
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index).value(),
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index_2).value(),
            bostan_mori(numerator.clone(), denominator.clone(), bostan_mori_index_3).value(),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bostan_mori_returns_zero_when_denominator_is_constant_and_index_exceeds_numerator_degree() {
        // Arrange
        let modulus = 998244353;
        let numerator = create_modint_vector(&[2, 4, 6], modulus);
        let denominator = create_modint_vector(&[3], modulus);
        let index = 5;
        let expected_value = 0;

        // Act
        let actual_value = bostan_mori(numerator, denominator, index).value();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn bostan_mori_matches_fibonacci_coefficients_for_x_over_one_minus_x_minus_x_squared() {
        // Arrange
        let modulus = 998244353;
        let numerator = create_modint_vector(&[0, 1], modulus);
        let denominator = create_modint_vector(&[1, -1, -1], modulus);
        let indices = vec![0usize, 1, 2, 3, 4, 5, 6, 7, 10];
        let expected_values = vec![0, 1, 1, 2, 3, 5, 8, 13, 55];

        // Act
        let actual_values: Vec<u32> = indices
            .iter()
            .map(|&index| bostan_mori(numerator.clone(), denominator.clone(), index).value())
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bostan_mori_matches_naive_series_expansion_for_deterministic_random_small_case() {
        // Arrange
        let modulus = 998244353;
        let seed = 20260211u64;
        let numerator_coefficients = create_deterministic_random_coefficients(seed, 4);
        let denominator_tail_coefficients = create_deterministic_random_coefficients(seed + 1, 3);
        let mut denominator_coefficients = vec![1];
        denominator_coefficients.extend(denominator_tail_coefficients);
        let numerator = create_modint_vector(&numerator_coefficients, modulus);
        let denominator = create_modint_vector(&denominator_coefficients, modulus);
        let indices = vec![0usize, 1, 2, 3, 7, 12];

        let expected_values: Vec<u32> = indices
            .iter()
            .map(|&index| {
                calculate_naive_rational_coefficient(&numerator, &denominator, index).value()
            })
            .collect();

        // Act
        let actual_values: Vec<u32> = indices
            .iter()
            .map(|&index| bostan_mori(numerator.clone(), denominator.clone(), index).value())
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn bostan_mori_accepts_different_modulus_inputs_and_uses_numerator_modulus_for_result() {
        // Arrange
        let numerator = create_modint_vector(&[6], 11);
        let denominator = create_modint_vector(&[2], 13);
        let index = 0;
        let expected_value = 9;
        let expected_modulus = 11;

        // Act
        let actual_coefficient = bostan_mori(numerator, denominator, index);
        let actual_value = actual_coefficient.value();
        let actual_modulus = actual_coefficient.modulus();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_modulus, expected_modulus);
    }
}
