#[derive(Debug, Clone, Copy)]
pub struct Circle {
    pub x: i128,
    pub y: i128,
    pub r: i128,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CircleRelation {
    Coincident,
    Contained,
    InternalTangent,
    Intersecting,
    ExternalTangent,
    Separate,
}

impl Circle {
    pub fn new(x: i128, y: i128, r: i128) -> Self {
        Self { x, y, r }
    }

    pub fn relation_to(&self, other: &Circle) -> CircleRelation {
        let distance_squared = self.distance_squared_to(other);
        let radius_sum_squared = self.radius_sum_squared(other);
        let radius_diff_squared = self.radius_diff_squared(other);

        if distance_squared == 0 && self.r == other.r {
            CircleRelation::Coincident
        } else if radius_sum_squared < distance_squared {
            CircleRelation::Separate
        } else if distance_squared == radius_sum_squared {
            CircleRelation::ExternalTangent
        } else if radius_diff_squared < distance_squared && distance_squared < radius_sum_squared {
            CircleRelation::Intersecting
        } else if distance_squared == radius_diff_squared {
            CircleRelation::InternalTangent
        } else {
            CircleRelation::Contained
        }
    }

    pub fn has_common_point_with(&self, other: &Circle) -> bool {
        matches!(
            self.relation_to(other),
            CircleRelation::Coincident
                | CircleRelation::InternalTangent
                | CircleRelation::Intersecting
                | CircleRelation::ExternalTangent
        )
    }

    pub fn is_centre_included(&self, other: &Circle) -> bool {
        self.distance_squared_to(other) <= self.r * self.r
    }

    pub fn is_cross(&self, other: &Circle) -> bool {
        self.relation_to(other) == CircleRelation::Intersecting
    }

    pub fn is_contact(&self, other: &Circle) -> bool {
        matches!(
            self.relation_to(other),
            CircleRelation::InternalTangent | CircleRelation::ExternalTangent
        )
    }

    pub fn distance_squared_to(&self, other: &Circle) -> i128 {
        let dx = self.x - other.x;
        let dy = self.y - other.y;
        dx * dx + dy * dy
    }

    fn radius_sum_squared(&self, other: &Circle) -> i128 {
        let radius_sum = self.r + other.r;
        radius_sum * radius_sum
    }

