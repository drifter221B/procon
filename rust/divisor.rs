pub fn divisors(n: u64) -> Vec<u64> {
    if n == 0 {
        return vec![];
    }

    let mut small_divisors = Vec::new();
    let mut large_divisors = Vec::new();
    let mut divisor_candidate = 1u64;

    while is_search_candidate_within_sqrt_limit(divisor_candidate, n) {
        if n % divisor_candidate == 0 {
            small_divisors.push(divisor_candidate);
            let paired_divisor = n / divisor_candidate;
            if divisor_candidate != paired_divisor {
                large_divisors.push(paired_divisor);
            }
        }
        divisor_candidate += 1;
    }

    large_divisors.reverse();
    small_divisors.extend(large_divisors);
    small_divisors
}

fn is_search_candidate_within_sqrt_limit(divisor_candidate: u64, target_number: u64) -> bool {
    if divisor_candidate == 0 {
        return false;
    }

    divisor_candidate <= target_number / divisor_candidate
}

#[cfg(test)]
mod tests {
    use super::{divisors, is_search_candidate_within_sqrt_limit};

    #[test]
    fn divisors_with_zero_returns_empty_vector() {
        // Arrange
        let input_number = 0u64;
        let expected_divisors: Vec<u64> = vec![];

        // Act
        let actual_divisors = divisors(input_number);

        // Assert
        assert_eq!(actual_divisors, expected_divisors);
    }

    #[test]
    fn divisors_with_boundary_values_x_minus_1_x_x_plus_1_returns_expected_vectors() {
        // Arrange
        let x_value = 1u64;
        let input_x_minus_1 = x_value - 1;
        let input_x = x_value;
        let input_x_plus_1 = x_value + 1;
        let expected_x_minus_1_divisors: Vec<u64> = vec![];
        let expected_x_divisors = vec![1];
        let expected_x_plus_1_divisors = vec![1, 2];

        // Act
        let actual_x_minus_1_divisors = divisors(input_x_minus_1);
        let actual_x_divisors = divisors(input_x);
        let actual_x_plus_1_divisors = divisors(input_x_plus_1);

        // Assert
        assert_eq!(actual_x_minus_1_divisors, expected_x_minus_1_divisors);
        assert_eq!(actual_x_divisors, expected_x_divisors);
        assert_eq!(actual_x_plus_1_divisors, expected_x_plus_1_divisors);
    }

    #[test]
    fn divisors_with_prime_input_returns_only_one_and_itself() {
        // Arrange
        let input_number = 13u64;
        let expected_divisors = vec![1, 13];

        // Act
        let actual_divisors = divisors(input_number);

        // Assert
        assert_eq!(actual_divisors, expected_divisors);
    }

    #[test]
    fn divisors_with_composite_input_returns_all_divisors_in_ascending_order() {
        // Arrange
        let input_number = 12u64;
        let expected_divisors = vec![1, 2, 3, 4, 6, 12];

        // Act
        let actual_divisors = divisors(input_number);

        // Assert
        assert_eq!(actual_divisors, expected_divisors);
    }

    #[test]
    fn divisors_with_perfect_square_contains_square_root_once() {
        // Arrange
        let input_number = 36u64;
        let square_root_value = 6u64;
        let expected_square_root_count = 1usize;

        // Act
        let actual_divisors = divisors(input_number);
        let actual_square_root_count = actual_divisors
            .iter()
            .filter(|&&divisor_value| divisor_value == square_root_value)
            .count();

        // Assert
        assert_eq!(actual_square_root_count, expected_square_root_count);
    }

    #[test]
    fn divisors_with_prime_power_input_returns_all_power_of_two_divisors() {
        // Arrange
        let input_number = 1024u64;
        let expected_divisors = vec![1, 2, 4, 8, 16, 32, 64, 128, 256, 512, 1024];

        // Act
        let actual_divisors = divisors(input_number);

        // Assert
        assert_eq!(actual_divisors, expected_divisors);
    }

    #[test]
    fn divisors_result_is_sorted_in_ascending_order() {
        // Arrange
        let input_number = 18u64;
        let expected_is_ascending = true;

        // Act
        let actual_divisors = divisors(input_number);
        let actual_is_ascending = actual_divisors
            .windows(2)
            .all(|adjacent_values| adjacent_values[0] <= adjacent_values[1]);

        // Assert
        assert_eq!(actual_is_ascending, expected_is_ascending);
    }

    #[test]
    fn divisors_each_element_divides_input_number() {
        // Arrange
        let input_number = 840u64;
        let expected_all_elements_are_divisors = true;

        // Act
        let actual_divisors = divisors(input_number);
        let actual_all_elements_are_divisors = actual_divisors
            .iter()
            .all(|&divisor_value| input_number % divisor_value == 0);

        // Assert
        assert_eq!(
            actual_all_elements_are_divisors,
            expected_all_elements_are_divisors
        );
    }

    #[test]
    fn divisors_result_contains_no_duplicate_elements() {
        // Arrange
        let input_number = 360u64;

        // Act
        let actual_divisors = divisors(input_number);
        let mut deduplicated_divisors = actual_divisors.clone();
        deduplicated_divisors.dedup();
        let actual_unique_count = deduplicated_divisors.len();
        let actual_original_count = actual_divisors.len();

        // Assert
        assert_eq!(actual_original_count, actual_unique_count);
    }

    #[test]
    fn search_limit_check_with_u64_max_around_square_root_boundary_avoids_overflow() {
        // Arrange
        let target_number = u64::MAX;
        let boundary_candidate = u32::MAX as u64;
        let boundary_plus_one_candidate = boundary_candidate + 1;
        let expected_boundary_result = true;
        let expected_boundary_plus_one_result = false;

        // Act
        let actual_boundary_result =
            is_search_candidate_within_sqrt_limit(boundary_candidate, target_number);
        let actual_boundary_plus_one_result =
            is_search_candidate_within_sqrt_limit(boundary_plus_one_candidate, target_number);

        // Assert
        assert_eq!(actual_boundary_result, expected_boundary_result);
        assert_eq!(
            actual_boundary_plus_one_result,
            expected_boundary_plus_one_result
        );
    }
}
