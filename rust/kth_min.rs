use std::collections::{BTreeSet, HashMap};

#[derive(Clone, Copy, Debug)]
struct Key<T> {
    cnt: usize,
    val: T,
}

impl<T: Ord> PartialEq for Key<T> {
    fn eq(&self, other: &Self) -> bool {
        self.cnt == other.cnt && self.val == other.val
    }
}

impl<T: Ord> Eq for Key<T> {}

impl<T: Ord> PartialOrd for Key<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl<T: Ord> Ord for Key<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match self.cnt.cmp(&other.cnt) {
            std::cmp::Ordering::Equal => self.val.cmp(&other.val),
            o => o,
        }
    }
}

impl<T: Copy + Into<i128>> Key<T> {

    pub fn new(cnt: usize, x: T) -> Self {
        Self { cnt, val: x }
    }

    fn count(&self) -> usize {
        self.cnt
    }

    fn value(&self) -> T {
        self.val
    }

    pub fn total(&self) -> i128 {
        (self.count() as i128) * (self.value().into())
    }

}

pub struct KthMin<T> {
    capacity: usize,
    total: i128,
    counter: HashMap<T, usize>,
    ins: BTreeSet<Key<T>>,
    outs: BTreeSet<Key<T>>,
}

impl<T: Ord + Copy + Into<i128> + std::hash::Hash> KthMin<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            capacity,
            total: 0,
            counter: HashMap::new(),
            ins: BTreeSet::new(),
            outs: BTreeSet::new(),
        }
    }

    #[inline]
    fn condition(&self) -> bool {
        self.kind_condition()
    }

    #[inline]
    fn kind_condition(&self) -> bool {
        self.ins.len() < self.capacity
    }

    #[inline]
    fn sum(&self) -> i128 {
        self.total
    }

    fn insert(&mut self, v: T) {

        let counter = *self.counter.get(&v).unwrap_or(&0);
        let mut key = Key::new(counter, v);

        let counter = self.counter.entry(v).or_insert(0);
        *counter += 1;

        if let Some(mut node) = self.ins.take(&key) {
            self.total += v.into();
            node.cnt += 1;
            self.ins.insert(node);
        } else {
            self.outs.remove(&key);
            key.cnt += 1;
            self.outs.insert(key);

            self.swap();
        }

    }

    pub fn erase(&mut self, del_v: T) {

        let Some(&cnt) = self.counter.get(&del_v) else {
            return;
        };

        let del_key = Key::new(cnt, del_v);
        let is_key_in_ins = self.ins.contains(&del_key);

        if is_key_in_ins {
            let mut ins_key = self.ins.take(&del_key).unwrap();
            self.total -= del_v.into();

            if cnt == 1 {
                self.counter.remove(&del_v);
            } else {
                self.counter.insert(del_v, cnt - 1);
                ins_key.cnt -= 1;
                self.ins.insert(ins_key);
                self.swap();
            }
        } else {
            self.outs.remove(&del_key);

            if cnt == 1 {
                self.counter.remove(&del_v);
            } else {
                self.counter.insert(del_v, cnt - 1);
                self.outs.insert(Key::new(cnt - 1, del_v));
            }
        }

        self.merge();
    }

    fn merge(&mut self) {

        while !self.outs.is_empty() && self.condition() {
            let max_out = *self.outs.iter().next_back().unwrap();
            self.outs.take(&max_out);
            self.total += max_out.total();
            self.ins.insert(max_out);
        }

    }

    fn swap(&mut self) -> bool {

        if self.outs.is_empty() || self.ins.is_empty() {
            return false;
        }

        let min_in = *self.ins.iter().next().unwrap();
        let max_out = *self.outs.iter().next_back().unwrap();

        if min_in.cmp(&max_out) != std::cmp::Ordering::Less {
            return false;
        }

        self.ins.take(&min_in);
        self.total -= min_in.total();
        self.outs.insert(min_in);

        self.outs.take(&max_out);
        self.total += max_out.total();
        self.ins.insert(max_out);

        true
    }

    fn init(&mut self, k: usize, vec: &[T]) {

        self.total = 0;
        self.counter.clear();
        self.ins.clear();
        self.outs.clear();

        vec.iter()
            .take(k)
            .for_each(|v| {
                    let counter = self.counter.entry(*v).or_insert(0);
                    *counter += 1;
            });

        for (&val, &cnt) in self.counter.iter() {
            self.outs.insert(Key::new(cnt, val));
        }

        self.merge();
    }

    pub fn answers(&mut self, k: usize, nums: &[T]) -> Vec<i128> {

        let n = nums.len();

        assert!(k <= n, "nums length must be >= k");

        let mut ans = Vec::with_capacity(n - k + 1);

        self.init(k, nums);
        ans.push(self.sum());

        for i in k..n {
            self.insert(nums[i]);
            self.erase(nums[i - k]);
            ans.push(self.sum());
        }

        ans
    }

}


