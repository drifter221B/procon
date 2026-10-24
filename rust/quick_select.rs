use rand::Rng;

#[derive(Debug, Clone)]
struct S {
    v: (i32, i32),
}

impl PartialEq for S {
    fn eq(&self, other: &Self) -> bool {
        self.v.0 == other.v.0
    }
}

impl Eq for S {}

impl PartialOrd for S {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.v.0.cmp(&other.v.0))
    }
}

impl Ord for S {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.v.0.cmp(&other.v.0)
    }
}

fn quick_select(nums: &[S], k: usize, is_max: bool) -> S {
    assert!(1 <= k);
    assert!(k <= nums.len());

    let (k, is_max) = match is_max {
        true => (nums.len() - k + 1, false),
        false => (k, is_max),
    };

    let pivot_index = rand::thread_rng().gen_range(0..nums.len());
    let pivot = nums[pivot_index].clone();

    let mut left = vec![];
    let mut mid = vec![];
    let mut right = vec![];

    for num in nums.iter() {
        match pivot.cmp(num) {
            std::cmp::Ordering::Less => right.push(num.to_owned()),
            std::cmp::Ordering::Greater => left.push(num.to_owned()),
            std::cmp::Ordering::Equal => mid.push(num.to_owned()),
        }
    }

    if k <= left.len() {
        quick_select(&left, k, is_max)
    } else if k <= left.len() + mid.len() {
        pivot.to_owned()
    } else {
        quick_select(&right, k - (left.len() + mid.len()), is_max)
    }
}

#[cfg(test)]
mod tests {
    use super::{quick_select, S};

    fn create_elements(value_pairs: &[(i32, i32)]) -> Vec<S> {
        value_pairs
            .iter()
            .map(|value_pair| S { v: *value_pair })
            .collect()
    }

    fn sorted_first_values(nums: &[S]) -> Vec<i32> {
        let mut first_values: Vec<i32> = nums.iter().map(|num| num.v.0).collect();
        first_values.sort();
        first_values
    }

    fn expected_kth_min_first_value(nums: &[S], k: usize) -> i32 {
        let sorted_values = sorted_first_values(nums);
        sorted_values[k - 1]
    }

    fn expected_kth_max_first_value(nums: &[S], k: usize) -> i32 {
        let sorted_values = sorted_first_values(nums);
        sorted_values[sorted_values.len() - k]
    }

