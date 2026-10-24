fn binary_search_left<T: Ord>(vec: &Vec<T>, target: T) -> usize {
    let mut right = vec.len();
    let mut left = 0;

    while left < right {
        let mid = left + (right - left) / 2;

        if vec[mid] < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left.checked_sub(1).unwrap_or(usize::MAX)
}

fn binary_search_right<T: Ord>(vec: &Vec<T>, target: T) -> usize {
    let mut right = vec.len();
    let mut left = 0;

    while left < right {
        let mid = left + (right - left) / 2;

        if vec[mid] < target {
            left = mid + 1;
        } else {
            right = mid;
        }
    }

    left
}
#[cfg(test)]
mod tests {
    use super::{binary_search_left, binary_search_right};

    #[test]
    fn empty_vector_returns_current_sentinel_values() {
        // Arrange
        let values: Vec<i32> = vec![];
        let target = 3;
        let expected_values = (usize::MAX, 0);

        // Act
        let actual_values = (
            binary_search_left(&values, target),
            binary_search_right(&values, target),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn duplicate_values_return_last_less_and_first_ge_positions() {
        // Arrange
        let values = vec![1, 2, 2, 2, 3];
        let target = 2;
        let expected_values = (0, 1);

        // Act
        let actual_values = (
            binary_search_left(&values, target),
            binary_search_right(&values, target),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn duplicate_values_return_neighbor_positions_for_present_target() {
        // Arrange
        let values = vec![1, 2, 2, 2, 3];
        let target = 3;
        let expected_values = (3, 4);

        // Act
        let actual_values = (
            binary_search_left(&values, target),
            binary_search_right(&values, target),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn missing_target_between_duplicate_values_returns_neighbor_positions() {
        // Arrange
        let values = vec![1, 2, 2, 4];
        let target = 3;
        let expected_values = (2, 3);

        // Act
        let actual_values = (
            binary_search_left(&values, target),
            binary_search_right(&values, target),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn singleton_values_cover_before_equal_and_after_targets() {
        // Arrange
        let values = vec![2];
        let targets = [1, 2, 3];
        let expected_values = [(usize::MAX, 0), (usize::MAX, 0), (0, 1)];

        // Act
        let actual_values = targets.map(|target| {
            (
                binary_search_left(&values, target),
                binary_search_right(&values, target),
            )
        });

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
