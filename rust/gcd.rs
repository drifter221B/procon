fn gcd<T>(x: T, y: T) -> T
where T: Copy + Default + PartialEq + std::ops::Rem<Output = T> {

    let mut x = x;
    let mut y = y;

    while y != T::default() {
        (x, y) = (y, x % y);
    }

    x
}

#[cfg(test)]
mod todo_tests {
    use super::gcd;

    #[test]
    fn zero_and_one_boundaries_return_expected_values() {
        // Arrange
        let inputs = [(0i64, 0i64), (0i64, 1i64), (1i64, 0i64), (1i64, 1i64)];
        let expected_values = (0, 1, 1, 1);

        // Act
        let actual_values = (
            gcd(inputs[0].0, inputs[0].1),
            gcd(inputs[1].0, inputs[1].1),
            gcd(inputs[2].0, inputs[2].1),
            gcd(inputs[3].0, inputs[3].1),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn positive_numbers_return_expected_gcd_for_order_coprime_multiple_and_neighbors() {
        // Arrange
        let inputs = [
            (14i64, 21i64),
            (21i64, 14i64),
            (35i64, 64i64),
            (24i64, 8i64),
            (9i64, 10i64),
            (10i64, 10i64),
            (11i64, 10i64),
        ];
        let expected_values = (7, 7, 1, 8, 1, 10, 1);

        // Act
        let actual_values = (
            gcd(inputs[0].0, inputs[0].1),
            gcd(inputs[1].0, inputs[1].1),
            gcd(inputs[2].0, inputs[2].1),
            gcd(inputs[3].0, inputs[3].1),
            gcd(inputs[4].0, inputs[4].1),
            gcd(inputs[5].0, inputs[5].1),
            gcd(inputs[6].0, inputs[6].1),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn signed_numbers_follow_current_sign_behavior() {
        // Arrange
        let inputs = [(-12i64, 18i64), (12i64, -18i64), (-12i64, -18i64)];
        let expected_values = (6, -6, -6);

        // Act
        let actual_values = (
            gcd(inputs[0].0, inputs[0].1),
            gcd(inputs[1].0, inputs[1].1),
            gcd(inputs[2].0, inputs[2].1),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn unsigned_upper_bound_returns_expected_gcd() {
        // Arrange
        let max_input = (0u64, u64::MAX);
        let neighbor_input = (u64::MAX, u64::MAX - 1);
        let expected_values = (u64::MAX, 1);

        // Act
        let actual_values = (
            gcd(max_input.0, max_input.1),
            gcd(neighbor_input.0, neighbor_input.1),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn gcd_is_commutative_and_divides_both_inputs() {
        // Arrange
        let inputs = (48u64, 18u64);
        let expected_gcd = 6u64;

        // Act
        let actual_ab = gcd(inputs.0, inputs.1);
        let actual_ba = gcd(inputs.1, inputs.0);
        let actual_divisibility = inputs.0 % actual_ab == 0 && inputs.1 % actual_ab == 0;

        // Assert
        assert_eq!(actual_ab, expected_gcd);
        assert_eq!(actual_ab, actual_ba);
        assert!(actual_divisibility);
    }
}
