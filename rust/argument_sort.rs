use std::cmp::Ordering;

pub type Point = (i64, i64);
type Vector = (i128, i128);

#[derive(Debug, PartialEq, Eq)]
pub enum ArgumentSortError {
    EmptyInput,
}

fn shift_to_origin(point: Point, origin: Point) -> Vector {
    (
        i128::from(point.0) - i128::from(origin.0),
        i128::from(point.1) - i128::from(origin.1),
    )
}

fn is_half_plane(vector: Vector) -> u8 {
    if 0 < vector.1 {
        return 0;
    }

    if vector.1 == 0 && 0 <= vector.0 {
        return 0;
    }

    1
}

fn compare_signed_products(
    lhs_first_factor: i128,
    lhs_second_factor: i128,
    rhs_first_factor: i128,
    rhs_second_factor: i128,
) -> Ordering {
    let lhs_sign = lhs_first_factor.signum() * lhs_second_factor.signum();
    let rhs_sign = rhs_first_factor.signum() * rhs_second_factor.signum();

    let sign_order = lhs_sign.cmp(&rhs_sign);
    if sign_order != Ordering::Equal {
        return sign_order;
    }

    let lhs_magnitude = lhs_first_factor.unsigned_abs() * lhs_second_factor.unsigned_abs();
    let rhs_magnitude = rhs_first_factor.unsigned_abs() * rhs_second_factor.unsigned_abs();

    if lhs_sign < 0 {
        rhs_magnitude.cmp(&lhs_magnitude)
    } else {
        lhs_magnitude.cmp(&rhs_magnitude)
    }
}

fn compare_cross_product_to_zero(lhs: Vector, rhs: Vector) -> Ordering {
    compare_signed_products(lhs.0, rhs.1, lhs.1, rhs.0)
}

fn squared_length(vector: Vector) -> (bool, u128) {
    let x = vector.0.unsigned_abs();
    let y = vector.1.unsigned_abs();
    let (low, carry) = (x * x).overflowing_add(y * y);

    (carry, low)
}

fn compare_by_argument(lhs: Vector, rhs: Vector) -> Ordering {
    let lhs_half = is_half_plane(lhs);
    let rhs_half = is_half_plane(rhs);

    if lhs_half != rhs_half {
        return lhs_half.cmp(&rhs_half);
    }

    let cross_order = compare_cross_product_to_zero(lhs, rhs);
    if cross_order != Ordering::Equal {
        return cross_order.reverse();
    }

    let lhs_length = squared_length(lhs);
    let rhs_length = squared_length(rhs);

    lhs_length.cmp(&rhs_length)
}

pub fn argument_sort(points: &[Point], origin: Point) -> Result<Vec<Point>, ArgumentSortError> {
    if points.is_empty() {
        return Err(ArgumentSortError::EmptyInput);
    }

    let mut shifted_points: Vec<(Point, Vector)> = points
        .iter()
        .copied()
        .filter(|&point| point != origin)
        .map(|point| (point, shift_to_origin(point, origin)))
        .collect();

    shifted_points.sort_by(|lhs, rhs| compare_by_argument(lhs.1, rhs.1));

    Ok(shifted_points.into_iter().map(|(point, _)| point).collect())
}

#[cfg(test)]
mod tests {
    use std::cmp::Ordering;

    use super::{argument_sort, shift_to_origin, ArgumentSortError, Point};

    fn sort_points_by_argument_sort(points: &[Point], origin: Point) -> Vec<Point> {
        let sorted_points_result = argument_sort(points, origin);
        sorted_points_result.expect("argument_sort should succeed")
    }

    fn sort_points_by_naive_atan2(points: &[Point], origin: Point) -> Vec<Point> {
        let mut shifted_points_with_angle: Vec<(Point, f64, i128)> = points
            .iter()
            .copied()
            .filter(|&point| point != origin)
            .map(|point| {
                let shifted_vector = shift_to_origin(point, origin);
                let mut normalized_angle = (shifted_vector.1 as f64).atan2(shifted_vector.0 as f64);
                if normalized_angle < 0.0 {
                    normalized_angle += std::f64::consts::TAU;
                }
                let squared_distance =
                    shifted_vector.0 * shifted_vector.0 + shifted_vector.1 * shifted_vector.1;
                (point, normalized_angle, squared_distance)
            })
            .collect();

        shifted_points_with_angle.sort_by(|lhs, rhs| {
            let angle_order = lhs
                .1
                .partial_cmp(&rhs.1)
                .expect("atan2 result should be comparable");
            if angle_order != Ordering::Equal {
                return angle_order;
            }
            lhs.2.cmp(&rhs.2)
        });

        shifted_points_with_angle
            .into_iter()
            .map(|(point, _, _)| point)
            .collect()
    }

