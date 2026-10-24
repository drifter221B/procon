use std::cell::RefCell;
use std::cmp;
use std::fmt::Debug;
use std::rc::Rc;
use std::time::{SystemTime, UNIX_EPOCH};

const DEFAULT_MAX_LEVEL: usize = 16;

#[derive(Debug)]
struct SkipNode<T: Ord + Copy + Default + Debug> {
    key: T,
    count: usize,
    forward: Vec<Option<Rc<RefCell<SkipNode<T>>>>>,
}

impl<T: Ord + Copy + Default + Debug> SkipNode<T> {
    fn new(key: T, level: usize) -> Self {
        Self {
            key,
            count: 1,
            forward: vec![None; level + 1],
        }
    }
}

#[derive(Clone)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let initial_state = if seed == 0 { 0x4d595df4d0f33173 } else { seed };
        Self { state: initial_state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 7;
        x ^= x >> 9;
        x ^= x << 8;
        self.state = x;
        x
    }

    fn next_bit_is_zero(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }
}

pub struct SkipList<T: Ord + Copy + Default + Debug> {
    head: Rc<RefCell<SkipNode<T>>>,
    max_level: usize,
    current_level: usize,
    length: usize,
    random: XorShift64,
}

impl<T: Ord + Copy + Default + Debug> Default for SkipList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Copy + Default + Debug> SkipList<T> {
    pub fn new() -> Self {
        let seed = Self::seed_from_time();
        Self::with_seed(DEFAULT_MAX_LEVEL, seed)
    }

    pub fn with_seed(max_level: usize, seed: u64) -> Self {
        let level_cap = cmp::max(1, max_level);
        Self {
            head: Rc::new(RefCell::new(SkipNode::new(T::default(), level_cap - 1))),
            max_level: level_cap,
            current_level: 0,
            length: 0,
            random: XorShift64::new(seed),
        }
    }

    pub fn len(&self) -> usize {
        self.length
    }

    pub fn is_empty(&self) -> bool {
        self.length == 0
    }

    pub fn contains(&self, key: T) -> bool {
        let mut current = self.head.clone();

        for level in (0..=self.current_level).rev() {
            loop {
                let next_node = current.borrow().forward[level].clone();
                match next_node {
                    Some(ref node) if node.borrow().key < key => {
                        current = node.clone();
                    }
                    _ => break,
                }
            }
        }

        let contains_key = current
            .borrow()
            .forward
            .get(0)
            .and_then(|next| next.clone())
            .map(|node| node.borrow().key == key)
            .unwrap_or(false);

        contains_key
    }

    pub fn insert(&mut self, key: T) {
        let mut update_nodes = self.collect_predecessors(key);

        let next = update_nodes[0].borrow().forward[0].clone();
        if let Some(node) = next {
            if node.borrow().key == key {
                node.borrow_mut().count += 1;
                self.length += 1;
                return;
            }
        }

        let node_level = self.random_level();
        if self.current_level < node_level {
            for level in self.current_level + 1..=node_level {
                update_nodes[level] = self.head.clone();
            }
            self.current_level = node_level;
        }

        let new_node = Rc::new(RefCell::new(SkipNode::new(key, node_level)));
        for level in 0..=node_level {
            let next = update_nodes[level].borrow().forward[level].clone();
            new_node.borrow_mut().forward[level] = next;
            update_nodes[level].borrow_mut().forward[level] = Some(new_node.clone());
        }

        self.length += 1;
    }