#[cfg(test)]
mod tests {
    use super::{Key, KthMin};
    use std::collections::HashMap;
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn capture_panic<F, R>(func: F) -> std::thread::Result<R>
    where
        F: FnOnce() -> R,
    {
        catch_unwind(AssertUnwindSafe(func))
    }

    fn calculate_reference_window_sum(capacity: usize, window_values: &[i64]) -> i128 {
        let mut count_by_value = HashMap::new();

        window_values.iter().for_each(|value| {
            let count = count_by_value.entry(*value).or_insert(0usize);
            *count += 1;
        });

        let mut count_and_value_pairs: Vec<(usize, i64)> = count_by_value
            .into_iter()
            .map(|(value, count)| (count, value))
            .collect();

        count_and_value_pairs.sort_unstable_by(
            |(left_count, left_value), (right_count, right_value)| {
                right_count
                    .cmp(left_count)
                    .then(right_value.cmp(left_value))
            },
        );

        count_and_value_pairs
            .iter()
            .take(capacity)
            .map(|(count, value)| (*count as i128) * (*value as i128))
            .sum()
    }

    fn calculate_reference_answers(capacity: usize, window_size: usize, numbers: &[i64]) -> Vec<i128> {
        assert!(window_size <= numbers.len());

        (0..=(numbers.len() - window_size))
            .map(|window_start_index| {
                let window_end_index = window_start_index + window_size;
                calculate_reference_window_sum(capacity, &numbers[window_start_index..window_end_index])
            })
            .collect()
    }

    fn next_seed(current_seed: u64) -> u64 {
        current_seed
            .wrapping_mul(6364136223846793005u64)
            .wrapping_add(1u64)
    }

    fn generate_deterministic_values(seed: u64, length: usize) -> Vec<i64> {
        let mut current_seed = seed;

        (0..length)
            .map(|_| {
                current_seed = next_seed(current_seed);
                let value_bucket = ((current_seed >> 32) % 11) as i64;
                value_bucket - 5
            })
            .collect()
    }

    #[test]
    fn erase_from_multiple_count_value_reinserts_key_with_decremented_count() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let initial_numbers = vec![5i64, 5, 1];
        let erased_value = 5i64;
        let expected_count_after_erase = Some(1usize);
        let expected_key_in_ins = Key::new(1usize, 5i64);
        let mut kth_min = KthMin::new(capacity);
        kth_min.init(window_size, &initial_numbers);

        // Act
        kth_min.erase(erased_value);
        let actual_count_after_erase = kth_min.counter.get(&erased_value).copied();
        let actual_contains_expected_key = kth_min.ins.contains(&expected_key_in_ins);

