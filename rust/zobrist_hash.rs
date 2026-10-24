use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};

const DEFAULT_SEED: u64 = 0x9e37_79b9_7f4a_7c15;

#[derive(Clone, Debug)]
pub struct ZobristHash<T> {
    seed: u64,
    hash_value: u64,
    element_count: usize,
    element_counts: HashMap<T, usize>,
}

impl<T: Eq + Hash> Default for ZobristHash<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Eq + Hash> ZobristHash<T> {
    pub fn new() -> Self {
        Self::with_seed(DEFAULT_SEED)
    }

    pub fn with_seed(seed: u64) -> Self {
        Self {
            seed,
            hash_value: 0,
            element_count: 0,
            element_counts: HashMap::new(),
        }
    }

    pub fn from_slice(values: &[T]) -> Self
    where
        T: Clone,
    {
        let mut zobrist_hash = Self::new();

        for value in values {
            zobrist_hash.insert(value.clone());
        }

        zobrist_hash
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.element_count
    }

    #[inline]
    pub fn distinct_len(&self) -> usize {
        self.element_counts.len()
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.element_count == 0
    }

    #[inline]
    pub fn hash(&self) -> u64 {
        self.hash_value
    }

    pub fn contains(&self, value: &T) -> bool {
        self.element_counts.contains_key(value)
    }

    pub fn count(&self, value: &T) -> usize {
        self.element_counts.get(value).copied().unwrap_or(0)
    }

    pub fn insert(&mut self, value: T) {
        self.insert_multiple(value, 1);
    }

    pub fn insert_multiple(&mut self, value: T, insert_count: usize) {
        if insert_count == 0 {
            return;
        }

        let current_count = self.count(&value);
        let next_count = current_count + insert_count;

        self.apply_count_change(&value, current_count, next_count);
        self.element_count += insert_count;
        self.element_counts.insert(value, next_count);
    }

    pub fn remove(&mut self, value: &T) -> bool {
        self.remove_multiple(value, 1) == 1
    }

    pub fn remove_multiple(&mut self, value: &T, remove_count: usize) -> usize {
        if remove_count == 0 {
            return 0;
        }

        let Some(current_count) = self.count_existing(value) else {
            return 0;
        };

        let removed_count = current_count.min(remove_count);
        let next_count = current_count - removed_count;

        self.apply_count_change(value, current_count, next_count);
        self.element_count -= removed_count;

        if next_count == 0 {
            self.element_counts.remove(value);
        } else if let Some(count) = self.element_counts.get_mut(value) {
            *count = next_count;
        }

        removed_count
    }

    pub fn remove_all(&mut self, value: &T) -> usize {
        let current_count = self.count(value);

        self.remove_multiple(value, current_count)
    }

    pub fn clear(&mut self) {
        self.hash_value = 0;
        self.element_count = 0;
        self.element_counts.clear();
    }

    fn count_existing(&self, value: &T) -> Option<usize> {
        self.element_counts.get(value).copied()
    }

    fn apply_count_change(&mut self, value: &T, current_count: usize, next_count: usize) {
        if 0 < current_count {
            self.hash_value ^= Self::element_count_hash(self.seed, value, current_count);
        }

        if 0 < next_count {
            self.hash_value ^= Self::element_count_hash(self.seed, value, next_count);
        }
    }

    fn element_count_hash(seed: u64, value: &T, count: usize) -> u64 {
        let mut hasher = DefaultHasher::new();
        seed.hash(&mut hasher);
        value.hash(&mut hasher);
        count.hash(&mut hasher);

        Self::splitmix64(hasher.finish())
    }