    pub fn remove(&mut self, key: T) -> bool {
        let mut update_nodes = self.collect_predecessors(key);

        let target = update_nodes[0].borrow().forward[0].clone();
        if let Some(node) = target {
            if node.borrow().key == key {
                if 1 < node.borrow().count {
                    node.borrow_mut().count -= 1;
                    self.length -= 1;
                    return true;
                }

                for level in 0..=self.current_level {
                    let should_relink = {
                        let current_forward = update_nodes[level].borrow();
                        current_forward.forward[level]
                            .as_ref()
                            .map(|next| Rc::ptr_eq(next, &node))
                            .unwrap_or(false)
                    };

                    if should_relink {
                        let next = node.borrow().forward[level].clone();
                        update_nodes[level].borrow_mut().forward[level] = next;
                    }
                }

                while 0 < self.current_level
                    && self.head.borrow().forward[self.current_level].is_none()
                {
                    self.current_level -= 1;
                }

                self.length -= 1;
                return true;
            }
        }

        false
    }

    pub fn iter(&self) -> SkipListIter<T> {
        let start = self.head.borrow().forward[0].clone();
        SkipListIter {
            current: start,
            remaining_in_node: 0,
        }
    }

    fn collect_predecessors(&self, key: T) -> Vec<Rc<RefCell<SkipNode<T>>>> {
        let mut update_nodes = vec![self.head.clone(); self.max_level];
        let mut current = self.head.clone();

        for level in (0..=self.current_level).rev() {
            loop {
                let next_node = current.borrow().forward[level].clone();
                match next_node {
                    Some(ref node) if node.borrow().key < key => {
                        current = node.clone();
                    }
                    _ => break,
                }
            }
            update_nodes[level] = current.clone();
        }

        update_nodes
    }

    fn random_level(&mut self) -> usize {
        let mut level = 0;
        while level + 1 < self.max_level && self.random.next_bit_is_zero() {
            level += 1;
        }
        level
    }

    fn seed_from_time() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos() as u64)
            .unwrap_or(0x1234_5678_9abc_def0)
    }
}

pub struct SkipListIter<T: Ord + Copy + Default + Debug> {
    current: Option<Rc<RefCell<SkipNode<T>>>>,
    remaining_in_node: usize,
}

impl<T: Ord + Copy + Default + Debug> Iterator for SkipListIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.current.clone() {
                Some(node) => {
                    if self.remaining_in_node == 0 {
                        self.remaining_in_node = node.borrow().count;
                    }

                    if 0 < self.remaining_in_node {
                        self.remaining_in_node -= 1;
                        let value = node.borrow().key;
                        if self.remaining_in_node == 0 {
                            self.current = node.borrow().forward[0].clone();
                        }
                        return Some(value);
                    }
                }
                None => return None,
            }
        }
    }
}
#[cfg(test)]
mod todo_tests {
    use super::SkipList;

    fn collect(list: &SkipList<i32>) -> Vec<i32> {
        list.iter().collect::<Vec<_>>()
    }

    #[test]
    fn initialization_variants_start_empty() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 1;
        let new_list = SkipList::<i32>::new();
        let default_list = SkipList::<i32>::default();
        let seeded_zero = SkipList::<i32>::with_seed(input_value_1, input_value_2);
        let seeded_one = SkipList::<i32>::with_seed(input_value_3, input_value_4);
        let expected_check_1 = ((0, true), (0, true), (0, true), (0, true));

        // Act
        let actual_values = (
            (new_list.len(), new_list.is_empty()),
            (default_list.len(), default_list.is_empty()),
            (seeded_zero.len(), seeded_zero.is_empty()),
            (seeded_one.len(), seeded_one.is_empty()),
        );