        // Assert
        assert_eq!(actual_count_after_erase, expected_count_after_erase);
        assert!(actual_contains_expected_key);
    }

    #[test]
    fn insert_when_value_is_in_ins_increases_total_only_by_inserted_value() {
        // Arrange
        let capacity = 1usize;
        let window_size = 2usize;
        let initial_numbers = vec![2i64, 1];
        let inserted_value = 2i64;
        let mut kth_min = KthMin::new(capacity);
        kth_min.init(window_size, &initial_numbers);
        let total_before_insert = kth_min.sum();
        let expected_total_after_insert = total_before_insert + (inserted_value as i128);

        // Act
        kth_min.insert(inserted_value);
        let actual_total_after_insert = kth_min.sum();

        // Assert
        assert_eq!(actual_total_after_insert, expected_total_after_insert);
    }

    #[test]
    fn insert_that_requires_swap_updates_ins_and_total_correctly() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let initial_numbers = vec![1i64, 2, 3];
        let inserted_value = 2i64;
        let expected_total = 4i128;
        let expected_key = Key::new(2usize, inserted_value);
        let mut kth_min = KthMin::new(capacity);
        kth_min.init(window_size, &initial_numbers);

        // Act
        kth_min.insert(inserted_value);
        let actual_total = kth_min.sum();
        let actual_contains_expected_key = kth_min.ins.contains(&expected_key);

        // Assert
        assert_eq!(actual_total, expected_total);
        assert!(actual_contains_expected_key);
    }

    #[test]
    fn answers_reflects_insert_then_erase_for_each_sliding_step() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let input_numbers = vec![2i64, 2, 1, 3];
        let expected_answers = vec![4i128, 3];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn erase_when_count_becomes_zero_removes_value_from_counter() {
        // Arrange
        let capacity = 1usize;
        let window_size = 1usize;
        let initial_numbers = vec![7i64];
        let erased_value = 7i64;
        let expected_total = 0i128;
        let mut kth_min = KthMin::new(capacity);
        kth_min.init(window_size, &initial_numbers);

        // Act
        kth_min.erase(erased_value);
        let actual_contains_value = kth_min.counter.contains_key(&erased_value);
        let actual_total = kth_min.sum();

        // Assert
        assert!(!actual_contains_value);
        assert_eq!(actual_total, expected_total);
    }

    #[test]
    fn erase_non_existent_value_keeps_internal_state_unchanged() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let initial_numbers = vec![1i64, 2, 2];
        let erased_value = 3i64;
        let mut kth_min = KthMin::new(capacity);
        kth_min.init(window_size, &initial_numbers);
        let expected_total = kth_min.sum();
        let expected_counter = kth_min.counter.clone();
        let expected_ins = kth_min.ins.clone();
        let expected_outs = kth_min.outs.clone();

        // Act
        kth_min.erase(erased_value);
        let actual_total = kth_min.sum();
        let actual_counter = kth_min.counter.clone();
        let actual_ins = kth_min.ins.clone();
        let actual_outs = kth_min.outs.clone();

        // Assert
        assert_eq!(actual_total, expected_total);
        assert_eq!(actual_counter, expected_counter);
        assert_eq!(actual_ins, expected_ins);
        assert_eq!(actual_outs, expected_outs);
    }

    #[test]
    fn erase_then_merge_fills_ins_when_current_size_is_below_capacity() {
        // Arrange
        let capacity = 2usize;
        let window_size = 3usize;
        let initial_numbers = vec![10i64, 9, 1];
        let erased_value = 10i64;
        let expected_ins_size = 2usize;
        let expected_total = 10i128;
        let expected_key = Key::new(1usize, 1i64);
        let mut kth_min = KthMin::new(capacity);
        kth_min.init(window_size, &initial_numbers);

        // Act
        kth_min.erase(erased_value);
        let actual_ins_size = kth_min.ins.len();
        let actual_total = kth_min.sum();
        let actual_contains_expected_key = kth_min.ins.contains(&expected_key);

        // Assert
        assert_eq!(actual_ins_size, expected_ins_size);
        assert_eq!(actual_total, expected_total);
        assert!(actual_contains_expected_key);
    }

    #[test]
    fn answers_with_capacity_greater_than_unique_value_count_matches_window_sum() {
        // Arrange
        let capacity = 5usize;
        let window_size = 3usize;
        let input_numbers = vec![4i64, -1, 4, 2];
        let expected_answers = vec![7i128, 5];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_single_value_multiset_returns_count_times_value() {
        // Arrange
        let capacity = 2usize;
        let window_size = 3usize;
        let input_numbers = vec![5i64, 5, 5, 5];
        let expected_answers = vec![15i128, 15];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_window_size_one_matches_each_value_when_capacity_is_one() {
        // Arrange
        let capacity = 1usize;
        let window_size = 1usize;
        let input_numbers = vec![3i64, -2, 7];
        let expected_answers = vec![3i128, -2, 7];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_window_size_equal_to_input_length_returns_single_full_window_value() {
        // Arrange
        let capacity = 2usize;
        let input_numbers = vec![5i64, 5, 1, 2];
        let window_size = input_numbers.len();
        let expected_answers = vec![12i128];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_distinct_values_prefers_larger_value_for_equal_frequency() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let input_numbers = vec![1i64, 2, 3];
        let expected_answers = vec![3i128];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_negative_tie_prefers_greater_value_closer_to_zero() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let input_numbers = vec![-5i64, -4, -3];
        let expected_answers = vec![-3i128];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_panics_when_window_size_is_greater_than_input_length() {
        // Arrange
        let capacity = 2usize;
        let window_size = 2usize;
        let input_numbers = vec![10i64];

        // Act
        let panic_result = capture_panic(|| {
            let mut kth_min = KthMin::new(capacity);
            let _actual_answers = kth_min.answers(window_size, &input_numbers);
        });

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn answers_with_empty_input_and_zero_window_returns_single_zero() {
        // Arrange
        let capacity = 3usize;
        let window_size = 0usize;
        let input_numbers: Vec<i64> = vec![];
        let expected_answers = vec![0i128];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_zero_window_on_non_empty_input_returns_all_zeros() {
        // Arrange
        let capacity = 2usize;
        let window_size = 0usize;
        let input_numbers = vec![4i64, 1, -2];
        let expected_answers = vec![0i128; input_numbers.len() + 1];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_zero_capacity_returns_zero_for_each_window() {
        // Arrange
        let capacity = 0usize;
        let window_size = 2usize;
        let input_numbers = vec![1i64, 2, 3];
        let expected_answers = vec![0i128, 0];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_capacity_one_uses_only_the_most_frequent_key() {
        // Arrange
        let capacity = 1usize;
        let window_size = 4usize;
        let input_numbers = vec![2i64, 2, 1, 1, 1];
        let expected_answers = vec![4i128, 3];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_x_minus_one_x_x_plus_one_tie_prefers_x_plus_one() {
        // Arrange
        let capacity = 1usize;
        let window_size = 3usize;
        let x = 10i64;
        let input_numbers = vec![x - 1, x, x + 1];
        let expected_answers = vec![(x + 1) as i128];
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_matches_reference_implementation_for_multiple_fixed_cases() {
        // Arrange
        let fixed_cases = vec![
            (0usize, 0usize, vec![]),
            (1usize, 1usize, vec![3i64]),
            (2usize, 2usize, vec![1i64, 2, 1, 3]),
            (3usize, 3usize, vec![-1i64, -1, 2, 2, 2]),
            (2usize, 4usize, vec![4i64, 3, 2, 1, 0]),
        ];
        let expected_answers = fixed_cases.iter().map(|(capacity, window_size, input_numbers)| {
            calculate_reference_answers(*capacity, *window_size, input_numbers)
        }).collect::<Vec<_>>();

        // Act
        let actual_answers = fixed_cases
            .iter()
            .map(|(capacity, window_size, input_numbers)| {
                let mut kth_min = KthMin::new(*capacity);
                kth_min.answers(*window_size, input_numbers)
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_matches_reference_implementation_for_deterministic_random_cases() {
        // Arrange
        let input_lengths = vec![0usize, 1, 2, 3, 5];
        let random_case_count = 25usize;
        let capacities = vec![0usize, 1, 2, 3];
        let initial_seed = 20260211u64;
        let mut current_seed = initial_seed;
        let mut cases = Vec::new();
        for case_index in 0..random_case_count {
            let target_length = input_lengths[case_index % input_lengths.len()];
            current_seed = next_seed(current_seed);
            let input_numbers = generate_deterministic_values(current_seed, target_length);
            for capacity in capacities.iter().copied() {
                for window_size in 0..=target_length {
                    let expected_answers = calculate_reference_answers(capacity, window_size, &input_numbers);
                    cases.push((capacity, window_size, input_numbers.clone(), expected_answers));
                }
            }
        }

        // Act
        let actual_answers = cases.iter().map(|(capacity, window_size, input_numbers, _)| {
            KthMin::new(*capacity).answers(*window_size, input_numbers)
        }).collect::<Vec<_>>();
        let expected_answers = cases.into_iter().map(|(_, _, _, expected_answers)| expected_answers).collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_with_i64_min_and_i64_max_matches_reference_without_panic() {
        // Arrange
        let capacity = 2usize;
        let window_size = 4usize;
        let input_numbers = vec![i64::MIN, 0, i64::MAX, i64::MIN, i64::MAX];
        let expected_answers = calculate_reference_answers(capacity, window_size, &input_numbers);
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_answers = kth_min.answers(window_size, &input_numbers);

        // Assert
        assert_eq!(actual_answers, expected_answers);
    }

    #[test]
    fn answers_called_twice_on_same_instance_resets_previous_state() {
        // Arrange
        let capacity = 2usize;
        let first_window_size = 2usize;
        let second_window_size = 2usize;
        let first_numbers = vec![1i64, 2, 2];
        let second_numbers = vec![3i64, 3, 4];
        let expected_first_answers =
            calculate_reference_answers(capacity, first_window_size, &first_numbers);
        let expected_second_answers =
            calculate_reference_answers(capacity, second_window_size, &second_numbers);
        let mut kth_min = KthMin::new(capacity);

        // Act
        let actual_first_answers = kth_min.answers(first_window_size, &first_numbers);
        let actual_second_answers = kth_min.answers(second_window_size, &second_numbers);

        // Assert
        assert_eq!(actual_first_answers, expected_first_answers);
        assert_eq!(actual_second_answers, expected_second_answers);
    }
}
