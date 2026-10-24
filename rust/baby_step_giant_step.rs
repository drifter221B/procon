use std::collections::HashMap;

fn mod_mul(x: i64, y: i64, modulo: i64) -> i64 {
    ((x.rem_euclid(modulo) as i128 * y.rem_euclid(modulo) as i128) % modulo as i128) as i64
}

fn mod_pow(mut base: i64, mut exp: i64, modulo: i64) -> i64 {
    base = base.rem_euclid(modulo);
    let mut result = 1 % modulo;

    while 0 < exp {
        if (exp & 1) == 1 {
            result = mod_mul(result, base, modulo);
        }

        base = mod_mul(base, base, modulo);
        exp >>= 1;
    }

    result
}

fn integer_sqrt(n: i64) -> i64 {
    assert!(0 <= n, "n must be non-negative");

    let n_i128 = n as i128;
    let mut s = (n as f64).sqrt() as i64;

    while ((s + 1) as i128) * ((s + 1) as i128) <= n_i128 {
        s += 1;
    }

    while (s as i128) * (s as i128) > n_i128 {
        s -= 1;
    }

    s
}

// Xi = a * Xi-1 + b (mod modulo)
// Assumes `modulo` is prime so `a.pow(modulo - 2)` yields the modular inverse of `a`.
fn baby_step_giant_step(modulo: i64, a: i64, b: i64, init_value: i64, target_value: i64) -> i64 {
    assert!(0 < modulo, "modulo must be positive");

    let mut ans = -1;
    let a_mod = a.rem_euclid(modulo);
    let b_mod = b.rem_euclid(modulo);
    let init_mod = init_value.rem_euclid(modulo);
    let target_mod = target_value.rem_euclid(modulo);

    if init_mod == target_mod {
        return 0;
    } else if a_mod == 0 {
        if b_mod == target_mod {
            return 1;
        } else {
            return -1;
        }
    }

    let inv_a = mod_pow(a_mod, modulo - 2, modulo);
    let inv_b = mod_mul(-b_mod, inv_a, modulo);
    let mut big_a = 1 % modulo;
    let mut big_b = 0;
    let sqrt_p = integer_sqrt(modulo);

    let mut small = HashMap::new();
    let mut current_v = target_mod;

    let mut j = 0;
    while j < sqrt_p {
        small.entry(current_v).or_insert(j);

        current_v = (mod_mul(inv_a, current_v, modulo) + inv_b).rem_euclid(modulo);
        big_a = mod_mul(big_a, a_mod, modulo);
        big_b = (mod_mul(big_b, a_mod, modulo) + b_mod).rem_euclid(modulo);

        j += 1;
    }

    current_v = init_mod;

    let limit = modulo / sqrt_p + 1;
    let mut j = 0;
    while j < limit {
        if let Some(&k) = small.get(&current_v) {
            ans = j * sqrt_p + k;
            break;
        }

        current_v = (mod_mul(big_a, current_v, modulo) + big_b).rem_euclid(modulo);
        j += 1;
    }

    ans
}

#[cfg(test)]
mod tests {
    use super::{baby_step_giant_step, integer_sqrt, mod_mul, mod_pow};

    fn calculate_next_value(current_value: i64, a_mod: i64, b_mod: i64, modulo: i64) -> i64 {
        (mod_mul(a_mod, current_value, modulo) + b_mod).rem_euclid(modulo)
    }

    fn brute_force_min_steps(
        modulo: i64,
        a: i64,
        b: i64,
        init_value: i64,
        target_value: i64,
    ) -> i64 {
        let a_mod = a.rem_euclid(modulo);
        let b_mod = b.rem_euclid(modulo);
        let target_mod = target_value.rem_euclid(modulo);
        let mut current_value = init_value.rem_euclid(modulo);
        let mut step = 0;

        while step <= modulo {
            if current_value == target_mod {
                return step;
            }
            current_value = calculate_next_value(current_value, a_mod, b_mod, modulo);
            step += 1;
        }

        -1
    }

