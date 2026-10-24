pub fn pre_sum<S, T>(nums: &[S]) -> Vec<T>
where
    S: Copy,
    T: std::ops::Add<Output = T> + From<S> + Copy + Default
{
    std::iter::once(T::default())
        .chain(nums.iter().scan(T::default(), |pre_sum, x| {
            *pre_sum = *pre_sum + T::from(*x);
            Some(*pre_sum)
        }))
        .collect::<Vec<T>>()
}

pub fn suf_sum<S, T>(nums: &[S]) -> Vec<T>
where
    S: Copy,
    T: std::ops::Add<Output = T> + From<S> + Copy + Default
{
    std::iter::once(T::default())
        .chain(nums.iter().rev().scan(T::default(), |suf_sum, x| {
            *suf_sum = *suf_sum + T::from(*x);
            Some(*suf_sum)
        }))
        .collect::<Vec<T>>()
}

#[cfg(test)]
mod tests {
    use super::{pre_sum, suf_sum};

    #[test]
    fn computes_prefix_and_suffix_sums() {
        // Arrange
        let nums = vec![1i64, 2, 3];
        let expected_prefix_sums = vec![0, 1, 3, 6];
        let expected_suffix_sums = vec![0, 3, 5, 6];

        // Act
        let actual_prefix_sums = pre_sum::<i64, i64>(&nums);
        let actual_suffix_sums = suf_sum::<i64, i64>(&nums);

        // Assert
        assert_eq!(actual_prefix_sums, expected_prefix_sums);
        assert_eq!(actual_suffix_sums, expected_suffix_sums);
    }

    #[test]
    fn empty_slice_returns_identity_only_prefix_and_suffix_sums() {
        // Arrange
        let nums: Vec<i32> = vec![];
        let expected_prefix_sums = vec![0];
        let expected_suffix_sums = vec![0];

        // Act
        let actual_prefix_sums = pre_sum::<i32, i32>(&nums);
        let actual_suffix_sums = suf_sum::<i32, i32>(&nums);

        // Assert
        assert_eq!(actual_prefix_sums, expected_prefix_sums);
        assert_eq!(actual_suffix_sums, expected_suffix_sums);
    }

    #[test]
    fn single_element_input_produces_length_two_results() {
        // Arrange
        let values = vec![5i32];
        let expected_values = (vec![0i32, 5], vec![0i32, 5]);

        // Act
        let actual_values = (pre_sum::<i32, i32>(&values), suf_sum::<i32, i32>(&values));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn negative_numbers_are_accumulated_correctly() {
        // Arrange
        let values = vec![-1i32, 2, -3];
        let expected_values = (vec![0i32, -1, 1, -2], vec![0i32, -3, -1, -2]);

        // Act
        let actual_values = (pre_sum::<i32, i32>(&values), suf_sum::<i32, i32>(&values));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pre_sum_converts_i32_input_to_i64_cumulative_sums() {
        // Arrange
        let values = vec![1i32, 2, 3];
        let expected_values = vec![0i64, 1, 3, 6];

        // Act
        let actual_values = pre_sum::<i32, i64>(&values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn suffix_sum_keeps_reverse_accumulation_order() {
        // Arrange
        let values = vec![1i32, 2, 3];
        let expected_values = vec![0i32, 3, 5, 6];

        // Act
        let actual_values = suf_sum::<i32, i32>(&values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
