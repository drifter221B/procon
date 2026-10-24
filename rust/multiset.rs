use std::collections::BTreeMap;
use std::fmt;

pub struct Multiset<T: Ord> {
    size: usize,
    btree_map: BTreeMap<T, usize>,
}

pub struct MultisetIter<'a, T: Ord> {
    map_iter: std::collections::btree_map::Iter<'a, T, usize>,
    current_key: Option<&'a T>,
    current_remaining_count: usize,
}

impl<'a, T: Ord> MultisetIter<'a, T> {
    fn new(map_iter: std::collections::btree_map::Iter<'a, T, usize>) -> Self {
        Self {
            map_iter,
            current_key: None,
            current_remaining_count: 0,
        }
    }
}

impl<'a, T: Ord> Iterator for MultisetIter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        if 1 <= self.current_remaining_count {
            self.current_remaining_count -= 1;
            return self.current_key;
        }

        if let Some((key, count)) = self.map_iter.next() {
            self.current_key = Some(key);
            self.current_remaining_count = count.saturating_sub(1);

            return Some(key);
        }

        self.current_key = None;

        None
    }
}

impl<T: Ord> Default for Multiset<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> Multiset<T> {
    pub fn new() -> Self {
        Multiset {
            size: 0,
            btree_map: BTreeMap::new(),
        }
    }

    pub fn from_slice(values: &[T]) -> Self
    where
        T: Clone,
    {
        values.iter().fold(Self::new(), |mut s, v| {
            s.insert(v.clone());

            s
        })
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn iter(&self) -> MultisetIter<'_, T> {
        MultisetIter::new(self.btree_map.iter())
    }

    pub fn iter_mut(&mut self) -> std::collections::btree_map::IterMut<'_, T, usize> {
        self.btree_map.iter_mut()
    }

    pub fn is_empty(&self) -> bool {
        self.btree_map.is_empty()
    }

    pub fn first(&self) -> Option<&T> {
        self.btree_map.first_key_value().map(|entry| entry.0)
    }

    pub fn last(&self) -> Option<&T> {
        self.btree_map.last_key_value().map(|entry| entry.0)
    }

    pub fn pop_first(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let mut min_entry = self.btree_map.first_entry()?;

        if 2 <= *min_entry.get() {
            let popped_value = min_entry.key().clone();
            *min_entry.get_mut() -= 1;
            self.size -= 1;
            Some(popped_value)
        } else {
            self.size -= 1;
            Some(min_entry.remove_entry().0)
        }
    }

    pub fn pop_last(&mut self) -> Option<T>
    where
        T: Clone,
    {
        let mut max_entry = self.btree_map.last_entry()?;

        if 2 <= *max_entry.get() {
            let popped_value = max_entry.key().clone();
            *max_entry.get_mut() -= 1;
            self.size -= 1;
            Some(popped_value)
        } else {
            self.size -= 1;
            Some(max_entry.remove_entry().0)
        }
    }

    pub fn clear(&mut self) {
        self.size = 0;
        self.btree_map.clear();
    }

    pub fn contains(&self, key: &T) -> bool {
        self.btree_map.contains_key(key)
    }

    pub fn count(&self, key: &T) -> usize {
        self.btree_map.get(key).map_or(0, |x| *x)
    }

    pub fn insert(&mut self, key: T) {
        self.size += 1;
        self.btree_map
            .entry(key)
            .and_modify(|x| *x += 1)
            .or_insert(1);
    }

    pub fn insert_multiple(&mut self, key: T, insert_count: usize) {
        if insert_count == 0 {
            return;
        }

        self.size += insert_count;
        self.btree_map
            .entry(key)
            .and_modify(|count| *count += insert_count)
            .or_insert(insert_count);
    }

    pub fn remove(&mut self, key: &T) -> bool {
        self.erase(key, 1) == 1
    }

    pub fn remove_up_to(&mut self, key: &T, delete_count: usize) -> usize {
        self.erase(key, delete_count)
    }

    pub fn remove_all(&mut self, key: &T) -> usize {
        let current_count = self.count(key);
        self.erase(key, current_count)
    }

    fn erase(&mut self, key: &T, delete_count: usize) -> usize {
        if let Some(count) = self.btree_map.get_mut(key) {
            let result = (*count).min(delete_count);
            self.size -= result;

            if *count <= delete_count {
                self.btree_map.remove(key);
            } else {
                *count -= delete_count;
            }

            result
        } else {
            0
        }
    }

    pub fn lower_bound(&self, key: &T) -> Option<&T> {
        self.btree_map.range(key..).next().map(|x| x.0)
    }

    pub fn upper_bound(&self, key: &T) -> Option<&T> {
        self.btree_map.range(..key).next_back().map(|x| x.0)
    }
}

impl<'a, T: Ord> IntoIterator for &'a Multiset<T> {
    type Item = &'a T;
    type IntoIter = MultisetIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a, T: Ord> IntoIterator for &'a mut Multiset<T> {
    type Item = (&'a T, &'a mut usize);
    type IntoIter = std::collections::btree_map::IterMut<'a, T, usize>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

impl<T: Ord + fmt::Debug> fmt::Debug for Multiset<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_list = f.debug_list();

        for value in self.iter() {
            debug_list.entry(value);
        }

        debug_list.finish()
    }
}

#[cfg(test)]
mod tests {

    use super::Multiset;

    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct NonCloneValue(i32);