    fn sort_points_by_i128_reference(points: &[Point], origin: Point) -> Vec<Point> {
        let mut shifted_points: Vec<(Point, (i128, i128))> = points
            .iter()
            .copied()
            .filter(|&point| point != origin)
            .map(|point| {
                let shifted_vector = (
                    point.0 as i128 - origin.0 as i128,
                    point.1 as i128 - origin.1 as i128,
                );
                (point, shifted_vector)
            })
            .collect();

        shifted_points.sort_by(|lhs, rhs| {
            let lhs_vector = lhs.1;
            let rhs_vector = rhs.1;
            let lhs_half = lhs_vector.1 < 0 || (lhs_vector.1 == 0 && lhs_vector.0 < 0);
            let rhs_half = rhs_vector.1 < 0 || (rhs_vector.1 == 0 && rhs_vector.0 < 0);

            if lhs_half != rhs_half {
                return lhs_half.cmp(&rhs_half);
            }

            let cross = lhs_vector.0 * rhs_vector.1 - lhs_vector.1 * rhs_vector.0;
            if cross != 0 {
                return if 0 < cross {
                    Ordering::Less
                } else {
                    Ordering::Greater
                };
            }

            let lhs_length = lhs_vector.0 * lhs_vector.0 + lhs_vector.1 * lhs_vector.1;
            let rhs_length = rhs_vector.0 * rhs_vector.0 + rhs_vector.1 * rhs_vector.1;
            lhs_length.cmp(&rhs_length)
        });

        shifted_points.into_iter().map(|(point, _)| point).collect()
    }

    #[test]
    fn argument_sort_orders_same_ray_points_when_squared_length_sum_exceeds_i64_range() {
        // Arrange
        let origin = (0, 0);
        let farther_point = (2_147_483_648, 2_147_483_648);
        let nearer_point = (1, 1);
        let points = vec![farther_point, nearer_point];
        let expected_sorted_points = vec![nearer_point, farther_point];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_returns_empty_input_error_when_points_is_empty() {
        // Arrange
        let points: Vec<Point> = vec![];
        let origin = (0, 0);
        let expected_error = ArgumentSortError::EmptyInput;

        // Act
        let actual_result = argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_result, Err(expected_error));
    }

