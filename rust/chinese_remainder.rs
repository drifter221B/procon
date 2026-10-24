fn compute_extended_gcd(left_value: i64, right_value: i64) -> (i64, i64, i64) {

    if right_value == 0 {
        return (left_value.abs(), left_value.signum(), 0);
    }

    let (gcd_value, coefficient_left_next, coefficient_right_next) =
        compute_extended_gcd(right_value, left_value % right_value);
    let coefficient_left = coefficient_right_next;
    let coefficient_right = coefficient_left_next - (left_value / right_value) * coefficient_right_next;

    (gcd_value, coefficient_left, coefficient_right)
}

fn normalize_congruence(remainder_value: i64, modulus_value: i64) -> Option<(i64, i64)> {

    if modulus_value == 0 {
        return None;
    }

    let positive_modulus = modulus_value.abs();
    let normalized_remainder = remainder_value.rem_euclid(positive_modulus);

    Some((normalized_remainder, positive_modulus))
}

fn merge_congruences(current: (i64, i64), next: (i64, i64)) -> Option<(i64, i64)> {

    let (current_remainder, current_modulus) = current;
    let (next_remainder, next_modulus) = next;

    let (gcd_value, coefficient_current, _coefficient_next) =
        compute_extended_gcd(current_modulus, next_modulus);
    let remainder_gap = next_remainder - current_remainder;

    if remainder_gap % gcd_value != 0 {
        return None;
    }

    let scaled_current_modulus = current_modulus / gcd_value;
    let combined_modulus_value = (scaled_current_modulus as i128) * (next_modulus as i128);

    if combined_modulus_value <= 0 || (i64::MAX as i128) < combined_modulus_value {
        return None;
    }

    let combined_step = (remainder_gap / gcd_value) as i128;
    let combined_remainder_value = current_remainder as i128 + combined_step * (coefficient_current as i128) * (current_modulus as i128);
    let normalized_remainder = ((combined_remainder_value % combined_modulus_value) + combined_modulus_value) % combined_modulus_value;

    Some((normalized_remainder as i64, combined_modulus_value as i64))
}

pub fn chinese_remainder(congruences: &[(i64, i64)]) -> Option<(i64, i64)> {

    let normalized = congruences.iter().map(|&(remainder_value, modulus_value)| normalize_congruence(remainder_value, modulus_value)).collect::<Option<Vec<_>>>()?;

    normalized.into_iter().fold(Some((0, 1)), |accumulated, next| {
        accumulated.and_then(|current| merge_congruences(current, next))
    })
}

#[cfg(test)]
mod tests {
    use super::{chinese_remainder, merge_congruences};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn satisfies_congruence(solution_remainder: i64, input_remainder: i64, modulus_value: i64) -> bool {
        let positive_modulus = modulus_value.abs();
        let normalized_input_remainder = input_remainder.rem_euclid(positive_modulus);
        let normalized_solution_remainder = solution_remainder.rem_euclid(positive_modulus);
        normalized_solution_remainder == normalized_input_remainder
    }

