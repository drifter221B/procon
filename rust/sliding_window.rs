use std::collections::VecDeque;

/// 単調キューで長さ `window` の区間ごとの最小値/最大値を返す。
pub fn sliding_window(values: &[i64], window: usize, is_max: bool) -> Option<Vec<i64>> {

    if window == 0 || values.len() < window {
        return None;
    }

    let mut deque: VecDeque<usize> = VecDeque::new();
    let mut results = Vec::with_capacity(values.len() - window + 1);

    for i in 0..values.len() {

        while let Some(&back) = deque.back() {
            let cond = if is_max {
                values[i] >= values[back]
            } else {
                values[i] <= values[back]
            };

            if cond {
                deque.pop_back();
            } else {
                break;
            }
        }

        deque.push_back(i);

        if i + 1 >= window {
            let left_bound = i + 1 - window;
            if let Some(&front) = deque.front() {
                if front < left_bound {
                    deque.pop_front();
                }
            }

            let index = deque.front().copied().unwrap();
            results.push(values[index]);
        }
    }

    Some(results)
}

#[cfg(test)]
mod tests {
    use super::sliding_window;

    #[test]
    fn zero_and_larger_than_length_windows_return_none() {
        // Arrange
        let inputs = [(&[1, 2][..], 0), (&[1][..], 2)];
        let is_max = true;
        let expected_values = [None, None];

        // Act
        let actual_values = inputs.map(|(values, window)| sliding_window(values, window, is_max));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn computes_maximums_and_minimums() {
        // Arrange
        let values = vec![1, 3, -1, -3, 5, 3, 6, 7];
        let window = 3;
        let expected_values = (
            Some(vec![3, 3, 5, 5, 6, 7]),
            Some(vec![-1, -3, -3, -3, 3, 3]),
        );

        // Act
        let actual_values = (
            sliding_window(&values, window, true),
            sliding_window(&values, window, false),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn singleton_and_full_length_windows_return_expected_values() {
        // Arrange
        let inputs = [(&[5][..], 1, true), (&[2, 1][..], 2, false)];
        let expected_values = [Some(vec![5]), Some(vec![1])];

        // Act
        let actual_values =
            inputs.map(|(values, window, is_max)| sliding_window(values, window, is_max));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn window_equal_to_length_returns_single_value() {
        // Arrange
        let values = vec![3, 1, 2];
        let expected_values = (Some(vec![3]), Some(vec![1]));

        // Act
        let actual_values = (
            sliding_window(&values, values.len(), true),
            sliding_window(&values, values.len(), false),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn duplicate_values_are_handled_for_max_and_min() {
        // Arrange
        let values = vec![2, 2, 2, 2];
        let expected_values = (Some(vec![2, 2, 2]), Some(vec![2, 2, 2]));

        // Act
        let actual_values = (
            sliding_window(&values, 2, true),
            sliding_window(&values, 2, false),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn all_negative_values_are_handled_correctly() {
        // Arrange
        let values = vec![-5, -1, -3];
        let expected_values = (Some(vec![-1, -1]), Some(vec![-5, -3]));

        // Act
        let actual_values = (
            sliding_window(&values, 2, true),
            sliding_window(&values, 2, false),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn empty_input_with_zero_or_one_window_returns_none() {
        // Arrange
        let values: [i64; 0] = [];
        let windows = [0, 1];
        let expected_values = [None, None];

        // Act
        let actual_values = windows.map(|window| sliding_window(&values, window, true));

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
