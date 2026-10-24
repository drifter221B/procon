pub fn at_most_k<T, F>(k: usize, nums: &[T], f: F) -> usize
where
    F: Fn(&T) -> usize,
{
    let mut right = 0;
    let mut cur = 0usize;
    let mut result = 0;

    for left in 0..nums.len() {
        while right < nums.len() {
            let right_weight = f(&nums[right]);
            let Some(next_cur) = cur.checked_add(right_weight) else {
                break;
            };
            if next_cur > k {
                break;
            }

            cur = next_cur;
            right += 1;
        }

        result += right - left;

        if right == left {
            right += 1;
        } else {
            cur -= f(&nums[left]);
        }
    }

    result
}

#[cfg(test)]
mod tests {
    use super::at_most_k;

    fn count_subarrays_at_most_k_bruteforce<T, F>(k: usize, nums: &[T], f: F) -> usize
    where
        F: Fn(&T) -> usize,
    {
        let mut valid_subarray_count = 0;
        for left_index in 0..nums.len() {
            let mut current_weight_sum = 0usize;
            for right_index in left_index..nums.len() {
                let Some(next_weight_sum) = current_weight_sum.checked_add(f(&nums[right_index]))
                else {
                    break;
                };
                if next_weight_sum > k {
                    break;
                }

                current_weight_sum = next_weight_sum;
                valid_subarray_count += 1;
            }
        }
        valid_subarray_count
    }

    fn calculate_total_subarray_count(array_length: usize) -> usize {
        array_length * (array_length + 1) / 2
    }

    #[test]
    fn returns_zero_for_empty_slice() {
        // Arrange
        let nums: Vec<i32> = vec![];
        let k = 1usize;
        let expected_subarray_count = 0usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |v| (*v as usize) & 1);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn at_most_k_counts_single_odd_weight_at_zero_and_one_boundaries() {
        // Arrange
        let nums = vec![3];
        let lower_k = 0usize;
        let upper_k = 1usize;
        let expected_subarray_count_for_lower_k = 0usize;
        let expected_subarray_count_for_upper_k = 1usize;

        // Act
        let actual_subarray_count_for_lower_k = at_most_k(lower_k, &nums, |v| (*v as usize) & 1);
        let actual_subarray_count_for_upper_k = at_most_k(upper_k, &nums, |v| (*v as usize) & 1);

        // Assert
        assert_eq!(
            actual_subarray_count_for_lower_k,
            expected_subarray_count_for_lower_k
        );
        assert_eq!(
            actual_subarray_count_for_upper_k,
            expected_subarray_count_for_upper_k
        );
    }

    #[test]
    fn counts_subarrays_with_sum_boundaries() {
        // Arrange
        let nums = vec![1, 1, 1];
        let k = 2usize;
        let expected_subarray_count = 5usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |v| *v as usize);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn at_most_k_with_odd_weight_function_counts_eight_subarrays() {
        // Arrange
        let nums = vec![1, 2, 3, 4];
        let k = 1usize;
        let expected_subarray_count = 8usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |v| (*v as usize) & 1);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_subarrays_for_k_minus_one_k_and_k_plus_one_on_same_input() {
        // Arrange
        let nums = vec![1usize, 2, 1];
        let base_k = 3usize;
        let lower_k = base_k - 1;
        let upper_k = base_k + 1;
        let expected_subarray_count_for_lower_k = 3usize;
        let expected_subarray_count_for_base_k = 5usize;
        let expected_subarray_count_for_upper_k = 6usize;

        // Act
        let actual_subarray_count_for_lower_k = at_most_k(lower_k, &nums, |value| *value);
        let actual_subarray_count_for_base_k = at_most_k(base_k, &nums, |value| *value);
        let actual_subarray_count_for_upper_k = at_most_k(upper_k, &nums, |value| *value);

        // Assert
        assert_eq!(
            actual_subarray_count_for_lower_k,
            expected_subarray_count_for_lower_k
        );
        assert_eq!(
            actual_subarray_count_for_base_k,
            expected_subarray_count_for_base_k
        );
        assert_eq!(
            actual_subarray_count_for_upper_k,
            expected_subarray_count_for_upper_k
        );
    }