    fn splitmix64(value: u64) -> u64 {
        let mut x = value.wrapping_add(0x9e37_79b9_7f4a_7c15);
        x = (x ^ (x >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        x = (x ^ (x >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);

        x ^ (x >> 31)
    }
}

#[cfg(test)]
mod tests {
    use super::ZobristHash;

    const TEST_SEED: u64 = 1_234_567_891;

    fn build_seeded_hash(values: &[i32]) -> ZobristHash<i32> {
        let mut zobrist_hash = ZobristHash::with_seed(TEST_SEED);

        for &value in values {
            zobrist_hash.insert(value);
        }

        zobrist_hash
    }

    #[test]
    fn new_returns_empty_hash_with_zero_element_count() {
        // Arrange
        let expected_len = 0usize;
        let expected_distinct_len = 0usize;
        let expected_is_empty = true;
        let expected_hash = 0u64;

        // Act
        let zobrist_hash = ZobristHash::<i32>::new();
        let actual_len = zobrist_hash.len();
        let actual_distinct_len = zobrist_hash.distinct_len();
        let actual_is_empty = zobrist_hash.is_empty();
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_distinct_len, expected_distinct_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn with_seed_uses_seed_to_change_hash_value() {
        // Arrange
        let first_seed = 100u64;
        let second_seed = 200u64;
        let inserted_value = 42i32;
        let mut first_hash = ZobristHash::with_seed(first_seed);
        let mut second_hash = ZobristHash::with_seed(second_seed);

        // Act
        first_hash.insert(inserted_value);
        second_hash.insert(inserted_value);
        let actual_first_hash = first_hash.hash();
        let actual_second_hash = second_hash.hash();

        // Assert
        assert_ne!(actual_first_hash, actual_second_hash);
    }

    #[test]
    fn from_slice_with_empty_slice_returns_empty_hash() {
        // Arrange
        let values: Vec<i32> = vec![];
        let expected_len = 0usize;
        let expected_is_empty = true;

        // Act
        let zobrist_hash = ZobristHash::from_slice(&values);
        let actual_len = zobrist_hash.len();
        let actual_is_empty = zobrist_hash.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn from_slice_with_single_element_returns_one_element_count() {
        // Arrange
        let values = vec![7i32];
        let target_value = 7i32;
        let expected_len = 1usize;
        let expected_count = 1usize;

        // Act
        let zobrist_hash = ZobristHash::from_slice(&values);
        let actual_len = zobrist_hash.len();
        let actual_count = zobrist_hash.count(&target_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn from_slice_with_multiple_elements_counts_each_value() {
        // Arrange
        let values = vec![1i32, 2, 1, 3, 2, 1];
        let first_value = 1i32;
        let second_value = 2i32;
        let third_value = 3i32;
        let expected_len = 6usize;
        let expected_first_count = 3usize;
        let expected_second_count = 2usize;
        let expected_third_count = 1usize;

        // Act
        let zobrist_hash = ZobristHash::from_slice(&values);
        let actual_len = zobrist_hash.len();
        let actual_first_count = zobrist_hash.count(&first_value);
        let actual_second_count = zobrist_hash.count(&second_value);
        let actual_third_count = zobrist_hash.count(&third_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_first_count, expected_first_count);
        assert_eq!(actual_second_count, expected_second_count);
        assert_eq!(actual_third_count, expected_third_count);
    }

    #[test]
    fn contains_with_registered_value_returns_true() {
        // Arrange
        let registered_value = 10i32;
        let expected_contains = true;
        let zobrist_hash = build_seeded_hash(&[registered_value]);

        // Act
        let actual_contains = zobrist_hash.contains(&registered_value);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn count_with_unregistered_value_returns_zero() {
        // Arrange
        let registered_value = 10i32;
        let unregistered_value = 20i32;
        let expected_count = 0usize;
        let zobrist_hash = build_seeded_hash(&[registered_value]);

        // Act
        let actual_count = zobrist_hash.count(&unregistered_value);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn insert_with_new_value_updates_len_count_and_hash() {
        // Arrange
        let inserted_value = 10i32;
        let expected_len = 1usize;
        let expected_count = 1usize;
        let expected_is_empty = false;
        let unexpected_empty_hash = 0u64;
        let mut zobrist_hash = ZobristHash::with_seed(TEST_SEED);

        // Act
        zobrist_hash.insert(inserted_value);
        let actual_len = zobrist_hash.len();
        let actual_count = zobrist_hash.count(&inserted_value);
        let actual_is_empty = zobrist_hash.is_empty();
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_ne!(actual_hash, unexpected_empty_hash);
    }

    #[test]
    fn insert_with_duplicate_value_keeps_count_in_hash_state() {
        // Arrange
        let duplicated_value = 10i32;
        let expected_len = 2usize;
        let expected_count = 2usize;
        let mut duplicate_hash = ZobristHash::with_seed(TEST_SEED);
        let single_hash = build_seeded_hash(&[duplicated_value]);

        // Act
        duplicate_hash.insert(duplicated_value);
        duplicate_hash.insert(duplicated_value);
        let actual_len = duplicate_hash.len();
        let actual_count = duplicate_hash.count(&duplicated_value);
        let actual_duplicate_hash = duplicate_hash.hash();
        let actual_single_hash = single_hash.hash();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
        assert_ne!(actual_duplicate_hash, actual_single_hash);
    }

    #[test]
    fn insert_order_does_not_change_hash_for_same_multiset() {
        // Arrange
        let first_order_values = vec![1i32, 2, 1, 3];
        let second_order_values = vec![3i32, 1, 2, 1];
        let first_hash = build_seeded_hash(&first_order_values);

        // Act
        let second_hash = build_seeded_hash(&second_order_values);
        let actual_first_hash = first_hash.hash();
        let actual_second_hash = second_hash.hash();

        // Assert
        assert_eq!(actual_first_hash, actual_second_hash);
    }

    #[test]
    fn insert_multiple_with_zero_count_does_not_change_state() {
        // Arrange
        let inserted_value = 10i32;
        let insert_count = 0usize;
        let mut zobrist_hash = build_seeded_hash(&[inserted_value]);
        let expected_len = zobrist_hash.len();
        let expected_count = zobrist_hash.count(&inserted_value);
        let expected_hash = zobrist_hash.hash();

        // Act
        zobrist_hash.insert_multiple(inserted_value, insert_count);
        let actual_len = zobrist_hash.len();
        let actual_count = zobrist_hash.count(&inserted_value);
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn insert_multiple_for_new_key_matches_repeated_single_insertions() {
        // Arrange
        let target_value = 10i32;
        let other_value = 20i32;
        let count = 3;
        let mut hash = build_seeded_hash(&[other_value]);
        let expected_hash =
            build_seeded_hash(&[other_value, target_value, target_value, target_value]);
        let expected_state = (
            expected_hash.hash(),
            expected_hash.len(),
            expected_hash.count(&target_value),
            expected_hash.distinct_len(),
        );

        // Act
        hash.insert_multiple(target_value, count);
        let actual_state = (
            hash.hash(),
            hash.len(),
            hash.count(&target_value),
            hash.distinct_len(),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn insert_multiple_for_existing_key_matches_repeated_single_insertions() {
        // Arrange
        let target_value = 10i32;
        let other_value = 20i32;
        let count = 2;
        let mut hash = build_seeded_hash(&[other_value, target_value]);
        let expected_hash =
            build_seeded_hash(&[other_value, target_value, target_value, target_value]);
        let expected_state = (
            expected_hash.hash(),
            expected_hash.len(),
            expected_hash.count(&target_value),
            expected_hash.distinct_len(),
        );

        // Act
        hash.insert_multiple(target_value, count);
        let actual_state = (
            hash.hash(),
            hash.len(),
            hash.count(&target_value),
            hash.distinct_len(),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn remove_with_existing_value_decrements_count_and_hash() {
        // Arrange
        let target_value = 10i32;
        let expected_removed = true;
        let expected_len = 1usize;
        let expected_count = 1usize;
        let expected_hash = build_seeded_hash(&[target_value]).hash();
        let mut zobrist_hash = build_seeded_hash(&[target_value, target_value]);

        // Act
        let actual_removed = zobrist_hash.remove(&target_value);
        let actual_len = zobrist_hash.len();
        let actual_count = zobrist_hash.count(&target_value);
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn remove_with_last_existing_value_removes_key_and_hash() {
        // Arrange
        let target_value = 10i32;
        let expected_removed = true;
        let expected_len = 0usize;
        let expected_count = 0usize;
        let expected_contains = false;
        let expected_hash = 0u64;
        let mut zobrist_hash = build_seeded_hash(&[target_value]);

        // Act
        let actual_removed = zobrist_hash.remove(&target_value);
        let actual_len = zobrist_hash.len();
        let actual_count = zobrist_hash.count(&target_value);
        let actual_contains = zobrist_hash.contains(&target_value);
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn remove_with_missing_value_returns_false_and_keeps_state() {
        // Arrange
        let registered_value = 10i32;
        let missing_value = 20i32;
        let expected_removed = false;
        let mut zobrist_hash = build_seeded_hash(&[registered_value]);
        let expected_len = zobrist_hash.len();
        let expected_hash = zobrist_hash.hash();

        // Act
        let actual_removed = zobrist_hash.remove(&missing_value);
        let actual_len = zobrist_hash.len();
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn remove_multiple_with_zero_count_does_not_change_state() {
        // Arrange
        let target_value = 10i32;
        let remove_count = 0usize;
        let expected_removed_count = 0usize;
        let mut zobrist_hash = build_seeded_hash(&[target_value, target_value]);
        let expected_len = zobrist_hash.len();
        let expected_hash = zobrist_hash.hash();

        // Act
        let actual_removed_count = zobrist_hash.remove_multiple(&target_value, remove_count);
        let actual_len = zobrist_hash.len();
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn remove_multiple_with_more_than_existing_count_removes_all_existing_values() {
        // Arrange
        let target_value = 10i32;
        let remove_count = 5usize;
        let expected_removed_count = 2usize;
        let expected_len = 0usize;
        let expected_count = 0usize;
        let expected_hash = 0u64;
        let mut zobrist_hash = build_seeded_hash(&[target_value, target_value]);

        // Act
        let actual_removed_count = zobrist_hash.remove_multiple(&target_value, remove_count);
        let actual_len = zobrist_hash.len();
        let actual_count = zobrist_hash.count(&target_value);
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn remove_multiple_handles_count_minus_one_count_and_count_plus_one_with_other_values() {
        // Arrange
        let target_value = 10i32;
        let other_value = 20i32;
        let initial_values = [target_value, target_value, target_value, other_value];
        let existing_count = 3usize;
        let remove_counts = [existing_count - 1, existing_count, existing_count + 1];
        let expected_remaining_hash = build_seeded_hash(&[target_value, other_value]).hash();
        let expected_other_hash = build_seeded_hash(&[other_value]).hash();
        let expected_states = [
            (2, 1, 2, 2, expected_remaining_hash),
            (3, 0, 1, 1, expected_other_hash),
            (3, 0, 1, 1, expected_other_hash),
        ];

        // Act
        let actual_states = remove_counts.map(|remove_count| {
            let mut zobrist_hash = build_seeded_hash(&initial_values);
            let removed = zobrist_hash.remove_multiple(&target_value, remove_count);
            (
                removed,
                zobrist_hash.count(&target_value),
                zobrist_hash.len(),
                zobrist_hash.distinct_len(),
                zobrist_hash.hash(),
            )
        });

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn remove_all_with_existing_value_removes_only_target_value() {
        // Arrange
        let target_value = 10i32;
        let remaining_value = 20i32;
        let expected_removed_count = 2usize;
        let expected_len = 1usize;
        let expected_target_count = 0usize;
        let expected_remaining_count = 1usize;
        let expected_hash = build_seeded_hash(&[remaining_value]).hash();
        let mut zobrist_hash = build_seeded_hash(&[target_value, remaining_value, target_value]);

        // Act
        let actual_removed_count = zobrist_hash.remove_all(&target_value);
        let actual_len = zobrist_hash.len();
        let actual_target_count = zobrist_hash.count(&target_value);
        let actual_remaining_count = zobrist_hash.count(&remaining_value);
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_target_count, expected_target_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn remove_all_with_missing_value_returns_zero_and_keeps_state() {
        // Arrange
        let registered_value = 10i32;
        let missing_value = 20i32;
        let expected_removed_count = 0usize;
        let mut zobrist_hash = build_seeded_hash(&[registered_value]);
        let expected_len = zobrist_hash.len();
        let expected_hash = zobrist_hash.hash();

        // Act
        let actual_removed_count = zobrist_hash.remove_all(&missing_value);
        let actual_len = zobrist_hash.len();
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_hash, expected_hash);
    }

    #[test]
    fn clear_with_multiple_values_resets_hash_and_counts() {
        // Arrange
        let first_value = 10i32;
        let second_value = 20i32;
        let expected_len = 0usize;
        let expected_distinct_len = 0usize;
        let expected_first_count = 0usize;
        let expected_second_count = 0usize;
        let expected_hash = 0u64;
        let mut zobrist_hash = build_seeded_hash(&[first_value, second_value, first_value]);

        // Act
        zobrist_hash.clear();
        let actual_len = zobrist_hash.len();
        let actual_distinct_len = zobrist_hash.distinct_len();
        let actual_first_count = zobrist_hash.count(&first_value);
        let actual_second_count = zobrist_hash.count(&second_value);
        let actual_hash = zobrist_hash.hash();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_distinct_len, expected_distinct_len);
        assert_eq!(actual_first_count, expected_first_count);
        assert_eq!(actual_second_count, expected_second_count);
        assert_eq!(actual_hash, expected_hash);
    }
}
