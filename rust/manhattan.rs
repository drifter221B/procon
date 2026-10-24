/// 各点 (x, y) を 45 度回転して `x - y`, `x + y` を返す。
pub fn manhattan_transform(xs: &[i64], ys: &[i64]) -> Option<(Vec<i64>, Vec<i64>)> {

    if xs.len() != ys.len() {
        return None;
    }

    let mut dx = Vec::with_capacity(xs.len());
    let mut dy = Vec::with_capacity(xs.len());

    for (x, y) in xs.iter().zip(ys.iter()) {
        dx.push(*x - *y);
        dy.push(*x + *y);
    }

    Some((dx, dy))
}

#[cfg(test)]
mod tests {
    use std::panic::{catch_unwind, AssertUnwindSafe};


    use super::manhattan_transform;

    #[test]
    fn returns_none_when_lengths_differ() {
        // Arrange
        let xs = [1, 2];
        let ys = [1];
        let expected_value = None;

        // Act
        let actual_value = manhattan_transform(&xs, &ys);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn manhattan_transform_empty_points_returns_empty_coordinates() {
        // Arrange
        let xs: [i64; 0] = [];
        let ys: [i64; 0] = [];
        let expected_value = Some((vec![], vec![]));

        // Act
        let actual_value = manhattan_transform(&xs, &ys);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn manhattan_transform_single_point_returns_rotated_coordinates() {
        // Arrange
        let xs = [3];
        let ys = [1];
        let expected_value = Some((vec![2], vec![4]));

        // Act
        let actual_value = manhattan_transform(&xs, &ys);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn manhattan_transform_multiple_points_preserves_transformed_order() {
        // Arrange
        let xs = vec![1, 2, -1];
        let ys = vec![0, 2, 3];
        let expected_value = Some((vec![1, 0, -4], vec![1, 4, 2]));

        // Act
        let actual_value = manhattan_transform(&xs, &ys);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn duplicate_points_and_origin_keep_order() {
        // Arrange
        let xs = vec![0, 0, 2, 2];
        let ys = vec![0, 0, -1, -1];
        let expected_value = Some((vec![0, 0, 3, 3], vec![0, 0, 1, 1]));

        // Act
        let actual_value = manhattan_transform(&xs, &ys);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn overflow_near_i64_boundaries_panics_in_debug_builds() {
        // Arrange
        let xs = vec![i64::MAX];
        let ys = vec![-1];

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| manhattan_transform(&xs, &ys)));

        // Assert
        assert!(actual_result.is_err());
    }
}
