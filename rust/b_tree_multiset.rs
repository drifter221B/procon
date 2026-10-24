const DEFAULT_MIN_DEGREE: usize = 8;
const DEFAULT_MAX_KEYS: usize = 2 * DEFAULT_MIN_DEGREE - 1;

#[derive(Debug)]
struct Entry<T: Ord> {
    key: T,
    count: usize,
}

impl<T: Ord> Entry<T> {
    fn new(key: T, count: usize) -> Self {
        Self { key, count }
    }
}

#[derive(Debug)]
struct Node<T: Ord> {
    entries: Vec<Entry<T>>,
    children: Vec<Node<T>>,
    subtree_size: usize,
}

impl<T: Ord> Node<T> {
    fn new_leaf(entry: Entry<T>) -> Self {
        Self {
            subtree_size: entry.count,
            entries: vec![entry],
            children: Vec::new(),
        }
    }

    fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }

    fn is_full(&self) -> bool {
        self.entries.len() == DEFAULT_MAX_KEYS
    }

    fn lower_bound_entry_index(&self, key: &T) -> usize {
        self.entries.partition_point(|entry| entry.key < *key)
    }

    fn child_subtree_size(&self, child_index: usize) -> usize {
        self.children
            .get(child_index)
            .map_or(0, |child_node| child_node.subtree_size)
    }

    fn refresh_subtree_size(&mut self) {
        self.subtree_size = self.entries.iter().map(|entry| entry.count).sum::<usize>()
            + self
                .children
                .iter()
                .map(|child_node| child_node.subtree_size)
                .sum::<usize>();
    }
}

#[derive(Debug)]
pub struct BTreeMultiset<T: Ord> {
    root: Option<Node<T>>,
}

impl<T: Ord> Default for BTreeMultiset<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord> BTreeMultiset<T> {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn len(&self) -> usize {
        self.root
            .as_ref()
            .map_or(0, |root_node| root_node.subtree_size)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn contains(&self, key: &T) -> bool {
        0 < self.count(key)
    }

    pub fn count(&self, key: &T) -> usize {
        self.root
            .as_ref()
            .and_then(|root_node| Self::find_entry(root_node, key))
            .map_or(0, |entry| entry.count)
    }

    pub fn insert(&mut self, key: T) {
        self.insert_multiple(key, 1);
    }

    pub fn insert_multiple(&mut self, key: T, insert_count: usize) {
        if insert_count == 0 {
            return;
        }

        if self.root.is_none() {
            self.root = Some(Node::new_leaf(Entry::new(key, insert_count)));
            return;
        }

        let root_is_full = self.root.as_ref().is_some_and(Node::is_full);
        if root_is_full {
            let previous_root = self.root.take().expect("root exists when splitting");
            let mut new_root = Node {
                entries: Vec::new(),
                children: vec![previous_root],
                subtree_size: 0,
            };
            Self::split_child(&mut new_root, 0);
            Self::insert_non_full(&mut new_root, key, insert_count);
            new_root.refresh_subtree_size();
            self.root = Some(new_root);
        } else if let Some(root_node) = self.root.as_mut() {
            Self::insert_non_full(root_node, key, insert_count);
        }
    }

    pub fn lower_bound(&self, key: &T) -> Option<&T> {
        self.root
            .as_ref()
            .and_then(|root_node| Self::lower_bound_in_node(root_node, key))
    }

    pub fn upper_bound(&self, key: &T) -> Option<&T> {
        self.root
            .as_ref()
            .and_then(|root_node| Self::upper_bound_in_node(root_node, key))
    }

    pub fn kth_min(&self, index: usize) -> Option<&T> {
        if self.len() <= index {
            None
        } else {
            self.root
                .as_ref()
                .and_then(|root_node| Self::kth_min_in_node(root_node, index))
        }
    }

    fn find_entry<'a>(node: &'a Node<T>, key: &T) -> Option<&'a Entry<T>> {
        let entry_index = node.lower_bound_entry_index(key);
        if entry_index < node.entries.len() && *key == node.entries[entry_index].key {
            Some(&node.entries[entry_index])
        } else if node.is_leaf() {
            None
        } else {
            Self::find_entry(&node.children[entry_index], key)
        }
    }

    fn insert_non_full(node: &mut Node<T>, key: T, insert_count: usize) {
        let mut entry_index = node.lower_bound_entry_index(&key);

        if entry_index < node.entries.len() && key == node.entries[entry_index].key {
            node.entries[entry_index].count += insert_count;
            node.subtree_size += insert_count;
            return;
        }

        if node.is_leaf() {
            node.entries
                .insert(entry_index, Entry::new(key, insert_count));
            node.subtree_size += insert_count;
            return;
        }

        if node.children[entry_index].is_full() {
            Self::split_child(node, entry_index);

            if key == node.entries[entry_index].key {
                node.entries[entry_index].count += insert_count;
                node.refresh_subtree_size();
                return;
            }

            if node.entries[entry_index].key < key {
                entry_index += 1;
            }
        }

        Self::insert_non_full(&mut node.children[entry_index], key, insert_count);
        node.refresh_subtree_size();
    }

    fn split_child(parent_node: &mut Node<T>, child_index: usize) {
        let mut target_child_node = parent_node.children.remove(child_index);

        let right_entries = target_child_node.entries.split_off(DEFAULT_MIN_DEGREE);
        let middle_entry = target_child_node
            .entries
            .pop()
            .expect("full child has a middle entry");
        let right_children = if target_child_node.is_leaf() {
            Vec::new()
        } else {
            target_child_node.children.split_off(DEFAULT_MIN_DEGREE)
        };

        target_child_node.refresh_subtree_size();

        let mut right_child_node = Node {
            entries: right_entries,
            children: right_children,
            subtree_size: 0,
        };
        right_child_node.refresh_subtree_size();

        parent_node.entries.insert(child_index, middle_entry);
        parent_node.children.insert(child_index, target_child_node);
        parent_node
            .children
            .insert(child_index + 1, right_child_node);
        parent_node.refresh_subtree_size();
    }

    fn lower_bound_in_node<'a>(node: &'a Node<T>, key: &T) -> Option<&'a T> {
        let entry_index = node.lower_bound_entry_index(key);

        if node.is_leaf() {
            node.entries.get(entry_index).map(|entry| &entry.key)
        } else if let Some(child_candidate) =
            Self::lower_bound_in_node(&node.children[entry_index], key)
        {
            Some(child_candidate)
        } else {
            node.entries.get(entry_index).map(|entry| &entry.key)
        }
    }

    fn upper_bound_in_node<'a>(node: &'a Node<T>, key: &T) -> Option<&'a T> {
        let entry_index = node.lower_bound_entry_index(key);

        if node.is_leaf() {
            if entry_index == 0 {
                None
            } else {
                Some(&node.entries[entry_index - 1].key)
            }
        } else if let Some(child_candidate) =
            Self::upper_bound_in_node(&node.children[entry_index], key)
        {
            Some(child_candidate)
        } else if entry_index == 0 {
            None
        } else {
            Some(&node.entries[entry_index - 1].key)
        }
    }

    fn kth_min_in_node(node: &Node<T>, mut index: usize) -> Option<&T> {
        for entry_index in 0..node.entries.len() {
            let left_child_subtree_size = node.child_subtree_size(entry_index);

            if index < left_child_subtree_size {
                return Self::kth_min_in_node(&node.children[entry_index], index);
            }
            index -= left_child_subtree_size;

            let current_entry_count = node.entries[entry_index].count;
            if index < current_entry_count {
                return Some(&node.entries[entry_index].key);
            }
            index -= current_entry_count;
        }

        if node.is_leaf() {
            None
        } else {
            Self::kth_min_in_node(&node.children[node.entries.len()], index)
        }
    }
}

impl<T: Ord + Clone> BTreeMultiset<T> {
    pub fn from_slice(values: &[T]) -> Self {
        values
            .iter()
            .cloned()
            .fold(Self::new(), |mut multiset, value| {
                multiset.insert(value);
                multiset
            })
    }
}

#[cfg(test)]
mod tests {
    use super::{BTreeMultiset, Node, DEFAULT_MAX_KEYS, DEFAULT_MIN_DEGREE};
    use std::collections::BTreeMap;

    const RANDOM_SEED: u64 = 0xD1B5_4A32_D192_ED03;

    #[derive(Debug, Eq, Ord, PartialEq, PartialOrd)]
    struct NonCloneOrdKey(i32);

    #[derive(Clone, Copy)]
    enum InsertOperation {
        Single(i32),
        Multiple(i32, usize),
    }

    #[derive(Debug, Eq, PartialEq)]
    struct QuerySnapshot {
        length: usize,
        is_empty: bool,
        counts: Vec<usize>,
        contains: Vec<bool>,
        lower_bounds: Vec<Option<i32>>,
        upper_bounds: Vec<Option<i32>>,
        kth_values: Vec<Option<i32>>,
    }

