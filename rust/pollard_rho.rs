fn gcd<T>(x: T, y: T) -> T
where T: Copy + Default + PartialEq + std::ops::Rem<Output = T> {

    let mut x = x;
    let mut y = y;

    while y != T::default() {
        (x, y) = (y, x % y);
    }

    x
}

fn add_mod(value1: u128, value2: u128, modulo: u128) -> u128 {

    let value1 = value1 % modulo;
    let value2 = value2 % modulo;

    if value1 < modulo - value2 {
        value1 + value2
    } else {
        value1 - (modulo - value2)
    }
}

fn multiply_mod(value1: u128, value2: u128, modulo: u128) -> u128 {

    let mut result= 0;
    let mut addend_value = value1 % modulo;
    let mut multiple= value2;

    while multiple != 0 {

        if multiple % 2 == 1 {
            result = add_mod(result, addend_value, modulo);
        }

        addend_value = add_mod(addend_value, addend_value, modulo);
        multiple >>= 1;
    }

    result
}

fn power_mod(base: u128, p: u128, modulo: u128) -> u128 {

    let mut result= 1 % modulo;
    let mut base = base % modulo;
    let mut p = p;

    while p != 0 {

        if p % 2 == 1 {
            result = multiply_mod(result, base, modulo);
        }

        base = multiply_mod(base, base, modulo);
        p >>= 1;
    }

    result
}

fn is_witness_composite(
    tested_value: u128,
    base_value: u128,
    odd_part_value: u128,
    exponent_of_two: u32,
) -> bool {

    let mut witness_value = power_mod(base_value, odd_part_value, tested_value);

    if witness_value == 1 || witness_value == tested_value - 1 {
        return false;
    }

    for _ in 1..exponent_of_two {
        witness_value = multiply_mod(witness_value, witness_value, tested_value);

        if witness_value == tested_value - 1 {
            return false;
        }
    }

    true
}

fn is_prime_by_miller_rabin(target_value: u128) -> bool {

    if target_value <= 1 {
        return false;
    }

    if target_value <= 3 {
        return true;
    }

    if target_value % 2 == 0 {
        return false;
    }

    let mut odd_part_value = target_value - 1;
    let mut exponent_of_two = 0;

    while odd_part_value % 2 == 0 {
        odd_part_value >>= 1;
        exponent_of_two += 1;
    }

    const DETERMINISTIC_BASES: [u128; 12] = [2, 3, 5, 7, 11, 13, 17, 19, 23, 29, 31, 37];

    for &base in DETERMINISTIC_BASES.iter() {

        if base <= target_value - 2 {
            if is_witness_composite(target_value, base, odd_part_value, exponent_of_two) {
                return false;
            }
        }
    }

    true
}

fn iterate_polynomial(value: u128, constant_value: u128, modulo_value: u128) -> u128 {

    let squared_value = multiply_mod(value, value, modulo_value);
    add_mod(squared_value, constant_value, modulo_value)
}

fn calculate_pollard_rho_divisor(target_value: u128) -> u128 {

    if target_value % 2 == 0 {
        return 2;
    }

    let mut constant_value = 1;

    loop {

        let mut tortoise_value = 2;
        let mut hare_value = 2;
        let mut divisor_value = 1;

        while divisor_value == 1 {
            tortoise_value = iterate_polynomial(tortoise_value, constant_value, target_value);
            hare_value = iterate_polynomial(hare_value, constant_value, target_value);
            hare_value = iterate_polynomial(hare_value, constant_value, target_value);

            let difference_value = if tortoise_value <= hare_value {
                hare_value - tortoise_value
            } else {
                tortoise_value - hare_value
            };

            divisor_value = gcd(difference_value, target_value);
        }

        if divisor_value != target_value {
            return divisor_value;
        }

        constant_value += 1;
    }
}

fn factorize_recursively(target_value: u128, factors: &mut Vec<u128>) {

    if target_value <= 1 {
        return;
    }

    if is_prime_by_miller_rabin(target_value) {
        factors.push(target_value);
        return;
    }

    let divisor_value = calculate_pollard_rho_divisor(target_value);
    factorize_recursively(divisor_value, factors);
    factorize_recursively(target_value / divisor_value, factors);
}

fn factorize_by_pollard_rho(target_value: u128) -> Vec<u128> {

    let mut factors = Vec::new();
    factorize_recursively(target_value, &mut factors);
    factors.sort();
    factors
}

#[cfg(test)]
mod tests {
    use super::{
        calculate_pollard_rho_divisor, factorize_by_pollard_rho, gcd, is_prime_by_miller_rabin,
        iterate_polynomial, multiply_mod, power_mod,
    };

    fn is_sorted_in_ascending_order(values: &[u128]) -> bool {
        values
            .windows(2)
            .all(|adjacent_values| adjacent_values[0] <= adjacent_values[1])
    }

    fn calculate_product(values: &[u128]) -> u128 {
        values.iter().copied().product()
    }

