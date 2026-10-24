#[derive(Debug)]
struct Fraction {
    numerator: i32,
    denominator: i32,
}

impl Fraction {

    fn new(numerator: i32, denominator: i32) -> Self {
        Self {
            numerator,
            denominator,
        }
    }

    fn compare(&self, other: &Self) -> std::cmp::Ordering {
        let left_product = self.numerator as i64 * other.denominator as i64;
        let right_product = self.denominator as i64 * other.numerator as i64;
        let ordering = left_product.cmp(&right_product);

        if (self.denominator as i64) * (other.denominator as i64) < 0 {
            ordering.reverse()
        } else {
            ordering
        }
    }
}

impl PartialEq for Fraction {
    fn eq(&self, other: &Self) -> bool {
        self.compare(other) == std::cmp::Ordering::Equal
    }
}

impl Eq for Fraction {}

impl PartialOrd for Fraction {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.compare(other))
    }
}

impl Ord for Fraction {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.compare(other)
    }

}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn sort_keeps_singleton_fraction_set_unchanged() {
        // Arrange
        let mut target_fractions = vec![Fraction::new(1, 2)];
        let expected_fractions = vec![Fraction::new(1, 2)];

        // Act
        target_fractions.sort();
        let actual_fractions = target_fractions;

        // Assert
        assert_eq!(actual_fractions, expected_fractions);
    }

    #[test]
    fn sort_orders_multiple_fractions_in_ascending_rational_order() {
        // Arrange
        let mut target_fractions = vec![
            Fraction::new(3, 4),
            Fraction::new(1, 2),
            Fraction::new(2, 3),
        ];
        let expected_fractions = vec![
            Fraction::new(1, 2),
            Fraction::new(2, 3),
            Fraction::new(3, 4),
        ];

        // Act
        target_fractions.sort();
        let actual_fractions = target_fractions;

        // Assert
        assert_eq!(actual_fractions, expected_fractions);
    }

    #[test]
    fn compare_returns_equal_for_identical_fraction_values() {
        // Arrange
        let left_fraction = Fraction::new(1, 2);
        let right_fraction = Fraction::new(1, 2);
        let expected_ordering = std::cmp::Ordering::Equal;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_returns_less_for_positive_fraction_pair() {
        // Arrange
        let left_fraction = Fraction::new(1, 3);
        let right_fraction = Fraction::new(1, 2);
        let expected_ordering = std::cmp::Ordering::Less;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_returns_greater_for_positive_fraction_pair() {
        // Arrange
        let left_fraction = Fraction::new(3, 4);
        let right_fraction = Fraction::new(2, 3);
        let expected_ordering = std::cmp::Ordering::Greater;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_handles_negative_and_positive_fraction_values() {
        // Arrange
        let left_fraction = Fraction::new(-1, 2);
        let right_fraction = Fraction::new(1, 3);
        let expected_ordering = std::cmp::Ordering::Less;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_treats_negative_denominator_as_equivalent_value() {
        // Arrange
        let left_fraction = Fraction::new(1, -2);
        let right_fraction = Fraction::new(-1, 2);
        let expected_ordering = std::cmp::Ordering::Equal;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_with_negative_denominators_uses_mathematical_ordering() {
        // Arrange
        let negative_half = Fraction::new(1, -2);
        let positive_third = Fraction::new(1, 3);
        let negative_third = Fraction::new(1, -3);
        let expected_orderings = (std::cmp::Ordering::Less, std::cmp::Ordering::Less);

        // Act
        let actual_orderings = (
            negative_half.compare(&positive_third),
            negative_half.compare(&negative_third),
        );

        // Assert
        assert_eq!(actual_orderings, expected_orderings);
    }

    #[test]
    fn compare_handles_zero_numerator_against_negative_fraction() {
        // Arrange
        let left_fraction = Fraction::new(0, 5);
        let right_fraction = Fraction::new(-1, 7);
        let expected_ordering = std::cmp::Ordering::Greater;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_returns_less_equal_greater_around_five_sevenths() {
        // Arrange
        let base_fraction = Fraction::new(5, 7);
        let left_boundary_fraction = Fraction::new(4, 7);
        let center_boundary_fraction = Fraction::new(5, 7);
        let right_boundary_fraction = Fraction::new(6, 7);
        let expected_left_ordering = std::cmp::Ordering::Less;
        let expected_center_ordering = std::cmp::Ordering::Equal;
        let expected_right_ordering = std::cmp::Ordering::Greater;

        // Act
        let actual_left_ordering = left_boundary_fraction.compare(&base_fraction);
        let actual_center_ordering = center_boundary_fraction.compare(&base_fraction);
        let actual_right_ordering = right_boundary_fraction.compare(&base_fraction);

        // Assert
        assert_eq!(actual_left_ordering, expected_left_ordering);
        assert_eq!(actual_center_ordering, expected_center_ordering);
        assert_eq!(actual_right_ordering, expected_right_ordering);
    }

    #[test]
    fn compare_handles_i32_max_nearby_values_without_overflow() {
        // Arrange
        let left_fraction = Fraction::new(i32::MAX, i32::MAX - 1);
        let right_fraction = Fraction::new(i32::MAX - 1, i32::MAX - 2);
        let expected_ordering = std::cmp::Ordering::Less;

        // Act
        let actual_ordering = left_fraction.compare(&right_fraction);

        // Assert
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn compare_with_zero_denominator_does_not_panic_and_returns_greater() {
        // Arrange
        let zero_denominator_fraction = Fraction::new(1, 0);
        let normal_fraction = Fraction::new(1, 1);
        let expected_ordering = std::cmp::Ordering::Greater;

        // Act
        let actual_ordering_result =
            std::panic::catch_unwind(|| zero_denominator_fraction.compare(&normal_fraction));

        // Assert
        assert!(actual_ordering_result.is_ok());
        assert_eq!(actual_ordering_result.unwrap(), expected_ordering);
    }

    #[test]
    fn compare_satisfies_transitivity_for_three_ordered_fractions() {
        // Arrange
        let fraction_a = Fraction::new(1, 3);
        let fraction_b = Fraction::new(1, 2);
        let fraction_c = Fraction::new(2, 3);
        let expected_ordering = std::cmp::Ordering::Less;

        // Act
        let actual_ordering_ab = fraction_a.cmp(&fraction_b);
        let actual_ordering_bc = fraction_b.cmp(&fraction_c);
        let actual_ordering_ac = fraction_a.cmp(&fraction_c);

        // Assert
        assert_eq!(actual_ordering_ab, expected_ordering);
        assert_eq!(actual_ordering_bc, expected_ordering);
        assert_eq!(actual_ordering_ac, expected_ordering);
    }

    #[test]
    fn eq_and_ord_consistency_for_equivalent_rational_values() {
        // Arrange
        let left_fraction = Fraction::new(1, 2);
        let right_fraction = Fraction::new(2, 4);
        let expected_equality = true;
        let expected_ordering = std::cmp::Ordering::Equal;

        // Act
        let actual_equality = left_fraction == right_fraction;
        let actual_ordering = left_fraction.cmp(&right_fraction);

        // Assert
        assert_eq!(actual_equality, expected_equality);
        assert_eq!(actual_ordering, expected_ordering);
    }

    #[test]
    fn partial_cmp_always_returns_some_ordering() {
        // Arrange
        let left_fraction = Fraction::new(1, 3);
        let right_fraction = Fraction::new(1, 2);
        let expected_partial_ordering = Some(std::cmp::Ordering::Less);

        // Act
        let actual_partial_ordering = left_fraction.partial_cmp(&right_fraction);

        // Assert
        assert_eq!(actual_partial_ordering, expected_partial_ordering);
    }

    #[test]
    fn cmp_and_partial_cmp_return_consistent_results() {
        // Arrange
        let left_fraction = Fraction::new(5, 9);
        let right_fraction = Fraction::new(4, 7);

        // Act
        let actual_ordering_by_cmp = left_fraction.cmp(&right_fraction);
        let actual_ordering_by_partial_cmp = left_fraction.partial_cmp(&right_fraction);

        // Assert
        assert_eq!(actual_ordering_by_partial_cmp, Some(actual_ordering_by_cmp));
    }

    #[test]
    fn sort_does_not_panic_for_empty_fraction_set() {
        // Arrange
        let mut target_fractions: Vec<Fraction> = vec![];
        let expected_fractions: Vec<Fraction> = vec![];

        // Act
        target_fractions.sort();
        let actual_fractions = target_fractions;

        // Assert
        assert_eq!(actual_fractions, expected_fractions);
    }
}
