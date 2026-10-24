use std::cmp::Ordering;

pub type Point = (i64, i64);

fn cross_product(origin: Point, point_a: Point, point_b: Point) -> i128 {
    let vector_a_x = point_a.0 - origin.0;
    let vector_a_y = point_a.1 - origin.1;
    let vector_b_x = point_b.0 - origin.0;
    let vector_b_y = point_b.1 - origin.1;
    let dx = i128::from(vector_a_x);
    let dy = i128::from(vector_a_y);
    let ex = i128::from(vector_b_x);
    let ey = i128::from(vector_b_y);
    dx * ey - dy * ex
}

fn sort_points(points: &[Point]) -> Vec<Point> {
    let mut sorted_points = points.to_vec();
    sorted_points.sort_by(|lhs, rhs| match lhs.0.cmp(&rhs.0) {
        Ordering::Equal => lhs.1.cmp(&rhs.1),
        other => other,
    });
    sorted_points.dedup();
    sorted_points
}

fn build_half_hull(sorted_points: &[Point]) -> Vec<Point> {
    let mut hull: Vec<Point> = Vec::with_capacity(sorted_points.len());

    for &point in sorted_points {
        while 2 <= hull.len() {
            let last = hull[hull.len() - 1];
            let second_last = hull[hull.len() - 2];
            if cross_product(second_last, last, point) <= 0 {
                hull.pop();
            } else {
                break;
            }
        }
        hull.push(point);
    }

    hull
}

pub fn convex_hull(points: &[Point]) -> Vec<Point> {
    let sorted_points = sort_points(points);
    if sorted_points.len() <= 1 {
        return sorted_points;
    }

    let mut lower_hull = build_half_hull(&sorted_points);
    let reversed_points: Vec<Point> = sorted_points.iter().rev().copied().collect();
    let mut upper_hull = build_half_hull(&reversed_points);

    if 1 < lower_hull.len() {
        lower_hull.pop();
    }

    if 1 < upper_hull.len() {
        upper_hull.pop();
    }

    lower_hull.extend(upper_hull);
    lower_hull
}

#[cfg(test)]
mod tests {
    use super::{build_half_hull, convex_hull, cross_product, sort_points, Point};

    fn create_points(points: &[(i64, i64)]) -> Vec<Point> {
        points.to_vec()
    }

    fn has_only_left_turns(points: &[Point]) -> bool {
        points
            .windows(3)
            .all(|window| cross_product(window[0], window[1], window[2]) > 0)
    }