    #[test]
    fn quick_select_when_nums_is_empty_and_k_is_zero_then_panics() {
        // Arrange
        let nums: Vec<S> = vec![];
        let k = 0;
        let is_max = false;

        // Act
        let panic_result = std::panic::catch_unwind(|| quick_select(&nums, k, is_max));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn quick_select_when_nums_is_empty_and_k_is_one_then_panics() {
        // Arrange
        let nums: Vec<S> = vec![];
        let k = 1;
        let is_max = false;

        // Act
        let panic_result = std::panic::catch_unwind(|| quick_select(&nums, k, is_max));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn quick_select_when_k_exceeds_length_then_panics() {
        // Arrange
        let nums = create_elements(&[(4, 10), (2, 20), (7, 30)]);
        let k = nums.len() + 1;
        let is_max = false;

        // Act
        let panic_result = std::panic::catch_unwind(|| quick_select(&nums, k, is_max));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn quick_select_when_single_element_and_is_max_false_then_returns_same_element() {
        // Arrange
        let value_pair = (42, 7);
        let nums = create_elements(&[value_pair]);
        let k = 1;
        let is_max = false;
        let expected_first_value = value_pair.0;

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_single_element_and_is_max_true_then_returns_same_element() {
        // Arrange
        let value_pair = (42, 7);
        let nums = create_elements(&[value_pair]);
        let k = 1;
        let is_max = true;
        let expected_first_value = value_pair.0;

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_is_max_false_and_k_is_one_then_returns_minimum_first_value() {
        // Arrange
        let nums = create_elements(&[(8, 1), (2, 2), (10, 3), (4, 4), (6, 5)]);
        let k = 1;
        let is_max = false;
        let expected_first_value = expected_kth_min_first_value(&nums, k);

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_is_max_false_and_k_is_n_then_returns_maximum_first_value() {
        // Arrange
        let nums = create_elements(&[(8, 1), (2, 2), (10, 3), (4, 4), (6, 5)]);
        let k = nums.len();
        let is_max = false;
        let expected_first_value = expected_kth_min_first_value(&nums, k);

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_is_max_true_and_k_is_one_then_returns_maximum_first_value() {
        // Arrange
        let nums = create_elements(&[(8, 1), (2, 2), (10, 3), (4, 4), (6, 5)]);
        let k = 1;
        let is_max = true;
        let expected_first_value = expected_kth_max_first_value(&nums, k);

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_is_max_true_and_k_is_n_then_returns_minimum_first_value() {
        // Arrange
        let nums = create_elements(&[(8, 1), (2, 2), (10, 3), (4, 4), (6, 5)]);
        let k = nums.len();
        let is_max = true;
        let expected_first_value = expected_kth_max_first_value(&nums, k);

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_k_boundary_around_one_then_panics_only_for_zero() {
        // Arrange
        let nums = create_elements(&[(3, 1), (9, 2)]);
        let k_minus_one = 0;
        let k = 1;
        let k_plus_one = 2;
        let is_max = false;
        let expected_k_first_value = expected_kth_min_first_value(&nums, k);
        let expected_k_plus_one_first_value = expected_kth_min_first_value(&nums, k_plus_one);

        // Act
        let panic_result = std::panic::catch_unwind(|| quick_select(&nums, k_minus_one, is_max));
        let selected_at_k = quick_select(&nums, k, is_max);
        let selected_at_k_plus_one = quick_select(&nums, k_plus_one, is_max);

        // Assert
        assert!(panic_result.is_err());
        assert_eq!(selected_at_k.v.0, expected_k_first_value);
        assert_eq!(selected_at_k_plus_one.v.0, expected_k_plus_one_first_value);
    }

    #[test]
    fn quick_select_when_k_boundary_around_n_then_panics_only_for_n_plus_one() {
        // Arrange
        let nums = create_elements(&[(3, 1), (9, 2), (6, 3), (1, 4)]);
        let n = nums.len();
        let k_minus_one = n - 1;
        let k = n;
        let k_plus_one = n + 1;
        let is_max = false;
        let expected_k_minus_one_first_value = expected_kth_min_first_value(&nums, k_minus_one);
        let expected_k_first_value = expected_kth_min_first_value(&nums, k);

        // Act
        let selected_at_k_minus_one = quick_select(&nums, k_minus_one, is_max);
        let selected_at_k = quick_select(&nums, k, is_max);
        let panic_result = std::panic::catch_unwind(|| quick_select(&nums, k_plus_one, is_max));

        // Assert
        assert_eq!(
            selected_at_k_minus_one.v.0,
            expected_k_minus_one_first_value
        );
        assert_eq!(selected_at_k.v.0, expected_k_first_value);
        assert!(panic_result.is_err());
    }

    #[test]
    fn quick_select_when_first_value_has_duplicates_with_different_second_value_then_returns_expected_ranked_first_value(
    ) {
        // Arrange
        let nums = create_elements(&[(5, 10), (1, 20), (5, 30), (3, 40)]);
        let k = 3;
        let is_max = false;
        let expected_first_value = expected_kth_min_first_value(&nums, k);

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_all_first_values_are_identical_then_returns_same_first_value_for_any_k_and_mode(
    ) {
        // Arrange
        let nums = create_elements(&[(4, 1), (4, 2), (4, 3), (4, 4)]);
        let ranks: Vec<usize> = (1..=nums.len()).collect();
        let modes = [false, true];
        let expected_first_value = 4;

        // Act
        let mut selected_first_values: Vec<i32> = vec![];
        for k in ranks {
            for is_max in modes {
                let selected = quick_select(&nums, k, is_max);
                selected_first_values.push(selected.v.0);
            }
        }

        // Assert
        for selected_first_value in selected_first_values {
            assert_eq!(selected_first_value, expected_first_value);
        }
    }

    #[test]
    fn quick_select_when_values_include_negative_zero_and_positive_then_returns_expected_ranked_first_value(
    ) {
        // Arrange
        let nums = create_elements(&[(-10, 1), (0, 2), (5, 3), (-3, 4), (8, 5)]);
        let k = 3;
        let is_max = false;
        let expected_first_value = expected_kth_min_first_value(&nums, k);

        // Act
        let selected = quick_select(&nums, k, is_max);

        // Assert
        assert_eq!(selected.v.0, expected_first_value);
    }

    #[test]
    fn quick_select_when_same_input_is_executed_multiple_times_then_ranked_first_value_is_consistent(
    ) {
        // Arrange
        let nums = create_elements(&[(9, 1), (1, 2), (5, 3), (7, 4), (3, 5), (11, 6)]);
        let k = 4;
        let is_max = false;
        let repetition = 200;
        let expected_first_value = expected_kth_min_first_value(&nums, k);

        // Act
        let mut selected_first_values: Vec<i32> = vec![];
        for _ in 0..repetition {
            let selected = quick_select(&nums, k, is_max);
            selected_first_values.push(selected.v.0);
        }

        // Assert
        for selected_first_value in selected_first_values {
            assert_eq!(selected_first_value, expected_first_value);
        }
    }

    #[test]
    fn quick_select_when_called_then_input_slice_content_and_order_remain_unchanged() {
        // Arrange
        let nums = create_elements(&[(9, 11), (1, 22), (5, 33), (7, 44), (3, 55)]);
        let k = 2;
        let is_max = false;
        let expected_pairs: Vec<(i32, i32)> = nums.iter().map(|num| num.v).collect();

        // Act
        let _selected = quick_select(&nums, k, is_max);
        let actual_pairs: Vec<(i32, i32)> = nums.iter().map(|num| num.v).collect();

        // Assert
        assert_eq!(actual_pairs, expected_pairs);
    }
}