    fn create_multiset_for_bound_tests() -> Multiset<i32> {
        let mut ms = Multiset::new();
        ms.insert(1);
        ms.insert(1);
        ms.insert(1);
        ms.insert(3);
        ms
    }

    #[test]
    pub fn default_creates_same_initial_state_as_new() {
        // Arrange
        let default_ms: Multiset<i32> = Multiset::default();
        let new_ms: Multiset<i32> = Multiset::new();

        // Act
        let actual_default_len = default_ms.len();
        let actual_default_is_empty = default_ms.is_empty();
        let actual_default_min = default_ms.first();
        let actual_default_max = default_ms.last();
        let expected_len = new_ms.len();
        let expected_is_empty = new_ms.is_empty();
        let expected_min = new_ms.first();
        let expected_max = new_ms.last();

        // Assert
        assert_eq!(actual_default_len, expected_len);
        assert_eq!(actual_default_is_empty, expected_is_empty);
        assert_eq!(actual_default_min, expected_min);
        assert_eq!(actual_default_max, expected_max);
    }

    #[test]
    fn len_returns_zero_when_multiset_is_new() {
        // Arrange
        let ms: Multiset<i32> = Multiset::new();
        let expected_len = 0usize;

        // Act
        let actual_len = ms.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    pub fn is_empty_returns_true_when_multiset_is_new() {
        // Arrange
        let ms: Multiset<i32> = Multiset::new();
        let expected_is_empty = true;

        // Act
        let actual_is_empty = ms.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn from_slice_returns_empty_multiset_when_input_is_empty() {
        // Arrange
        let values: [i32; 0] = [];
        let expected_len = 0usize;
        let expected_is_empty = true;

        // Act
        let ms = Multiset::from_slice(&values);
        let actual_len = ms.len();
        let actual_is_empty = ms.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn from_slice_sets_count_to_one_when_input_has_single_value() {
        // Arrange
        let single_value = 10;
        let values = [single_value];
        let expected_len = 1usize;
        let expected_count = 1usize;

        // Act
        let ms = Multiset::from_slice(&values);
        let actual_len = ms.len();
        let actual_count = ms.count(&single_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn from_slice_counts_duplicates_when_input_has_multiple_values() {
        // Arrange
        let boundary_base_value = 1;
        let lower_boundary_value = boundary_base_value - 1;
        let boundary_value = boundary_base_value;
        let upper_boundary_value = boundary_base_value + 1;
        let values = [
            lower_boundary_value,
            boundary_value,
            boundary_value,
            upper_boundary_value,
        ];
        let expected_len = 4usize;
        let expected_lower_count = 1usize;
        let expected_boundary_count = 2usize;
        let expected_upper_count = 1usize;

        // Act
        let ms = Multiset::from_slice(&values);
        let actual_len = ms.len();
        let actual_lower_count = ms.count(&lower_boundary_value);
        let actual_boundary_count = ms.count(&boundary_value);
        let actual_upper_count = ms.count(&upper_boundary_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_boundary_count, expected_boundary_count);
        assert_eq!(actual_upper_count, expected_upper_count);
    }

    #[test]
    pub fn from_slice_collects_counts_and_returns_sorted_iteration_for_unsorted_input() {
        // Arrange
        let lower_value = 1;
        let middle_value = 2;
        let upper_value = 3;
        let values = [upper_value, lower_value, middle_value, lower_value];
        let expected_lower_count = 2usize;
        let expected_middle_count = 1usize;
        let expected_upper_count = 1usize;
        let expected_sorted_values = vec![lower_value, lower_value, middle_value, upper_value];

        // Act
        let ms = Multiset::from_slice(&values);
        let actual_lower_count = ms.count(&lower_value);
        let actual_middle_count = ms.count(&middle_value);
        let actual_upper_count = ms.count(&upper_value);
        let actual_sorted_values = ms.iter().copied().collect::<Vec<i32>>();

        // Assert
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_middle_count, expected_middle_count);
        assert_eq!(actual_upper_count, expected_upper_count);
        assert_eq!(actual_sorted_values, expected_sorted_values);
    }

    #[test]
    pub fn from_slice_with_string_type_updates_counts_and_iterates_in_sorted_order() {
        // Arrange
        let lower_key = String::from("apple");
        let upper_key = String::from("banana");
        let values = [upper_key.clone(), lower_key.clone(), upper_key.clone()];
        let expected_upper_count = 2usize;
        let expected_sorted_values = vec![lower_key.clone(), upper_key.clone(), upper_key.clone()];

        // Act
        let ms = Multiset::from_slice(&values);
        let actual_upper_count = ms.count(&upper_key);
        let actual_sorted_values = ms.iter().cloned().collect::<Vec<String>>();

        // Assert
        assert_eq!(actual_upper_count, expected_upper_count);
        assert_eq!(actual_sorted_values, expected_sorted_values);
    }

    #[test]
    fn len_returns_one_when_single_value_is_inserted() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 1;
        let expected_len = 1usize;

        // Act
        ms.insert(inserted_value);
        let actual_len = ms.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    pub fn iter_next_returns_none_repeatedly_after_reaching_iterator_end() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_key = 1;
        let insert_count = 2usize;
        let expected_first_none = None;
        let expected_second_none = None;
        ms.insert_multiple(inserted_key, insert_count);

        // Act
        let mut iter = ms.iter();
        let _ = iter.next();
        let _ = iter.next();
        let actual_first_none = iter.next();
        let actual_second_none = iter.next();

        // Assert
        assert_eq!(actual_first_none, expected_first_none);
        assert_eq!(actual_second_none, expected_second_none);
    }

    #[test]
    pub fn iter_mut_count_update_changes_count_but_len_remains_original() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let upper_key = 2;
        let updated_lower_count = 3usize;
        let expected_lower_count = updated_lower_count;
        let expected_len = 2usize;
        let expected_iter_count = updated_lower_count + 1;
        ms.insert(lower_key);
        ms.insert(upper_key);

        // Act
        if let Some((_key, count)) = ms.iter_mut().next() {
            *count = updated_lower_count;
        }
        let actual_lower_count = ms.count(&lower_key);
        let actual_len = ms.len();
        let actual_iter_count = ms.iter().count();

        // Assert
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_iter_count, expected_iter_count);
    }

    #[test]
    pub fn iter_returns_none_when_empty() {
        // Arrange
        let ms: Multiset<i32> = Multiset::new();
        let expected_first_value = None;

        // Act
        let actual_first_value = ms.iter().next();

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
    }

    #[test]
    pub fn iter_returns_single_value_for_single_element() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 10;
        let expected_values = vec![inserted_value];
        ms.insert(inserted_value);

        // Act
        let actual_values = ms.iter().copied().collect::<Vec<i32>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    pub fn iter_returns_sorted_values_with_duplicates_and_boundary_values() {
        // Arrange
        let mut ms = Multiset::new();
        let boundary_base_value = 10;
        let lower_boundary_value = boundary_base_value - 1;
        let boundary_value = boundary_base_value;
        let upper_boundary_value = boundary_base_value + 1;
        let expected_values = vec![
            lower_boundary_value,
            boundary_value,
            boundary_value,
            upper_boundary_value,
            upper_boundary_value,
        ];
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);
        ms.insert(lower_boundary_value);
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);