    fn build_multiset_from_values(values: &[i32]) -> BTreeMultiset<i32> {
        BTreeMultiset::from_slice(values)
    }

    fn collect_kth_values(multiset: &BTreeMultiset<i32>) -> Vec<i32> {
        (0..multiset.len())
            .map(|index| *multiset.kth_min(index).expect("index in range"))
            .collect()
    }

    fn collect_kth_values_generic<T: Ord + Clone>(multiset: &BTreeMultiset<T>) -> Vec<T> {
        (0..multiset.len())
            .map(|index| multiset.kth_min(index).cloned().expect("index in range"))
            .collect()
    }

    fn reference_count_map(values: &[i32]) -> BTreeMap<i32, usize> {
        let mut count_map = BTreeMap::new();
        for &value in values {
            count_map
                .entry(value)
                .and_modify(|count| *count += 1)
                .or_insert(1);
        }
        count_map
    }

    fn reference_lower_bound(count_map: &BTreeMap<i32, usize>, key: i32) -> Option<i32> {
        count_map.range(key..).next().map(|(value, _)| *value)
    }

    fn reference_upper_bound(count_map: &BTreeMap<i32, usize>, key: i32) -> Option<i32> {
        count_map.range(..key).next_back().map(|(value, _)| *value)
    }

    fn reference_kth_min(count_map: &BTreeMap<i32, usize>, index: usize) -> Option<i32> {
        let mut traversed_count = 0usize;
        for (&value, &count) in count_map {
            if index < traversed_count + count {
                return Some(value);
            }
            traversed_count += count;
        }
        None
    }

    fn validate_node_invariants(
        node: &Node<i32>,
        lower_limit: Option<i32>,
        upper_limit: Option<i32>,
        is_root: bool,
    ) -> usize {
        let mut leaf_depth = None;
        validate_node_invariants_at_depth(
            node,
            lower_limit,
            upper_limit,
            is_root,
            0,
            &mut leaf_depth,
        )
    }

    fn validate_node_invariants_at_depth(
        node: &Node<i32>,
        lower_limit: Option<i32>,
        upper_limit: Option<i32>,
        is_root: bool,
        depth: usize,
        leaf_depth: &mut Option<usize>,
    ) -> usize {
        if is_root {
            assert!(!node.entries.is_empty());
        }
        if !is_root {
            assert!(DEFAULT_MIN_DEGREE - 1 <= node.entries.len());
        }
        assert!(node.entries.len() <= DEFAULT_MAX_KEYS);
        assert!(node.entries.iter().all(|entry| 0 < entry.count));

        if node.is_leaf() {
            assert!(node.children.is_empty());
            if let Some(expected_leaf_depth) = *leaf_depth {
                assert_eq!(depth, expected_leaf_depth);
            } else {
                *leaf_depth = Some(depth);
            }
        } else {
            assert_eq!(node.children.len(), node.entries.len() + 1);
        }

        for entry_pair in node.entries.windows(2) {
            assert!(entry_pair[0].key < entry_pair[1].key);
        }

        if let Some(lower_value) = lower_limit {
            if let Some(first_entry) = node.entries.first() {
                assert!(lower_value < first_entry.key);
            }
        }

        if let Some(upper_value) = upper_limit {
            if let Some(last_entry) = node.entries.last() {
                assert!(last_entry.key < upper_value);
            }
        }

        let mut subtree_size_total = 0usize;
        for entry_index in 0..node.entries.len() {
            if !node.is_leaf() {
                let child_lower_limit = if entry_index == 0 {
                    lower_limit
                } else {
                    Some(node.entries[entry_index - 1].key)
                };
                let child_upper_limit = Some(node.entries[entry_index].key);
                subtree_size_total += validate_node_invariants_at_depth(
                    &node.children[entry_index],
                    child_lower_limit,
                    child_upper_limit,
                    false,
                    depth + 1,
                    leaf_depth,
                );
            }
            subtree_size_total += node.entries[entry_index].count;
        }

        if !node.is_leaf() {
            let last_child_lower_limit = node.entries.last().map(|entry| entry.key);
            subtree_size_total += validate_node_invariants_at_depth(
                &node.children[node.entries.len()],
                last_child_lower_limit,
                upper_limit,
                false,
                depth + 1,
                leaf_depth,
            );
        }

        assert_eq!(node.subtree_size, subtree_size_total);
        subtree_size_total
    }

    fn node_height(node: &Node<i32>) -> usize {
        if node.is_leaf() {
            1
        } else {
            1 + node_height(&node.children[0])
        }
    }

    fn search_path_subtree_sizes(node: &Node<i32>, key: &i32) -> Vec<usize> {
        let mut subtree_sizes = vec![node.subtree_size];
        let entry_index = node.lower_bound_entry_index(key);

        if entry_index < node.entries.len() && *key == node.entries[entry_index].key {
            return subtree_sizes;
        }
        if !node.is_leaf() {
            subtree_sizes.extend(search_path_subtree_sizes(&node.children[entry_index], key));
        }
        subtree_sizes
    }

    fn validate_multiset_invariants(multiset: &BTreeMultiset<i32>) -> usize {
        match multiset.root.as_ref() {
            Some(root_node) => {
                assert!(!multiset.is_empty());
                let validated_size = validate_node_invariants(root_node, None, None, true);
                assert_eq!(validated_size, multiset.len());
                validated_size
            }
            None => {
                assert!(multiset.is_empty());
                assert_eq!(multiset.len(), 0);
                0
            }
        }
    }

    fn collect_internal_separator_keys(node: &Node<i32>, separator_keys: &mut Vec<i32>) {
        if node.is_leaf() {
            return;
        }

        separator_keys.extend(node.entries.iter().map(|entry| entry.key));
        for child_node in &node.children {
            collect_internal_separator_keys(child_node, separator_keys);
        }
    }

    fn insert_into_reference(count_map: &mut BTreeMap<i32, usize>, key: i32, count: usize) {
        if count == 0 {
            return;
        }

        count_map
            .entry(key)
            .and_modify(|current_count| *current_count += count)
            .or_insert(count);
    }

    fn next_pseudo_random(state: &mut u64) -> u64 {
        *state ^= *state << 13;
        *state ^= *state >> 7;
        *state ^= *state << 17;
        *state
    }

    fn multiset_query_snapshot(multiset: &BTreeMultiset<i32>, query_keys: &[i32]) -> QuerySnapshot {
        QuerySnapshot {
            length: multiset.len(),
            is_empty: multiset.is_empty(),
            counts: query_keys
                .iter()
                .map(|query_key| multiset.count(query_key))
                .collect(),
            contains: query_keys
                .iter()
                .map(|query_key| multiset.contains(query_key))
                .collect(),
            lower_bounds: query_keys
                .iter()
                .map(|query_key| multiset.lower_bound(query_key).copied())
                .collect(),
            upper_bounds: query_keys
                .iter()
                .map(|query_key| multiset.upper_bound(query_key).copied())
                .collect(),
            kth_values: (0..=multiset.len())
                .map(|index| multiset.kth_min(index).copied())
                .collect(),
        }
    }

    fn reference_query_snapshot(
        count_map: &BTreeMap<i32, usize>,
        query_keys: &[i32],
    ) -> QuerySnapshot {
        let expected_values: Vec<i32> = count_map
            .iter()
            .flat_map(|(&key, &count)| std::iter::repeat_n(key, count))
            .collect();

        QuerySnapshot {
            length: expected_values.len(),
            is_empty: expected_values.is_empty(),
            counts: query_keys
                .iter()
                .map(|query_key| count_map.get(query_key).copied().unwrap_or(0))
                .collect(),
            contains: query_keys
                .iter()
                .map(|query_key| count_map.contains_key(query_key))
                .collect(),
            lower_bounds: query_keys
                .iter()
                .map(|query_key| reference_lower_bound(count_map, *query_key))
                .collect(),
            upper_bounds: query_keys
                .iter()
                .map(|query_key| reference_upper_bound(count_map, *query_key))
                .collect(),
            kth_values: expected_values
                .into_iter()
                .map(Some)
                .chain(std::iter::once(None))
                .collect(),
        }
    }

