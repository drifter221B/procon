pub fn prime_sieve(x: usize) -> Vec<bool> {
    let mut sieve = vec![true; x + 1];

    if x == 0 {
        sieve[0] = false;
        return sieve;
    }

    sieve[0] = false;
    sieve[1] = false;

    let mut y = 2;
    while y * y <= x {
        if sieve[y] {
            for z in ((y * y)..=x).step_by(y) {
                sieve[z] = false;
            }
        }
        y += 1;
    }

    sieve
}

pub const fn prime_sieve_const<const N: usize>() -> [bool; N] {
    let mut sieve = [true; N];

    if N == 0 {
        return sieve;
    }

    sieve[0] = false;

    if N == 1 {
        return sieve;
    }

    sieve[1] = false;

    let max_index = N - 1;
    let mut y = 2;
    while y <= max_index / y {
        if sieve[y] {
            let mut z = y * y;
            while z <= max_index {
                sieve[z] = false;
                if y <= max_index - z {
                    z += y;
                } else {
                    break;
                }
            }
        }
        y += 1;
    }

    sieve
}

#[allow(dead_code)]
fn smallest_prime_factors(x: usize) -> Vec<usize> {

    let mut spf = (0..=x).collect::<Vec<usize>>();
    let mut primes = Vec::with_capacity(x + 1);

    for i in 2..=x {

        if spf[i] == i {
            primes.push(i);
        }

        for &prime in &primes {

            let multiple = prime * i;
            if x < multiple {
                break;
            }

            spf[multiple] = prime;

            if spf[i] == prime {
                break;
            }

        }

    }

    spf
}

pub fn divisor(x: i64) -> Vec<i64> {
    let mut forward_results = Vec::new();
    let mut backward_results = Vec::new();
    let mut divisor_candidate = 1;

    while divisor_candidate * divisor_candidate <= x {
        if x % divisor_candidate == 0 {
            forward_results.push(divisor_candidate);
            if divisor_candidate != x / divisor_candidate {
                backward_results.push(x / divisor_candidate);
            }
        }
        divisor_candidate += 1;
    }

    backward_results.reverse();
    forward_results.extend_from_slice(&backward_results);
    forward_results
}

pub fn prime_factorise(x: i64) -> Vec<(i64, i32)> {
    let mut results = Vec::new();
    let mut divisor_candidate = 2;
    let mut remaining_value = x;

    while divisor_candidate * divisor_candidate <= remaining_value {
        let mut count = 0;
        while remaining_value % divisor_candidate == 0 {
            count += 1;
            remaining_value /= divisor_candidate;
        }

        if 0 < count {
            results.push((divisor_candidate, count));
        }

        divisor_candidate += 2 - ((divisor_candidate % 2 == 0) as i64);
    }

    if 1 < remaining_value {
        results.push((remaining_value, 1));
    }

    results
}

pub fn gcd(x: i64, y: i64) -> i64 {
    let mut x = x;
    let mut y = y;

    while y != 0 {
        (x, y) = (y, x % y);
    }

    x
}

pub fn extend_gcd(a: i64, b: i64) -> (i64, i64, i64) {
    if b != 0 {
        let (d, y, x) = extend_gcd(b, a % b);
        return (d, x, y - (a / b) * x);
    }

    match a.cmp(&0) {
        std::cmp::Ordering::Less => (-a, -1, 0),
        _ => (a, 1, 0),
    }
}

pub fn extend_gcd_min_pair(a: i64, b: i64) -> (i64, i64, i64) {
    let (g, x0, y0) = extend_gcd(a, b);

    if g == 0 || b == 0 {
        return (g, x0, y0);
    }

    let step = b.abs() / g;
    let x_min = (step + x0 % step) % step;
    let denom = b / g;
    let k = (x_min - x0) / denom;
    let y_min = (y0 as i128 - (k as i128 * (a / g) as i128)) as i64;

    (g, x_min, y_min)
}