    fn is_prime_by_trial_division(target_value: u128) -> bool {
        if target_value <= 1 {
            return false;
        }

        if target_value == 2 {
            return true;
        }

        if target_value % 2 == 0 {
            return false;
        }

        let mut divisor_candidate = 3u128;

        while divisor_candidate <= target_value / divisor_candidate {
            if target_value % divisor_candidate == 0 {
                return false;
            }

            divisor_candidate += 2;
        }

        true
    }

    #[test]
    fn gcd_with_both_zero_returns_zero() {
        // Arrange
        let x = 0u128;
        let y = 0u128;
        let expected_gcd = 0u128;

        // Act
        let actual_gcd = gcd(x, y);

        // Assert
        assert_eq!(actual_gcd, expected_gcd);
    }

    #[test]
    fn gcd_with_left_zero_returns_right_value() {
        // Arrange
        let x = 0u128;
        let y = 15u128;
        let expected_gcd = 15u128;

        // Act
        let actual_gcd = gcd(x, y);

        // Assert
        assert_eq!(actual_gcd, expected_gcd);
    }

    #[test]
    fn gcd_with_common_divisor_returns_expected_value() {
        // Arrange
        let x = 18u128;
        let y = 24u128;
        let expected_gcd = 6u128;

        // Act
        let actual_gcd = gcd(x, y);

        // Assert
        assert_eq!(actual_gcd, expected_gcd);
    }

    #[test]
    fn gcd_with_coprime_values_returns_one() {
        // Arrange
        let x = 17u128;
        let y = 19u128;
        let expected_gcd = 1u128;

        // Act
        let actual_gcd = gcd(x, y);

        // Assert
        assert_eq!(actual_gcd, expected_gcd);
    }