    #[test]
    fn returns_zero_when_k_is_zero_and_all_weights_are_positive() {
        // Arrange
        let nums = vec![1usize, 2, 3];
        let k = 0usize;
        let expected_subarray_count = 0usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_all_subarrays_when_k_is_zero_and_all_weights_are_zero() {
        // Arrange
        let nums = vec![0usize, 0, 0, 0];
        let k = 0usize;
        let nums_length = nums.len();
        let expected_subarray_count = calculate_total_subarray_count(nums_length);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn returns_zero_when_every_weight_is_greater_than_k() {
        // Arrange
        let nums = vec![5usize, 6, 7];
        let k = 4usize;
        let expected_subarray_count = 0usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_all_subarrays_when_k_is_equal_to_total_weight() {
        // Arrange
        let nums = vec![2usize, 1, 3];
        let k = 6usize;
        let nums_length = nums.len();
        let expected_subarray_count = calculate_total_subarray_count(nums_length);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_subarrays_of_zero_weight_runs_when_weights_are_mixed() {
        // Arrange
        let nums = vec![0usize, 0, 1, 0];
        let k = 0usize;
        let expected_subarray_count = 4usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn continues_counting_after_first_element_exceeds_k() {
        // Arrange
        let nums = vec![5usize, 1, 1];
        let k = 2usize;
        let expected_subarray_count = 3usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn handles_last_element_exceeding_k_without_losing_previous_counts() {
        // Arrange
        let nums = vec![1usize, 1, 5];
        let k = 2usize;
        let expected_subarray_count = 3usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_length_two_case_with_mixed_valid_and_invalid_ranges() {
        // Arrange
        let nums = vec![1usize, 2];
        let k = 1usize;
        let expected_subarray_count = 1usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn matches_bruteforce_result_for_length_three_input() {
        // Arrange
        let nums = vec![1usize, 0, 2];
        let k = 2usize;
        let expected_subarray_count =
            count_subarrays_at_most_k_bruteforce(k, &nums, |value| *value);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn matches_bruteforce_result_for_length_four_input() {
        // Arrange
        let nums = vec![2usize, 0, 1, 2];
        let k = 3usize;
        let expected_subarray_count =
            count_subarrays_at_most_k_bruteforce(k, &nums, |value| *value);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn matches_bruteforce_result_for_length_five_input() {
        // Arrange
        let nums = vec![1usize, 2, 1, 0, 1];
        let k = 3usize;
        let expected_subarray_count =
            count_subarrays_at_most_k_bruteforce(k, &nums, |value| *value);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_all_subarrays_when_k_is_greater_than_total_weight() {
        // Arrange
        let nums = vec![2usize, 1, 3];
        let k = 7usize;
        let nums_length = nums.len();
        let expected_subarray_count = calculate_total_subarray_count(nums_length);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn matches_bruteforce_when_weights_greater_than_k_are_mixed() {
        // Arrange
        let nums = vec![5usize, 1, 6, 1, 7];
        let k = 2usize;
        let expected_subarray_count =
            count_subarrays_at_most_k_bruteforce(k, &nums, |value| *value);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn matches_bruteforce_when_zero_and_positive_weights_alternate() {
        // Arrange
        let nums = vec![0usize, 1, 0, 1, 0];
        let k = 1usize;
        let expected_subarray_count =
            count_subarrays_at_most_k_bruteforce(k, &nums, |value| *value);

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn at_most_k_with_struct_weight_field_counts_four_subarrays() {
        // Arrange
        struct WeightedValue {
            weight: usize,
        }

        let nums = vec![
            WeightedValue { weight: 1 },
            WeightedValue { weight: 1 },
            WeightedValue { weight: 2 },
        ];
        let k = 2usize;
        let expected_subarray_count = 4usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| value.weight);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn counts_all_subarrays_when_cumulative_weight_equals_usize_max() {
        // Arrange
        let nums = vec![usize::MAX - 1, 1];
        let k = usize::MAX;
        let expected_subarray_count = 3usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn excludes_subarray_when_cumulative_weight_would_overflow_usize() {
        // Arrange
        let nums = vec![usize::MAX, 1];
        let k = usize::MAX;
        let expected_subarray_count = 2usize;

        // Act
        let actual_subarray_count = at_most_k(k, &nums, |value| *value);

        // Assert
        assert_eq!(actual_subarray_count, expected_subarray_count);
    }

    #[test]
    fn matches_overflow_safe_bruteforce_for_all_small_weight_arrays() {
        // Arrange
        let max_weight = 3usize;
        let max_array_length = 6usize;
        let weight_variant_count = max_weight + 1;
        let mut test_cases = Vec::new();

        for array_length in 0..=max_array_length {
            let array_variant_count = weight_variant_count.pow(array_length as u32);

            for encoded_array in 0..array_variant_count {
                let mut remaining_code = encoded_array;
                let mut nums = Vec::with_capacity(array_length);

                for _ in 0..array_length {
                    nums.push(remaining_code % weight_variant_count);
                    remaining_code /= weight_variant_count;
                }

                let maximum_k = nums.iter().sum::<usize>() + 1;
                for k in 0..=maximum_k {
                    let expected_subarray_count =
                        count_subarrays_at_most_k_bruteforce(k, &nums, |value| *value);
                    test_cases.push((nums.clone(), k, expected_subarray_count));
                }
            }
        }

        // Act
        let actual_subarray_counts = test_cases
            .iter()
            .map(|(nums, k, _)| at_most_k(*k, nums, |value| *value))
            .collect::<Vec<_>>();

        // Assert
        for ((nums, k, expected_subarray_count), actual_subarray_count) in
            test_cases.iter().zip(actual_subarray_counts)
        {
            assert_eq!(
                actual_subarray_count, *expected_subarray_count,
                "nums = {nums:?}, k = {k}"
            );
        }
    }
}
