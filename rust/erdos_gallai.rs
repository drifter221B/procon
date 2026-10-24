pub mod erdos_gallai {
    pub fn is_graphic_sequence<T>(in_degrees: &[T]) -> bool
    where
        T: Copy + TryInto<usize>,
    {
        let n = in_degrees.len();
        let mut degrees = convert_degrees(in_degrees);

        assert!(n == degrees.len());

        if degrees.iter().copied().map(|d| d & 1).sum::<usize>() % 2 != 0 {
            return false;
        }

        degrees.sort_unstable_by(|a, b| b.cmp(a));

        let pre_sum = std::iter::once(0)
            .chain(degrees.iter().copied().scan(0, |acc, x| {
                *acc += x;

                Some(*acc)
            }))
            .collect::<Vec<usize>>();

        for k in 1..=n {
            let left = pre_sum[k];

            let index = degrees.partition_point(|&x| k < x);
            let right =
                k * (k - 1) + pre_sum[n] - pre_sum[index.max(k)] + index.saturating_sub(k) * k;

            if right < left {
                return false;
            }
        }

        true
    }

    fn convert_degrees<T>(in_degrees: &[T]) -> Vec<usize>
    where
        T: Copy + TryInto<usize>,
    {
        let n = in_degrees.len();
        let mut degrees = Vec::with_capacity(n);

        for &degree in in_degrees {
            let degree = degree
                .try_into()
                .ok()
                .filter(|degree| *degree < n)
                .expect("A degree should be non negative and degree <= n - 1");
            degrees.push(degree);
        }

        degrees
    }
}

#[cfg(test)]
mod tests {
    use super::erdos_gallai::is_graphic_sequence;

    #[test]
    fn is_graphic_sequence_returns_true_for_empty_set() {
        // Arrange
        let in_degrees: Vec<i64> = vec![];
        let expected = true;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_returns_true_for_single_zero_degree() {
        // Arrange
        let in_degrees = vec![0i64];
        let expected = true;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_returns_true_for_cycle_degree_sequence() {
        // Arrange
        let in_degrees = vec![2usize, 2, 2];
        let expected = true;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_returns_false_when_degree_sum_is_odd() {
        // Arrange
        let in_degrees = vec![1usize, 1, 1];
        let expected = false;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_returns_false_when_erdos_gallai_inequality_fails() {
        // Arrange
        let in_degrees = vec![3usize, 3, 1, 1];
        let expected = false;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_sorts_unsorted_input_before_judging() {
        // Arrange
        let in_degrees = vec![1usize, 2, 1, 2];
        let expected = true;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_accepts_complete_graph_degree_boundary() {
        // Arrange
        let in_degrees = vec![3usize, 3, 3, 3];
        let expected = true;

        // Act
        let actual = is_graphic_sequence(&in_degrees);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_graphic_sequence_panics_when_degree_is_at_least_vertex_count() {
        // Arrange
        let in_degrees = vec![2usize, 0];
        let expected_message = "A degree should be non negative and degree <= n - 1";

        // Act
        let actual_result = std::panic::catch_unwind(|| is_graphic_sequence(&in_degrees));
        let actual_panic = actual_result.expect_err("invalid degree must panic");
        let actual_message = actual_panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| actual_panic.downcast_ref::<&str>().copied());

        // Assert
        assert_eq!(actual_message, Some(expected_message));
    }

    #[test]
    fn is_graphic_sequence_panics_when_degree_is_negative() {
        // Arrange
        let in_degrees = vec![1i64, -1];
        let expected_message = "A degree should be non negative and degree <= n - 1";

        // Act
        let actual_result = std::panic::catch_unwind(|| is_graphic_sequence(&in_degrees));
        let actual_panic = actual_result.expect_err("invalid degree must panic");
        let actual_message = actual_panic
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| actual_panic.downcast_ref::<&str>().copied());

        // Assert
        assert_eq!(actual_message, Some(expected_message));
    }
}
