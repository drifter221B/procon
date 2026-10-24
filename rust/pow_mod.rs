fn modular_inverse(n: usize, modulo: usize) -> usize {
    let (mut remainder, mut next_remainder) = (modulo as i128, n as i128);
    let (mut coefficient, mut next_coefficient) = (0, 1);

    // 拡張ユークリッド法なら、法が合成数でも互いに素な値の逆元を求められる。
    while next_remainder != 0 {
        let quotient = remainder / next_remainder;
        (remainder, next_remainder) = (next_remainder, remainder - quotient * next_remainder);
        (coefficient, next_coefficient) =
            (next_coefficient, coefficient - quotient * next_coefficient);
    }

    assert!(
        remainder == 1,
        "n and modulo must be coprime for negative exponent"
    );
    coefficient.rem_euclid(modulo as i128) as usize
}

fn pow_mod(n: usize, p: i64, modulo: usize) -> usize {
    assert!(modulo > 0, "modulo must be positive");

    let mut result = 1 % modulo;
    let mut x = n % modulo;
    let mut exponent = p.unsigned_abs();

    if p < 0 {
        x = modular_inverse(x, modulo);
    }

    while exponent != 0 {
        if exponent % 2 == 1 {
            result = (result as u128 * x as u128 % modulo as u128) as usize;
        }

        x = (x as u128 * x as u128 % modulo as u128) as usize;
        exponent >>= 1;
    }

    result
}


#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};

    use super::pow_mod;

    #[test]
    fn pow_mod_zero_exponent_returns_one() {
        // Arrange
        let n = 5;
        let exponent = 0;
        let modulo = 7;
        let expected_value = 1;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_positive_exponent_returns_power_modulo() {
        // Arrange
        let n = 2;
        let exponent = 3;
        let modulo = 1_000;
        let expected_value = 8;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_negative_exponent_returns_multiplicative_inverse() {
        // Arrange
        let n = 3;
        let exponent = -1;
        let modulo = 1_000_000_007;
        let expected_value = 333_333_336;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_negative_exponent_on_composite_modulus_returns_inverse_power() {
        // Arrange
        let n = 5;
        let exponent = -3;
        let modulo = 12;
        let expected_value = 5;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_minimum_signed_exponent_returns_modular_power() {
        // Arrange
        let n = 3;
        let exponent = i64::MIN;
        let modulo = 7;
        let expected_value = 4;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_large_base_is_squared_without_overflow() {
        // Arrange
        let modulo = usize::MAX;
        let n = modulo - 1;
        let exponent = 2;
        let expected_value = 1;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_large_modulus_returns_multiplicative_inverse() {
        // Arrange
        let modulo = usize::MAX;
        let n = modulo - 1;
        let exponent = -1;
        let expected_value = n;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pow_mod_zero_exponent_with_modulo_one_returns_zero() {
        // Arrange
        let n = 5;
        let exponent = 0;
        let modulo = 1;
        let expected_value = 0;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn modulo_one_returns_zero_for_positive_exponent() {
        // Arrange
        let n = 5;
        let exponent = 3;
        let modulo = 1;
        let expected_value = 0;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn zero_base_with_positive_exponent_returns_zero() {
        // Arrange
        let n = 0;
        let exponent = 3;
        let modulo = 7;
        let expected_value = 0;

        // Act
        let actual_value = pow_mod(n, exponent, modulo);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn negative_exponent_with_non_coprime_inputs_panics() {
        // Arrange
        let n = 2;
        let exponent = -1;
        let modulo = 4;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| pow_mod(n, exponent, modulo)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn zero_modulo_panics() {
        // Arrange
        let n = 2;
        let exponent = 3;
        let modulo = 0;

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| pow_mod(n, exponent, modulo)));

        // Assert
        assert!(actual_result.is_err());
    }
}