pub fn is_collinear(coordinates: Vec<Vec<i64>>) -> bool {
    if coordinates.len() <= 2 {
        return true;
    }

    let origin = &coordinates[0];
    let Some(other) = coordinates.iter().find(|point| *point != origin) else {
        return true;
    };
    let delta_x = origin[0] - other[0];
    let delta_y = origin[1] - other[1];

    coordinates.iter().all(|point| {
        delta_y * (point[0] - coordinates[0][0]) == (point[1] - coordinates[0][1]) * delta_x
    })
}

#[cfg(test)]
mod tests {
    use super::{
        divisor, extend_gcd, extend_gcd_min_pair, gcd, is_collinear, prime_factorise, prime_sieve,
        prime_sieve_const,
    };

    #[test]
    fn prime_sieve_const_returns_empty_when_n_is_zero() {
        // Arrange
        const N: usize = 0;
        let expected_length = 0;
        // Act
        let actual_result = prime_sieve_const::<N>();
        let actual_length = actual_result.len();
        // Assert
        assert_eq!(actual_length, expected_length);
    }

    #[test]
    fn prime_sieve_const_sets_zero_false_when_n_is_one() {
        // Arrange
        const N: usize = 1;
        let expected_result = [false];
        // Act
        let actual_result = prime_sieve_const::<N>();
        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn prime_sieve_const_marks_two_as_prime_in_minimum_range() {
        // Arrange
        const N: usize = 3;
        let prime_index = 2;
        let expected_value = true;
        // Act
        let actual_result = prime_sieve_const::<N>();
        let actual_value = actual_result[prime_index];
        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn prime_sieve_const_creates_sieve_up_to_ten() {
        // Arrange
        const N: usize = 11;
        let expected = [
            false, false, true, true, false, true, false, true, false, false, false,
        ];
        // Act
        let actual_result = prime_sieve_const::<N>();
        // Assert
        assert_eq!(actual_result, expected);
    }

    #[test]
    fn divisor_of_one_returns_singleton_vector() {
        // Arrange
        let value = 1;
        let expected = vec![1];

        // Act
        let actual_values = divisor(value);

        // Assert
        assert_eq!(actual_values, expected);
    }

    #[test]
    fn prime_factorise_of_one_returns_empty_vector() {
        // Arrange
        let value = 1;

        // Act
        let actual_values = prime_factorise(value);

        // Assert
        assert!(actual_values.is_empty());
    }

    #[test]
    fn prime_factorise_groups_duplicate_prime_factors() {
        // Arrange
        let value = 72;
        let expected = vec![(2, 3), (3, 2)];

        // Act
        let actual_values = prime_factorise(value);

        // Assert
        assert_eq!(actual_values, expected);
    }

    #[test]
    fn extend_gcd_returns_coefficients_that_satisfy_bezout_identity() {
        // Arrange
        let a = 30;
        let b = 18;
        let expected_gcd = 6;

        // Act
        let actual_values = extend_gcd(a, b);
        let actual_identity = a * actual_values.1 + b * actual_values.2;

        // Assert
        assert_eq!(actual_values.0, expected_gcd);
        assert_eq!(actual_identity, actual_values.0);
    }

    #[test]
    fn extend_gcd_min_pair_keeps_current_behavior_for_opposite_sign_inputs() {
        // Arrange
        let a = -6;
        let b = 9;
        let expected = (3, 1, 1);

        // Act
        let actual_values = extend_gcd_min_pair(a, b);
        let actual_identity = a * actual_values.1 + b * actual_values.2;

        // Assert
        assert_eq!(actual_values, expected);
        assert_eq!(actual_identity, actual_values.0);
    }

    #[test]
    fn is_collinear_returns_false_for_non_collinear_points() {
        // Arrange
        let points = vec![vec![0, 0], vec![1, 1], vec![2, 3]];

        // Act
        let actual_value = is_collinear(points);

        // Assert
        assert!(!actual_value);
    }

    #[test]
    fn gcd_with_both_zero_returns_current_signed_result() {
        // Arrange
        let left = 0;
        let right = 0;
        let expected = 0;

        // Act
        let actual = gcd(left, right);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn gcd_with_zero_and_negative_returns_current_signed_result() {
        // Arrange
        let left = 0;
        let right = -5;
        let expected = -5;

        // Act
        let actual = gcd(left, right);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn gcd_with_negative_and_positive_returns_current_signed_result() {
        // Arrange
        let left = -12;
        let right = 18;
        let expected = 6;

        // Act
        let actual = gcd(left, right);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_empty_returns_true() {
        // Arrange
        let points = Vec::<Vec<i64>>::new();
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_singleton_returns_true() {
        // Arrange
        let points = vec![vec![1, 2]];
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_two_points_returns_true() {
        // Arrange
        let points = vec![vec![1, 2], vec![3, 4]];
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_three_points_returns_true() {
        // Arrange
        let points = vec![vec![0, 0], vec![1, 2], vec![2, 4]];
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_identical_points_returns_true() {
        // Arrange
        let points = vec![vec![1, 2]; 3];
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_duplicate_first_pair_returns_true() {
        // Arrange
        let points = vec![vec![0, 0], vec![0, 0], vec![1, 2], vec![2, 4]];
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_reordered_points_returns_true() {
        // Arrange
        let points = vec![vec![2, 4], vec![0, 0], vec![1, 2], vec![0, 0]];
        let expected = true;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_collinear_with_duplicate_first_pair_and_non_collinear_remainder_returns_false() {
        // Arrange
        let points = vec![vec![0, 0], vec![0, 0], vec![1, 1], vec![2, 3]];
        let expected = false;

        // Act
        let actual = is_collinear(points);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn prime_sieve_with_upper_bound_zero_through_three_marks_only_primes() {
        // Arrange
        let cases = [
            (0, vec![false]),
            (1, vec![false, false]),
            (2, vec![false, false, true]),
            (3, vec![false, false, true, true]),
        ];
        let expected: Vec<_> = cases.iter().map(|(_, values)| values.clone()).collect();

        // Act
        let actual: Vec<_> = cases.iter().map(|(upper, _)| prime_sieve(*upper)).collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn extend_gcd_with_positive_a_and_zero_b_returns_base_coefficients() {
        // Arrange
        let a = 7;
        let b = 0;
        let expected = (7, 1, 0);

        // Act
        let actual = extend_gcd(a, b);
        let actual_identity = a * actual.1 + b * actual.2;

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_identity, expected.0);
    }

    #[test]
    fn extend_gcd_with_negative_a_and_zero_b_returns_base_coefficients() {
        // Arrange
        let a = -7;
        let b = 0;
        let expected = (7, -1, 0);

        // Act
        let actual = extend_gcd(a, b);
        let actual_identity = a * actual.1 + b * actual.2;

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_identity, expected.0);
    }

    #[test]
    fn extend_gcd_with_zero_a_and_zero_b_returns_base_coefficients() {
        // Arrange
        let a = 0;
        let b = 0;
        let expected = (0, 1, 0);

        // Act
        let actual = extend_gcd(a, b);
        let actual_identity = a * actual.1 + b * actual.2;

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_identity, expected.0);
    }

    #[test]
    fn extend_gcd_min_pair_with_positive_a_and_zero_b_returns_base_coefficients() {
        // Arrange
        let a = 7;
        let b = 0;
        let expected = (7, 1, 0);

        // Act
        let actual = extend_gcd_min_pair(a, b);
        let actual_identity = a * actual.1 + b * actual.2;

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_identity, expected.0);
    }

    #[test]
    fn extend_gcd_min_pair_with_negative_a_and_zero_b_returns_base_coefficients() {
        // Arrange
        let a = -7;
        let b = 0;
        let expected = (7, -1, 0);

        // Act
        let actual = extend_gcd_min_pair(a, b);
        let actual_identity = a * actual.1 + b * actual.2;

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_identity, expected.0);
    }

    #[test]
    fn extend_gcd_min_pair_with_zero_a_and_zero_b_returns_base_coefficients() {
        // Arrange
        let a = 0;
        let b = 0;
        let expected = (0, 1, 0);

        // Act
        let actual = extend_gcd_min_pair(a, b);
        let actual_identity = a * actual.1 + b * actual.2;

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_identity, expected.0);
    }
}
