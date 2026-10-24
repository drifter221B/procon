mod hash_table {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    pub struct HashTable<K, V> {
        size: usize,
        hash_table: Vec<Vec<(K, V)>>,
    }

    impl<K: Eq + Hash, V> HashTable<K, V> {

        pub fn new(size: usize) -> Self {
            HashTable {
                size,
                hash_table: (0..size).map(|_| Vec::new()).collect(),
            }
        }

        fn hash(&self, key: &K) -> usize {
            let mut hasher = DefaultHasher::new();
            key.hash(&mut hasher);
            (hasher.finish() as usize) % self.size
        }

        pub fn len(&self) -> usize {
            self.size
        }

        pub fn set(&mut self, key: K, value: V) {

            let index = self.hash(&key);

            if let Some(bucket) = self.hash_table.get_mut(index) {

                for (k, v) in bucket.iter_mut() {

                    if *k == key {
                        *v = value;
                        return;
                    }
                }

                bucket.push((key, value));
            }
        }

        pub fn get(&self, key: &K) -> Option<&V> {

            let index = self.hash(&key);

            if let Some(bucket) = self.hash_table.get(index) {
                for (k, v) in bucket.iter() {
                    if *k == *key {
                        return Some(v);
                    }
                }
            }

            None
        }
    }

    #[cfg(test)]
    mod tests {
        use super::HashTable;
        use std::hash::{Hash, Hasher};
        use std::panic::{self, AssertUnwindSafe};

        #[derive(Clone, Debug, Eq, PartialEq)]
        struct CollisionKey {
            key_id: i32,
            forced_hash: u64,
        }

        impl Hash for CollisionKey {
            fn hash<H: Hasher>(&self, state: &mut H) {
                self.forced_hash.hash(state);
            }
        }

        #[test]
        fn new_and_len_with_size_boundary_values_returns_each_size() {
            // Arrange
            let size_x_minus_1 = 0usize;
            let size_x = 1usize;
            let size_x_plus_1 = 2usize;

            // Act
            let hash_table_x_minus_1 = HashTable::<i32, i32>::new(size_x_minus_1);
            let hash_table_x = HashTable::<i32, i32>::new(size_x);
            let hash_table_x_plus_1 = HashTable::<i32, i32>::new(size_x_plus_1);
            let actual_len_x_minus_1 = hash_table_x_minus_1.len();
            let actual_len_x = hash_table_x.len();
            let actual_len_x_plus_1 = hash_table_x_plus_1.len();

            // Assert
            assert_eq!(actual_len_x_minus_1, size_x_minus_1);
            assert_eq!(actual_len_x, size_x);
            assert_eq!(actual_len_x_plus_1, size_x_plus_1);
        }

        #[test]
        fn set_with_zero_size_table_panics_as_current_behavior() {
            // Arrange
            let table_size = 0usize;
            let insert_key = 1i32;
            let insert_value = 10i32;
            let mut hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            let actual_panic_result = panic::catch_unwind(AssertUnwindSafe(|| {
                hash_table.set(insert_key, insert_value);
            }));

            // Assert
            assert!(actual_panic_result.is_err());
        }

        #[test]
        fn set_and_get_with_single_entry_returns_registered_value() {
            // Arrange
            let table_size = 3usize;
            let registered_key = 7i32;
            let registered_value = 70i32;
            let mut hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            hash_table.set(registered_key, registered_value);
            let actual_value = hash_table.get(&registered_key);

            // Assert
            assert_eq!(actual_value, Some(&registered_value));
        }

        #[test]
        fn set_and_get_with_multiple_entries_returns_each_registered_value() {
            // Arrange
            let table_size = 5usize;
            let first_key = 1i32;
            let second_key = 2i32;
            let third_key = 3i32;
            let first_value = 11i32;
            let second_value = 22i32;
            let third_value = 33i32;
            let mut hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            hash_table.set(first_key, first_value);
            hash_table.set(second_key, second_value);
            hash_table.set(third_key, third_value);
            let actual_first_value = hash_table.get(&first_key);
            let actual_second_value = hash_table.get(&second_key);
            let actual_third_value = hash_table.get(&third_key);

            // Assert
            assert_eq!(actual_first_value, Some(&first_value));
            assert_eq!(actual_second_value, Some(&second_value));
            assert_eq!(actual_third_value, Some(&third_value));
        }

        #[test]
        fn set_with_same_key_twice_overwrites_with_latest_value() {
            // Arrange
            let table_size = 5usize;
            let target_key = 9i32;
            let first_value = 90i32;
            let second_value = 99i32;
            let mut hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            hash_table.set(target_key, first_value);
            hash_table.set(target_key, second_value);
            let actual_value = hash_table.get(&target_key);

            // Assert
            assert_eq!(actual_value, Some(&second_value));
        }

        #[test]
        fn set_updating_existing_key_keeps_other_keys_unchanged() {
            // Arrange
            let table_size = 5usize;
            let update_target_key = 1i32;
            let untouched_key = 2i32;
            let before_update_target_value = 10i32;
            let before_update_untouched_value = 20i32;
            let after_update_target_value = 30i32;
            let mut hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            hash_table.set(update_target_key, before_update_target_value);
            hash_table.set(untouched_key, before_update_untouched_value);
            hash_table.set(update_target_key, after_update_target_value);
            let actual_update_target_value = hash_table.get(&update_target_key);
            let actual_untouched_value = hash_table.get(&untouched_key);

            // Assert
            assert_eq!(actual_update_target_value, Some(&after_update_target_value));
            assert_eq!(actual_untouched_value, Some(&before_update_untouched_value));
        }

        #[test]
        fn set_updating_one_collided_key_keeps_other_collided_key_unchanged() {
            // Arrange
            let table_size = 8usize;
            let collided_hash = 1u64;
            let update_target_key = CollisionKey {
                key_id: 10,
                forced_hash: collided_hash,
            };
            let untouched_key = CollisionKey {
                key_id: 11,
                forced_hash: collided_hash,
            };
            let before_update_target_value = 101i32;
            let before_update_untouched_value = 202i32;
            let after_update_target_value = 303i32;
            let mut hash_table = HashTable::<CollisionKey, i32>::new(table_size);

            // Act
            hash_table.set(update_target_key.clone(), before_update_target_value);
            hash_table.set(untouched_key.clone(), before_update_untouched_value);
            hash_table.set(update_target_key.clone(), after_update_target_value);
            let actual_update_target_value = hash_table.get(&update_target_key);
            let actual_untouched_value = hash_table.get(&untouched_key);

            // Assert
            assert_eq!(actual_update_target_value, Some(&after_update_target_value));
            assert_eq!(actual_untouched_value, Some(&before_update_untouched_value));
        }

        #[test]
        fn get_with_zero_size_table_panics_as_current_behavior() {
            // Arrange
            let table_size = 0usize;
            let lookup_key = 1i32;
            let hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            let actual_panic_result = panic::catch_unwind(|| {
                let _actual_value = hash_table.get(&lookup_key);
            });

            // Assert
            assert!(actual_panic_result.is_err());
        }

        #[test]
        fn get_with_empty_table_returns_none_for_unregistered_key() {
            // Arrange
            let table_size = 3usize;
            let unregistered_key = 10i32;
            let expected_value = None;
            let hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            let actual_value = hash_table.get(&unregistered_key);

            // Assert
            assert_eq!(actual_value, expected_value);
        }

        #[test]
        fn get_with_collided_keys_returns_both_values() {
            // Arrange
            let table_size = 8usize;
            let collided_hash = 0u64;
            let left_key = CollisionKey {
                key_id: 1,
                forced_hash: collided_hash,
            };
            let right_key = CollisionKey {
                key_id: 2,
                forced_hash: collided_hash,
            };
            let left_value = 100i32;
            let right_value = 200i32;
            let mut hash_table = HashTable::<CollisionKey, i32>::new(table_size);

            // Act
            hash_table.set(left_key.clone(), left_value);
            hash_table.set(right_key.clone(), right_value);
            let actual_left_value = hash_table.get(&left_key);
            let actual_right_value = hash_table.get(&right_key);

            // Assert
            assert_eq!(actual_left_value, Some(&left_value));
            assert_eq!(actual_right_value, Some(&right_value));
        }

        #[test]
        fn get_with_unregistered_key_after_multiple_sets_returns_none_and_existing_values_stay_same() {
            // Arrange
            let table_size = 7usize;
            let first_key = 100i32;
            let second_key = 200i32;
            let third_key = 300i32;
            let unregistered_key = 999i32;
            let first_value = 1i32;
            let second_value = 2i32;
            let third_value = 3i32;
            let expected_unregistered_value = None;
            let mut hash_table = HashTable::<i32, i32>::new(table_size);

            // Act
            hash_table.set(first_key, first_value);
            hash_table.set(second_key, second_value);
            hash_table.set(third_key, third_value);
            let actual_unregistered_value = hash_table.get(&unregistered_key);
            let actual_first_value = hash_table.get(&first_key);
            let actual_second_value = hash_table.get(&second_key);
            let actual_third_value = hash_table.get(&third_key);

            // Assert
            assert_eq!(actual_unregistered_value, expected_unregistered_value);
            assert_eq!(actual_first_value, Some(&first_value));
            assert_eq!(actual_second_value, Some(&second_value));
            assert_eq!(actual_third_value, Some(&third_value));
        }
    }
}