    #[test]
    fn argument_sort_returns_empty_vector_when_points_contains_only_origin_singleton() {
        // Arrange
        let origin = (2, -3);
        let points = vec![origin];
        let expected_sorted_points: Vec<Point> = vec![];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_returns_empty_vector_when_points_contains_only_origin_multiple_elements() {
        // Arrange
        let origin = (4, 7);
        let points = vec![origin, origin, origin];
        let expected_sorted_points: Vec<Point> = vec![];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_returns_the_same_point_when_points_contains_one_non_origin_point() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(2, -3)];
        let expected_sorted_points = vec![(2, -3)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_filters_out_only_origin_points_when_origin_points_are_mixed() {
        // Arrange
        let origin = (0, 0);
        let points = vec![origin, (1, 0), origin, (0, 1)];
        let expected_sorted_points = vec![(1, 0), (0, 1)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_sorts_by_angle_after_translation_when_origin_is_not_zero() {
        // Arrange
        let origin = (10, -5);
        let points = vec![origin, (10, -6), (9, -5), (12, -5), (10, -4)];
        let expected_sorted_points = vec![(12, -5), (10, -4), (9, -5), (10, -6)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_eight_direction_points_in_counterclockwise_order() {
        // Arrange
        let origin = (0, 0);
        let points = vec![
            (-1, -1),
            (1, 1),
            (0, -1),
            (-1, 1),
            (1, 0),
            (1, -1),
            (0, 1),
            (-1, 0),
        ];
        let expected_sorted_points = vec![
            (1, 0),
            (1, 1),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_treats_positive_x_axis_as_upper_half_plane_at_y_minus_one_zero_and_one() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(1, -1), (1, 0), (1, 1)];
        let expected_sorted_points = vec![(1, 0), (1, 1), (1, -1)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_treats_negative_x_axis_as_lower_half_plane_at_y_minus_one_zero_and_one() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(-1, 0), (-1, -1), (-1, 1)];
        let expected_sorted_points = vec![(-1, 1), (-1, 0), (-1, -1)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_same_direction_points_by_distance_ascending() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(3, 3), (1, 1), (2, 2)];
        let expected_sorted_points = vec![(1, 1), (2, 2), (3, 3)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_prioritizes_half_plane_over_distance_for_opposite_direction_collinear_points()
    {
        // Arrange
        let origin = (0, 0);
        let points = vec![(-1, 0), (2, 0)];
        let expected_sorted_points = vec![(2, 0), (-1, 0)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_preserves_duplicate_points_in_result() {
        // Arrange
        let origin = (0, 0);
        let duplicate_point = (1, 0);
        let points = vec![duplicate_point, (0, 1), duplicate_point, (0, -1)];
        let expected_sorted_points = vec![duplicate_point, duplicate_point, (0, 1), (0, -1)];
        let expected_duplicate_count = 2usize;
        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);
        let actual_duplicate_count = actual_sorted_points
            .iter()
            .filter(|&&point| point == duplicate_point)
            .count();
        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
        assert_eq!(actual_duplicate_count, expected_duplicate_count);
    }

    #[test]
    fn argument_sort_returns_same_result_for_different_input_orders() {
        // Arrange
        let origin = (0, 0);
        let points_order_a = vec![(1, -1), (0, 1), (-1, -1), (2, 0), (-1, 1)];
        let points_order_b = vec![(-1, 1), (2, 0), (-1, -1), (1, -1), (0, 1)];

        // Act
        let actual_sorted_points_from_a = sort_points_by_argument_sort(&points_order_a, origin);
        let actual_sorted_points_from_b = sort_points_by_argument_sort(&points_order_b, origin);

        // Assert
        assert_eq!(actual_sorted_points_from_a, actual_sorted_points_from_b);
    }

    #[test]
    fn argument_sort_matches_naive_atan2_based_sort_for_multiple_points() {
        // Arrange
        let origin = (0, 0);
        let points = vec![
            (5, 1),
            (2, 3),
            (-1, 4),
            (-4, 1),
            (-3, -2),
            (-1, -4),
            (3, -3),
            (4, -1),
            (2, -5),
        ];
        let expected_sorted_points = sort_points_by_naive_atan2(&points, origin);

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_returns_only_non_origin_point_when_filtering_leaves_one_point() {
        // Arrange
        let origin = (3, -4);
        let non_origin = (5, 6);
        let points = vec![origin, non_origin, origin];
        let expected_sorted_points = vec![non_origin];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_same_ray_points_by_distance_from_non_zero_origin() {
        // Arrange
        let origin = (-10, 0);
        let points = vec![(-8, 0), (-9, 0)];
        let expected_sorted_points = vec![(-9, 0), (-8, 0)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_distinguishes_near_equal_extreme_angles_without_floating_point_rounding() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(i64::MAX - 1, 1), (i64::MAX, 1)];
        let expected_sorted_points = vec![(i64::MAX, 1), (i64::MAX - 1, 1)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_matches_i128_reference_for_deterministic_bounded_cases() {
        // Arrange
        let origins = [(0, 0), (7, -11), (-19, 23)];
        let relative_vectors = [
            (0, 0),
            (-1, -1),
            (2, 0),
            (0, 1),
            (1, 0),
            (-1, 0),
            (1, -1),
            (-1, 1),
            (4, 2),
            (2, 1),
            (1, 0),
            (0, -1),
        ];
        let expected_relative_vectors = [
            (1, 0),
            (1, 0),
            (2, 0),
            (2, 1),
            (4, 2),
            (0, 1),
            (-1, 1),
            (-1, 0),
            (-1, -1),
            (0, -1),
            (1, -1),
        ];
        let points_by_case: Vec<Vec<Point>> = origins
            .iter()
            .map(|&origin| {
                relative_vectors
                    .iter()
                    .map(|&vector| (origin.0 + vector.0, origin.1 + vector.1))
                    .collect()
            })
            .collect();
        let expected_sorted_points_by_case: Vec<Vec<Point>> = points_by_case
            .iter()
            .zip(origins.iter())
            .map(|(_, &origin)| {
                expected_relative_vectors
                    .iter()
                    .map(|&vector| (origin.0 + vector.0, origin.1 + vector.1))
                    .collect()
            })
            .collect();

        // Act
        let reference_sorted_points_by_case: Vec<Vec<Point>> = points_by_case
            .iter()
            .zip(origins.iter())
            .map(|(points, &origin)| sort_points_by_i128_reference(points, origin))
            .collect();
        let actual_sorted_points_by_case: Vec<Vec<Point>> = points_by_case
            .iter()
            .zip(origins.iter())
            .map(|(points, &origin)| sort_points_by_argument_sort(points, origin))
            .collect();

        // Assert
        assert_eq!(
            reference_sorted_points_by_case,
            expected_sorted_points_by_case
        );
        assert_eq!(
            actual_sorted_points_by_case,
            reference_sorted_points_by_case
        );
    }

    #[test]
    fn argument_sort_handles_x_translation_beyond_i64_range() {
        // Arrange
        let origin = (i64::MIN, 0);
        let points = vec![(i64::MAX, 0), (i64::MIN, 1)];
        let expected_sorted_points = vec![(i64::MAX, 0), (i64::MIN, 1)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_handles_y_translation_beyond_i64_range() {
        // Arrange
        let origin = (0, i64::MIN);
        let points = vec![(0, i64::MAX), (-1, i64::MIN)];
        let expected_sorted_points = vec![(0, i64::MAX), (-1, i64::MIN)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_points_when_cross_product_multiplication_exceeds_i64_range() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(1, 2), (i64::MAX, 1)];
        let expected_sorted_points = vec![(i64::MAX, 1), (1, 2)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_preserves_order_independence_when_cross_product_difference_exceeds_i64_range()
    {
        // Arrange
        let origin = (0, 0);
        let first_quadrant_point = (2_147_483_648, 2_147_483_648);
        let second_quadrant_point = (-2_147_483_648, 2_147_483_648);
        let points_order_a = vec![first_quadrant_point, second_quadrant_point];
        let points_order_b = vec![second_quadrant_point, first_quadrant_point];
        let expected_sorted_points = vec![first_quadrant_point, second_quadrant_point];

        // Act
        let actual_sorted_points_from_a = sort_points_by_argument_sort(&points_order_a, origin);
        let actual_sorted_points_from_b = sort_points_by_argument_sort(&points_order_b, origin);

        // Assert
        assert_eq!(actual_sorted_points_from_a, expected_sorted_points);
        assert_eq!(actual_sorted_points_from_b, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_same_ray_points_when_component_square_exceeds_i64_range() {
        // Arrange
        let origin = (0, 0);
        let farther_point = (3_037_000_500, 0);
        let nearer_point = (3_037_000_499, 0);
        let points = vec![farther_point, nearer_point];
        let expected_sorted_points = vec![nearer_point, farther_point];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_points_when_cross_product_exceeds_i128_range() {
        // Arrange
        let origin = (i64::MIN, i64::MIN);
        let higher_angle_point = (i64::MIN + 1, i64::MAX);
        let lower_angle_point = (i64::MAX, i64::MIN + 1);
        let points = vec![higher_angle_point, lower_angle_point];
        let expected_sorted_points = vec![lower_angle_point, higher_angle_point];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_same_ray_points_when_squared_lengths_exceed_u128_range() {
        // Arrange
        let origin = (i64::MIN, i64::MIN);
        let farther_point = (i64::MAX, i64::MAX);
        let nearer_point = (i64::MAX - 1, i64::MAX - 1);
        let points = vec![farther_point, nearer_point];
        let expected_sorted_points = vec![nearer_point, farther_point];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }

    #[test]
    fn argument_sort_orders_extreme_perpendicular_points_without_overflow() {
        // Arrange
        let origin = (0, 0);
        let points = vec![(1, i64::MAX), (i64::MAX, 1)];
        let expected_sorted_points = vec![(i64::MAX, 1), (1, i64::MAX)];

        // Act
        let actual_sorted_points = sort_points_by_argument_sort(&points, origin);

        // Assert
        assert_eq!(actual_sorted_points, expected_sorted_points);
    }
}