    #[test]
    fn cross_product_returns_positive_for_counter_clockwise_points() {
        // Arrange
        let origin = (0, 0);
        let point_a = (1, 0);
        let point_b = (0, 1);
        let expected_value = 1;

        // Act
        let actual_value = cross_product(origin, point_a, point_b);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn cross_product_returns_negative_for_clockwise_points() {
        // Arrange
        let origin = (0, 0);
        let point_a = (0, 1);
        let point_b = (1, 0);
        let expected_value = -1;

        // Act
        let actual_value = cross_product(origin, point_a, point_b);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn cross_product_returns_zero_for_collinear_points() {
        // Arrange
        let origin = (0, 0);
        let point_a = (1, 1);
        let point_b = (2, 2);
        let expected_value = 0;

        // Act
        let actual_value = cross_product(origin, point_a, point_b);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn cross_product_preserves_current_behavior_for_subtraction_overflow_near_i64_limits() {
        // Arrange
        let origin = (i64::MIN, 0);
        let point_a = (i64::MAX, 0);
        let point_b = (0, i64::MAX);

        // Act
        let actual_result = std::panic::catch_unwind(|| cross_product(origin, point_a, point_b));

        // Assert
        #[cfg(debug_assertions)]
        assert!(actual_result.is_err());
        #[cfg(not(debug_assertions))]
        assert!(actual_result.is_ok());
    }

    #[test]
    fn sort_points_returns_empty_vector_for_empty_input() {
        // Arrange
        let points: Vec<Point> = vec![];
        let expected_points: Vec<Point> = vec![];

        // Act
        let actual_points = sort_points(&points);

        // Assert
        assert_eq!(actual_points, expected_points);
    }

    #[test]
    fn sort_points_returns_single_point_for_singleton_input() {
        // Arrange
        let points = create_points(&[(4, -3)]);
        let expected_points = create_points(&[(4, -3)]);

        // Act
        let actual_points = sort_points(&points);

        // Assert
        assert_eq!(actual_points, expected_points);
    }

    #[test]
    fn sort_points_removes_duplicate_points_from_multi_element_input() {
        // Arrange
        let points = create_points(&[(1, 1), (2, 0), (1, 1), (2, 0), (3, 3)]);
        let expected_points = create_points(&[(1, 1), (2, 0), (3, 3)]);

        // Act
        let actual_points = sort_points(&points);

        // Assert
        assert_eq!(actual_points, expected_points);
    }

    #[test]
    fn sort_points_orders_points_with_same_x_by_y_ascending() {
        // Arrange
        let points = create_points(&[(2, 3), (2, -1), (2, 0), (2, 2)]);
        let expected_points = create_points(&[(2, -1), (2, 0), (2, 2), (2, 3)]);

        // Act
        let actual_points = sort_points(&points);

        // Assert
        assert_eq!(actual_points, expected_points);
    }

    #[test]
    fn sort_points_orders_points_in_lexicographical_order() {
        // Arrange
        let points = create_points(&[(3, 0), (-1, 5), (3, -2), (0, 0), (-1, 4)]);
        let expected_points = create_points(&[(-1, 4), (-1, 5), (0, 0), (3, -2), (3, 0)]);

        // Act
        let actual_points = sort_points(&points);

        // Assert
        assert_eq!(actual_points, expected_points);
    }

    #[test]
    fn build_half_hull_returns_empty_vector_for_empty_input() {
        // Arrange
        let sorted_points: Vec<Point> = vec![];
        let expected_hull: Vec<Point> = vec![];

        // Act
        let actual_hull = build_half_hull(&sorted_points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn build_half_hull_returns_single_point_for_singleton_input() {
        // Arrange
        let sorted_points = create_points(&[(1, 2)]);
        let expected_hull = create_points(&[(1, 2)]);

        // Act
        let actual_hull = build_half_hull(&sorted_points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn build_half_hull_returns_two_points_for_two_point_input() {
        // Arrange
        let sorted_points = create_points(&[(0, 0), (2, 1)]);
        let expected_hull = create_points(&[(0, 0), (2, 1)]);

        // Act
        let actual_hull = build_half_hull(&sorted_points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn build_half_hull_removes_middle_point_for_clockwise_three_points() {
        // Arrange
        let sorted_points = create_points(&[(0, 0), (1, 1), (2, 0)]);
        let expected_hull = create_points(&[(0, 0), (2, 0)]);

        // Act
        let actual_hull = build_half_hull(&sorted_points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn build_half_hull_removes_middle_point_for_collinear_three_points() {
        // Arrange
        let sorted_points = create_points(&[(0, 0), (1, 1), (2, 2)]);
        let expected_hull = create_points(&[(0, 0), (2, 2)]);

        // Act
        let actual_hull = build_half_hull(&sorted_points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn build_half_hull_keeps_only_left_turn_chain_after_multiple_consecutive_pops() {
        // Arrange
        let sorted_points = create_points(&[(0, 0), (1, 4), (2, 1), (3, 3), (4, 2), (5, 5)]);
        let expected_hull = create_points(&[(0, 0), (4, 2), (5, 5)]);

        // Act
        let actual_hull = build_half_hull(&sorted_points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_empty_vector_for_empty_input() {
        // Arrange
        let points: Vec<Point> = vec![];
        let expected_hull: Vec<Point> = vec![];

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_single_point_for_multi_element_same_point_input() {
        // Arrange
        let points = create_points(&[(7, -2), (7, -2), (7, -2), (7, -2)]);
        let expected_hull = create_points(&[(7, -2)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_single_point_for_singleton_input() {
        // Arrange
        let points = create_points(&[(3, 8)]);
        let expected_hull = create_points(&[(3, 8)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_two_points_for_two_point_input() {
        // Arrange
        let points = create_points(&[(3, 1), (1, 2)]);
        let expected_hull = create_points(&[(1, 2), (3, 1)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_counter_clockwise_triangle_for_unsorted_three_points() {
        // Arrange
        let points = create_points(&[(1, 1), (0, 0), (2, 0)]);
        let expected_hull = create_points(&[(0, 0), (2, 0), (1, 1)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_excludes_concave_vertex_from_point_set() {
        // Arrange
        let points = create_points(&[(0, 0), (3, 0), (3, 3), (2, 1), (0, 3)]);
        let expected_hull = create_points(&[(0, 0), (3, 0), (3, 3), (0, 3)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_excludes_interior_points_from_square_vertices_with_inner_points() {
        // Arrange
        let points = create_points(&[(0, 0), (2, 0), (2, 2), (0, 2), (1, 1)]);
        let expected_hull = create_points(&[(0, 0), (2, 0), (2, 2), (0, 2)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_excludes_non_endpoint_points_on_edges() {
        // Arrange
        let points = create_points(&[(0, 0), (1, 0), (2, 0), (2, 2), (1, 2), (0, 2)]);
        let expected_hull = create_points(&[(0, 0), (2, 0), (2, 2), (0, 2)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_two_endpoints_for_horizontal_collinear_boundary_values() {
        // Arrange
        let points = create_points(&[(0, 0), (1, 0), (2, 0)]);
        let expected_hull = create_points(&[(0, 0), (2, 0)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_returns_two_endpoints_for_vertical_collinear_boundary_values() {
        // Arrange
        let points = create_points(&[(0, -1), (0, 0), (0, 1)]);
        let expected_hull = create_points(&[(0, -1), (0, 1)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_result_is_invariant_to_input_order() {
        // Arrange
        let points_in_first_order = create_points(&[(0, 0), (2, 0), (2, 2), (0, 2), (1, 1)]);
        let points_in_second_order = create_points(&[(1, 1), (2, 2), (0, 2), (2, 0), (0, 0)]);

        // Act
        let actual_hull_first_order = convex_hull(&points_in_first_order);
        let actual_hull_second_order = convex_hull(&points_in_second_order);

        // Assert
        assert_eq!(actual_hull_first_order, actual_hull_second_order);
    }

    #[test]
    fn convex_hull_builds_correct_hull_for_negative_coordinate_points() {
        // Arrange
        let points = create_points(&[(-2, 0), (0, -2), (2, 0), (0, 2), (0, 0), (-1, 0)]);
        let expected_hull = create_points(&[(-2, 0), (0, -2), (2, 0), (0, 2)]);

        // Act
        let actual_hull = convex_hull(&points);

        // Assert
        assert_eq!(actual_hull, expected_hull);
    }

    #[test]
    fn convex_hull_preserves_current_behavior_for_overflow_near_i64_limits() {
        // Arrange
        let points = create_points(&[(i64::MIN, 0), (0, 0), (i64::MAX, 0)]);

        // Act
        let actual_result = std::panic::catch_unwind(|| convex_hull(&points));

        // Assert
        #[cfg(debug_assertions)]
        assert!(actual_result.is_err());
        #[cfg(not(debug_assertions))]
        assert!(actual_result.is_ok());
    }
}
