const EPSILON: f64 = 1e-9;
const DEGREE_THRESHOLD: f64 = 180.0 + EPSILON;

/// (x2, y2) -> (x, y) -> (x1, y1) が時計回りに並んでいるときに、内角が 180 度未満かを判定する。
pub fn is_convex(x: f64, y: f64, x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    let vec1_x = x1 - x;
    let vec1_y = y1 - y;
    let vec2_x = x2 - x;
    let vec2_y = y2 - y;

    let rad1 = vec1_y.atan2(vec1_x);
    let rad2 = vec2_y.atan2(vec2_x);

    let mut degree = rad1.to_degrees() - rad2.to_degrees();
    let full_turn = 360.0_f64;
    if degree < 0.0 {
        degree += full_turn;
    }

    degree < DEGREE_THRESHOLD
}

#[cfg(test)]
mod tests {
    use super::is_convex;

    fn point_on_unit_circle_from_degree(degree_value: f64) -> (f64, f64) {
        let radian_value = degree_value.to_radians();
        (radian_value.cos(), radian_value.sin())
    }

    #[test]
    fn is_convex_with_clockwise_right_angle_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 1.0;
        let y1 = 0.0;
        let x2 = 0.0;
        let y2 = -1.0;
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_clockwise_acute_angle_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 1.0;
        let y1 = 0.0;
        let (x2, y2) = point_on_unit_circle_from_degree(-45.0);
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_clockwise_obtuse_angle_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 1.0;
        let y1 = 0.0;
        let (x2, y2) = point_on_unit_circle_from_degree(-135.0);
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_clockwise_reflex_angle_returns_false() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 1.0;
        let y1 = 0.0;
        let x2 = 0.0;
        let y2 = 1.0;
        let expected_is_convex = false;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_boundary_degrees_around_180_returns_expected_values() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let base_degree = -180.0;
        let boundary_degree = 180.0;
        let angle_x_minus_1 = boundary_degree - 0.000000001;
        let angle_x = boundary_degree;
        let angle_x_plus_1 = boundary_degree + 0.000000001;
        let angle_after_threshold = boundary_degree + 0.000000002;
        let (x1_x_minus_1, y1_x_minus_1) =
            point_on_unit_circle_from_degree(base_degree + angle_x_minus_1);
        let (x1_x, y1_x) = point_on_unit_circle_from_degree(base_degree + angle_x);
        let (x1_x_plus_1, y1_x_plus_1) =
            point_on_unit_circle_from_degree(base_degree + angle_x_plus_1);
        let (x1_after_threshold, y1_after_threshold) =
            point_on_unit_circle_from_degree(base_degree + angle_after_threshold);
        let (x2, y2) = point_on_unit_circle_from_degree(base_degree);
        let expected_x_minus_1 = true;
        let expected_x = true;
        let expected_x_plus_1 = false;
        let expected_after_threshold = false;

        // Act
        let actual_x_minus_1 = is_convex(x, y, x1_x_minus_1, y1_x_minus_1, x2, y2);
        let actual_x = is_convex(x, y, x1_x, y1_x, x2, y2);
        let actual_x_plus_1 = is_convex(x, y, x1_x_plus_1, y1_x_plus_1, x2, y2);
        let actual_after_threshold = is_convex(x, y, x1_after_threshold, y1_after_threshold, x2, y2);

        // Assert
        assert_eq!(actual_x_minus_1, expected_x_minus_1);
        assert_eq!(actual_x, expected_x);
        assert_eq!(actual_x_plus_1, expected_x_plus_1);
        assert_eq!(actual_after_threshold, expected_after_threshold);
    }

    #[test]
    fn is_convex_with_negative_degree_difference_applies_full_turn_adjustment() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let (x1, y1) = point_on_unit_circle_from_degree(-170.0);
        let (x2, y2) = point_on_unit_circle_from_degree(170.0);
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_non_negative_degree_difference_skips_full_turn_adjustment() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let (x1, y1) = point_on_unit_circle_from_degree(170.0);
        let (x2, y2) = point_on_unit_circle_from_degree(-170.0);
        let expected_is_convex = false;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_counterclockwise_point_order_returns_false() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 0.0;
        let y1 = -1.0;
        let x2 = 1.0;
        let y2 = 0.0;
        let expected_is_convex = false;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_zero_length_first_vector_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = x;
        let y1 = y;
        let x2 = 0.0;
        let y2 = -1.0;
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_zero_length_second_vector_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 1.0;
        let y1 = 0.0;
        let x2 = x;
        let y2 = y;
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_both_zero_length_vectors_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = x;
        let y1 = y;
        let x2 = x;
        let y2 = y;
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_collinear_same_direction_vectors_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = 1.0;
        let y1 = 0.0;
        let x2 = 2.0;
        let y2 = 0.0;
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_is_invariant_under_translation() {
        // Arrange
        let base_x = 0.0;
        let base_y = 0.0;
        let base_x1 = 1.0;
        let base_y1 = 0.0;
        let base_x2 = 0.0;
        let base_y2 = -1.0;
        let translate_x = 10.0;
        let translate_y = -3.0;

        // Act
        let actual_base_result = is_convex(base_x, base_y, base_x1, base_y1, base_x2, base_y2);
        let actual_translated_result = is_convex(
            base_x + translate_x,
            base_y + translate_y,
            base_x1 + translate_x,
            base_y1 + translate_y,
            base_x2 + translate_x,
            base_y2 + translate_y,
        );

        // Assert
        assert_eq!(actual_base_result, actual_translated_result);
    }

    #[test]
    fn is_convex_is_invariant_under_positive_scaling() {
        // Arrange
        let base_x = 0.0;
        let base_y = 0.0;
        let base_x1 = 1.0;
        let base_y1 = 0.0;
        let base_x2 = 0.0;
        let base_y2 = -1.0;
        let scale_factor = 3.5;

        // Act
        let actual_base_result = is_convex(base_x, base_y, base_x1, base_y1, base_x2, base_y2);
        let actual_scaled_result = is_convex(
            base_x * scale_factor,
            base_y * scale_factor,
            base_x1 * scale_factor,
            base_y1 * scale_factor,
            base_x2 * scale_factor,
            base_y2 * scale_factor,
        );

        // Assert
        assert_eq!(actual_base_result, actual_scaled_result);
    }

    #[test]
    fn is_convex_with_f64_max_boundaries_returns_false() {
        // Arrange
        let x = f64::MAX;
        let y = f64::MAX;
        let x1 = -f64::MAX;
        let y1 = f64::MAX;
        let x2 = f64::MAX;
        let y2 = -f64::MAX;
        let expected_is_convex = false;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);
        let actual_first_vector_x_is_infinite = (x1 - x).is_infinite();
        let actual_second_vector_y_is_infinite = (y2 - y).is_infinite();

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
        assert!(actual_first_vector_x_is_infinite);
        assert!(actual_second_vector_y_is_infinite);
    }

    #[test]
    fn is_convex_with_nan_argument_returns_false() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = f64::NAN;
        let y1 = 0.0;
        let x2 = 1.0;
        let y2 = 0.0;
        let expected_is_convex = false;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }

    #[test]
    fn is_convex_with_infinite_arguments_returns_true() {
        // Arrange
        let x = 0.0;
        let y = 0.0;
        let x1 = f64::INFINITY;
        let y1 = 0.0;
        let x2 = 0.0;
        let y2 = f64::NEG_INFINITY;
        let expected_is_convex = true;

        // Act
        let actual_is_convex = is_convex(x, y, x1, y1, x2, y2);

        // Assert
        assert_eq!(actual_is_convex, expected_is_convex);
    }
}