        // Assert
        assert_eq!(actual_values, expected_check_1);
    }

    #[test]
    fn same_seed_is_deterministic_and_different_seed_keeps_public_behavior() {
        // Arrange
        let input_value_1 = 8;
        let input_value_2 = 1;
        let input_value_3 = 8;
        let input_value_4 = 1;
        let input_value_5 = 8;
        let input_value_6 = 2;
        let mut left = SkipList::with_seed(input_value_1, input_value_2);
        let mut right = SkipList::with_seed(input_value_3, input_value_4);
        let mut other = SkipList::with_seed(input_value_5, input_value_6);

        let values = [3, 1, 2, 2];

        // Act
        for value in values {
            left.insert(value);
            right.insert(value);
            other.insert(value);
        }
        let actual_values = (collect(&left), collect(&right), collect(&other));

        // Assert
        assert_eq!(actual_values.0, actual_values.1);
        assert_eq!(actual_values.0, actual_values.2);
    }

    #[test]
    fn contains_handles_empty_boundaries_and_extremes() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 10;
        let input_value_4 = i32::MIN;
        let input_value_5 = i32::MAX;
        let input_value_6 = 4;
        let input_value_7 = 1;
        let input_value_8 = 0;
        let input_value_9 = 9;
        let input_value_10 = 10;
        let input_value_11 = 11;
        let input_value_12 = i32::MIN;
        let input_value_13 = i32::MAX;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.insert(input_value_5);
        let expected_check_1 = (false, false, true, false, true, true);

        // Act
        let actual_values = (
            SkipList::<i32>::with_seed(input_value_6, input_value_7).contains(input_value_8),
            list.contains(input_value_9),
            list.contains(input_value_10),
            list.contains(input_value_11),
            list.contains(input_value_12),
            list.contains(input_value_13),
        );

        // Assert
        assert_eq!(actual_values, expected_check_1);
    }

    #[test]
    fn absent_middle_and_outside_values_return_false() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 10;
        let input_value_4 = 20;
        let input_value_5 = 30;
        let input_value_6 = 15;
        let input_value_7 = 5;
        let input_value_8 = 35;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.insert(input_value_5);
        let expected_check_1 = (false, false, false);

        // Act
        let actual_values = (
            list.contains(input_value_6),
            list.contains(input_value_7),
            list.contains(input_value_8),
        );

        // Assert
        assert_eq!(actual_values, expected_check_1);
    }

    #[test]
    fn unsorted_insertions_iterate_in_sorted_order() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 3;
        let input_value_4 = 1;
        let input_value_5 = 2;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        let expected_check_1 = vec![1, 2, 3];

        // Act
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.insert(input_value_5);
        let actual_values = collect(&list);

        // Assert
        assert_eq!(actual_values, expected_check_1);
    }

    #[test]
    fn duplicates_affect_len_and_iteration_counts() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 2;
        let input_value_4 = 2;
        let input_value_5 = 2;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        let expected_check_1 = 3;
        let expected_check_2 = vec![2, 2, 2];

        // Act
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.insert(input_value_5);
        let actual_len = list.len();
        let actual_values = collect(&list);

        // Assert
        assert_eq!(actual_len, expected_check_1);
        assert_eq!(actual_values, expected_check_2);
    }

    #[test]
    fn remove_missing_keeps_state_unchanged() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        let expected_values = (false, 1, vec![1]);

        // Act
        let actual_removed = list.remove(input_value_4);
        let actual_values = (actual_removed, list.len(), collect(&list));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn remove_single_and_duplicate_values_updates_membership_stepwise() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 5;
        let input_value_4 = 5;
        let input_value_5 = 5;
        let input_value_6 = 5;
        let input_value_7 = 5;
        let input_value_8 = 5;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        list.insert(input_value_4);
        let expected_check_2 = (true, 1, vec![5]);
        let expected_check_4 = (false, 0, vec![]);

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        let first_remove = list.remove(input_value_5);
        let state_after_first = (list.contains(input_value_6), list.len(), collect(&list));
        let second_remove = list.remove(input_value_7);
        let state_after_second = (list.contains(input_value_8), list.len(), collect(&list));

        // Assert
        assert_eq!(first_remove, expected_boolean_1);
        assert_eq!(state_after_first, expected_check_2);
        assert_eq!(second_remove, expected_boolean_2);
        assert_eq!(state_after_second, expected_check_4);
    }

    #[test]
    fn min_and_max_removals_keep_remaining_order() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 3;
        let input_value_6 = 1;
        let input_value_7 = 3;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.insert(input_value_5);
        let expected_check_1 = vec![2, 3];
        let expected_check_2 = vec![2];

        // Act
        list.remove(input_value_6);
        let after_min = collect(&list);
        list.remove(input_value_7);
        let after_max = collect(&list);

        // Assert
        assert_eq!(after_min, expected_check_1);
        assert_eq!(after_max, expected_check_2);
    }

    #[test]
    fn reuse_after_clearing_and_len_matches_iteration_count() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 1;
        let input_value_6 = 2;
        let input_value_7 = 4;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.remove(input_value_5);
        list.remove(input_value_6);
        list.insert(input_value_7);
        let expected_values = vec![4];

        // Act
        let actual_values = collect(&list);
        let actual_len = list.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, actual_values.len());
    }

    #[test]
    fn empty_iterator_returns_none_on_repeated_next_calls() {
        // Arrange
        let max_level = 4;
        let seed = 1;
        let list = SkipList::<i32>::with_seed(max_level, seed);
        let mut iterator = list.iter();
        let expected = (None, None);

        // Act
        let actual = (iterator.next(), iterator.next());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn alternating_insert_remove_preserves_other_duplicates() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let input_value_3 = 1;
        let input_value_4 = 2;
        let input_value_5 = 2;
        let input_value_6 = 1;
        let input_value_7 = 3;
        let input_value_8 = 2;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);
        list.insert(input_value_3);
        list.insert(input_value_4);
        list.insert(input_value_5);
        list.remove(input_value_6);
        list.insert(input_value_7);
        list.remove(input_value_8);
        let expected_check_1 = vec![2, 3];

        // Act
        let actual_values = collect(&list);

        // Assert
        assert_eq!(actual_values, expected_check_1);
    }

    #[test]
    fn large_number_of_insertions_and_removals_work() {
        // Arrange
        let input_value_1 = 8;
        let input_value_2 = 1;
        let mut list = SkipList::with_seed(input_value_1, input_value_2);

        let insertion_values = 0..1000;
        let search_values = insertion_values.clone();
        let removal_values = insertion_values.clone();
        let expected_check_2 = 0;

        let expected_boolean_1 = true;
        let expected_boolean_2 = true;

        // Act
        for value in insertion_values {
            list.insert(value);
        }
        let all_exist = search_values.clone().all(|value| list.contains(value));
        for value in removal_values {
            list.remove(value);
        }

        let actual_check_2 = list.len();
        let actual_check_3 = collect(&list).is_empty();

        // Assert
        assert_eq!(all_exist, expected_boolean_1);
        assert_eq!(actual_check_2, expected_check_2);
        assert_eq!(actual_check_3, expected_boolean_2);
    }
    #[test]
    fn remove_from_empty_list_returns_false() {
        // Arrange
        let max_level = 4;
        let seed = 1;
        let key = 1;
        let expected = false;
        let mut list = SkipList::with_seed(max_level, seed);

        // Act
        let actual = list.remove(key);

        // Assert
        assert_eq!(actual, expected);
    }
    #[test]
    fn level_zero_and_one_support_duplicate_insert_search_remove_and_ordering() {
        // Arrange
        let levels = [0, 1];
        let seed = 1;
        let keys = [3, 1, 2, 2];
        let removed_key = 2;
        let missing_key = 4;
        let expected = vec![(vec![1, 2, 2, 3], true, false, true, vec![1, 2, 3], 3); levels.len()];

        // Act
        let actual = levels
            .into_iter()
            .map(|level| {
                let mut list = SkipList::with_seed(level, seed);
                for key in keys {
                    list.insert(key);
                }
                let before = collect(&list);
                let present = list.contains(removed_key);
                let absent = list.contains(missing_key);
                let removed = list.remove(removed_key);
                (before, present, absent, removed, collect(&list), list.len())
            })
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }
}