        // Act
        let actual_values = ms.iter().copied().collect::<Vec<i32>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    pub fn iter_mut_returns_none_when_empty() {
        // Arrange
        let mut ms: Multiset<i32> = Multiset::new();
        let expected_first_item = None;

        // Act
        let actual_first_item = ms.iter_mut().next().map(|(key, count)| (*key, *count));

        // Assert
        assert_eq!(actual_first_item, expected_first_item);
    }

    #[test]
    pub fn iter_mut_returns_single_entry_for_single_element_set() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 10;
        let expected_entries = vec![(inserted_value, 1usize)];
        ms.insert(inserted_value);

        // Act
        let actual_entries = ms
            .iter_mut()
            .map(|(key, count)| (*key, *count))
            .collect::<Vec<(i32, usize)>>();

        // Assert
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    pub fn iter_mut_returns_sorted_entries_with_boundary_values() {
        // Arrange
        let mut ms = Multiset::new();
        let boundary_base_value = 10;
        let lower_boundary_value = boundary_base_value - 1;
        let boundary_value = boundary_base_value;
        let upper_boundary_value = boundary_base_value + 1;
        let expected_entries = vec![
            (lower_boundary_value, 1usize),
            (boundary_value, 2usize),
            (upper_boundary_value, 2usize),
        ];
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);
        ms.insert(lower_boundary_value);
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);

        // Act
        let actual_entries = ms
            .iter_mut()
            .map(|(key, count)| (*key, *count))
            .collect::<Vec<(i32, usize)>>();

        // Assert
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    pub fn is_empty_returns_false_when_multiset_has_element() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 1;
        let expected_is_empty = false;
        ms.insert(inserted_value);

        // Act
        let actual_is_empty = ms.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    pub fn first_returns_none_when_multiset_is_empty() {
        // Arrange
        let ms: Multiset<i32> = Multiset::new();
        let expected_min = None;

        // Act
        let actual_min = ms.first();

        // Assert
        assert_eq!(actual_min, expected_min);
    }

    #[test]
    pub fn first_returns_smallest_value_when_multiset_has_elements() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let expected_min = Some(&lower_value);
        ms.insert(lower_value);
        ms.insert(upper_value);

        // Act
        let actual_min = ms.first();

        // Assert
        assert_eq!(actual_min, expected_min);
    }

    #[test]
    pub fn first_and_last_work_without_clone_bound_on_container_type() {
        // Arrange
        let inserted_lower_value = NonCloneValue(1);
        let inserted_upper_value = NonCloneValue(2);
        let expected_first_value = NonCloneValue(1);
        let expected_last_value = NonCloneValue(2);
        let expected_len = 2usize;
        let expected_first = Some(&expected_first_value);
        let expected_last = Some(&expected_last_value);
        let mut ms: Multiset<NonCloneValue> = Multiset::new();

        // Act
        ms.insert(inserted_upper_value);
        ms.insert(inserted_lower_value);
        let actual_len = ms.len();
        let actual_first = ms.first();
        let actual_last = ms.last();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_last, expected_last);
    }

    #[test]
    pub fn last_returns_none_when_multiset_is_empty() {
        // Arrange
        let ms: Multiset<i32> = Multiset::new();
        let expected_max = None;

        // Act
        let actual_max = ms.last();

        // Assert
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    pub fn last_returns_largest_value_when_multiset_has_elements() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let expected_max = Some(&upper_value);
        ms.insert(lower_value);
        ms.insert(upper_value);

        // Act
        let actual_max = ms.last();

        // Assert
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    pub fn pop_first_updates_minimum_to_next_key_when_previous_min_count_is_one() {
        // Arrange
        let mut ms = Multiset::new();
        let previous_min_key = 1;
        let next_min_key = 2;
        let next_min_count = 2usize;
        let expected_popped_value = Some(previous_min_key);
        let expected_min_after_pop = Some(&next_min_key);
        ms.insert(previous_min_key);
        ms.insert_multiple(next_min_key, next_min_count);

        // Act
        let actual_popped_value = ms.pop_first();
        let actual_min_after_pop = ms.first();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_min_after_pop, expected_min_after_pop);
    }

    #[test]
    pub fn pop_first_decrements_len_by_one_when_value_is_removed() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let expected_len_after_pop = 1usize;
        ms.insert(lower_value);
        ms.insert(upper_value);

        // Act
        let _ = ms.pop_first();
        let actual_len_after_pop = ms.len();

        // Assert
        assert_eq!(actual_len_after_pop, expected_len_after_pop);
    }

    #[test]
    pub fn pop_first_returns_none_when_multiset_is_empty() {
        // Arrange
        let mut ms: Multiset<i32> = Multiset::new();
        let expected_result = None;

        // Act
        let actual_result = ms.pop_first();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn pop_first_returns_smallest_value_when_multiset_has_elements() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let expected_popped_value = Some(lower_value);
        ms.insert(lower_value);
        ms.insert(upper_value);

        // Act
        let actual_popped_value = ms.pop_first();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
    }

    #[test]
    pub fn pop_first_decrements_minimum_count_when_minimum_count_is_two() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let lower_value_count = 2usize;
        let upper_value = 2;
        let expected_popped_value = Some(lower_value);
        let expected_lower_value_count = lower_value_count - 1;
        let expected_len_after_pop = 2usize;
        ms.insert_multiple(lower_value, lower_value_count);
        ms.insert(upper_value);

        // Act
        let actual_popped_value = ms.pop_first();
        let actual_lower_value_count = ms.count(&lower_value);
        let actual_len_after_pop = ms.len();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_lower_value_count, expected_lower_value_count);
        assert_eq!(actual_len_after_pop, expected_len_after_pop);
    }

    #[test]
    pub fn pop_first_single_element_set_returns_element_and_becomes_empty() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_key = 10;
        let expected_popped_value = Some(inserted_key);
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_min = None;
        let expected_max = None;
        ms.insert(inserted_key);

        // Act
        let actual_popped_value = ms.pop_first();
        let actual_len = ms.len();
        let actual_is_empty = ms.is_empty();
        let actual_min = ms.first();
        let actual_max = ms.last();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    pub fn pop_first_and_pop_last_with_string_type_follow_lexicographic_order() {
        // Arrange
        let lower_key = String::from("apple");
        let middle_key = String::from("banana");
        let upper_key = String::from("cherry");
        let expected_min_value = Some(lower_key.clone());
        let expected_max_value = Some(upper_key.clone());
        let expected_remaining_values = vec![middle_key.clone()];
        let mut ms = Multiset::new();
        ms.insert(middle_key.clone());
        ms.insert(upper_key.clone());
        ms.insert(lower_key.clone());

        // Act
        let actual_min_value = ms.pop_first();
        let actual_max_value = ms.pop_last();
        let actual_remaining_values = ms.iter().cloned().collect::<Vec<String>>();

        // Assert
        assert_eq!(actual_min_value, expected_min_value);
        assert_eq!(actual_max_value, expected_max_value);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    pub fn pop_last_decrements_len_by_one_when_value_is_removed() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let expected_len_after_pop = 1usize;
        ms.insert(lower_value);
        ms.insert(upper_value);

        // Act
        let _ = ms.pop_last();
        let actual_len_after_pop = ms.len();

        // Assert
        assert_eq!(actual_len_after_pop, expected_len_after_pop);
    }

    #[test]
    pub fn pop_last_returns_none_when_multiset_is_empty() {
        // Arrange
        let mut ms: Multiset<i32> = Multiset::new();
        let expected_result = None;

        // Act
        let actual_result = ms.pop_last();

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn pop_last_returns_largest_value_when_multiset_has_elements() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let expected_popped_value = Some(upper_value);
        ms.insert(lower_value);
        ms.insert(upper_value);

        // Act
        let actual_popped_value = ms.pop_last();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
    }

    #[test]
    pub fn pop_last_decrements_maximum_count_when_maximum_count_is_two() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_value = 1;
        let upper_value = 2;
        let upper_value_count = 2usize;
        let expected_popped_value = Some(upper_value);
        let expected_upper_value_count = upper_value_count - 1;
        let expected_len_after_pop = 2usize;
        ms.insert(lower_value);
        ms.insert_multiple(upper_value, upper_value_count);

        // Act
        let actual_popped_value = ms.pop_last();
        let actual_upper_value_count = ms.count(&upper_value);
        let actual_len_after_pop = ms.len();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_upper_value_count, expected_upper_value_count);
        assert_eq!(actual_len_after_pop, expected_len_after_pop);
    }

    #[test]
    pub fn pop_last_single_element_set_returns_element_and_becomes_empty() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_key = 10;
        let expected_popped_value = Some(inserted_key);
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_min = None;
        let expected_max = None;
        ms.insert(inserted_key);

        // Act
        let actual_popped_value = ms.pop_last();
        let actual_len = ms.len();
        let actual_is_empty = ms.is_empty();
        let actual_min = ms.first();
        let actual_max = ms.last();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    pub fn pop_last_updates_maximum_to_previous_key_when_previous_max_count_is_one() {
        // Arrange
        let mut ms = Multiset::new();
        let previous_max_key = 3;
        let next_max_key = 1;
        let next_max_count = 2usize;
        let expected_popped_value = Some(previous_max_key);
        let expected_max_after_pop = Some(&next_max_key);
        ms.insert_multiple(next_max_key, next_max_count);
        ms.insert(previous_max_key);

        // Act
        let actual_popped_value = ms.pop_last();
        let actual_max_after_pop = ms.last();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_max_after_pop, expected_max_after_pop);
    }

    #[test]
    pub fn clear_sets_len_to_zero_when_multiset_has_elements() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 1;
        let inserted_count = 3usize;
        let expected_len_after_clear = 0usize;
        ms.insert_multiple(inserted_value, inserted_count);

        // Act
        ms.clear();
        let actual_len_after_clear = ms.len();

        // Assert
        assert_eq!(actual_len_after_clear, expected_len_after_clear);
    }

    #[test]
    pub fn clear_keeps_empty_state_when_multiset_is_already_empty() {
        // Arrange
        let mut ms: Multiset<i32> = Multiset::new();
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_min = None;
        let expected_max = None;

        // Act
        ms.clear();
        let actual_len = ms.len();
        let actual_is_empty = ms.is_empty();
        let actual_min = ms.first();
        let actual_max = ms.last();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    pub fn clear_removes_all_lookup_results_when_multiset_has_elements() {
        // Arrange
        let mut ms = Multiset::new();
        let target_key = 1;
        let other_key = 2;
        let insert_count = 2usize;
        let expected_is_empty = true;
        let expected_contains_target = false;
        let expected_contains_other = false;
        let expected_min = None;
        let expected_max = None;
        ms.insert_multiple(target_key, insert_count);
        ms.insert(other_key);

        // Act
        ms.clear();
        let actual_is_empty = ms.is_empty();
        let actual_contains_target = ms.contains(&target_key);
        let actual_contains_other = ms.contains(&other_key);
        let actual_min = ms.first();
        let actual_max = ms.last();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_contains_target, expected_contains_target);
        assert_eq!(actual_contains_other, expected_contains_other);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    pub fn contains_returns_true_when_target_key_exists() {
        // Arrange
        let mut ms = Multiset::new();
        let existing_value = 1;
        let expected_contains = true;
        ms.insert(existing_value);
        ms.insert(2);

        // Act
        let actual_contains = ms.contains(&existing_value);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    pub fn contains_returns_false_when_target_key_is_absent() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 1;
        let absent_value = 42;
        let expected_contains = false;
        ms.insert(inserted_value);
        ms.insert(2);

        // Act
        let actual_contains = ms.contains(&absent_value);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    pub fn count_returns_one_when_target_key_appears_once() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let expected_count = 1usize;
        ms.insert(target_value);
        ms.insert(42);
        ms.insert(42);

        // Act
        let actual_count = ms.count(&target_value);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn count_returns_two_when_target_key_appears_twice() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 42;
        let expected_count = 2usize;
        ms.insert(1);
        ms.insert(target_value);
        ms.insert(target_value);

        // Act
        let actual_count = ms.count(&target_value);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn count_returns_zero_when_target_key_is_absent() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 0;
        let expected_count = 0usize;
        ms.insert(1);
        ms.insert(42);
        ms.insert(42);

        // Act
        let actual_count = ms.count(&target_value);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn insert_increments_len_when_value_is_added() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 1;
        let expected_len = 1usize;

        // Act
        ms.insert(inserted_value);
        let actual_len = ms.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    pub fn insert_increments_count_when_same_value_is_added_twice() {
        // Arrange
        let mut ms = Multiset::new();
        let inserted_value = 1;
        let expected_count = 2usize;
        ms.insert(inserted_value);

        // Act
        ms.insert(inserted_value);
        let actual_count = ms.count(&inserted_value);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn insert_multiple_preserves_state_when_insert_count_is_zero() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let existing_count = 2usize;
        let insert_count = 0usize;
        ms.insert_multiple(target_value, existing_count);
        let expected_len = existing_count;
        let expected_count = existing_count;

        // Act
        ms.insert_multiple(target_value, insert_count);
        let actual_len = ms.len();
        let actual_count = ms.count(&target_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn insert_multiple_adds_one_element_when_insert_count_is_one() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let insert_count = 1usize;
        let expected_len = 1usize;
        let expected_count = 1usize;

        // Act
        ms.insert_multiple(target_value, insert_count);
        let actual_len = ms.len();
        let actual_count = ms.count(&target_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn insert_multiple_adds_two_elements_when_insert_count_is_two() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let insert_count = 2usize;
        let expected_len = 2usize;
        let expected_count = 2usize;

        // Act
        ms.insert_multiple(target_value, insert_count);
        let actual_len = ms.len();
        let actual_count = ms.count(&target_value);

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    pub fn insert_multiple_accumulates_count_when_target_key_already_exists() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let initial_insert_count = 2usize;
        let additional_insert_count = 3usize;
        let expected_total_count = initial_insert_count + additional_insert_count;
        ms.insert_multiple(target_value, initial_insert_count);

        // Act
        ms.insert_multiple(target_value, additional_insert_count);
        let actual_total_count = ms.count(&target_value);
        let actual_total_len = ms.len();

        // Assert
        assert_eq!(actual_total_count, expected_total_count);
        assert_eq!(actual_total_len, expected_total_count);
    }

    #[test]
    pub fn remove_up_to_removes_x_minus_1_count_when_delete_count_is_one() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let existing_count = 2usize;
        let delete_count = existing_count - 1;
        let expected_removed_count = delete_count;
        let expected_remaining_count = 1usize;
        let expected_remaining_len = 1usize;
        ms.insert_multiple(target_value, existing_count);

        // Act
        let actual_removed_count = ms.remove_up_to(&target_value, delete_count);
        let actual_remaining_count = ms.count(&target_value);
        let actual_remaining_len = ms.len();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_remaining_len, expected_remaining_len);
    }

    #[test]
    pub fn remove_up_to_removes_x_count_when_delete_count_equals_existing_count() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let existing_count = 2usize;
        let delete_count = existing_count;
        let expected_removed_count = existing_count;
        let expected_remaining_count = 0usize;
        let expected_remaining_len = 0usize;
        ms.insert_multiple(target_value, existing_count);

        // Act
        let actual_removed_count = ms.remove_up_to(&target_value, delete_count);
        let actual_remaining_count = ms.count(&target_value);
        let actual_remaining_len = ms.len();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_remaining_len, expected_remaining_len);
    }

    #[test]
    pub fn remove_up_to_removes_x_count_when_delete_count_is_x_plus_1() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let existing_count = 2usize;
        let delete_count = existing_count + 1;
        let expected_removed_count = existing_count;
        let expected_remaining_count = 0usize;
        let expected_remaining_len = 0usize;
        ms.insert_multiple(target_value, existing_count);

        // Act
        let actual_removed_count = ms.remove_up_to(&target_value, delete_count);
        let actual_remaining_count = ms.count(&target_value);
        let actual_remaining_len = ms.len();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_remaining_len, expected_remaining_len);
    }

    #[test]
    pub fn remove_up_to_preserves_state_when_delete_count_is_zero() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let delete_count = 0usize;
        let expected_removed_count = 0usize;
        ms.insert(target_value);
        ms.insert(target_value);
        let expected_len_after_remove = ms.len();
        let expected_count_after_remove = ms.count(&target_value);

        // Act
        let actual_removed_count = ms.remove_up_to(&target_value, delete_count);
        let actual_len_after_remove = ms.len();
        let actual_count_after_remove = ms.count(&target_value);

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len_after_remove, expected_len_after_remove);
        assert_eq!(actual_count_after_remove, expected_count_after_remove);
    }

    #[test]
    pub fn remove_all_removes_one_element_when_target_count_is_one() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let existing_count = 1usize;
        let expected_removed_count = existing_count;
        let expected_remaining_count = 0usize;
        let expected_remaining_len = 0usize;
        ms.insert_multiple(target_value, existing_count);

        // Act
        let actual_removed_count = ms.remove_all(&target_value);
        let actual_remaining_count = ms.count(&target_value);
        let actual_remaining_len = ms.len();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_remaining_len, expected_remaining_len);
    }

    #[test]
    pub fn remove_all_removes_all_elements_when_target_count_is_multiple() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let existing_count = 3usize;
        let expected_removed_count = existing_count;
        let expected_remaining_count = 0usize;
        let expected_remaining_len = 0usize;
        ms.insert_multiple(target_value, existing_count);

        // Act
        let actual_removed_count = ms.remove_all(&target_value);
        let actual_remaining_count = ms.count(&target_value);
        let actual_remaining_len = ms.len();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_remaining_len, expected_remaining_len);
    }

    #[test]
    pub fn remove_deletes_key_when_target_count_is_one() {
        // Arrange
        let mut ms = Multiset::new();
        let target_key = 2;
        let retained_key = 1;
        let expected_removed = true;
        let expected_target_contains = false;
        let expected_target_count = 0usize;
        let expected_retained_count = 1usize;
        ms.insert(target_key);
        ms.insert(retained_key);

        // Act
        let actual_removed = ms.remove(&target_key);
        let actual_target_contains = ms.contains(&target_key);
        let actual_target_count = ms.count(&target_key);
        let actual_retained_count = ms.count(&retained_key);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_target_contains, expected_target_contains);
        assert_eq!(actual_target_count, expected_target_count);
        assert_eq!(actual_retained_count, expected_retained_count);
    }

    #[test]
    pub fn remove_returns_true_when_key_exists_and_removes_only_one_element() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let expected_removed = true;
        let expected_count_after_remove = 2usize;
        let expected_len_after_remove = 2usize;
        ms.insert(target_value);
        ms.insert(target_value);
        ms.insert(target_value);

        // Act
        let actual_removed = ms.remove(&target_value);
        let actual_count_after_remove = ms.count(&target_value);
        let actual_len_after_remove = ms.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count_after_remove, expected_count_after_remove);
        assert_eq!(actual_len_after_remove, expected_len_after_remove);
    }

    #[test]
    pub fn remove_all_returns_zero_when_target_key_is_absent() {
        // Arrange
        let mut ms = Multiset::new();
        let target_value = 1;
        let expected_removed_count = 0usize;
        let expected_remaining_count = 0usize;
        let expected_remaining_len = 0usize;

        // Act
        let actual_removed_count = ms.remove_all(&target_value);
        let actual_remaining_count = ms.count(&target_value);
        let actual_remaining_len = ms.len();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_remaining_count, expected_remaining_count);
        assert_eq!(actual_remaining_len, expected_remaining_len);
    }

    #[test]
    pub fn remove_returns_false_and_preserves_other_entries_when_target_key_is_absent() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let upper_key = 3;
        let absent_key = 2;
        let lower_count = 2usize;
        let upper_count = 1usize;
        let expected_removed = false;
        let expected_len = lower_count + upper_count;
        let expected_lower_count = lower_count;
        let expected_upper_count = upper_count;
        ms.insert_multiple(lower_key, lower_count);
        ms.insert_multiple(upper_key, upper_count);

        // Act
        let actual_removed = ms.remove(&absent_key);
        let actual_len = ms.len();
        let actual_lower_count = ms.count(&lower_key);
        let actual_upper_count = ms.count(&upper_key);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_upper_count, expected_upper_count);
    }

    #[test]
    pub fn remove_up_to_returns_zero_and_preserves_state_when_target_key_is_absent() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let upper_key = 3;
        let absent_key = 2;
        let delete_count = 2usize;
        let lower_count = 2usize;
        let upper_count = 1usize;
        let expected_removed_count = 0usize;
        let expected_len = lower_count + upper_count;
        let expected_lower_count = lower_count;
        let expected_upper_count = upper_count;
        ms.insert_multiple(lower_key, lower_count);
        ms.insert_multiple(upper_key, upper_count);

        // Act
        let actual_removed_count = ms.remove_up_to(&absent_key, delete_count);
        let actual_len = ms.len();
        let actual_lower_count = ms.count(&lower_key);
        let actual_upper_count = ms.count(&upper_key);

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_upper_count, expected_upper_count);
    }

    #[test]
    pub fn remove_up_to_updates_only_target_key_when_multiple_keys_exist() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let target_key = 2;
        let upper_key = 3;
        let lower_count = 1usize;
        let target_count = 3usize;
        let upper_count = 1usize;
        let delete_count = 2usize;
        let expected_removed_count = delete_count;
        let expected_lower_count = lower_count;
        let expected_target_count = target_count - delete_count;
        let expected_upper_count = upper_count;
        let expected_entries = vec![
            (lower_key, expected_lower_count),
            (target_key, expected_target_count),
            (upper_key, expected_upper_count),
        ];
        ms.insert_multiple(lower_key, lower_count);
        ms.insert_multiple(target_key, target_count);
        ms.insert_multiple(upper_key, upper_count);

        // Act
        let actual_removed_count = ms.remove_up_to(&target_key, delete_count);
        let actual_lower_count = ms.count(&lower_key);
        let actual_target_count = ms.count(&target_key);
        let actual_upper_count = ms.count(&upper_key);
        let actual_entries = ms
            .iter_mut()
            .map(|(key, count)| (*key, *count))
            .collect::<Vec<(i32, usize)>>();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_target_count, expected_target_count);
        assert_eq!(actual_upper_count, expected_upper_count);
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    pub fn remove_all_returns_zero_and_preserves_existing_keys_when_target_key_is_absent() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let upper_key = 3;
        let absent_key = 2;
        let lower_count = 2usize;
        let upper_count = 1usize;
        let expected_removed_count = 0usize;
        let expected_len = lower_count + upper_count;
        let expected_entries = vec![(lower_key, lower_count), (upper_key, upper_count)];
        ms.insert_multiple(lower_key, lower_count);
        ms.insert_multiple(upper_key, upper_count);

        // Act
        let actual_removed_count = ms.remove_all(&absent_key);
        let actual_len = ms.len();
        let actual_entries = ms
            .iter_mut()
            .map(|(key, count)| (*key, *count))
            .collect::<Vec<(i32, usize)>>();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    pub fn remove_all_removes_only_target_key_and_keeps_other_keys() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let target_key = 2;
        let upper_key = 3;
        let lower_count = 2usize;
        let target_count = 3usize;
        let upper_count = 1usize;
        let expected_removed_count = target_count;
        let expected_target_count = 0usize;
        let expected_lower_count = lower_count;
        let expected_upper_count = upper_count;
        let expected_entries = vec![(lower_key, lower_count), (upper_key, upper_count)];
        let expected_len = lower_count + upper_count;
        ms.insert_multiple(lower_key, lower_count);
        ms.insert_multiple(target_key, target_count);
        ms.insert_multiple(upper_key, upper_count);

        // Act
        let actual_removed_count = ms.remove_all(&target_key);
        let actual_target_count = ms.count(&target_key);
        let actual_lower_count = ms.count(&lower_key);
        let actual_upper_count = ms.count(&upper_key);
        let actual_len = ms.len();
        let actual_entries = ms
            .iter_mut()
            .map(|(key, count)| (*key, *count))
            .collect::<Vec<(i32, usize)>>();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_target_count, expected_target_count);
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_upper_count, expected_upper_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    pub fn lower_bound_returns_next_greater_element_when_key_is_between_values() {
        // Arrange
        let ms = create_multiset_for_bound_tests();
        let target_key = 2;
        let expected_result = Some(&3);

        // Act
        let actual_result = ms.lower_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn lower_bound_returns_none_when_multiset_is_empty() {
        // Arrange
        let ms = Multiset::new();
        let target_key = 1;
        let expected_result = None;

        // Act
        let actual_result = ms.lower_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn lower_bound_returns_none_when_key_is_greater_than_all_elements() {
        // Arrange
        let ms = create_multiset_for_bound_tests();
        let target_key = 4;
        let expected_result = None;

        // Act
        let actual_result = ms.lower_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn lower_bound_returns_equal_element_when_key_exists() {
        // Arrange
        let ms = create_multiset_for_bound_tests();
        let target_key = 3;
        let expected_result = Some(&3);

        // Act
        let actual_result = ms.lower_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn lower_bound_returns_minimum_key_when_target_is_x_minus_1_from_minimum() {
        // Arrange
        let mut ms = Multiset::new();
        let minimum_key = 2;
        let upper_key = 4;
        let target_key = minimum_key - 1;
        let expected_result = Some(&minimum_key);
        ms.insert(minimum_key);
        ms.insert(upper_key);

        // Act
        let actual_result = ms.lower_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn upper_bound_returns_none_when_multiset_is_empty() {
        // Arrange
        let ms = Multiset::new();
        let target_key = 1;
        let expected_result = None;

        // Act
        let actual_result = ms.upper_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn upper_bound_returns_none_when_key_is_equal_to_minimum_element() {
        // Arrange
        let ms = create_multiset_for_bound_tests();
        let target_key = 1;
        let expected_result = None;

        // Act
        let actual_result = ms.upper_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn upper_bound_returns_previous_smaller_element_when_key_is_between_values() {
        // Arrange
        let ms = create_multiset_for_bound_tests();
        let target_key = 2;
        let expected_result = Some(&1);

        // Act
        let actual_result = ms.upper_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn upper_bound_returns_maximum_element_when_key_is_greater_than_all_elements() {
        // Arrange
        let ms = create_multiset_for_bound_tests();
        let target_key = 4;
        let expected_result = Some(&3);

        // Act
        let actual_result = ms.upper_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn upper_bound_returns_none_when_target_is_x_minus_1_from_minimum() {
        // Arrange
        let mut ms = Multiset::new();
        let minimum_key = 2;
        let upper_key = 4;
        let target_key = minimum_key - 1;
        let expected_result = None;
        ms.insert(minimum_key);
        ms.insert(upper_key);

        // Act
        let actual_result = ms.upper_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn upper_bound_returns_x_minus_1_key_when_target_key_is_existing_non_minimum_x() {
        // Arrange
        let mut ms = Multiset::new();
        let lower_key = 1;
        let target_key = 2;
        let upper_key = 3;
        let expected_result = Some(&lower_key);
        ms.insert(lower_key);
        ms.insert(target_key);
        ms.insert(upper_key);

        // Act
        let actual_result = ms.upper_bound(&target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    pub fn into_iterator_for_reference_returns_sorted_values() {
        // Arrange
        let mut ms = Multiset::new();
        let boundary_base_value = 5;
        let lower_boundary_value = boundary_base_value - 1;
        let boundary_value = boundary_base_value;
        let upper_boundary_value = boundary_base_value + 1;
        let expected_values = vec![lower_boundary_value, boundary_value, upper_boundary_value];
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);
        ms.insert(lower_boundary_value);

        // Act
        let actual_values = (&ms).into_iter().copied().collect::<Vec<i32>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    pub fn into_iterator_for_mutable_reference_returns_sorted_entries() {
        // Arrange
        let mut ms = Multiset::new();
        let boundary_base_value = 5;
        let lower_boundary_value = boundary_base_value - 1;
        let boundary_value = boundary_base_value;
        let upper_boundary_value = boundary_base_value + 1;
        let expected_entries = vec![
            (lower_boundary_value, 1usize),
            (boundary_value, 1usize),
            (upper_boundary_value, 1usize),
        ];
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);
        ms.insert(lower_boundary_value);

        // Act
        let actual_entries = (&mut ms)
            .into_iter()
            .map(|(key, count)| (*key, *count))
            .collect::<Vec<(i32, usize)>>();

        // Assert
        assert_eq!(actual_entries, expected_entries);
    }

    #[test]
    pub fn debug_formats_empty_multiset_as_empty_list() {
        // Arrange
        let ms: Multiset<i32> = Multiset::new();
        let expected_debug_output = "[]";

        // Act
        let actual_debug_output = format!("{:?}", ms);

        // Assert
        assert_eq!(actual_debug_output, expected_debug_output);
    }

    #[test]
    pub fn debug_formats_sorted_values_with_duplicates_and_boundary_values() {
        // Arrange
        let mut ms = Multiset::new();
        let boundary_base_value = 0;
        let lower_boundary_value = boundary_base_value - 1;
        let boundary_value = boundary_base_value;
        let upper_boundary_value = boundary_base_value + 1;
        let expected_debug_output = format!(
            "[{}, {}, {}, {}]",
            lower_boundary_value, boundary_value, boundary_value, upper_boundary_value,
        );
        ms.insert(upper_boundary_value);
        ms.insert(boundary_value);
        ms.insert(boundary_value);
        ms.insert(lower_boundary_value);

        // Act
        let actual_debug_output = format!("{:?}", ms);

        // Assert
        assert_eq!(actual_debug_output, expected_debug_output);
    }
}