    fn radius_diff_squared(&self, other: &Circle) -> i128 {
        let radius_diff = self.r - other.r;
        radius_diff * radius_diff
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn relation_to_same_circle_is_coincident() {
        // Arrange
        let circle = Circle::new(0, 0, 2);
        let other = Circle::new(0, 0, 2);
        let expected = CircleRelation::Coincident;

        // Act
        let actual = circle.relation_to(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn relation_to_separated_circles_is_separate() {
        // Arrange
        let circle = Circle::new(0, 0, 2);
        let other = Circle::new(5, 0, 2);
        let expected = CircleRelation::Separate;

        // Act
        let actual = circle.relation_to(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn relation_to_externally_tangent_circles_is_external_tangent() {
        // Arrange
        let circle = Circle::new(0, 0, 2);
        let other = Circle::new(5, 0, 3);
        let expected = CircleRelation::ExternalTangent;

        // Act
        let actual = circle.relation_to(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn relation_to_intersecting_circles_is_intersecting() {
        // Arrange
        let circle = Circle::new(0, 0, 3);
        let other = Circle::new(4, 0, 3);
        let expected = CircleRelation::Intersecting;

        // Act
        let actual = circle.relation_to(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn relation_to_internally_tangent_circles_is_internal_tangent() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let other = Circle::new(3, 0, 2);
        let expected = CircleRelation::InternalTangent;

        // Act
        let actual = circle.relation_to(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn relation_to_contained_circles_is_contained() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let other = Circle::new(2, 0, 2);
        let expected = CircleRelation::Contained;

        // Act
        let actual = circle.relation_to(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn has_common_point_with_sample_cases_returns_expected_results() {
        // Arrange
        let cases = vec![
            (Circle::new(0, 0, 2), Circle::new(2, 3, 2), true),
            (Circle::new(0, 0, 2), Circle::new(2, 3, 1), false),
            (Circle::new(1, 2, 5), Circle::new(3, 2, 1), false),
            (Circle::new(5, 4, 2), Circle::new(8, 8, 3), true),
            (Circle::new(2, 1, 5), Circle::new(5, 1, 2), true),
            (Circle::new(0, 0, 1), Circle::new(0, 0, 1), true),
            (
                Circle::new(0, 0, 500_000_000),
                Circle::new(1, 1_000_000_000, 500_000_000),
                false,
            ),
        ];

        let expected = cases
            .iter()
            .map(|(_, _, expected)| *expected)
            .collect::<Vec<_>>();

        // Act
        let actual = cases
            .iter()
            .map(|(circle, other, _)| circle.has_common_point_with(other))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_centre_included_when_other_center_is_on_circumference_returns_true() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let other = Circle::new(3, 4, 2);
        let expected = true;

        // Act
        let actual = circle.is_centre_included(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_centre_included_when_other_center_is_outside_returns_false() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let other = Circle::new(6, 0, 2);
        let expected = false;

        // Act
        let actual = circle.is_centre_included(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_cross_when_circles_intersect_at_two_points_returns_true() {
        // Arrange
        let circle = Circle::new(0, 0, 3);
        let other = Circle::new(4, 0, 3);
        let expected = true;

        // Act
        let actual = circle.is_cross(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_cross_when_circles_are_tangent_returns_false() {
        // Arrange
        let circle = Circle::new(0, 0, 2);
        let other = Circle::new(5, 0, 3);
        let expected = false;

        // Act
        let actual = circle.is_cross(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_contact_when_circles_are_externally_tangent_returns_true() {
        // Arrange
        let circle = Circle::new(0, 0, 2);
        let other = Circle::new(5, 0, 3);
        let expected = true;

        // Act
        let actual = circle.is_contact(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_contact_when_circles_intersect_at_two_points_returns_false() {
        // Arrange
        let circle = Circle::new(0, 0, 3);
        let other = Circle::new(4, 0, 3);
        let expected = false;

        // Act
        let actual = circle.is_contact(&other);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_centre_included_when_other_center_is_strictly_inside_returns_true() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let other = Circle::new(2, 3, 2);
        let expected = true;

        // Act
        let actual = circle.is_centre_included(&other);

        // Assert
        assert_eq!(actual, expected);
    }


    #[test]
    fn relation_to_near_internal_tangency_transitions_from_contained_to_intersecting() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let centers = [2, 3, 4];
        let other_circles = centers.map(|x| Circle::new(x, 0, 2));
        let expected_relations = [
            CircleRelation::Contained,
            CircleRelation::InternalTangent,
            CircleRelation::Intersecting,
        ];

        // Act
        let actual_relations = other_circles.map(|other| circle.relation_to(&other));

        // Assert
        assert_eq!(actual_relations, expected_relations);
    }


    #[test]
    fn relation_to_near_external_tangency_transitions_from_intersecting_to_separate() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let centers = [6, 7, 8];
        let other_circles = centers.map(|x| Circle::new(x, 0, 2));
        let expected_relations = [
            CircleRelation::Intersecting,
            CircleRelation::ExternalTangent,
            CircleRelation::Separate,
        ];

        // Act
        let actual_relations = other_circles.map(|other| circle.relation_to(&other));

        // Assert
        assert_eq!(actual_relations, expected_relations);
    }


    #[test]
    fn zero_radius_circle_on_circumference_is_tangent_with_common_point() {
        // Arrange
        let circle = Circle::new(0, 0, 5);
        let point_circle = Circle::new(5, 0, 0);
        let expected_relation = CircleRelation::ExternalTangent;
        let expected_contact = true;

        // Act
        let actual_relation = circle.relation_to(&point_circle);
        let actual_contact = circle.is_contact(&point_circle);

        // Assert
        assert_eq!(actual_relation, expected_relation);
        assert_eq!(actual_contact, expected_contact);
    }


    #[test]
    fn zero_radius_circles_at_same_center_are_coincident() {
        // Arrange
        let circle = Circle::new(1, 2, 0);
        let other = Circle::new(1, 2, 0);
        let expected_relation = CircleRelation::Coincident;

        // Act
        let actual_relation = circle.relation_to(&other);

        // Assert
        assert_eq!(actual_relation, expected_relation);
    }

}