    #[test]
    fn baby_step_giant_step_returns_zero_when_init_and_target_are_congruent() {
        // Arrange
        let modulo = 7;
        let a = 3;
        let b = 4;
        let init_value = -1;
        let target_value = 6;
        let expected_value = 0;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_one_when_a_is_zero_and_b_matches_target() {
        // Arrange
        let modulo = 7;
        let a = 0;
        let b = -3;
        let init_value = 2;
        let target_value = 4;
        let expected_value = 1;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_minus_one_when_a_is_zero_and_b_does_not_match_target() {
        // Arrange
        let modulo = 7;
        let a = 0;
        let b = 4;
        let init_value = 1;
        let target_value = 5;
        let expected_value = -1;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_min_steps_for_reachable_case() {
        // Arrange
        let modulo = 7;
        let a = 2;
        let b = 1;
        let init_value = 0;
        let target_value = 3;
        let expected_value = 2;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_minus_one_for_unreachable_case() {
        // Arrange
        let modulo = 7;
        let a = 2;
        let b = 1;
        let init_value = 0;
        let target_value = 2;
        let expected_value = -1;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_handles_negative_parameters() {
        // Arrange
        let modulo = 7;
        let a = -2;
        let b = -1;
        let init_value = -1;
        let target_value = -3;
        let expected_value = 2;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_zero_when_modulo_is_one() {
        // Arrange
        let modulo = 1;
        let a = 123;
        let b = -456;
        let init_value = 789;
        let target_value = -1011;
        let expected_value = 0;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_panics_when_modulo_is_not_positive() {
        // Arrange
        let zero_modulo = 0;
        let negative_modulo = -1;
        let a = 2;
        let b = 1;
        let init_value = 0;
        let target_value = 1;

        // Act
        let zero_modulo_result = std::panic::catch_unwind(|| {
            baby_step_giant_step(zero_modulo, a, b, init_value, target_value)
        });
        let negative_modulo_result = std::panic::catch_unwind(|| {
            baby_step_giant_step(negative_modulo, a, b, init_value, target_value)
        });

        // Assert
        assert!(zero_modulo_result.is_err());
        assert!(negative_modulo_result.is_err());
    }

    #[test]
    fn baby_step_giant_step_handles_non_representative_init_and_target_values() {
        // Arrange
        let modulo = 7;
        let a = 2;
        let b = 1;
        let init_value = 14;
        let target_value = 10;
        let expected_value = 2;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_steps_before_baby_giant_boundary() {
        // Arrange
        let modulo = 29;
        let a = 1;
        let b = 1;
        let init_value = 0;
        let target_value = 4;
        let expected_value = 4;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_steps_at_baby_giant_boundary() {
        // Arrange
        let modulo = 29;
        let a = 1;
        let b = 1;
        let init_value = 0;
        let target_value = 5;
        let expected_value = 5;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_steps_after_baby_giant_boundary() {
        // Arrange
        let modulo = 29;
        let a = 1;
        let b = 1;
        let init_value = 0;
        let target_value = 6;
        let expected_value = 6;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_returns_min_steps_when_baby_step_keys_repeat() {
        // Arrange
        let modulo = 17;
        let a = -1;
        let b = 0;
        let init_value = 1;
        let target_value = 16;
        let expected_value = 1;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_finds_solution_near_last_giant_step_for_long_cycle() {
        // Arrange
        let modulo = 1_000_003;
        let a = 2;
        let b = 37;
        let init_value = 123;
        let target_value = 43;
        let expected_value = 1_000_001;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_normalizes_i64_boundary_parameters() {
        // Arrange
        let modulo = 11;
        let a = i64::MIN;
        let b = i64::MAX;
        let init_value = i64::MIN;
        let target_value = i64::MAX;
        let expected_value = 3;

        // Act
        let actual_value = baby_step_giant_step(modulo, a, b, init_value, target_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn baby_step_giant_step_matches_bruteforce_for_small_prime_moduli() {
        // Arrange
        let prime_moduli = [2, 3, 5, 7];
        let value_min = -7;
        let value_max = 7;

        for &modulo in &prime_moduli {
            for a in value_min..=value_max {
                for b in value_min..=value_max {
                    for init_value in value_min..=value_max {
                        for target_value in value_min..=value_max {
                            // Arrange
                            let expected_value =
                                brute_force_min_steps(modulo, a, b, init_value, target_value);

                            // Act
                            let actual_value =
                                baby_step_giant_step(modulo, a, b, init_value, target_value);

                            // Assert
                            assert_eq!(
                                actual_value, expected_value,
                                "modulo={modulo}, a={a}, b={b}, init={init_value}, target={target_value}"
                            );
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn mod_mul_returns_expected_value_for_positive_operands() {
        // Arrange
        let x = 2;
        let y = 3;
        let modulo = 5;
        let expected_value = 1;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_mul_normalizes_negative_left_operand() {
        // Arrange
        let x = -2;
        let y = 3;
        let modulo = 5;
        let expected_value = 4;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_mul_normalizes_two_negative_operands() {
        // Arrange
        let x = -2;
        let y = -3;
        let modulo = 5;
        let expected_value = 1;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_mul_returns_zero_when_modulo_is_one() {
        // Arrange
        let x = 12345;
        let y = -98765;
        let modulo = 1;
        let expected_value = 0;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_mul_avoids_overflow_near_i64_upper_bound() {
        // Arrange
        let x = i64::MAX;
        let y = i64::MAX;
        let modulo = 1_000_000_007;
        let expected_value = 737564071;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_mul_returns_zero_when_left_operand_is_zero() {
        // Arrange
        let x = 0;
        let y = i64::MAX;
        let modulo = 97;
        let expected_value = 0;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_mul_avoids_overflow_when_reduced_product_exceeds_i64() {
        // Arrange
        let x = i64::MIN;
        let y = i64::MIN;
        let modulo = i64::MAX;
        let expected_value = 1;

        // Act
        let actual_value = mod_mul(x, y, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_returns_one_mod_modulo_when_exponent_is_zero() {
        // Arrange
        let base = 123;
        let exp = 0;
        let modulo = 17;
        let expected_value = 1 % modulo;

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_returns_base_modulo_when_exponent_is_one() {
        // Arrange
        let base: i64 = -12;
        let exp: i64 = 1;
        let modulo: i64 = 7;
        let expected_value = base.rem_euclid(modulo);

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_returns_expected_values_for_exponent_boundary_around_two() {
        // Arrange
        let base = 3;
        let modulo = 7;
        let exp_minus_one = 1;
        let exp_center = 2;
        let exp_plus_one = 3;
        let expected_value_minus_one = 3;
        let expected_value_center = 2;
        let expected_value_plus_one = 6;

        // Act
        let actual_value_minus_one = mod_pow(base, exp_minus_one, modulo);
        let actual_value_center = mod_pow(base, exp_center, modulo);
        let actual_value_plus_one = mod_pow(base, exp_plus_one, modulo);

        // Assert
        assert_eq!(actual_value_minus_one, expected_value_minus_one);
        assert_eq!(actual_value_center, expected_value_center);
        assert_eq!(actual_value_plus_one, expected_value_plus_one);
    }

    #[test]
    fn mod_pow_handles_negative_base() {
        // Arrange
        let base = -2;
        let exp = 3;
        let modulo = 5;
        let expected_value = 2;

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_returns_zero_when_modulo_is_one() {
        // Arrange
        let base = 987654;
        let exp = 12345;
        let modulo = 1;
        let expected_value = 0;

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_returns_known_value_for_large_exponent() {
        // Arrange
        let base = 2;
        let exp = 60;
        let modulo = 1_000_000_007;
        let expected_value = 536396504;

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_returns_zero_when_base_is_zero_and_exponent_is_positive() {
        // Arrange
        let base = 0;
        let exp = 5;
        let modulo = 7;
        let expected_value = 0;

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn mod_pow_handles_i64_input_and_exponent_bounds() {
        // Arrange
        let base = i64::MIN;
        let exp = i64::MAX;
        let modulo = i64::MAX;
        let expected_value = i64::MAX - 1;

        // Act
        let actual_value = mod_pow(base, exp, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_returns_zero_for_zero_input() {
        // Arrange
        let input_value = 0;
        let expected_value = 0;

        // Act
        let actual_value = integer_sqrt(input_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_returns_one_for_one_input() {
        // Arrange
        let input_value = 1;
        let expected_value = 1;

        // Act
        let actual_value = integer_sqrt(input_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_returns_exact_root_for_perfect_square() {
        // Arrange
        let input_value = 16;
        let expected_value = 4;

        // Act
        let actual_value = integer_sqrt(input_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_returns_floor_for_boundary_around_perfect_square() {
        // Arrange
        let input_value_minus_one = 15;
        let input_value_center = 16;
        let input_value_plus_one = 17;
        let expected_value_minus_one = 3;
        let expected_value_center = 4;
        let expected_value_plus_one = 4;

        // Act
        let actual_value_minus_one = integer_sqrt(input_value_minus_one);
        let actual_value_center = integer_sqrt(input_value_center);
        let actual_value_plus_one = integer_sqrt(input_value_plus_one);

        // Assert
        assert_eq!(actual_value_minus_one, expected_value_minus_one);
        assert_eq!(actual_value_center, expected_value_center);
        assert_eq!(actual_value_plus_one, expected_value_plus_one);
    }

    #[test]
    fn integer_sqrt_returns_exact_root_for_upper_bound_near_square() {
        // Arrange
        let input_value = 3_037_000_499_i64.pow(2);
        let expected_value = 3_037_000_499;

        // Act
        let actual_value = integer_sqrt(input_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_corrects_f64_overestimate_before_upper_bound_square() {
        // Arrange
        let root = 3_037_000_499_i64;
        let input_value = root.pow(2) - 1;
        let expected_value = root - 1;

        // Act
        let actual_value = integer_sqrt(input_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_returns_floor_for_i64_max() {
        // Arrange
        let input_value = i64::MAX;
        let expected_value = 3_037_000_499;

        // Act
        let actual_value = integer_sqrt(input_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn integer_sqrt_panics_for_negative_input() {
        // Arrange
        let input_value = -1;

        // Act
        let actual_result = std::panic::catch_unwind(|| integer_sqrt(input_value));

        // Assert
        assert!(actual_result.is_err());
    }
}
