const BUCKET_COUNT: usize = u64::BITS as usize + 1;

#[derive(Debug, Clone)]
pub struct RaddixHeap<T> {
    buckets: Vec<Vec<(u64, T)>>,
    size: usize,
    last: u64,
}

impl<T> Default for RaddixHeap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> RaddixHeap<T> {
    pub fn new() -> Self {
        let buckets = (0..BUCKET_COUNT)
            .map(|_| Vec::new())
            .collect::<Vec<Vec<(u64, T)>>>();

        Self {
            buckets,
            size: 0,
            last: 0,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn push(&mut self, key: u64, value: T) {
        assert!(
            self.last <= key,
            "radix heap requires non-decreasing pushed keys"
        );

        let bucket_index = Self::bucket_index_from_last(self.last, key);
        self.buckets[bucket_index].push((key, value));
        self.size += 1;
    }

    pub fn pop(&mut self) -> Option<(u64, T)> {
        if self.size == 0 {
            return None;
        }

        self.refill_bucket_zero_if_needed();

        self.size -= 1;
        self.buckets[0].pop()
    }

    fn bucket_index_from_last(last_key: u64, key: u64) -> usize {
        let xor_distance = last_key ^ key;
        if xor_distance == 0 {
            0
        } else {
            u64::BITS as usize - xor_distance.leading_zeros() as usize
        }
    }

    fn refill_bucket_zero_if_needed(&mut self) {
        if !self.buckets[0].is_empty() {
            return;
        }

        let first_non_empty_bucket_index = self.find_first_non_empty_bucket_index().unwrap();
        self.last = self.minimum_key_in_bucket(first_non_empty_bucket_index);
        self.redistribute_entries_from_bucket(first_non_empty_bucket_index);
    }

    fn find_first_non_empty_bucket_index(&self) -> Option<usize> {
        for bucket_index in 1..BUCKET_COUNT {
            if !self.buckets[bucket_index].is_empty() {
                return Some(bucket_index);
            }
        }

        None
    }

    fn minimum_key_in_bucket(&self, bucket_index: usize) -> u64 {
        self.buckets[bucket_index]
            .iter()
            .map(|(key, _)| *key)
            .min()
            .unwrap()
    }

    fn redistribute_entries_from_bucket(&mut self, bucket_index: usize) {
        let entries = std::mem::take(&mut self.buckets[bucket_index]);

        for (key, value) in entries {
            let redistributed_bucket_index = Self::bucket_index_from_last(self.last, key);
            self.buckets[redistributed_bucket_index].push((key, value));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_creates_same_empty_state_as_new() {
        // Arrange
        let default_heap: RaddixHeap<&str> = RaddixHeap::default();
        let new_heap: RaddixHeap<&str> = RaddixHeap::new();
        let expected_len = new_heap.len();
        let expected_is_empty = new_heap.is_empty();

        // Act
        let actual_len = default_heap.len();
        let actual_is_empty = default_heap.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn push_singleton_then_pop_returns_same_entry() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let single_key = 7u64;
        let single_value = "seven";
        let expected_popped_entry = Some((single_key, single_value));
        let expected_len_after_pop = 0usize;
        let expected_is_empty_after_pop = true;

        // Act
        heap.push(single_key, single_value);
        let actual_popped_entry = heap.pop();
        let actual_len_after_pop = heap.len();
        let actual_is_empty_after_pop = heap.is_empty();

        // Assert
        assert_eq!(actual_popped_entry, expected_popped_entry);
        assert_eq!(actual_len_after_pop, expected_len_after_pop);
        assert_eq!(actual_is_empty_after_pop, expected_is_empty_after_pop);
    }

    #[test]
    fn push_key_smaller_than_last_popped_key_panics() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let first_key = 5u64;
        let first_value = "first";
        let invalid_next_key = 4u64;
        let invalid_next_value = "invalid";
        heap.push(first_key, first_value);
        let popped_entry = heap.pop();
        let expected_popped_entry = Some((first_key, first_value));
        let expected_is_panic = true;

        // Act
        let push_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            heap.push(invalid_next_key, invalid_next_value);
        }));
        let actual_is_panic = push_result.is_err();

        // Assert
        assert_eq!(popped_entry, expected_popped_entry);
        assert_eq!(actual_is_panic, expected_is_panic);
    }