    #[test]
    fn new_returns_empty_and_query_methods_return_none() {
        // Arrange
        let multiset = BTreeMultiset::<i32>::new();
        let lower_bound_key = 0i32;
        let upper_bound_key = 0i32;
        let kth_index = 0usize;
        let expected_length = 0usize;
        let expected_is_empty = true;
        let expected_bound = None;
        let expected_kth_min = None;

        // Act
        let actual_length = multiset.len();
        let actual_is_empty = multiset.is_empty();
        let actual_lower_bound = multiset.lower_bound(&lower_bound_key);
        let actual_upper_bound = multiset.upper_bound(&upper_bound_key);
        let actual_kth_min = multiset.kth_min(kth_index);

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_lower_bound, expected_bound);
        assert_eq!(actual_upper_bound, expected_bound);
        assert_eq!(actual_kth_min, expected_kth_min);
    }

    #[test]
    fn new_returns_false_for_contains_and_zero_for_count() {
        // Arrange
        let multiset = BTreeMultiset::<i32>::new();
        let query_key = 42i32;

        let expected_count = 0;

        // Act
        let actual_contains = multiset.contains(&query_key);
        let actual_count = multiset.count(&query_key);

        // Assert
        assert!(!actual_contains);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn default_returns_same_initial_state_as_new() {
        // Arrange
        let new_multiset = BTreeMultiset::<i32>::new();
        let default_multiset = BTreeMultiset::<i32>::default();
        let query_key = 7i32;

        // Act
        let actual_new_length = new_multiset.len();
        let actual_new_is_empty = new_multiset.is_empty();
        let actual_new_contains = new_multiset.contains(&query_key);
        let actual_new_count = new_multiset.count(&query_key);
        let actual_default_length = default_multiset.len();
        let actual_default_is_empty = default_multiset.is_empty();
        let actual_default_contains = default_multiset.contains(&query_key);
        let actual_default_count = default_multiset.count(&query_key);

        // Assert
        assert_eq!(actual_default_length, actual_new_length);
        assert_eq!(actual_default_is_empty, actual_new_is_empty);
        assert_eq!(actual_default_contains, actual_new_contains);
        assert_eq!(actual_default_count, actual_new_count);
    }

    #[test]
    fn contains_matches_reference_and_count_positive_after_root_split() {
        // Arrange
        let inserted_values: Vec<i32> = (0..96)
            .map(|index| ((index * 7) % 23) as i32 - 11)
            .collect();
        let query_values: Vec<i32> = (-13..=13).collect();
        let count_map = reference_count_map(&inserted_values);
        let expected_contains: Vec<bool> = query_values
            .iter()
            .map(|query_value| count_map.contains_key(query_value))
            .collect();
        let multiset = build_multiset_from_values(&inserted_values);

        // Act
        let actual_contains: Vec<bool> = query_values
            .iter()
            .map(|query_value| multiset.contains(query_value))
            .collect();
        let actual_count_positive: Vec<bool> = query_values
            .iter()
            .map(|query_value| 0 < multiset.count(query_value))
            .collect();

        // Assert
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_contains, actual_count_positive);
    }

    #[test]
    fn absent_key_queries_do_not_change_existing_counts() {
        // Arrange
        let inserted_values = [1i32, 2, 2, 4];
        let missing_key = 3i32;
        let existing_key = 2i32;
        let multiset = build_multiset_from_values(&inserted_values);
        let expected_existing_count = 2usize;

        let expected_count = 0;

        // Act
        let actual_contains = multiset.contains(&missing_key);
        let actual_count = multiset.count(&missing_key);
        let actual_existing_count = multiset.count(&existing_key);

        // Assert
        assert!(!actual_contains);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_existing_count, expected_existing_count);
    }

    #[test]
    fn count_matches_btree_map_reference_after_root_split() {
        // Arrange
        let inserted_values: Vec<i32> = (0..96)
            .map(|index| ((index * 7) % 23) as i32 - 11)
            .collect();
        let query_values: Vec<i32> = (-13..=13).collect();
        let count_map = reference_count_map(&inserted_values);
        let expected_counts: Vec<usize> = query_values
            .iter()
            .map(|query_value| count_map.get(query_value).copied().unwrap_or(0))
            .collect();
        let multiset = build_multiset_from_values(&inserted_values);

        // Act
        let actual_counts: Vec<usize> = query_values
            .iter()
            .map(|query_value| multiset.count(query_value))
            .collect();

        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn many_insertions_preserve_b_tree_invariants_and_sorted_order() {
        // Arrange
        let inserted_values: Vec<i32> = (0..48)
            .map(|index| (index % 9) as i32 - 4)
            .chain((48..96).map(|index| index as i32))
            .collect();
        let multiset = build_multiset_from_values(&inserted_values);

        // Act
        let actual_total_size = multiset
            .root
            .as_ref()
            .map(|root_node| validate_node_invariants(root_node, None, None, true))
            .unwrap_or(0);
        let actual_kth_values = collect_kth_values(&multiset);
        let actual_is_sorted = actual_kth_values
            .windows(2)
            .all(|value_pair| value_pair[0] <= value_pair[1]);

        // Assert
        assert_eq!(actual_total_size, inserted_values.len());
        assert!(actual_is_sorted);
    }

    #[test]
    fn insert_returns_same_state_as_insert_multiple_with_count_one() {
        // Arrange
        let inserted_value = 13i32;
        let insert_count = 1usize;
        let expected_length = 1usize;
        let expected_is_empty = false;
        let expected_count = 1usize;
        let mut inserted_once_multiset = BTreeMultiset::<i32>::new();
        let mut inserted_multiple_multiset = BTreeMultiset::<i32>::new();

        // Act
        inserted_once_multiset.insert(inserted_value);
        inserted_multiple_multiset.insert_multiple(inserted_value, insert_count);
        let actual_inserted_once_length = inserted_once_multiset.len();
        let actual_inserted_once_is_empty = inserted_once_multiset.is_empty();
        let actual_inserted_once_count = inserted_once_multiset.count(&inserted_value);
        let actual_inserted_once_values = collect_kth_values(&inserted_once_multiset);
        let actual_inserted_multiple_length = inserted_multiple_multiset.len();
        let actual_inserted_multiple_is_empty = inserted_multiple_multiset.is_empty();
        let actual_inserted_multiple_count = inserted_multiple_multiset.count(&inserted_value);
        let actual_inserted_multiple_values = collect_kth_values(&inserted_multiple_multiset);

        // Assert
        assert_eq!(actual_inserted_once_length, expected_length);
        assert_eq!(actual_inserted_once_is_empty, expected_is_empty);
        assert_eq!(actual_inserted_once_count, expected_count);
        assert_eq!(actual_inserted_once_length, actual_inserted_multiple_length);
        assert_eq!(
            actual_inserted_once_is_empty,
            actual_inserted_multiple_is_empty
        );
        assert_eq!(actual_inserted_once_count, actual_inserted_multiple_count);
        assert_eq!(actual_inserted_once_values, actual_inserted_multiple_values);
    }

    #[test]
    fn insert_and_insert_multiple_for_same_key_accumulate_total_count() {
        // Arrange
        let target_key = 5i32;
        let additional_count = 3usize;
        let mut multiset = BTreeMultiset::<i32>::new();

        // Act
        multiset.insert(target_key);
        multiset.insert_multiple(target_key, additional_count);
        let actual_count = multiset.count(&target_key);
        let actual_length = multiset.len();

        // Assert
        assert_eq!(actual_count, additional_count + 1);
        assert_eq!(actual_length, additional_count + 1);
    }

    #[test]
    fn root_split_preserves_contains_and_sorted_kth_values() {
        // Arrange
        let inserted_values: Vec<i32> = (0..=(DEFAULT_MAX_KEYS as i32)).collect();
        let mut multiset = BTreeMultiset::<i32>::new();
        let expected_root_entry_count = 1usize;
        let expected_root_child_count = 2usize;
        let expected_child_entry_counts = vec![DEFAULT_MIN_DEGREE - 1, DEFAULT_MIN_DEGREE];
        let expected_child_subtree_sizes = expected_child_entry_counts.clone();
        let expected_root_subtree_size = inserted_values.len();

        // Act
        for inserted_value in &inserted_values {
            multiset.insert(*inserted_value);
        }
        let actual_contains: Vec<bool> = inserted_values
            .iter()
            .map(|inserted_value| multiset.contains(inserted_value))
            .collect();
        let actual_kth_values = collect_kth_values(&multiset);
        let actual_root_entry_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.entries.len());
        let actual_root_child_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.children.len());
        let actual_root_is_leaf = multiset
            .root
            .as_ref()
            .is_none_or(|root_node| root_node.is_leaf());
        let actual_child_entry_counts = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node
                .children
                .iter()
                .map(|child_node| child_node.entries.len())
                .collect()
        });
        let actual_child_subtree_sizes =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.subtree_size)
                    .collect()
            });
        let actual_root_subtree_size = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.subtree_size);

        // Assert
        assert!(actual_contains
            .into_iter()
            .all(|contains_result| contains_result));
        assert_eq!(actual_kth_values, inserted_values);
        assert_eq!(actual_root_entry_count, expected_root_entry_count);
        assert_eq!(actual_root_child_count, expected_root_child_count);
        assert!(!actual_root_is_leaf);
        assert_eq!(actual_child_entry_counts, expected_child_entry_counts);
        assert_eq!(actual_child_subtree_sizes, expected_child_subtree_sizes);
        assert_eq!(actual_root_subtree_size, expected_root_subtree_size);
    }

    #[test]
    fn full_leaf_root_at_max_keys_remains_unsplit_and_queries_are_correct() {
        // Arrange
        let inserted_values: Vec<i32> = (0..DEFAULT_MAX_KEYS as i32).collect();
        let query_values: Vec<i32> = (-1..=DEFAULT_MAX_KEYS as i32).collect();
        let expected_count_map = reference_count_map(&inserted_values);
        let expected_counts: Vec<usize> = query_values
            .iter()
            .map(|query_value| expected_count_map.get(query_value).copied().unwrap_or(0))
            .collect();
        let expected_contains: Vec<bool> = query_values
            .iter()
            .map(|query_value| expected_count_map.contains_key(query_value))
            .collect();
        let expected_lower_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| reference_lower_bound(&expected_count_map, *query_value))
            .collect();
        let expected_upper_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| reference_upper_bound(&expected_count_map, *query_value))
            .collect();
        let expected_length = inserted_values.len();
        let expected_root_entry_count = DEFAULT_MAX_KEYS;
        let expected_root_child_count = 0usize;

        // Act
        let multiset = build_multiset_from_values(&inserted_values);
        let actual_length = multiset.len();
        let actual_root_is_leaf = multiset
            .root
            .as_ref()
            .is_some_and(|root_node| root_node.is_leaf());
        let actual_root_entry_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.entries.len());
        let actual_root_child_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.children.len());
        let actual_counts: Vec<usize> = query_values
            .iter()
            .map(|query_value| multiset.count(query_value))
            .collect();
        let actual_contains: Vec<bool> = query_values
            .iter()
            .map(|query_value| multiset.contains(query_value))
            .collect();
        let actual_lower_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| multiset.lower_bound(query_value).copied())
            .collect();
        let actual_upper_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| multiset.upper_bound(query_value).copied())
            .collect();
        let actual_kth_values = collect_kth_values(&multiset);

        // Assert
        assert_eq!(actual_length, expected_length);
        assert!(actual_root_is_leaf);
        assert_eq!(actual_root_entry_count, expected_root_entry_count);
        assert_eq!(actual_root_child_count, expected_root_child_count);
        assert_eq!(actual_counts, expected_counts);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_lower_bounds, expected_lower_bounds);
        assert_eq!(actual_upper_bounds, expected_upper_bounds);
        assert_eq!(actual_kth_values, inserted_values);
    }

    #[test]
    fn descending_insertions_split_leftmost_leaf_and_preserve_invariants() {
        // Arrange
        let inserted_values: Vec<i32> = (0..=23).rev().collect();
        let expected_values: Vec<i32> = (0..=23).collect();
        let expected_root_entries = vec![8i32, 16];
        let expected_child_entries: Vec<Vec<i32>> =
            vec![(0..=7).collect(), (9..=15).collect(), (17..=23).collect()];
        let expected_child_subtree_sizes = vec![8usize, 7, 7];
        let expected_length = inserted_values.len();
        let mut multiset = BTreeMultiset::<i32>::new();

        // Act
        for inserted_value in &inserted_values {
            multiset.insert(*inserted_value);
        }
        let actual_values = collect_kth_values(&multiset);
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node.entries.iter().map(|entry| entry.key).collect()
        });
        let actual_child_entries: Vec<Vec<i32>> =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.entries.iter().map(|entry| entry.key).collect())
                    .collect()
            });
        let actual_child_subtree_sizes =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.subtree_size)
                    .collect()
            });
        let actual_total_size = multiset
            .root
            .as_ref()
            .map(|root_node| validate_node_invariants(root_node, None, None, true))
            .unwrap_or(0);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_root_entries, expected_root_entries);
        assert_eq!(actual_child_entries, expected_child_entries);
        assert_eq!(actual_child_subtree_sizes, expected_child_subtree_sizes);
        assert_eq!(actual_total_size, expected_length);
    }

    #[test]
    fn middle_leaf_split_preserves_parent_child_alignment() {
        // Arrange
        let initial_values: Vec<i32> = (0..=23).map(|value| value * 10).collect();
        let additional_values: Vec<i32> = (81..=89).collect();
        let mut expected_values = initial_values.clone();
        expected_values.extend(additional_values.iter().copied());
        expected_values.sort_unstable();
        let expected_root_entries = vec![70i32, 87, 150];
        let expected_child_entries = vec![
            (0..=6).map(|value| value * 10).collect::<Vec<_>>(),
            (80..=86).collect(),
            vec![88, 89, 90, 100, 110, 120, 130, 140],
            (16..=23).map(|value| value * 10).collect(),
        ];
        let expected_child_subtree_sizes = vec![7usize, 7, 8, 8];
        let expected_length = expected_values.len();
        let mut multiset = build_multiset_from_values(&initial_values);

        // Act
        for additional_value in &additional_values {
            multiset.insert(*additional_value);
        }
        let actual_values = collect_kth_values(&multiset);
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node.entries.iter().map(|entry| entry.key).collect()
        });
        let actual_child_entries: Vec<Vec<i32>> =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.entries.iter().map(|entry| entry.key).collect())
                    .collect()
            });
        let actual_child_subtree_sizes =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.subtree_size)
                    .collect()
            });
        let actual_total_size = multiset
            .root
            .as_ref()
            .map(|root_node| validate_node_invariants(root_node, None, None, true))
            .unwrap_or(0);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_root_entries, expected_root_entries);
        assert_eq!(actual_child_entries, expected_child_entries);
        assert_eq!(actual_child_subtree_sizes, expected_child_subtree_sizes);
        assert_eq!(actual_total_size, expected_length);
    }

    #[test]
    fn ascending_129_keys_split_full_internal_root_and_preserve_invariants() {
        // Arrange
        let keys_before_split: Vec<i32> = (0..128).collect();
        let split_trigger_key = 128i32;
        let expected_height_before_split = 2usize;
        let expected_root_entries_before_split = DEFAULT_MAX_KEYS;
        let expected_height_after_split = 3usize;
        let expected_root_entries_after_split = vec![63i32];
        let expected_root_child_count = 2usize;
        let expected_child_entry_counts = vec![DEFAULT_MIN_DEGREE - 1; 2];
        let expected_values: Vec<i32> = (0..=split_trigger_key).collect();
        let expected_length = expected_values.len();
        let mut multiset = build_multiset_from_values(&keys_before_split);
        let root_before_split = multiset.root.as_ref().expect("non-empty tree has a root");
        let actual_height_before_split = node_height(root_before_split);
        let actual_root_entries_before_split = root_before_split.entries.len();

        // Act
        multiset.insert(split_trigger_key);
        let root_after_split = multiset.root.as_ref().expect("non-empty tree has a root");
        let actual_height_after_split = node_height(root_after_split);
        let actual_root_entries_after_split: Vec<i32> = root_after_split
            .entries
            .iter()
            .map(|entry| entry.key)
            .collect();
        let actual_root_child_count = root_after_split.children.len();
        let actual_child_entry_counts: Vec<usize> = root_after_split
            .children
            .iter()
            .map(|child_node| child_node.entries.len())
            .collect();
        let actual_children_are_internal = root_after_split
            .children
            .iter()
            .all(|child_node| !child_node.is_leaf());
        let actual_total_size = validate_multiset_invariants(&multiset);
        let actual_values = collect_kth_values(&multiset);

        // Assert
        assert_eq!(actual_height_before_split, expected_height_before_split);
        assert_eq!(
            actual_root_entries_before_split,
            expected_root_entries_before_split
        );
        assert_eq!(actual_height_after_split, expected_height_after_split);
        assert_eq!(
            actual_root_entries_after_split,
            expected_root_entries_after_split
        );
        assert_eq!(actual_root_child_count, expected_root_child_count);
        assert_eq!(actual_child_entry_counts, expected_child_entry_counts);
        assert!(actual_children_are_internal);
        assert_eq!(actual_total_size, expected_length);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn ascending_193_keys_split_full_non_root_internal_child_and_preserve_invariants() {
        // Arrange
        let keys_before_split: Vec<i32> = (0..192).collect();
        let split_trigger_key = 192i32;
        let expected_height = 3usize;
        let expected_root_entries_before_split = vec![63i32];
        let expected_full_child_entry_count = DEFAULT_MAX_KEYS;
        let expected_root_entries_after_split = vec![63i32, 127];
        let expected_root_child_count = 3usize;
        let expected_child_entry_counts = vec![DEFAULT_MIN_DEGREE - 1; 3];
        let expected_values: Vec<i32> = (0..=split_trigger_key).collect();
        let expected_length = expected_values.len();
        let mut multiset = build_multiset_from_values(&keys_before_split);
        let root_before_split = multiset.root.as_ref().expect("non-empty tree has a root");
        let actual_height_before_split = node_height(root_before_split);
        let actual_root_entries_before_split: Vec<i32> = root_before_split
            .entries
            .iter()
            .map(|entry| entry.key)
            .collect();
        let actual_full_child_entry_count = root_before_split.children[1].entries.len();
        let actual_full_child_is_internal = !root_before_split.children[1].is_leaf();

        // Act
        multiset.insert(split_trigger_key);
        let root_after_split = multiset.root.as_ref().expect("non-empty tree has a root");
        let actual_height_after_split = node_height(root_after_split);
        let actual_root_entries_after_split: Vec<i32> = root_after_split
            .entries
            .iter()
            .map(|entry| entry.key)
            .collect();
        let actual_root_child_count = root_after_split.children.len();
        let actual_child_entry_counts: Vec<usize> = root_after_split
            .children
            .iter()
            .map(|child_node| child_node.entries.len())
            .collect();
        let actual_total_size = validate_multiset_invariants(&multiset);
        let actual_values = collect_kth_values(&multiset);

        // Assert
        assert_eq!(actual_height_before_split, expected_height);
        assert_eq!(
            actual_root_entries_before_split,
            expected_root_entries_before_split
        );
        assert_eq!(
            actual_full_child_entry_count,
            expected_full_child_entry_count
        );
        assert!(actual_full_child_is_internal);
        assert_eq!(actual_height_after_split, expected_height);
        assert_eq!(
            actual_root_entries_after_split,
            expected_root_entries_after_split
        );
        assert_eq!(actual_root_child_count, expected_root_child_count);
        assert_eq!(actual_child_entry_counts, expected_child_entry_counts);
        assert_eq!(actual_total_size, expected_length);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn insert_multiple_for_single_key_accumulates_count_and_len() {
        // Arrange
        let mut multiset = BTreeMultiset::<i32>::new();
        let target_key = 5i32;
        let insert_count = 3usize;
        let kth_indices = (0usize, insert_count - 1, insert_count);
        let expected_kth_value = Some(target_key);
        let expected_out_of_range_kth_value = None;

        // Act
        multiset.insert_multiple(target_key, insert_count);
        let actual_count = multiset.count(&target_key);
        let actual_contains = multiset.contains(&target_key);
        let actual_length = multiset.len();
        let actual_kth_zero = multiset.kth_min(kth_indices.0).copied();
        let actual_kth_last = multiset.kth_min(kth_indices.1).copied();
        let actual_kth_out_of_range = multiset.kth_min(kth_indices.2).copied();

        // Assert
        assert_eq!(actual_count, insert_count);
        assert!(actual_contains);
        assert_eq!(actual_length, insert_count);
        assert_eq!(actual_kth_zero, expected_kth_value);
        assert_eq!(actual_kth_last, expected_kth_value);
        assert_eq!(actual_kth_out_of_range, expected_out_of_range_kth_value);
    }

    #[test]
    fn insert_multiple_with_zero_on_empty_multiset_keeps_state_unchanged() {
        // Arrange
        let mut multiset = BTreeMultiset::<i32>::new();
        let target_key = 5i32;
        let insert_count = 0usize;

        let expected_length = 0;
        let expected_count = 0;

        // Act
        multiset.insert_multiple(target_key, insert_count);
        let actual_length = multiset.len();
        let actual_is_empty = multiset.is_empty();
        let actual_contains = multiset.contains(&target_key);
        let actual_count = multiset.count(&target_key);

        // Assert
        assert_eq!(actual_length, expected_length);
        assert!(actual_is_empty);
        assert!(!actual_contains);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn insert_multiple_with_zero_for_existing_key_keeps_len_count_and_order() {
        // Arrange
        let inserted_values = [1i32, 2, 2, 3];
        let target_key = 2i32;
        let insert_count = 0usize;
        let mut multiset = build_multiset_from_values(&inserted_values);
        let before_length = multiset.len();
        let before_count = multiset.count(&target_key);
        let before_values = collect_kth_values(&multiset);

        // Act
        multiset.insert_multiple(target_key, insert_count);
        let actual_length = multiset.len();
        let actual_count = multiset.count(&target_key);
        let actual_values = collect_kth_values(&multiset);

        // Assert
        assert_eq!(actual_length, before_length);
        assert_eq!(actual_count, before_count);
        assert_eq!(actual_values, before_values);
    }

    #[test]
    fn insert_multiple_for_promoted_key_after_child_split_updates_count_and_len() {
        // Arrange
        let largest_initial_value = (DEFAULT_MAX_KEYS + DEFAULT_MIN_DEGREE - 1) as i32;
        let inserted_values: Vec<i32> = (0..=largest_initial_value).collect();
        let promoted_key = DEFAULT_MAX_KEYS as i32;
        let additional_count = 3usize;
        let expected_root_entries = vec![(DEFAULT_MIN_DEGREE - 1) as i32, promoted_key];
        let mut multiset = BTreeMultiset::<i32>::new();

        for inserted_value in &inserted_values {
            multiset.insert(*inserted_value);
        }

        let expected_root_child_count = 3;

        // Act
        multiset.insert_multiple(promoted_key, additional_count);
        let actual_count = multiset.count(&promoted_key);
        let actual_length = multiset.len();
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node.entries.iter().map(|entry| entry.key).collect()
        });
        let actual_root_child_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.children.len());
        let actual_total_size = multiset
            .root
            .as_ref()
            .map(|root_node| validate_node_invariants(root_node, None, None, true))
            .unwrap_or(0);

        // Assert
        assert_eq!(actual_count, additional_count + 1);
        assert_eq!(actual_length, inserted_values.len() + additional_count);
        assert_eq!(actual_root_entries, expected_root_entries);
        assert_eq!(actual_root_child_count, expected_root_child_count);
        assert_eq!(actual_total_size, inserted_values.len() + additional_count);
    }

    #[test]
    fn insert_multiple_for_median_of_full_root_updates_promoted_entry() {
        // Arrange
        let inserted_values: Vec<i32> = (0..DEFAULT_MAX_KEYS as i32).collect();
        let median_key = (DEFAULT_MIN_DEGREE - 1) as i32;
        let additional_count = 3usize;
        let expected_count = additional_count + 1;
        let expected_length = inserted_values.len() + additional_count;
        let expected_root_entries = vec![(median_key, expected_count)];
        let expected_child_subtree_sizes = vec![DEFAULT_MIN_DEGREE - 1, DEFAULT_MIN_DEGREE - 1];
        let mut expected_values = inserted_values.clone();
        expected_values.extend(std::iter::repeat_n(median_key, additional_count));
        expected_values.sort_unstable();
        let mut multiset = build_multiset_from_values(&inserted_values);

        // Act
        multiset.insert_multiple(median_key, additional_count);
        let actual_count = multiset.count(&median_key);
        let actual_length = multiset.len();
        let actual_values = collect_kth_values(&multiset);
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node
                .entries
                .iter()
                .map(|entry| (entry.key, entry.count))
                .collect()
        });
        let actual_child_subtree_sizes =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.subtree_size)
                    .collect()
            });
        let actual_total_size = multiset
            .root
            .as_ref()
            .map(|root_node| validate_node_invariants(root_node, None, None, true))
            .unwrap_or(0);

        // Assert
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_root_entries, expected_root_entries);
        assert_eq!(actual_child_subtree_sizes, expected_child_subtree_sizes);
        assert_eq!(actual_total_size, expected_length);
    }

    #[test]
    fn insert_multiple_below_median_of_full_root_updates_left_child_sizes() {
        // Arrange
        let inserted_values: Vec<i32> = (0..DEFAULT_MAX_KEYS as i32).collect();
        let median_key = (DEFAULT_MIN_DEGREE - 1) as i32;
        let target_key = median_key - 4;
        let additional_count = 3usize;
        let expected_count = additional_count + 1;
        let expected_length = inserted_values.len() + additional_count;
        let expected_root_entries = vec![median_key];
        let expected_child_subtree_sizes = vec![
            DEFAULT_MIN_DEGREE - 1 + additional_count,
            DEFAULT_MIN_DEGREE - 1,
        ];
        let mut multiset = build_multiset_from_values(&inserted_values);

        // Act
        multiset.insert_multiple(target_key, additional_count);
        let actual_count = multiset.count(&target_key);
        let actual_length = multiset.len();
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node.entries.iter().map(|entry| entry.key).collect()
        });
        let actual_child_subtree_sizes =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.subtree_size)
                    .collect()
            });
        let actual_total_size = multiset
            .root
            .as_ref()
            .map(|root_node| validate_node_invariants(root_node, None, None, true))
            .unwrap_or(0);

        // Assert
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_root_entries, expected_root_entries);
        assert_eq!(actual_child_subtree_sizes, expected_child_subtree_sizes);
        assert_eq!(actual_total_size, expected_length);
    }

    #[test]
    fn zero_insert_into_full_root_does_not_split_or_change_state() {
        // Arrange
        let inserted_values: Vec<i32> = (0..DEFAULT_MAX_KEYS as i32).collect();
        let missing_key = -1i32;
        let insert_count = 0usize;
        let mut multiset = build_multiset_from_values(&inserted_values);
        let before_length = multiset.len();
        let before_values = collect_kth_values(&multiset);
        let before_counts: Vec<usize> = inserted_values
            .iter()
            .map(|inserted_value| multiset.count(inserted_value))
            .collect();
        let before_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node
                .entries
                .iter()
                .map(|entry| (entry.key, entry.count))
                .collect()
        });
        let before_root_child_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.children.len());
        let before_root_subtree_size = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.subtree_size);

        let expected_missing_count = 0;

        // Act
        multiset.insert_multiple(missing_key, insert_count);
        let actual_length = multiset.len();
        let actual_missing_count = multiset.count(&missing_key);
        let actual_values = collect_kth_values(&multiset);
        let actual_counts: Vec<usize> = inserted_values
            .iter()
            .map(|inserted_value| multiset.count(inserted_value))
            .collect();
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node
                .entries
                .iter()
                .map(|entry| (entry.key, entry.count))
                .collect()
        });
        let actual_root_child_count = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.children.len());
        let actual_root_subtree_size = multiset
            .root
            .as_ref()
            .map_or(0, |root_node| root_node.subtree_size);

        // Assert
        assert_eq!(actual_length, before_length);
        assert_eq!(actual_missing_count, expected_missing_count);
        assert_eq!(actual_values, before_values);
        assert_eq!(actual_counts, before_counts);
        assert_eq!(actual_root_entries, before_root_entries);
        assert_eq!(actual_root_child_count, before_root_child_count);
        assert_eq!(actual_root_subtree_size, before_root_subtree_size);
    }

    #[test]
    fn insert_multiple_for_new_key_in_height_three_tree_updates_order_and_path_sizes() {
        // Arrange
        let initial_key_count = 2 * DEFAULT_MIN_DEGREE * DEFAULT_MIN_DEGREE + 1;
        let initial_values: Vec<i32> = (0..initial_key_count as i32)
            .map(|value| value * 2)
            .collect();
        let target_key = initial_key_count as i32;
        let insert_count = 3usize;
        let expected_length = initial_values.len() + insert_count;
        let expected_count = insert_count;
        let duplicate_start_index = (target_key as usize).div_ceil(2);
        let boundary_indices = [
            duplicate_start_index - 1,
            duplicate_start_index,
            duplicate_start_index + 1,
            duplicate_start_index + insert_count - 1,
            duplicate_start_index + insert_count,
        ];
        let expected_boundary_values = vec![
            target_key - 1,
            target_key,
            target_key,
            target_key,
            target_key + 1,
        ];
        let expected_path_node_count = 3usize;
        let mut multiset = build_multiset_from_values(&initial_values);
        let before_path_subtree_sizes = multiset
            .root
            .as_ref()
            .map(|root_node| search_path_subtree_sizes(root_node, &target_key))
            .expect("non-empty tree has a root");
        let expected_path_subtree_sizes: Vec<usize> = before_path_subtree_sizes
            .iter()
            .map(|subtree_size| subtree_size + insert_count)
            .collect();

        // Act
        multiset.insert_multiple(target_key, insert_count);
        let actual_length = multiset.len();
        let actual_count = multiset.count(&target_key);
        let actual_boundary_values: Vec<i32> = boundary_indices
            .iter()
            .map(|index| *multiset.kth_min(*index).expect("index in range"))
            .collect();
        let actual_path_subtree_sizes = multiset
            .root
            .as_ref()
            .map(|root_node| search_path_subtree_sizes(root_node, &target_key))
            .expect("non-empty tree has a root");
        let actual_total_size = validate_multiset_invariants(&multiset);

        // Assert
        assert_eq!(before_path_subtree_sizes.len(), expected_path_node_count);
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_boundary_values, expected_boundary_values);
        assert_eq!(actual_path_subtree_sizes, expected_path_subtree_sizes);
        assert_eq!(actual_total_size, expected_length);
    }

    #[test]
    fn insert_multiple_with_usize_max_supports_last_valid_kth_index() {
        // Arrange
        let target_key = 42i32;
        let insert_count = usize::MAX;
        let last_valid_index = usize::MAX - 1;
        let first_out_of_range_index = usize::MAX;
        let expected_length = usize::MAX;
        let expected_count = usize::MAX;
        let expected_last_value = Some(target_key);
        let expected_out_of_range_value = None;
        let mut multiset = BTreeMultiset::<i32>::new();

        // Act
        multiset.insert_multiple(target_key, insert_count);
        let actual_length = multiset.len();
        let actual_count = multiset.count(&target_key);
        let actual_last_value = multiset.kth_min(last_valid_index).copied();
        let actual_out_of_range_value = multiset.kth_min(first_out_of_range_index).copied();

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_last_value, expected_last_value);
        assert_eq!(actual_out_of_range_value, expected_out_of_range_value);
    }

    #[test]
    fn lower_bound_handles_x_minus_1_x_x_plus_1_queries() {
        // Arrange
        let boundary_center_value = 10i32;
        let inserted_values = [
            boundary_center_value,
            boundary_center_value + 2,
            boundary_center_value + 2,
        ];
        let multiset = build_multiset_from_values(&inserted_values);
        let x_minus_1 = boundary_center_value - 1;
        let x = boundary_center_value;
        let x_plus_1 = boundary_center_value + 1;

        // Act
        let actual_x_minus_1 = multiset.lower_bound(&x_minus_1).copied();
        let actual_x = multiset.lower_bound(&x).copied();
        let actual_x_plus_1 = multiset.lower_bound(&x_plus_1).copied();

        // Assert
        assert_eq!(actual_x_minus_1, Some(boundary_center_value));
        assert_eq!(actual_x, Some(boundary_center_value));
        assert_eq!(actual_x_plus_1, Some(boundary_center_value + 2));
    }

    #[test]
    fn bounds_match_reference_around_internal_separators_in_height_three_tree() {
        // Arrange
        let inserted_values: Vec<i32> = (0..=128).map(|value| value * 2).collect();
        let count_map = reference_count_map(&inserted_values);
        let multiset = build_multiset_from_values(&inserted_values);
        let root_node = multiset.root.as_ref().expect("non-empty tree has a root");
        let expected_height = 3usize;
        let expected_total_size = inserted_values.len();
        let mut separator_keys = Vec::new();
        collect_internal_separator_keys(root_node, &mut separator_keys);
        let mut query_keys: Vec<i32> = separator_keys
            .iter()
            .flat_map(|separator| [separator - 1, *separator, separator + 1])
            .chain([-1, 257])
            .collect();
        query_keys.sort_unstable();
        query_keys.dedup();
        let expected_lower_bounds: Vec<Option<i32>> = query_keys
            .iter()
            .map(|query_key| reference_lower_bound(&count_map, *query_key))
            .collect();
        let expected_upper_bounds: Vec<Option<i32>> = query_keys
            .iter()
            .map(|query_key| reference_upper_bound(&count_map, *query_key))
            .collect();

        // Act
        let actual_height = node_height(root_node);
        let actual_lower_bounds: Vec<Option<i32>> = query_keys
            .iter()
            .map(|query_key| multiset.lower_bound(query_key).copied())
            .collect();
        let actual_upper_bounds: Vec<Option<i32>> = query_keys
            .iter()
            .map(|query_key| multiset.upper_bound(query_key).copied())
            .collect();
        let actual_total_size = validate_multiset_invariants(&multiset);

        // Assert
        assert!(!separator_keys.is_empty());
        assert_eq!(actual_height, expected_height);
        assert_eq!(actual_lower_bounds, expected_lower_bounds);
        assert_eq!(actual_upper_bounds, expected_upper_bounds);
        assert_eq!(actual_total_size, expected_total_size);
    }

    #[test]
    fn upper_bound_handles_x_minus_1_x_x_plus_1_queries() {
        // Arrange
        let boundary_center_value = 10i32;
        let inserted_values = [
            boundary_center_value - 2,
            boundary_center_value,
            boundary_center_value + 2,
        ];
        let multiset = build_multiset_from_values(&inserted_values);
        let x_minus_1 = boundary_center_value - 1;
        let x = boundary_center_value;
        let x_plus_1 = boundary_center_value + 1;

        // Act
        let actual_x_minus_1 = multiset.upper_bound(&x_minus_1).copied();
        let actual_x = multiset.upper_bound(&x).copied();
        let actual_x_plus_1 = multiset.upper_bound(&x_plus_1).copied();

        // Assert
        assert_eq!(actual_x_minus_1, Some(boundary_center_value - 2));
        assert_eq!(actual_x, Some(boundary_center_value - 2));
        assert_eq!(actual_x_plus_1, Some(boundary_center_value));
    }

    #[test]
    fn kth_min_counts_duplicate_values_as_individual_positions() {
        // Arrange
        let inserted_values = [1i32, 2, 2, 2, 3];
        let multiset = build_multiset_from_values(&inserted_values);
        let first_index = 0usize;
        let duplicate_start_index = 1usize;
        let duplicate_middle_index = 2usize;
        let duplicate_end_index = 3usize;
        let last_index = 4usize;
        let expected_values = (Some(1), Some(2), Some(2), Some(2), Some(3));

        // Act
        let actual_first_value = multiset.kth_min(first_index).copied();
        let actual_duplicate_start = multiset.kth_min(duplicate_start_index).copied();
        let actual_duplicate_middle = multiset.kth_min(duplicate_middle_index).copied();
        let actual_duplicate_end = multiset.kth_min(duplicate_end_index).copied();
        let actual_last_value = multiset.kth_min(last_index).copied();

        // Assert
        assert_eq!(actual_first_value, expected_values.0);
        assert_eq!(actual_duplicate_start, expected_values.1);
        assert_eq!(actual_duplicate_middle, expected_values.2);
        assert_eq!(actual_duplicate_end, expected_values.3);
        assert_eq!(actual_last_value, expected_values.4);
    }

    #[test]
    fn kth_min_handles_duplicate_range_at_internal_separator() {
        // Arrange
        let inserted_values: Vec<i32> = (0..=128).collect();
        let mut multiset = build_multiset_from_values(&inserted_values);
        let root_node = multiset.root.as_ref().expect("non-empty tree has a root");
        let separator_key = root_node.entries[0].key;
        let additional_count = 4usize;
        let expected_count = additional_count + 1;
        let duplicate_start_index = separator_key as usize;
        let boundary_indices = [
            duplicate_start_index - 1,
            duplicate_start_index,
            duplicate_start_index + expected_count - 1,
            duplicate_start_index + expected_count,
        ];
        let expected_boundary_values = vec![
            Some(separator_key - 1),
            Some(separator_key),
            Some(separator_key),
            Some(separator_key + 1),
        ];
        let expected_height = 3usize;
        let expected_total_size = inserted_values.len() + additional_count;

        // Act
        multiset.insert_multiple(separator_key, additional_count);
        let actual_boundary_values: Vec<Option<i32>> = boundary_indices
            .iter()
            .map(|index| multiset.kth_min(*index).copied())
            .collect();
        let actual_count = multiset.count(&separator_key);
        let actual_separator_is_internal = multiset.root.as_ref().is_some_and(|root_node| {
            !root_node.is_leaf()
                && root_node
                    .entries
                    .iter()
                    .any(|entry| entry.key == separator_key && entry.count == expected_count)
        });
        let actual_height = multiset.root.as_ref().map_or(0, node_height);
        let actual_total_size = validate_multiset_invariants(&multiset);

        // Assert
        assert_eq!(actual_boundary_values, expected_boundary_values);
        assert_eq!(actual_count, expected_count);
        assert!(actual_separator_is_internal);
        assert_eq!(actual_height, expected_height);
        assert_eq!(actual_total_size, expected_total_size);
    }

    #[test]
    fn from_slice_with_empty_slice_returns_empty_multiset() {
        // Arrange
        let values: Vec<i32> = Vec::new();
        let query_key = 0i32;
        let kth_index = 0usize;
        let expected_length = 0usize;
        let expected_is_empty = true;
        let expected_bound = None;
        let expected_kth_min = None;

        // Act
        let multiset = BTreeMultiset::from_slice(&values);
        let actual_length = multiset.len();
        let actual_is_empty = multiset.is_empty();
        let actual_lower_bound = multiset.lower_bound(&query_key);
        let actual_upper_bound = multiset.upper_bound(&query_key);
        let actual_kth_min = multiset.kth_min(kth_index);

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_lower_bound, expected_bound);
        assert_eq!(actual_upper_bound, expected_bound);
        assert_eq!(actual_kth_min, expected_kth_min);
    }

    #[test]
    fn from_slice_with_single_value_returns_singleton_multiset() {
        // Arrange
        let single_value = 11i32;
        let values = [single_value];
        let kth_index = 0usize;

        let expected_count = 1;

        // Act
        let multiset = BTreeMultiset::from_slice(&values);
        let actual_count = multiset.count(&single_value);
        let actual_contains = multiset.contains(&single_value);
        let actual_kth_min = multiset.kth_min(kth_index).copied();

        // Assert
        assert_eq!(actual_count, expected_count);
        assert!(actual_contains);
        assert_eq!(actual_kth_min, Some(single_value));
    }

    #[test]
    fn from_slice_with_strings_returns_sorted_values_with_duplicate_counts() {
        // Arrange
        let first_value = "a".to_string();
        let second_value = "b".to_string();
        let values = [
            second_value.clone(),
            first_value.clone(),
            second_value.clone(),
        ];
        let expected_values = vec![
            first_value.clone(),
            second_value.clone(),
            second_value.clone(),
        ];

        let expected_first_count = 1;
        let expected_second_count = 2;

        // Act
        let multiset = BTreeMultiset::from_slice(&values);
        let actual_first_count = multiset.count(&first_value);
        let actual_second_count = multiset.count(&second_value);
        let actual_kth_values = collect_kth_values_generic(&multiset);

        // Assert
        assert_eq!(actual_first_count, expected_first_count);
        assert_eq!(actual_second_count, expected_second_count);
        assert_eq!(actual_kth_values, expected_values);
    }

    #[test]
    fn query_methods_match_btree_map_reference_after_root_split() {
        // Arrange
        let inserted_values: Vec<i32> = (0..96)
            .map(|index| ((index * 7) % 23) as i32 - 11)
            .collect();
        let query_values: Vec<i32> = (-13..=13).collect();
        let multiset = build_multiset_from_values(&inserted_values);
        let count_map = reference_count_map(&inserted_values);

        // Act
        let actual_lower_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| multiset.lower_bound(query_value).copied())
            .collect();
        let actual_upper_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| multiset.upper_bound(query_value).copied())
            .collect();
        let actual_kth_values: Vec<Option<i32>> = (0..=multiset.len())
            .map(|index| multiset.kth_min(index).copied())
            .collect();

        let expected_lower_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| reference_lower_bound(&count_map, *query_value))
            .collect();
        let expected_upper_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| reference_upper_bound(&count_map, *query_value))
            .collect();
        let expected_kth_values: Vec<Option<i32>> = (0..=multiset.len())
            .map(|index| reference_kth_min(&count_map, index))
            .collect();

        // Assert
        assert_eq!(actual_lower_bounds, expected_lower_bounds);
        assert_eq!(actual_upper_bounds, expected_upper_bounds);
        assert_eq!(actual_kth_values, expected_kth_values);
    }

    #[test]
    fn query_methods_handle_i32_boundaries_in_sorted_order() {
        // Arrange
        let minimum_value = i32::MIN;
        let middle_value = 0i32;
        let maximum_value = i32::MAX;
        let minimum_successor = i32::MIN + 1;
        let maximum_predecessor = i32::MAX - 1;
        let kth_indices = (0usize, 1usize, 2usize);
        let expected_lower_bounds = (
            Some(minimum_value),
            Some(middle_value),
            Some(maximum_value),
            Some(maximum_value),
        );
        let expected_upper_bounds = (None, Some(minimum_value), Some(middle_value));
        let expected_kth_values = (Some(minimum_value), Some(middle_value), Some(maximum_value));
        let inserted_values = [minimum_value, middle_value, maximum_value];
        let multiset = build_multiset_from_values(&inserted_values);

        // Act
        let actual_lower_bound_at_minimum = multiset.lower_bound(&minimum_value).copied();
        let actual_lower_bound_after_minimum = multiset.lower_bound(&minimum_successor).copied();
        let actual_lower_bound_at_maximum_predecessor =
            multiset.lower_bound(&maximum_predecessor).copied();
        let actual_lower_bound_at_maximum = multiset.lower_bound(&maximum_value).copied();
        let actual_upper_bound_at_minimum = multiset.upper_bound(&minimum_value).copied();
        let actual_upper_bound_at_middle = multiset.upper_bound(&middle_value).copied();
        let actual_upper_bound_at_maximum = multiset.upper_bound(&maximum_value).copied();
        let actual_first_value = multiset.kth_min(kth_indices.0).copied();
        let actual_second_value = multiset.kth_min(kth_indices.1).copied();
        let actual_third_value = multiset.kth_min(kth_indices.2).copied();

        // Assert
        assert_eq!(actual_lower_bound_at_minimum, expected_lower_bounds.0);
        assert_eq!(actual_lower_bound_after_minimum, expected_lower_bounds.1);
        assert_eq!(
            actual_lower_bound_at_maximum_predecessor,
            expected_lower_bounds.2
        );
        assert_eq!(actual_lower_bound_at_maximum, expected_lower_bounds.3);
        assert_eq!(actual_upper_bound_at_minimum, expected_upper_bounds.0);
        assert_eq!(actual_upper_bound_at_middle, expected_upper_bounds.1);
        assert_eq!(actual_upper_bound_at_maximum, expected_upper_bounds.2);
        assert_eq!(actual_first_value, expected_kth_values.0);
        assert_eq!(actual_second_value, expected_kth_values.1);
        assert_eq!(actual_third_value, expected_kth_values.2);
    }

    #[test]
    fn fixed_seed_mixed_insertions_match_reference_after_every_operation() {
        // Arrange
        let mut random_state = RANDOM_SEED;
        let mut shuffled_keys: Vec<i32> = (-160..=160).collect();
        for upper_index in (1..shuffled_keys.len()).rev() {
            let swap_index =
                (next_pseudo_random(&mut random_state) % (upper_index as u64 + 1)) as usize;
            shuffled_keys.swap(upper_index, swap_index);
        }
        let mut operations = Vec::new();
        for (key_index, &key) in shuffled_keys.iter().enumerate() {
            if key_index % 2 == 0 {
                operations.push(InsertOperation::Single(key));
            } else {
                let insert_count = (next_pseudo_random(&mut random_state) % 3 + 1) as usize;
                operations.push(InsertOperation::Multiple(key, insert_count));
            }

            if key_index % 29 == 0 {
                let duplicate_key = shuffled_keys[key_index / 2];
                if (key_index / 29) % 2 == 0 {
                    operations.push(InsertOperation::Single(duplicate_key));
                } else {
                    let duplicate_count = (next_pseudo_random(&mut random_state) % 3 + 2) as usize;
                    operations.push(InsertOperation::Multiple(duplicate_key, duplicate_count));
                }
            }

            if key_index % 43 == 0 {
                let absent_key = -10_000 - key_index as i32;
                operations.push(InsertOperation::Multiple(absent_key, 0));
            }
        }
        let mut query_keys: Vec<i32> = (-162..=162)
            .chain(operations.iter().map(|operation| match operation {
                InsertOperation::Single(key) | InsertOperation::Multiple(key, _) => *key,
            }))
            .collect();
        query_keys.sort_unstable();
        query_keys.dedup();
        let expected_distinct_key_count = shuffled_keys.len();
        let mut reference_count_map = BTreeMap::new();
        let mut expected_snapshots = Vec::with_capacity(operations.len());
        for operation in &operations {
            match *operation {
                InsertOperation::Single(key) => {
                    insert_into_reference(&mut reference_count_map, key, 1)
                }
                InsertOperation::Multiple(key, count) => {
                    insert_into_reference(&mut reference_count_map, key, count)
                }
            }
            expected_snapshots.push(reference_query_snapshot(&reference_count_map, &query_keys));
        }
        let expected_has_negative_key = reference_count_map.keys().any(|key| *key < 0);
        let expected_has_duplicate_count = reference_count_map.values().any(|count| 1 < *count);
        let expected_single_operation_count = operations
            .iter()
            .filter(|operation| matches!(operation, InsertOperation::Single(_)))
            .count();
        let expected_multiple_operation_count = operations
            .iter()
            .filter(
                |operation| matches!(operation, InsertOperation::Multiple(_, count) if 0 < *count),
            )
            .count();
        let expected_zero_operation_count = operations
            .iter()
            .filter(|operation| matches!(operation, InsertOperation::Multiple(_, 0)))
            .count();
        let mut multiset = BTreeMultiset::<i32>::new();
        let mut snapshots = Vec::with_capacity(operations.len());

        // Act
        for (operation_index, operation) in operations.iter().enumerate() {
            match *operation {
                InsertOperation::Single(key) => multiset.insert(key),
                InsertOperation::Multiple(key, insert_count) => {
                    multiset.insert_multiple(key, insert_count);
                }
            }

            let actual_snapshot = multiset_query_snapshot(&multiset, &query_keys);
            let actual_invariant_size = validate_multiset_invariants(&multiset);
            snapshots.push((operation_index, actual_snapshot, actual_invariant_size));
        }
        let actual_distinct_key_count = multiset
            .root
            .as_ref()
            .map_or(0, |_| reference_count_map.len());

        // Assert
        for ((operation_index, actual, invariant_size), expected) in
            snapshots.into_iter().zip(expected_snapshots)
        {
            assert_eq!(
                invariant_size, expected.length,
                "operation {operation_index} invariant size"
            );
            assert_eq!(actual, expected, "operation {operation_index}");
        }
        assert_eq!(actual_distinct_key_count, expected_distinct_key_count);
        assert!(expected_has_negative_key);
        assert!(expected_has_duplicate_count);
        assert!(0 < expected_single_operation_count);
        assert!(0 < expected_multiple_operation_count);
        assert!(0 < expected_zero_operation_count);
    }

    #[test]
    fn non_clone_ord_key_queries_remain_correct_after_root_and_child_splits() {
        // Arrange
        let distinct_key_count = DEFAULT_MAX_KEYS + DEFAULT_MIN_DEGREE + 1;
        let query_values: Vec<i32> = (-1..=distinct_key_count as i32).collect();
        let expected_length = distinct_key_count;
        let expected_root_entries = vec![(DEFAULT_MIN_DEGREE - 1) as i32, DEFAULT_MAX_KEYS as i32];
        let expected_child_entry_counts = vec![
            DEFAULT_MIN_DEGREE - 1,
            DEFAULT_MIN_DEGREE - 1,
            DEFAULT_MIN_DEGREE,
        ];
        let expected_child_subtree_sizes = expected_child_entry_counts.clone();
        let expected_counts: Vec<usize> = query_values
            .iter()
            .map(|query_value| {
                usize::from(0 <= *query_value && *query_value < distinct_key_count as i32)
            })
            .collect();
        let expected_lower_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| {
                if *query_value < 0 {
                    Some(0)
                } else if *query_value < distinct_key_count as i32 {
                    Some(*query_value)
                } else {
                    None
                }
            })
            .collect();
        let expected_upper_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| {
                if *query_value <= 0 {
                    None
                } else if *query_value <= distinct_key_count as i32 {
                    Some(*query_value - 1)
                } else {
                    Some(distinct_key_count as i32 - 1)
                }
            })
            .collect();
        let expected_kth_values: Vec<Option<i32>> = (0..distinct_key_count as i32)
            .map(Some)
            .chain(std::iter::once(None))
            .collect();
        let mut multiset = BTreeMultiset::<NonCloneOrdKey>::new();

        // Act
        for key in 0..distinct_key_count as i32 {
            multiset.insert(NonCloneOrdKey(key));
        }
        let actual_length = multiset.len();
        let actual_counts: Vec<usize> = query_values
            .iter()
            .map(|query_value| multiset.count(&NonCloneOrdKey(*query_value)))
            .collect();
        let actual_lower_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| {
                multiset
                    .lower_bound(&NonCloneOrdKey(*query_value))
                    .map(|key| key.0)
            })
            .collect();
        let actual_upper_bounds: Vec<Option<i32>> = query_values
            .iter()
            .map(|query_value| {
                multiset
                    .upper_bound(&NonCloneOrdKey(*query_value))
                    .map(|key| key.0)
            })
            .collect();
        let actual_kth_values: Vec<Option<i32>> = (0..=multiset.len())
            .map(|index| multiset.kth_min(index).map(|key| key.0))
            .collect();
        let actual_root_entries = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node.entries.iter().map(|entry| entry.key.0).collect()
        });
        let actual_child_entry_counts = multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
            root_node
                .children
                .iter()
                .map(|child_node| child_node.entries.len())
                .collect()
        });
        let actual_child_subtree_sizes =
            multiset.root.as_ref().map_or_else(Vec::new, |root_node| {
                root_node
                    .children
                    .iter()
                    .map(|child_node| child_node.subtree_size)
                    .collect()
            });

        // Assert
        assert_eq!(actual_length, expected_length);
        assert_eq!(actual_counts, expected_counts);
        assert_eq!(actual_lower_bounds, expected_lower_bounds);
        assert_eq!(actual_upper_bounds, expected_upper_bounds);
        assert_eq!(actual_kth_values, expected_kth_values);
        assert_eq!(actual_root_entries, expected_root_entries);
        assert_eq!(actual_child_entry_counts, expected_child_entry_counts);
        assert_eq!(actual_child_subtree_sizes, expected_child_subtree_sizes);
    }
}