    #[test]
    fn multiply_mod_with_zero_multiplier_returns_zero() {
        // Arrange
        let value1 = 987654321u128;
        let value2 = 0u128;
        let modulo = 1234567u128;
        let expected_value = 0u128;

        // Act
        let actual_value = multiply_mod(value1, value2, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn multiply_mod_with_modulo_one_returns_zero() {
        // Arrange
        let value1 = 123u128;
        let value2 = 456u128;
        let modulo = 1u128;
        let expected_value = 0u128;

        // Act
        let actual_value = multiply_mod(value1, value2, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn multiply_mod_with_small_values_returns_expected_remainder() {
        // Arrange
        let value1 = 13u128;
        let value2 = 17u128;
        let modulo = 19u128;
        let expected_value = 12u128;

        // Act
        let actual_value = multiply_mod(value1, value2, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn multiply_mod_with_large_values_returns_expected_remainder() {
        // Arrange
        let value1 = u128::MAX - 1;
        let value2 = 2u128;
        let modulo = u128::MAX;
        let expected_value = u128::MAX - 2;

        // Act
        let actual_value = multiply_mod(value1, value2, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn power_mod_with_zero_exponent_returns_one_modulo_mod() {
        // Arrange
        let base = 321u128;
        let p = 0u128;
        let modulo = 97u128;
        let expected_value = 1u128 % modulo;

        // Act
        let actual_value = power_mod(base, p, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn power_mod_with_modulo_one_returns_zero_for_zero_and_positive_exponents() {
        // Arrange
        let base = 321u128;
        let exponents = (0u128, 5u128);
        let modulo = 1u128;
        let expected_values = (0u128, 0u128);

        // Act
        let actual_values = (
            power_mod(base, exponents.0, modulo),
            power_mod(base, exponents.1, modulo),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn multiply_mod_with_zero_modulo_panics() {
        // Arrange
        let value1 = 2u128;
        let value2 = 3u128;
        let modulo = 0u128;

        // Act
        let actual_result = std::panic::catch_unwind(|| multiply_mod(value1, value2, modulo));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn power_mod_with_zero_modulo_panics() {
        // Arrange
        let base = 2u128;
        let exponent = 3u128;
        let modulo = 0u128;

        // Act
        let actual_result = std::panic::catch_unwind(|| power_mod(base, exponent, modulo));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn power_mod_with_zero_base_and_positive_exponent_returns_zero() {
        // Arrange
        let base = 0u128;
        let p = 15u128;
        let modulo = 97u128;
        let expected_value = 0u128;

        // Act
        let actual_value = power_mod(base, p, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn power_mod_with_known_case_returns_expected_value() {
        // Arrange
        let base = 2u128;
        let p = 10u128;
        let modulo = 1000u128;
        let expected_value = 24u128;

        // Act
        let actual_value = power_mod(base, p, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn power_mod_with_fermat_little_theorem_case_returns_one() {
        // Arrange
        let base = 2u128;
        let p = 100u128;
        let modulo = 101u128;
        let expected_value = 1u128;

        // Act
        let actual_value = power_mod(base, p, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_zero_returns_false() {
        // Arrange
        let target_value = 0u128;
        let expected_is_prime = false;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_one_returns_false() {
        // Arrange
        let target_value = 1u128;
        let expected_is_prime = false;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_two_returns_true() {
        // Arrange
        let target_value = 2u128;
        let expected_is_prime = true;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_three_returns_true() {
        // Arrange
        let target_value = 3u128;
        let expected_is_prime = true;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_four_returns_false() {
        // Arrange
        let target_value = 4u128;
        let expected_is_prime = false;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_odd_composite_returns_false() {
        // Arrange
        let target_value = 9u128;
        let expected_is_prime = false;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_carmichael_number_returns_false() {
        // Arrange
        let target_value = 561u128;
        let expected_is_prime = false;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn is_prime_by_miller_rabin_with_large_known_prime_returns_true() {
        // Arrange
        let target_value = 2305843009213693951u128;
        let expected_is_prime = true;

        // Act
        let actual_is_prime = is_prime_by_miller_rabin(target_value);

        // Assert
        assert_eq!(actual_is_prime, expected_is_prime);
    }

    #[test]
    fn iterate_polynomial_returns_x_squared_plus_c_mod_n() {
        // Arrange
        let value = 9u128;
        let constant_value = 7u128;
        let modulo_value = 11u128;
        let expected_value = ((9u128 * 9u128) + 7u128) % 11u128;

        // Act
        let actual_value = iterate_polynomial(value, constant_value, modulo_value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn calculate_pollard_rho_divisor_with_even_input_returns_two() {
        // Arrange
        let target_value = 100u128;
        let expected_divisor = 2u128;

        // Act
        let actual_divisor = calculate_pollard_rho_divisor(target_value);

        // Assert
        assert_eq!(actual_divisor, expected_divisor);
    }

    #[test]
    fn calculate_pollard_rho_divisor_with_odd_composite_returns_non_trivial_divisor() {
        // Arrange
        let target_value = 8051u128;
        let expected_is_non_trivial_divisor = true;

        // Act
        let actual_divisor = calculate_pollard_rho_divisor(target_value);
        let actual_is_non_trivial_divisor = 1u128 < actual_divisor
            && actual_divisor < target_value
            && target_value % actual_divisor == 0;

        // Assert
        assert_eq!(
            actual_is_non_trivial_divisor,
            expected_is_non_trivial_divisor
        );
    }

    #[test]
    fn factorize_by_pollard_rho_with_zero_returns_empty_vector() {
        // Arrange
        let target_value = 0u128;
        let expected_factors: Vec<u128> = vec![];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_with_one_returns_empty_vector() {
        // Arrange
        let target_value = 1u128;
        let expected_factors: Vec<u128> = vec![];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_with_prime_returns_singleton_vector() {
        // Arrange
        let target_value = 97u128;
        let expected_factors = vec![97u128];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_with_prime_square_returns_duplicated_prime_vector() {
        // Arrange
        let target_value = 49u128;
        let expected_factors = vec![7u128, 7u128];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_with_power_of_two_returns_repeated_two_vector() {
        // Arrange
        let target_value = 64u128;
        let expected_factors = vec![2u128, 2u128, 2u128, 2u128, 2u128, 2u128];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_with_semiprime_returns_two_prime_factors() {
        // Arrange
        let target_value = 83u128 * 97u128;
        let expected_factors = vec![83u128, 97u128];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_with_multiple_factors_returns_all_prime_factors() {
        // Arrange
        let target_value = 2u128 * 2u128 * 3u128 * 5u128;
        let expected_factors = vec![2u128, 2u128, 3u128, 5u128];

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);

        // Assert
        assert_eq!(actual_factors, expected_factors);
    }

    #[test]
    fn factorize_by_pollard_rho_result_is_sorted_in_ascending_order() {
        // Arrange
        let target_value = 3u128 * 83u128 * 97u128 * 2u128;
        let expected_is_sorted = true;

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);
        let actual_is_sorted = is_sorted_in_ascending_order(&actual_factors);

        // Assert
        assert_eq!(actual_is_sorted, expected_is_sorted);
    }

    #[test]
    fn factorize_by_pollard_rho_result_product_matches_original_input() {
        // Arrange
        let target_value = 2u128 * 2u128 * 2u128 * 3u128 * 5u128 * 7u128;
        let expected_product = target_value;

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);
        let actual_product = calculate_product(&actual_factors);

        // Assert
        assert_eq!(actual_product, expected_product);
    }

    #[test]
    fn factorize_by_pollard_rho_result_elements_are_all_primes() {
        // Arrange
        let target_value = 2u128 * 2u128 * 3u128 * 5u128 * 7u128;
        let expected_all_elements_are_prime = true;

        // Act
        let actual_factors = factorize_by_pollard_rho(target_value);
        let actual_all_elements_are_prime = actual_factors
            .iter()
            .all(|&factor_value| is_prime_by_trial_division(factor_value));

        // Assert
        assert_eq!(
            actual_all_elements_are_prime,
            expected_all_elements_are_prime
        );
    }
}