    #[test]
    fn chinese_remainder_with_negative_remainder_normalizes_remainder() {
        // Arrange
        let input_congruences = vec![(-1_i64, 5_i64)];
        let expected_result = Some((4_i64, 5_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_negative_modulus_normalizes_to_positive_modulus() {
        // Arrange
        let input_congruences = vec![(7_i64, -5_i64)];
        let expected_result = Some((2_i64, 5_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn merge_congruences_with_extreme_remainder_gap_panics_on_overflow() {
        // Arrange
        let current_congruence = (i64::MIN, 1_i64);
        let next_congruence = (i64::MAX, 1_i64);
        let expected_is_panic = true;

        // Act
        let actual_is_panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = merge_congruences(current_congruence, next_congruence);
        }))
        .is_err();

        // Assert
        assert_eq!(actual_is_panic, expected_is_panic);
    }

    #[test]
    fn chinese_remainder_with_empty_congruence_set_returns_zero_mod_one() {
        // Arrange
        let input_congruences: Vec<(i64, i64)> = vec![];
        let expected_result = Some((0_i64, 1_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_single_positive_congruence_returns_same_pair() {
        // Arrange
        let input_congruences = vec![(2_i64, 5_i64)];
        let expected_result = Some((2_i64, 5_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_modulus_minus_one_returns_zero_mod_one() {
        // Arrange
        let input_congruences = vec![(0_i64, -1_i64)];
        let expected_result = Some((0_i64, 1_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_modulus_one_returns_zero_mod_one() {
        // Arrange
        let input_congruences = vec![(123_i64, 1_i64)];
        let expected_result = Some((0_i64, 1_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_modulus_zero_returns_none() {
        // Arrange
        let input_congruences = vec![(0_i64, 0_i64)];
        let expected_result: Option<(i64, i64)> = None;

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_two_coprime_moduli_returns_expected_solution() {
        // Arrange
        let input_congruences = vec![(2_i64, 3_i64), (3_i64, 5_i64)];
        let expected_result = Some((8_i64, 15_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_two_non_coprime_consistent_moduli_returns_solution() {
        // Arrange
        let input_congruences = vec![(2_i64, 4_i64), (6_i64, 8_i64)];
        let expected_result = Some((6_i64, 8_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_two_non_coprime_inconsistent_moduli_returns_none() {
        // Arrange
        let input_congruences = vec![(1_i64, 4_i64), (2_i64, 6_i64)];
        let expected_result: Option<(i64, i64)> = None;

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_three_congruences_merges_all_constraints() {
        // Arrange
        let input_congruences = vec![(2_i64, 3_i64), (3_i64, 5_i64), (2_i64, 7_i64)];
        let expected_result = Some((23_i64, 105_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_duplicate_congruences_keeps_same_result() {
        // Arrange
        let input_congruences = vec![(2_i64, 5_i64), (2_i64, 5_i64)];
        let expected_result = Some((2_i64, 5_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_modulus_one_constraint_keeps_other_solution() {
        // Arrange
        let input_congruences = vec![(2_i64, 3_i64), (0_i64, 1_i64)];
        let expected_result = Some((2_i64, 3_i64));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_permuted_input_order_returns_equivalent_solution() {
        // Arrange
        let input_congruences_order_a = vec![(2_i64, 3_i64), (3_i64, 5_i64), (2_i64, 7_i64)];
        let input_congruences_order_b = vec![(2_i64, 7_i64), (2_i64, 3_i64), (3_i64, 5_i64)];
        let expected_result = Some((23_i64, 105_i64));

        // Act
        let actual_result_order_a = chinese_remainder(&input_congruences_order_a);
        let actual_result_order_b = chinese_remainder(&input_congruences_order_b);

        // Assert
        assert_eq!(actual_result_order_a, expected_result);
        assert_eq!(actual_result_order_b, expected_result);
        assert_eq!(actual_result_order_a, actual_result_order_b);
    }

    #[test]
    fn chinese_remainder_with_zero_modulus_in_multiple_congruences_returns_none() {
        // Arrange
        let input_congruences = vec![(2_i64, 3_i64), (0_i64, 0_i64)];
        let expected_result: Option<(i64, i64)> = None;

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_combined_modulus_equal_i64_max_returns_some() {
        // Arrange
        let input_congruences = vec![(0_i64, i64::MAX)];
        let expected_result = Some((0_i64, i64::MAX));

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn chinese_remainder_with_combined_modulus_exceeding_i64_max_returns_none() {
        // Arrange
        let input_congruences = vec![(0_i64, 2_i64), (0_i64, i64::MAX)];
        let expected_result: Option<(i64, i64)> = None;

        // Act
        let actual_result = chinese_remainder(&input_congruences);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn chinese_remainder_with_modulus_i64_min_panics_on_abs_overflow() {
        // Arrange
        let input_congruences = vec![(0_i64, i64::MIN)];
        let expected_is_panic = true;

        // Act
        let actual_is_panic = catch_unwind(AssertUnwindSafe(|| {
            let _ = chinese_remainder(&input_congruences);
        }))
        .is_err();

        // Assert
        assert_eq!(actual_is_panic, expected_is_panic);
    }

    #[test]
    fn chinese_remainder_solution_remainder_is_within_zero_and_modulus_bounds() {
        // Arrange
        let input_congruences = vec![(14_i64, 21_i64), (35_i64, 28_i64)];
        let expected_solution_exists = true;
        let expected_remainder_in_range = true;
        let lower_bound = 0_i64;

        // Act
        let actual_solution = chinese_remainder(&input_congruences);
        let actual_solution_exists = actual_solution.is_some();
        let actual_remainder_in_range = actual_solution
            .map(|(solution_remainder, solution_modulus)| {
                lower_bound <= solution_remainder && solution_remainder < solution_modulus
            })
            .unwrap_or(false);

        // Assert
        assert_eq!(actual_solution_exists, expected_solution_exists);
        assert_eq!(actual_remainder_in_range, expected_remainder_in_range);
    }

    #[test]
    fn chinese_remainder_solution_satisfies_all_input_congruences() {
        // Arrange
        let input_congruences = vec![(-1_i64, 5_i64), (3_i64, 7_i64), (2_i64, 9_i64)];
        let expected_solution_exists = true;
        let expected_satisfies_all = true;

        // Act
        let actual_solution = chinese_remainder(&input_congruences);
        let actual_solution_exists = actual_solution.is_some();
        let actual_satisfies_all = actual_solution
            .map(|(solution_remainder, _)| {
                input_congruences
                    .iter()
                    .all(|&(input_remainder, modulus_value)| {
                        satisfies_congruence(solution_remainder, input_remainder, modulus_value)
                    })
            })
            .unwrap_or(false);

        // Assert
        assert_eq!(actual_solution_exists, expected_solution_exists);
        assert_eq!(actual_satisfies_all, expected_satisfies_all);
    }
}