    #[test]
    fn push_increments_len_and_makes_heap_non_empty() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let key = 4u64;
        let value = "four";
        let expected_len = 1usize;
        let expected_is_empty = false;

        // Act
        heap.push(key, value);
        let actual_len = heap.len();
        let actual_is_empty = heap.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn push_same_key_as_last_popped_key_is_allowed() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let key = 5u64;
        let first_value = "first";
        let second_value = "second";
        let expected_second_popped_key = Some(key);

        // Act
        heap.push(key, first_value);
        let first_popped_entry = heap.pop();
        heap.push(key, second_value);
        let actual_second_popped_key = heap.pop().map(|(popped_key, _)| popped_key);

        let actual_first_popped_entry_right = Some((key, first_value));

        // Assert
        assert_eq!(first_popped_entry, actual_first_popped_entry_right);
        assert_eq!(actual_second_popped_key, expected_second_popped_key);
    }

    #[test]
    fn push_greater_key_after_partial_pop_keeps_ascending_pop_order() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let first_entry = (2u64, "two");
        let second_entry = (4u64, "four");
        let third_entry = (6u64, "six");
        let expected_entries = vec![Some(first_entry), Some(second_entry), Some(third_entry)];

        // Act
        heap.push(first_entry.0, first_entry.1);
        heap.push(second_entry.0, second_entry.1);
        let actual_first_entry = heap.pop();
        heap.push(third_entry.0, third_entry.1);
        let actual_second_entry = heap.pop();
        let actual_third_entry = heap.pop();
        let actual_entries = vec![actual_first_entry, actual_second_entry, actual_third_entry];

        // Assert
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    fn duplicate_key_values_are_all_returned() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let duplicate_key = 3u64;
        let values = ["first", "second", "third"];
        let mut expected_entries = values
            .iter()
            .map(|&value| (duplicate_key, value))
            .collect::<Vec<_>>();
        expected_entries.sort_unstable();

        // Act
        for value in values {
            heap.push(duplicate_key, value);
        }
        let mut actual_entries = Vec::new();
        while let Some(entry) = heap.pop() {
            actual_entries.push(entry);
        }
        actual_entries.sort_unstable();

        // Assert
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    fn pop_with_empty_heap_returns_none_and_keeps_empty_state() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let expected_popped_entry: Option<(u64, &str)> = None;
        let expected_len = 0usize;
        let expected_is_empty = true;

        // Act
        let actual_popped_entry = heap.pop();
        let actual_len = heap.len();
        let actual_is_empty = heap.is_empty();

        // Assert
        assert_eq!(actual_popped_entry, expected_popped_entry);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn pop_with_multiple_entries_returns_entries_in_ascending_key_order() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let first_key = 8u64;
        let first_value = "eight";
        let second_key = 3u64;
        let second_value = "three";
        let third_key = 5u64;
        let third_value = "five";
        let expected_first_popped_entry = Some((second_key, second_value));
        let expected_second_popped_entry = Some((third_key, third_value));
        let expected_third_popped_entry = Some((first_key, first_value));
        let expected_fourth_popped_entry: Option<(u64, &str)> = None;

        // Act
        heap.push(first_key, first_value);
        heap.push(second_key, second_value);
        heap.push(third_key, third_value);
        let actual_first_popped_entry = heap.pop();
        let actual_second_popped_entry = heap.pop();
        let actual_third_popped_entry = heap.pop();
        let actual_fourth_popped_entry = heap.pop();

        // Assert
        assert_eq!(actual_first_popped_entry, expected_first_popped_entry);
        assert_eq!(actual_second_popped_entry, expected_second_popped_entry);
        assert_eq!(actual_third_popped_entry, expected_third_popped_entry);
        assert_eq!(actual_fourth_popped_entry, expected_fourth_popped_entry);
    }

    #[test]
    fn pop_boundary_values_x_minus_1_x_x_plus_1_returns_in_ascending_order() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let x_value = 100u64;
        let x_minus_one_key = x_value - 1;
        let x_key = x_value;
        let x_plus_one_key = x_value + 1;
        let x_minus_one_value = "x-1";
        let x_value_label = "x";
        let x_plus_one_value = "x+1";
        let expected_first_popped_entry = Some((x_minus_one_key, x_minus_one_value));
        let expected_second_popped_entry = Some((x_key, x_value_label));
        let expected_third_popped_entry = Some((x_plus_one_key, x_plus_one_value));

        // Act
        heap.push(x_plus_one_key, x_plus_one_value);
        heap.push(x_minus_one_key, x_minus_one_value);
        heap.push(x_key, x_value_label);
        let actual_first_popped_entry = heap.pop();
        let actual_second_popped_entry = heap.pop();
        let actual_third_popped_entry = heap.pop();

        // Assert
        assert_eq!(actual_first_popped_entry, expected_first_popped_entry);
        assert_eq!(actual_second_popped_entry, expected_second_popped_entry);
        assert_eq!(actual_third_popped_entry, expected_third_popped_entry);
    }

    #[test]
    fn pop_lower_bound_zero_key_then_positive_key_returns_zero_first() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let lower_bound_key = 0u64;
        let upper_key = 1u64;
        let lower_bound_value = "zero";
        let upper_value = "one";
        let expected_first_popped_entry = Some((lower_bound_key, lower_bound_value));
        let expected_second_popped_entry = Some((upper_key, upper_value));

        // Act
        heap.push(upper_key, upper_value);
        heap.push(lower_bound_key, lower_bound_value);
        let actual_first_popped_entry = heap.pop();
        let actual_second_popped_entry = heap.pop();

        // Assert
        assert_eq!(actual_first_popped_entry, expected_first_popped_entry);
        assert_eq!(actual_second_popped_entry, expected_second_popped_entry);
    }

    #[test]
    fn pop_upper_bound_u64_max_and_neighbor_returns_ascending_order() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let upper_bound_key = u64::MAX;
        let upper_bound_minus_one_key = upper_bound_key - 1;
        let upper_bound_minus_one_value = "max-1";
        let upper_bound_value = "max";
        let expected_first_popped_entry =
            Some((upper_bound_minus_one_key, upper_bound_minus_one_value));
        let expected_second_popped_entry = Some((upper_bound_key, upper_bound_value));

        // Act
        heap.push(upper_bound_key, upper_bound_value);
        heap.push(upper_bound_minus_one_key, upper_bound_minus_one_value);
        let actual_first_popped_entry = heap.pop();
        let actual_second_popped_entry = heap.pop();

        // Assert
        assert_eq!(actual_first_popped_entry, expected_first_popped_entry);
        assert_eq!(actual_second_popped_entry, expected_second_popped_entry);
    }

    #[test]
    fn pop_with_duplicate_keys_returns_all_elements_for_that_key() {
        // Arrange
        let mut heap: RaddixHeap<&str> = RaddixHeap::new();
        let duplicate_key = 9u64;
        let first_value = "first";
        let second_value = "second";
        let expected_first_key = Some(duplicate_key);
        let expected_second_key = Some(duplicate_key);
        let expected_len_after_two_pops = 0usize;

        // Act
        heap.push(duplicate_key, first_value);
        heap.push(duplicate_key, second_value);
        let actual_first_popped_key = heap.pop().map(|(key, _)| key);
        let actual_second_popped_key = heap.pop().map(|(key, _)| key);
        let actual_len_after_two_pops = heap.len();

        // Assert
        assert_eq!(actual_first_popped_key, expected_first_key);
        assert_eq!(actual_second_popped_key, expected_second_key);
        assert_eq!(actual_len_after_two_pops, expected_len_after_two_pops);
    }
}
