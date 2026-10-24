#[derive(Default)]
pub struct Node {
    pub children: [Option<Box<Node>>; 2],
    pub counter: usize,
    pub end_count: usize,
}

impl Node {

    pub fn new() -> Self {
        Self {
            children: [None, None],
            counter: 0,
            end_count: 0,
        }
    }

    pub fn is_leaf(&self) -> bool {
        0 < self.end_count
    }

}

pub struct BinaryTrie {
    root: Box<Node>,
    bit_size: usize,
}

impl BinaryTrie {

    pub fn new(bit_size: usize) -> Self {
        assert!(bit_size <= 64, "bit_size must be 64 or less");
        Self {
            root: Box::new(Node::new()),
            bit_size,
        }
    }

    pub fn insert(&mut self, x: u64) {

        let mut node = &mut self.root;

        for i in (0..self.bit_size).rev() {

            let bit = ((x >> i) & 1) as usize;

            let child = node
                .children[bit]
                .get_or_insert_with(|| Box::new(Node::new()));

            node = child;
            node.counter += 1;
        }

        node.end_count += 1;
    }

    pub fn search(&self, x: u64) -> bool {

        let mut node: &Node = &self.root;

        for i in (0..self.bit_size).rev() {

            let bit = ((x >> i) & 1) as usize;

            match node.children[bit].as_deref() {
                Some(child) => node = child,
                None => return false,
            }
        }

        node.is_leaf()
    }

    pub fn count(&self, x: u64) -> usize {

        let mut node: &Node = &self.root;

        for i in (0..self.bit_size).rev() {

            let bit = ((x >> i) & 1) as usize;

            match node.children[bit].as_deref() {
                Some(child) => node = child,
                None => return 0,
            }
        }

        node.end_count
    }

    pub fn remove(&mut self, x: u64) -> bool {
        Self::remove_internal(&mut self.root, x, self.bit_size)
    }

    pub fn erase(&mut self, x: u64) -> usize {

        let delete_count = self.count(x);
        if delete_count == 0 {
            return 0;
        }

        Self::erase_internal(&mut self.root, x, self.bit_size, delete_count);

        delete_count
    }

    pub fn dfs<'a>(&'a self, bit_flg: usize, node: Option<&'a Node>) -> Option<&'a Node> {

        if 1 < bit_flg {
            return None;
        }

        let node = node.unwrap_or(&self.root);
        node.children[bit_flg].as_deref()
    }

    pub fn lcp(&self, x: u64) -> u64 {

        let mut node: &Node = &self.root;
        let mut result = 0;

        for i in (0..self.bit_size).rev() {

            let bit = ((x >> i) & 1) as usize;

            match node.children[bit].as_deref() {
                Some(child) => {
                    node = child;
                    result |= (bit as u64) << i;
                }
                None => return result,
            }
        }

        result
    }

    pub fn min_xor(&self, x: u64) -> u64 {
        if !self.has_stored_value() {
            return 0;
        }

        let mut node: &Node = &self.root;
        let mut result: u64 = 0;

        for i in (0..self.bit_size).rev() {
            let bit = ((x >> i) & 1) as usize;

            match (
                node.children[bit].as_deref(),
                node.children[bit ^ 1].as_deref(),
            ) {
                (None, None) => return result | (x & ((1u64 << i) - 1)),
                (Some(next), _) => node = next,
                (None, Some(next)) => {
                    node = next;
                    result |= 1u64 << i;
                }
            }
        }

        result
    }

    pub fn max_xor(&self, x: u64) -> u64 {
        if !self.has_stored_value() {
            return 0;
        }

        let mut node: &Node = &self.root;
        let mut result: u64 = 0;

        for i in (0..self.bit_size).rev() {
            let desired = ((x >> i) & 1) as usize ^ 1;

            match (
                node.children[desired].as_deref(),
                node.children[desired ^ 1].as_deref(),
            ) {
                (None, None) => return result | (x & ((1u64 << i) - 1)),
                (Some(next), _) => {
                    node = next;
                    result |= 1u64 << i;
                }
                (None, Some(next)) => node = next,
            }
        }

        result
    }

    fn remove_internal(node: &mut Node, x: u64, level: usize) -> bool {

        if level == 0 {

            if node.end_count == 0 {
                return false;
            }

            node.end_count -= 1;

            return true;
        }

        let bit = ((x >> (level - 1)) & 1) as usize;

        match node.children[bit].as_mut() {
            None => false,
            Some(child) => {
                if Self::remove_internal(child, x, level - 1) {
                    child.counter = child.counter.saturating_sub(1);
                    if child.counter == 0 {
                        node.children[bit] = None;
                    }
                    true
                } else {
                    false
                }
            }
        }
    }

    fn erase_internal(node: &mut Node, x: u64, level: usize, delete_count: usize) -> bool {

        if level == 0 {
            node.end_count = node.end_count.saturating_sub(delete_count);
            return true;
        }

        let bit = ((x >> (level - 1)) & 1) as usize;

        match node.children[bit].as_mut() {
            None => false,
            Some(child) => {
                if Self::erase_internal(child, x, level - 1, delete_count) {

                    child.counter = child.counter.saturating_sub(delete_count);
                    if child.counter == 0 {
                        node.children[bit] = None;
                    }

                    true
                } else {
                    false
                }
            }
        }
    }

    fn has_stored_value(&self) -> bool {
        0 < self.root.end_count
            || self.root.children[0].is_some()
            || self.root.children[1].is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::{BinaryTrie, Node};
    use std::collections::HashMap;

    fn bit_mask(bit_size: usize) -> u64 {
        if bit_size == 0 {
            0
        } else if bit_size < 64 {
            (1u64 << bit_size) - 1
        } else {
            u64::MAX
        }
    }

    fn normalize_value(value: u64, bit_size: usize) -> u64 {
        value & bit_mask(bit_size)
    }

    fn create_trie(bit_size: usize, values: &[u64]) -> BinaryTrie {
        let mut trie = BinaryTrie::new(bit_size);
        for &value in values {
            trie.insert(value);
        }
        trie
    }

    fn counter_pruning_path_counts(
        trie: &BinaryTrie,
        value: u64,
    ) -> Option<Vec<(usize, usize)>> {
        let mut node: &Node = &trie.root;
        let mut counts = Vec::with_capacity(trie.bit_size);

        for bit_index in (0..trie.bit_size).rev() {
            let bit = ((value >> bit_index) & 1) as usize;
            node = node.children[bit].as_deref()?;
            counts.push((node.counter, node.end_count));
        }

        Some(counts)
    }

    fn counter_pruning_tree_snapshot(
        trie: &BinaryTrie,
    ) -> Vec<(Vec<usize>, usize, usize, [bool; 2])> {
        fn visit(
            node: &Node,
            path: &mut Vec<usize>,
            snapshot: &mut Vec<(Vec<usize>, usize, usize, [bool; 2])>,
        ) {
            snapshot.push((
                path.clone(),
                node.counter,
                node.end_count,
                [node.children[0].is_some(), node.children[1].is_some()],
            ));

            for bit in 0..=1 {
                if let Some(child) = node.children[bit].as_deref() {
                    path.push(bit);
                    visit(child, path, snapshot);
                    path.pop();
                }
            }
        }

        let mut snapshot = Vec::new();
        visit(&trie.root, &mut Vec::new(), &mut snapshot);
        snapshot
    }

    fn insert_reference_count(reference_counts: &mut HashMap<u64, usize>, key: u64) {
        let current_count = reference_counts.get(&key).copied().unwrap_or(0);
        reference_counts.insert(key, current_count + 1);
    }

    fn remove_one_reference_count(reference_counts: &mut HashMap<u64, usize>, key: u64) -> bool {
        let current_count = reference_counts.get(&key).copied().unwrap_or(0);
        if current_count == 0 {
            return false;
        }

        if current_count == 1 {
            reference_counts.remove(&key);
        } else {
            reference_counts.insert(key, current_count - 1);
        }
        true
    }

    fn erase_all_reference_count(reference_counts: &mut HashMap<u64, usize>, key: u64) -> usize {
        reference_counts.remove(&key).unwrap_or(0)
    }

    fn reference_min_xor(reference_counts: &HashMap<u64, usize>, query: u64) -> Option<u64> {
        reference_counts
            .keys()
            .map(|key| query ^ key)
            .min()
    }

    fn reference_max_xor(reference_counts: &HashMap<u64, usize>, query: u64) -> Option<u64> {
        reference_counts
            .keys()
            .map(|key| query ^ key)
            .max()
    }

    fn reference_lcp(
        reference_counts: &HashMap<u64, usize>,
        query: u64,
        bit_size: usize,
    ) -> u64 {
        let normalized_query = normalize_value(query, bit_size);
        let mut result = 0u64;

        for bit_index in (0..bit_size).rev() {
            let prefix_mask = bit_mask(bit_size) & !bit_mask(bit_index);
            let has_matching_prefix = reference_counts
                .keys()
                .any(|key| ((key ^ normalized_query) & prefix_mask) == 0);
            if !has_matching_prefix {
                break;
            }

            result |= normalized_query & (1u64 << bit_index);
        }

        result
    }

    fn assert_counter_invariants(node: &Node, is_root: bool) -> usize {
        let children_count = node
            .children
            .iter()
            .filter_map(|child| child.as_deref())
            .map(|child| assert_counter_invariants(child, false))
            .sum::<usize>();
        let subtree_count = node.end_count + children_count;

        if is_root {
            assert_eq!(node.counter, 0);
        } else {
            assert_eq!(node.counter, subtree_count);
            assert!(0 < node.counter);
        }

        subtree_count
    }

    type QuerySnapshot = (bool, usize, u64, u64, u64);

    fn reference_queries(
        reference: &HashMap<u64, usize>,
        bit_size: usize,
        queries: &[u64],
    ) -> Vec<QuerySnapshot> {
        queries
            .iter()
            .map(|&query| {
                let normalized = normalize_value(query, bit_size);
                let count = reference.get(&normalized).copied().unwrap_or(0);
                (
                    count > 0,
                    count,
                    reference_lcp(reference, query, bit_size),
                    reference_min_xor(reference, normalized).unwrap_or(0),
                    reference_max_xor(reference, normalized).unwrap_or(0),
                )
            })
            .collect()
    }

    fn actual_queries(trie: &BinaryTrie, queries: &[u64]) -> Vec<QuerySnapshot> {
        queries
            .iter()
            .map(|&query| {
                (
                    trie.search(query),
                    trie.count(query),
                    trie.lcp(query),
                    trie.min_xor(query),
                    trie.max_xor(query),
                )
            })
            .collect()
    }

    #[derive(Clone, Copy)]
    enum ReferenceOperation {
        Insert(u64),
        Remove(u64),
        Erase(u64),
    }

    fn deterministic_operations(bit_size: usize) -> (Vec<ReferenceOperation>, Vec<u64>) {
        let primary = normalize_value(
            (1u64 << bit_size.saturating_sub(1)) | 0b10101,
            bit_size,
        );
        let secondary = if bit_size == 0 {
            u64::MAX
        } else {
            primary ^ 1
        };
        let alias = if bit_size < 64 {
            primary | (1u64 << bit_size)
        } else {
            primary
        };
        let absent = normalize_value(primary.wrapping_add(2), bit_size);

        let mut operations = vec![
            ReferenceOperation::Remove(absent),
            ReferenceOperation::Erase(absent),
            ReferenceOperation::Insert(primary),
            ReferenceOperation::Insert(alias),
        ];
        if bit_size != 0 {
            operations.push(ReferenceOperation::Remove(secondary));
        }
        operations.extend([
            ReferenceOperation::Insert(secondary),
            ReferenceOperation::Remove(alias),
            ReferenceOperation::Erase(secondary),
            ReferenceOperation::Erase(primary),
            ReferenceOperation::Remove(absent),
            ReferenceOperation::Insert(secondary),
        ]);

        let queries = vec![
            0,
            1,
            bit_mask(bit_size),
            primary,
            secondary,
            alias,
            absent,
            u64::MAX,
        ];

        (operations, queries)
    }

    #[test]
    fn node_new_returns_zero_initialized_fields() {
        // Arrange
        let expected_counter = 0;
        let expected_end_count = 0;

        // Act
        let node = Node::new();
        let actual_children = node.children;
        let actual_counter = node.counter;
        let actual_end_count = node.end_count;

        // Assert
        assert!(actual_children[0].is_none());
        assert!(actual_children[1].is_none());
        assert_eq!(actual_counter, expected_counter);
        assert_eq!(actual_end_count, expected_end_count);
    }

    #[test]
    fn node_is_leaf_returns_false_when_end_count_is_zero() {
        // Arrange
        let initial_end_count = 0usize;
        let mut node = Node::new();
        node.end_count = initial_end_count;

        // Act
        let actual_is_leaf = node.is_leaf();

        // Assert
        assert!(!actual_is_leaf);
    }

    #[test]
    fn node_is_leaf_returns_true_when_end_count_is_one() {
        // Arrange
        let initial_end_count = 1usize;
        let mut node = Node::new();
        node.end_count = initial_end_count;

        // Act
        let actual_is_leaf = node.is_leaf();

        // Assert
        assert!(actual_is_leaf);
    }

    #[test]
    fn node_is_leaf_returns_true_when_end_count_is_two_or_more() {
        // Arrange
        let mut node = Node::new();
        node.end_count = 2;

        // Act
        let actual_is_leaf = node.is_leaf();

        // Assert
        assert!(actual_is_leaf);
    }

    #[test]
    fn node_is_leaf_uses_end_count_not_children() {
        // Arrange
        let mut node = Node::new();
        node.children[0] = Some(Box::new(Node::new()));

        // Act
        let actual_is_leaf = node.is_leaf();

        // Assert
        assert!(!actual_is_leaf);
    }

    #[test]
    fn node_default_matches_node_new_initial_state() {
        // Arrange
        let expected_counter = 0usize;
        let expected_end_count = 0usize;
        let new_node = Node::new();

        // Act
        let default_node = Node::default();

        let actual_default_child_absence = default_node
            .children
            .iter()
            .map(Option::is_none)
            .collect::<Vec<_>>();
        let actual_new_child_absence = new_node
            .children
            .iter()
            .map(Option::is_none)
            .collect::<Vec<_>>();

        // Assert
        assert!(default_node.children[0].is_none());
        assert!(default_node.children[1].is_none());
        assert_eq!(default_node.counter, expected_counter);
        assert_eq!(default_node.end_count, expected_end_count);
        assert_eq!(default_node.counter, new_node.counter);
        assert_eq!(default_node.end_count, new_node.end_count);
        assert_eq!(actual_default_child_absence, actual_new_child_absence,);
    }

    #[test]
    fn new_with_bit_size_zero_creates_empty_trie_that_searches_false() {
        // Arrange
        let bit_size = 0usize;
        let query_value = 13u64;
        let trie = BinaryTrie::new(bit_size);

        // Act
        let actual_search_result = trie.search(query_value);

        // Assert
        assert!(!actual_search_result);
    }

    #[test]
    fn new_with_bit_size_one_can_distinguish_zero_and_one() {
        // Arrange
        let bit_size = 1usize;
        let zero_value = 0u64;
        let one_value = 1u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(zero_value);
        trie.insert(one_value);

        // Act
        let actual_search_zero = trie.search(zero_value);
        let actual_search_one = trie.search(one_value);

        // Assert
        assert!(actual_search_zero);
        assert!(actual_search_one);
    }

    #[test]
    fn new_with_bit_size_sixty_four_can_store_u64_min_and_max() {
        // Arrange
        let bit_size = 64usize;
        let min_value = u64::MIN;
        let max_value = u64::MAX;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(min_value);
        trie.insert(max_value);

        // Act
        let actual_search_min = trie.search(min_value);
        let actual_search_max = trie.search(max_value);

        // Assert
        assert!(actual_search_min);
        assert!(actual_search_max);
    }

    #[test]
    fn new_with_bit_size_sixty_five_panics() {
        // Arrange
        let invalid_bit_size = 65usize;
        let expected_message = "bit_size must be 64 or less";

        // Act
        let actual_result = std::panic::catch_unwind(|| BinaryTrie::new(invalid_bit_size));

        // Assert
        let payload = actual_result.err().expect("invalid bit size must panic");
        let message = payload
            .downcast_ref::<String>()
            .map(String::as_str)
            .or_else(|| payload.downcast_ref::<&str>().copied());
        assert_eq!(message, Some(expected_message));
    }

    #[test]
    fn insert_once_makes_search_true_and_count_one() {
        // Arrange
        let bit_size = 8usize;
        let inserted_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);

        let expected_count_right = 1;

        // Act
        trie.insert(inserted_value);
        let actual_search_result = trie.search(inserted_value);
        let actual_count = trie.count(inserted_value);

        // Assert
        assert!(actual_search_result);
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn insert_x_minus_one_x_x_plus_one_makes_each_searchable() {
        // Arrange
        let bit_size = 8usize;
        let center_value = 40u64;
        let left_value = center_value - 1;
        let right_value = center_value + 1;
        let mut trie = BinaryTrie::new(bit_size);

        // Act
        trie.insert(left_value);
        trie.insert(center_value);
        trie.insert(right_value);
        let actual_left_search = trie.search(left_value);
        let actual_center_search = trie.search(center_value);
        let actual_right_search = trie.search(right_value);

        // Assert
        assert!(actual_left_search);
        assert!(actual_center_search);
        assert!(actual_right_search);
    }

    #[test]
    fn insert_twice_increments_count_to_two() {
        // Arrange
        let bit_size = 8usize;
        let inserted_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);

        let expected_count_right = 2;

        // Act
        trie.insert(inserted_value);
        trie.insert(inserted_value);
        let actual_count = trie.count(inserted_value);

        // Assert
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn insert_distinct_values_does_not_interfere_counts() {
        // Arrange
        let bit_size = 8usize;
        let first_value = 42u64;
        let second_value = 43u64;
        let mut trie = BinaryTrie::new(bit_size);

        let expected_first_count_right = 1;
        let expected_second_count_right = 1;

        // Act
        trie.insert(first_value);
        trie.insert(second_value);
        let actual_first_count = trie.count(first_value);
        let actual_second_count = trie.count(second_value);

        // Assert
        assert_eq!(actual_first_count, expected_first_count_right);
        assert_eq!(actual_second_count, expected_second_count_right);
    }

    #[test]
    fn insert_updates_each_node_counter_to_its_subtree_multiplicity() {
        // Arrange
        let bit_size = 4usize;
        let duplicate_value = 0b1010u64;
        let shared_prefix_value = 0b1011u64;
        let other_branch_value = 0b0010u64;
        let values = [
            duplicate_value,
            duplicate_value,
            shared_prefix_value,
            other_branch_value,
        ];
        let expected_duplicate_path = vec![(3, 0), (3, 0), (3, 0), (2, 2)];
        let expected_shared_prefix_path = vec![(3, 0), (3, 0), (3, 0), (1, 1)];
        let expected_other_branch_path = vec![(1, 0), (1, 0), (1, 0), (1, 1)];
        let expected_total_count = values.len();

        // Act
        let trie = create_trie(bit_size, &values);
        let actual_duplicate_path = counter_pruning_path_counts(&trie, duplicate_value)
            .expect("duplicate value path must exist");
        let actual_shared_prefix_path = counter_pruning_path_counts(&trie, shared_prefix_value)
            .expect("shared-prefix value path must exist");
        let actual_other_branch_path = counter_pruning_path_counts(&trie, other_branch_value)
            .expect("other branch value path must exist");
        let actual_total_count = assert_counter_invariants(&trie.root, true);

        // Assert
        assert_eq!(actual_duplicate_path, expected_duplicate_path);
        assert_eq!(actual_shared_prefix_path, expected_shared_prefix_path);
        assert_eq!(actual_other_branch_path, expected_other_branch_path);
        assert_eq!(actual_total_count, expected_total_count);
    }

    #[test]
    fn search_returns_false_for_empty_trie() {
        // Arrange
        let bit_size = 8usize;
        let query_value = 123u64;
        let trie = BinaryTrie::new(bit_size);

        // Act
        let actual_search_result = trie.search(query_value);

        // Assert
        assert!(!actual_search_result);
    }

    #[test]
    fn count_returns_zero_for_empty_trie() {
        // Arrange
        let bit_size = 8usize;
        let query_value = 123u64;
        let trie = BinaryTrie::new(bit_size);

        let expected_count_right = 0;

        // Act
        let actual_count = trie.count(query_value);

        // Assert
        assert_eq!(actual_count, expected_count_right);
    }

    #[test]
    fn remove_single_existing_value_clears_search_hit() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(target_value);

        // Act
        let actual_removed = trie.remove(target_value);
        let actual_search_after_remove = trie.search(target_value);

        // Assert
        assert!(actual_removed);
        assert!(!actual_search_after_remove);
    }

    #[test]
    fn remove_non_existing_value_returns_false_and_keeps_existing_count() {
        // Arrange
        let bit_size = 8usize;
        let existing_value = 42u64;
        let absent_value = 43u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(existing_value);

        let expected_existing_count_right = 1;

        // Act
        let actual_removed = trie.remove(absent_value);
        let actual_existing_count = trie.count(existing_value);

        // Assert
        assert!(!actual_removed);
        assert_eq!(actual_existing_count, expected_existing_count_right);
    }

    #[test]
    fn remove_one_from_duplicate_value_decrements_count_by_one() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(target_value);
        trie.insert(target_value);

        let expected_count_after_remove_right = 1;

        // Act
        let actual_removed = trie.remove(target_value);
        let actual_count_after_remove = trie.count(target_value);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_count_after_remove, expected_count_after_remove_right);
    }

    #[test]
    fn remove_one_duplicate_decrements_path_counters_without_pruning() {
        // Arrange
        let bit_size = 4usize;
        let duplicate_value = 0b1010u64;
        let shared_prefix_value = 0b1011u64;
        let values = [duplicate_value, duplicate_value, shared_prefix_value];
        let expected_duplicate_path = vec![(2, 0), (2, 0), (2, 0), (1, 1)];
        let expected_shared_prefix_path = vec![(2, 0), (2, 0), (2, 0), (1, 1)];
        let expected_duplicate_count = 1usize;
        let mut trie = create_trie(bit_size, &values);

        // Act
        let actual_removed = trie.remove(duplicate_value);
        let actual_duplicate_path = counter_pruning_path_counts(&trie, duplicate_value)
            .expect("one duplicate path must remain");
        let actual_shared_prefix_path = counter_pruning_path_counts(&trie, shared_prefix_value)
            .expect("shared-prefix path must remain");
        let actual_duplicate_count = trie.count(duplicate_value);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_duplicate_path, expected_duplicate_path);
        assert_eq!(actual_shared_prefix_path, expected_shared_prefix_path);
        assert_eq!(actual_duplicate_count, expected_duplicate_count);
    }

    #[test]
    fn remove_and_erase_accept_aliases_above_bit_size_and_prune_key_path() {
        // Arrange
        let bit_size = 4usize;
        let normalized_value = 0b1010u64;
        let inserted_values = [
            normalized_value,
            normalized_value | (1u64 << bit_size),
            normalized_value | (1u64 << 63),
        ];
        let remove_alias = normalized_value | (1u64 << 20);
        let erase_alias = normalized_value | (1u64 << 40);
        let mut trie = create_trie(bit_size, &inserted_values);

        let expected_count_after_remove_right = 2;
        let expected_erased_count_right = 2;
        let expected_count_after_erase_right = 0;

        let dfs_branch = 1;
        let dfs_node = None;

        // Act
        let actual_removed = trie.remove(remove_alias);
        let actual_count_after_remove = trie.count(normalized_value);
        let actual_erased_count = trie.erase(erase_alias);
        let actual_count_after_erase = trie.count(normalized_value);
        let actual_root_one_child = trie.dfs(dfs_branch, dfs_node);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_count_after_remove, expected_count_after_remove_right);
        assert_eq!(actual_erased_count, expected_erased_count_right);
        assert_eq!(actual_count_after_erase, expected_count_after_erase_right);
        assert!(actual_root_one_child.is_none());
    }

    #[test]
    fn remove_from_empty_trie_returns_false() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 55u64;
        let mut trie = BinaryTrie::new(bit_size);

        // Act
        let actual_removed = trie.remove(target_value);

        // Assert
        assert!(!actual_removed);
    }

    #[test]
    fn remove_duplicate_value_twice_then_third_time_returns_false() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(target_value);
        trie.insert(target_value);

        // Act
        let actual_first_removed = trie.remove(target_value);
        let actual_second_removed = trie.remove(target_value);
        let actual_third_removed = trie.remove(target_value);

        // Assert
        assert!(actual_first_removed);
        assert!(actual_second_removed);
        assert!(!actual_third_removed);
    }

    #[test]
    fn remove_one_value_with_shared_prefix_keeps_other_value() {
        // Arrange
        let bit_size = 4usize;
        let removed_value = 0b1010u64;
        let kept_value = 0b1011u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(removed_value);
        trie.insert(kept_value);

        // Act
        let actual_removed = trie.remove(removed_value);
        let actual_kept_search = trie.search(kept_value);

        // Assert
        assert!(actual_removed);
        assert!(actual_kept_search);
    }

    #[test]
    fn remove_center_of_x_minus_one_x_x_plus_one_keeps_neighbors() {
        // Arrange
        let bit_size = 8usize;
        let center_value = 100u64;
        let left_value = center_value - 1;
        let right_value = center_value + 1;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(left_value);
        trie.insert(center_value);
        trie.insert(right_value);

        // Act
        let actual_removed = trie.remove(center_value);
        let actual_left_search = trie.search(left_value);
        let actual_right_search = trie.search(right_value);

        // Assert
        assert!(actual_removed);
        assert!(actual_left_search);
        assert!(actual_right_search);
    }

    #[test]
    fn remove_last_shared_prefix_value_prunes_only_its_unique_branch() {
        // Arrange
        let bit_size = 4usize;
        let removed_value = 0b1011u64;
        let kept_value = 0b1010u64;
        let expected_kept_path = vec![(1, 0), (1, 0), (1, 0), (1, 1)];
        let expected_kept_count = 1usize;
        let expected_lcp = kept_value;
        let expected_xor = removed_value ^ kept_value;
        let mut trie = create_trie(bit_size, &[removed_value, kept_value]);

        // Act
        let actual_removed = trie.remove(removed_value);
        let actual_removed_path = counter_pruning_path_counts(&trie, removed_value);
        let actual_kept_path = counter_pruning_path_counts(&trie, kept_value)
            .expect("kept value path must remain");
        let actual_kept_search = trie.search(kept_value);
        let actual_kept_count = trie.count(kept_value);
        let actual_lcp = trie.lcp(removed_value);
        let actual_min_xor = trie.min_xor(removed_value);
        let actual_max_xor = trie.max_xor(removed_value);

        // Assert
        assert!(actual_removed);
        assert!(actual_removed_path.is_none());
        assert_eq!(actual_kept_path, expected_kept_path);
        assert!(actual_kept_search);
        assert_eq!(actual_kept_count, expected_kept_count);
        assert_eq!(actual_lcp, expected_lcp);
        assert_eq!(actual_min_xor, expected_xor);
        assert_eq!(actual_max_xor, expected_xor);
    }

    #[test]
    fn remove_last_stored_item_restores_observably_empty_trie() {
        // Arrange
        let bit_size = 4usize;
        let stored_value = 0b1011u64;
        let zero_bit = 0usize;
        let one_bit = 1usize;
        let expected_count = 0usize;
        let expected_empty_result = 0u64;
        let mut trie = create_trie(bit_size, &[stored_value]);

        let dfs_node = None;

        // Act
        let actual_removed = trie.remove(stored_value);
        let actual_zero_child = trie.dfs(zero_bit, dfs_node);
        let actual_one_child = trie.dfs(one_bit, dfs_node);
        let actual_search = trie.search(stored_value);
        let actual_count = trie.count(stored_value);
        let actual_lcp = trie.lcp(stored_value);
        let actual_min_xor = trie.min_xor(stored_value);
        let actual_max_xor = trie.max_xor(stored_value);

        // Assert
        assert!(actual_removed);
        assert!(actual_zero_child.is_none());
        assert!(actual_one_child.is_none());
        assert!(!actual_search);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_lcp, expected_empty_result);
        assert_eq!(actual_min_xor, expected_empty_result);
        assert_eq!(actual_max_xor, expected_empty_result);
    }

    #[test]
    fn erase_non_existing_value_returns_zero_and_keeps_existing_count() {
        // Arrange
        let bit_size = 8usize;
        let existing_value = 42u64;
        let absent_value = 43u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(existing_value);

        let expected_erased_count_right = 0;
        let expected_existing_count_right = 1;

        // Act
        let actual_erased_count = trie.erase(absent_value);
        let actual_existing_count = trie.count(existing_value);

        // Assert
        assert_eq!(actual_erased_count, expected_erased_count_right);
        assert_eq!(actual_existing_count, expected_existing_count_right);
    }

    #[test]
    fn erase_single_existing_value_returns_one_and_clears_count() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(target_value);

        let expected_erased_count_right = 1;
        let expected_count_after_erase_right = 0;

        // Act
        let actual_erased_count = trie.erase(target_value);
        let actual_count_after_erase = trie.count(target_value);

        // Assert
        assert_eq!(actual_erased_count, expected_erased_count_right);
        assert_eq!(actual_count_after_erase, expected_count_after_erase_right);
    }

    #[test]
    fn erase_duplicate_value_returns_all_count_and_removes_all() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 42u64;
        let duplicate_count = 3usize;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(target_value);
        trie.insert(target_value);
        trie.insert(target_value);

        let expected_count_after_erase_right = 0;

        // Act
        let actual_erased_count = trie.erase(target_value);
        let actual_count_after_erase = trie.count(target_value);

        // Assert
        assert_eq!(actual_erased_count, duplicate_count);
        assert_eq!(actual_count_after_erase, expected_count_after_erase_right);
    }

    #[test]
    fn erase_duplicates_decrements_counters_by_all_removed_items_and_keeps_sibling() {
        // Arrange
        let bit_size = 4usize;
        let erased_value = 0b1011u64;
        let kept_value = 0b1010u64;
        let erased_count = 3usize;
        let values = [erased_value, erased_value, erased_value, kept_value];
        let expected_kept_path = vec![(1, 0), (1, 0), (1, 0), (1, 1)];
        let expected_kept_count = 1usize;
        let mut trie = create_trie(bit_size, &values);

        // Act
        let actual_erased_count = trie.erase(erased_value);
        let actual_erased_path = counter_pruning_path_counts(&trie, erased_value);
        let actual_kept_path = counter_pruning_path_counts(&trie, kept_value)
            .expect("kept sibling path must remain");
        let actual_kept_search = trie.search(kept_value);
        let actual_kept_count = trie.count(kept_value);

        // Assert
        assert_eq!(actual_erased_count, erased_count);
        assert!(actual_erased_path.is_none());
        assert_eq!(actual_kept_path, expected_kept_path);
        assert!(actual_kept_search);
        assert_eq!(actual_kept_count, expected_kept_count);
    }

    #[test]
    fn erase_from_empty_trie_returns_zero() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 77u64;
        let mut trie = BinaryTrie::new(bit_size);

        let expected_erased_count_right = 0;

        // Act
        let actual_erased_count = trie.erase(target_value);

        // Assert
        assert_eq!(actual_erased_count, expected_erased_count_right);
    }

    #[test]
    fn erase_one_value_with_shared_prefix_keeps_other_value() {
        // Arrange
        let bit_size = 4usize;
        let erased_value = 0b1010u64;
        let kept_value = 0b1011u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(erased_value);
        trie.insert(kept_value);

        let expected_erased_count_right = 1;

        // Act
        let actual_erased_count = trie.erase(erased_value);
        let actual_kept_search = trie.search(kept_value);

        // Assert
        assert_eq!(actual_erased_count, expected_erased_count_right);
        assert!(actual_kept_search);
    }

    #[test]
    fn erase_all_stored_duplicates_prunes_every_path_to_root() {
        // Arrange
        let bit_size = 4usize;
        let stored_value = 0b1011u64;
        let duplicate_count = 3usize;
        let zero_bit = 0usize;
        let one_bit = 1usize;
        let expected_empty_result = 0u64;
        let mut trie = create_trie(bit_size, &[stored_value, stored_value, stored_value]);

        let dfs_node = None;

        // Act
        let actual_erased_count = trie.erase(stored_value);
        let actual_zero_child = trie.dfs(zero_bit, dfs_node);
        let actual_one_child = trie.dfs(one_bit, dfs_node);
        let actual_lcp = trie.lcp(stored_value);
        let actual_min_xor = trie.min_xor(stored_value);
        let actual_max_xor = trie.max_xor(stored_value);

        // Assert
        assert_eq!(actual_erased_count, duplicate_count);
        assert!(actual_zero_child.is_none());
        assert!(actual_one_child.is_none());
        assert_eq!(actual_lcp, expected_empty_result);
        assert_eq!(actual_min_xor, expected_empty_result);
        assert_eq!(actual_max_xor, expected_empty_result);
    }

    #[test]
    fn dfs_on_empty_trie_returns_none_for_zero_and_one_children() {
        // Arrange
        let bit_size = 4usize;
        let trie = BinaryTrie::new(bit_size);
        let zero_bit = 0usize;
        let one_bit = 1usize;

        let dfs_node = None;

        // Act
        let actual_zero_child = trie.dfs(zero_bit, dfs_node);
        let actual_one_child = trie.dfs(one_bit, dfs_node);

        // Assert
        assert!(actual_zero_child.is_none());
        assert!(actual_one_child.is_none());
    }

    #[test]
    fn dfs_returns_none_when_bit_flag_is_two_or_more() {
        // Arrange
        let bit_size = 4usize;
        let invalid_bit_flag = 2usize;
        let trie = BinaryTrie::new(bit_size);

        let dfs_node = None;

        // Act
        let actual_child = trie.dfs(invalid_bit_flag, dfs_node);

        // Assert
        assert!(actual_child.is_none());
    }

    #[test]
    fn dfs_on_root_returns_existing_child_node() {
        // Arrange
        let bit_size = 3usize;
        let inserted_value = 0b101u64;
        let expected_root_child_bit = 1usize;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(inserted_value);

        let dfs_node = None;

        // Act
        let actual_child = trie.dfs(expected_root_child_bit, dfs_node);

        // Assert
        assert!(actual_child.is_some());
    }

    #[test]
    fn dfs_with_some_node_traverses_from_given_node() {
        // Arrange
        let bit_size = 3usize;
        let first_value = 0b010u64;
        let second_value = 0b011u64;
        let root_child_bit = 0usize;
        let nested_child_bit = 1usize;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(first_value);
        trie.insert(second_value);
        let root_child = trie.dfs(root_child_bit, None).expect("root child must exist");

        // Act
        let actual_nested_child = trie.dfs(nested_child_bit, Some(root_child));

        // Assert
        assert!(actual_nested_child.is_some());
    }

    #[test]
    fn dfs_returns_none_for_non_existing_branch() {
        // Arrange
        let bit_size = 3usize;
        let inserted_value = 0b101u64;
        let absent_root_child_bit = 0usize;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(inserted_value);

        let dfs_node = None;

        // Act
        let actual_absent_child = trie.dfs(absent_root_child_bit, dfs_node);

        // Assert
        assert!(actual_absent_child.is_none());
    }

    #[test]
    fn lcp_normalizes_query_bits_above_bit_size() {
        // Arrange
        let bit_size = 5usize;
        let stored_value = 0b10110u64;
        let normalized_query = stored_value;
        let aliased_query = stored_value | (1u64 << 20) | (1u64 << 63);
        let trie = create_trie(bit_size, &[stored_value]);

        // Act
        let actual_normalized_lcp = trie.lcp(normalized_query);
        let actual_aliased_lcp = trie.lcp(aliased_query);

        // Assert
        assert_eq!(actual_normalized_lcp, stored_value);
        assert_eq!(actual_aliased_lcp, stored_value);
    }

    #[test]
    fn lcp_returns_zero_for_empty_trie() {
        // Arrange
        let bit_size = 8usize;
        let query_value = 42u64;
        let trie = BinaryTrie::new(bit_size);

        let expected_lcp_right = 0;

        // Act
        let actual_lcp = trie.lcp(query_value);

        // Assert
        assert_eq!(actual_lcp, expected_lcp_right);
    }

    #[test]
    fn lcp_returns_query_prefix_when_fully_matched() {
        // Arrange
        let bit_size = 4usize;
        let inserted_value = 0b1010u64;
        let expected_lcp = inserted_value;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(inserted_value);

        // Act
        let actual_lcp = trie.lcp(inserted_value);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn lcp_returns_only_matched_prefix_bits_on_partial_match() {
        // Arrange
        let bit_size = 4usize;
        let inserted_value = 0b1011u64;
        let query_value = 0b1001u64;
        let expected_lcp = 0b1000u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(inserted_value);

        // Act
        let actual_lcp = trie.lcp(query_value);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn lcp_returns_zero_when_first_bit_mismatches() {
        // Arrange
        let bit_size = 4usize;
        let inserted_value = 0b1000u64;
        let query_value = 0b0001u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(inserted_value);

        let expected_lcp_right = 0;

        // Act
        let actual_lcp = trie.lcp(query_value);

        // Assert
        assert_eq!(actual_lcp, expected_lcp_right);
    }

    #[test]
    fn lcp_for_x_minus_one_x_x_plus_one_changes_with_prefix_match_length() {
        // Arrange
        let bit_size = 4usize;
        let center_value = 0b1010u64;
        let left_value = center_value - 1;
        let right_value = center_value + 1;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(center_value);

        let expected_left_lcp_right = 0b1000;
        let expected_center_lcp_right = 0b1010;
        let expected_right_lcp_right = 0b1010;

        // Act
        let actual_left_lcp = trie.lcp(left_value);
        let actual_center_lcp = trie.lcp(center_value);
        let actual_right_lcp = trie.lcp(right_value);

        // Assert
        assert_eq!(actual_left_lcp, expected_left_lcp_right);
        assert_eq!(actual_center_lcp, expected_center_lcp_right);
        assert_eq!(actual_right_lcp, expected_right_lcp_right);
    }

    #[test]
    fn lcp_for_multiple_prefix_lengths_matches_reference_for_all_queries() {
        // Arrange
        let bit_size = 6usize;
        let stored_values = [
            0b000000u64,
            0b001100u64,
            0b010101u64,
            0b100000u64,
            0b101110u64,
            0b111111u64,
        ];
        let query_values = (0..=bit_mask(bit_size)).collect::<Vec<_>>();
        let trie = create_trie(bit_size, &stored_values);
        let mut reference_counts = HashMap::<u64, usize>::new();
        for &value in &stored_values {
            insert_reference_count(&mut reference_counts, normalize_value(value, bit_size));
        }
        let expected_lcps = query_values
            .iter()
            .map(|&query| reference_lcp(&reference_counts, query, bit_size))
            .collect::<Vec<_>>();

        // Act
        let actual_lcps = query_values
            .iter()
            .map(|&query| trie.lcp(query))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_lcps, expected_lcps);
    }

    #[test]
    fn min_xor_returns_zero_for_empty_trie() {
        // Arrange
        let bit_size = 8usize;
        let query_value = 123u64;
        let trie = BinaryTrie::new(bit_size);

        let expected_min_xor_right = 0;

        // Act
        let actual_min_xor = trie.min_xor(query_value);

        // Assert
        assert_eq!(actual_min_xor, expected_min_xor_right);
    }

    #[test]
    fn min_xor_for_single_value_equals_query_xor_value() {
        // Arrange
        let bit_size = 8usize;
        let stored_value = 77u64;
        let query_value = 123u64;
        let expected_min_xor = query_value ^ stored_value;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(stored_value);

        // Act
        let actual_min_xor = trie.min_xor(query_value);

        // Assert
        assert_eq!(actual_min_xor, expected_min_xor);
    }

    #[test]
    fn min_xor_for_multiple_values_returns_global_minimum_xor() {
        // Arrange
        let bit_size = 8usize;
        let stored_values = [1u64, 9u64, 15u64, 31u64];
        let query_value = 10u64;
        let trie = create_trie(bit_size, &stored_values);
        let expected_min_xor = stored_values
            .iter()
            .map(|value| query_value ^ value)
            .min()
            .unwrap_or(0);

        // Act
        let actual_min_xor = trie.min_xor(query_value);

        // Assert
        assert_eq!(actual_min_xor, expected_min_xor);
    }

    #[test]
    fn min_xor_for_x_minus_one_x_x_plus_one_queries_matches_reference() {
        // Arrange
        let bit_size = 8usize;
        let query_center = 20u64;
        let query_left = query_center - 1;
        let query_right = query_center + 1;
        let stored_values = [5u64, 10u64, 17u64];
        let trie = create_trie(bit_size, &stored_values);
        let expected_left = stored_values.iter().map(|value| query_left ^ value).min().unwrap_or(0);
        let expected_center = stored_values.iter().map(|value| query_center ^ value).min().unwrap_or(0);
        let expected_right = stored_values.iter().map(|value| query_right ^ value).min().unwrap_or(0);

        // Act
        let actual_left = trie.min_xor(query_left);
        let actual_center = trie.min_xor(query_center);
        let actual_right = trie.min_xor(query_right);

        // Assert
        assert_eq!(actual_left, expected_left);
        assert_eq!(actual_center, expected_center);
        assert_eq!(actual_right, expected_right);
    }

    #[test]
    fn min_xor_returns_same_result_for_values_sharing_lower_bits() {
        // Arrange
        let bit_size = 4usize;
        let first_value = 0b0011u64;
        let second_value = 0b10011u64;
        let first_query = 0b0010u64;
        let second_query = 0b10010u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(first_value);
        trie.insert(second_value);

        // Act
        let actual_first_query_min = trie.min_xor(first_query);
        let actual_second_query_min = trie.min_xor(second_query);

        // Assert
        assert_eq!(actual_first_query_min, actual_second_query_min);
    }

    #[test]
    fn max_xor_returns_zero_for_empty_trie() {
        // Arrange
        let bit_size = 8usize;
        let query_value = 123u64;
        let trie = BinaryTrie::new(bit_size);

        let expected_max_xor_right = 0;

        // Act
        let actual_max_xor = trie.max_xor(query_value);

        // Assert
        assert_eq!(actual_max_xor, expected_max_xor_right);
    }

    #[test]
    fn max_xor_for_single_value_equals_query_xor_value() {
        // Arrange
        let bit_size = 8usize;
        let stored_value = 77u64;
        let query_value = 123u64;
        let expected_max_xor = query_value ^ stored_value;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(stored_value);

        // Act
        let actual_max_xor = trie.max_xor(query_value);

        // Assert
        assert_eq!(actual_max_xor, expected_max_xor);
    }

    #[test]
    fn max_xor_for_multiple_values_returns_global_maximum_xor() {
        // Arrange
        let bit_size = 8usize;
        let stored_values = [1u64, 9u64, 15u64, 31u64];
        let query_value = 10u64;
        let trie = create_trie(bit_size, &stored_values);
        let expected_max_xor = stored_values
            .iter()
            .map(|value| query_value ^ value)
            .max()
            .unwrap_or(0);

        // Act
        let actual_max_xor = trie.max_xor(query_value);

        // Assert
        assert_eq!(actual_max_xor, expected_max_xor);
    }

    #[test]
    fn max_xor_for_x_minus_one_x_x_plus_one_queries_matches_reference() {
        // Arrange
        let bit_size = 8usize;
        let query_center = 20u64;
        let query_left = query_center - 1;
        let query_right = query_center + 1;
        let stored_values = [5u64, 10u64, 17u64];
        let trie = create_trie(bit_size, &stored_values);
        let expected_left = stored_values.iter().map(|value| query_left ^ value).max().unwrap_or(0);
        let expected_center = stored_values.iter().map(|value| query_center ^ value).max().unwrap_or(0);
        let expected_right = stored_values.iter().map(|value| query_right ^ value).max().unwrap_or(0);

        // Act
        let actual_left = trie.max_xor(query_left);
        let actual_center = trie.max_xor(query_center);
        let actual_right = trie.max_xor(query_right);

        // Assert
        assert_eq!(actual_left, expected_left);
        assert_eq!(actual_center, expected_center);
        assert_eq!(actual_right, expected_right);
    }

    #[test]
    fn max_xor_returns_same_result_for_values_sharing_lower_bits() {
        // Arrange
        let bit_size = 4usize;
        let first_value = 0b0011u64;
        let second_value = 0b10011u64;
        let first_query = 0b0010u64;
        let second_query = 0b10010u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(first_value);
        trie.insert(second_value);

        // Act
        let actual_first_query_max = trie.max_xor(first_query);
        let actual_second_query_max = trie.max_xor(second_query);

        // Assert
        assert_eq!(actual_first_query_max, actual_second_query_max);
    }

    #[test]
    fn search_count_remove_erase_match_reference_on_empty_single_and_multi_sets() {
        // Arrange
        let bit_size = 6usize;
        let cases = [
            (vec![], 63u64),
            (vec![17u64], 17u64),
            (vec![1u64, 2, 2, 3], 2u64),
        ];

        for (values, query) in cases {
            // Arrange
            let mut trie = create_trie(bit_size, &values);
            let mut reference = HashMap::<u64, usize>::new();
            for value in values {
                insert_reference_count(&mut reference, normalize_value(value, bit_size));
            }
            let key = normalize_value(query, bit_size);
            let before_count = reference.get(&key).copied().unwrap_or(0);
            let removed = remove_one_reference_count(&mut reference, key);
            let after_remove_count = reference.get(&key).copied().unwrap_or(0);
            let erased = erase_all_reference_count(&mut reference, key);
            let after_erase_count = reference.get(&key).copied().unwrap_or(0);
            let expected = (
                before_count > 0,
                before_count,
                removed,
                after_remove_count > 0,
                after_remove_count,
                erased,
                after_erase_count,
            );

            // Act
            let before_search = trie.search(query);
            let before_count = trie.count(query);
            let removed = trie.remove(query);
            let after_remove_search = trie.search(query);
            let after_remove_count = trie.count(query);
            let erased = trie.erase(query);
            let after_erase_count = trie.count(query);
            let actual = (
                before_search,
                before_count,
                removed,
                after_remove_search,
                after_remove_count,
                erased,
                after_erase_count,
            );

            // Assert
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn min_xor_and_max_xor_match_reference_after_mixed_operations() {
        // Arrange
        let bit_size = 6usize;
        let initial_values = [1u64, 4u64, 7u64, 7u64];
        let inserted_value = 2u64;
        let removed_once_value = 7u64;
        let erased_value = 4u64;
        let query_values = [0u64, 3u64, 6u64];
        let mut trie = create_trie(bit_size, &initial_values);
        let mut reference_counts = HashMap::<u64, usize>::new();
        for &value in &initial_values {
            insert_reference_count(&mut reference_counts, normalize_value(value, bit_size));
        }

        // Act
        trie.insert(inserted_value);
        insert_reference_count(&mut reference_counts, normalize_value(inserted_value, bit_size));

        let _actual_removed_once = trie.remove(removed_once_value);
        let _reference_removed_once = remove_one_reference_count(
            &mut reference_counts,
            normalize_value(removed_once_value, bit_size),
        );

        let _actual_erased_count = trie.erase(erased_value);
        let _reference_erased_count = erase_all_reference_count(
            &mut reference_counts,
            normalize_value(erased_value, bit_size),
        );

        let mut actual_min_xors = Vec::new();
        let mut actual_max_xors = Vec::new();
        let mut expected_min_xors = Vec::new();
        let mut expected_max_xors = Vec::new();
        for &query in &query_values {
            let normalized_query = normalize_value(query, bit_size);
            actual_min_xors.push(trie.min_xor(normalized_query));
            actual_max_xors.push(trie.max_xor(normalized_query));
            expected_min_xors.push(reference_min_xor(&reference_counts, normalized_query).unwrap_or(0));
            expected_max_xors.push(reference_max_xor(&reference_counts, normalized_query).unwrap_or(0));
        }

        // Assert
        assert_eq!(actual_min_xors, expected_min_xors);
        assert_eq!(actual_max_xors, expected_max_xors);
    }

    #[test]
    fn zero_bit_trie_normalizes_all_values_to_empty_key() {
        // Arrange
        let raw_values = [0u64, 1u64, u64::MAX];
        let query_values = [0u64, 42u64, u64::MAX];
        let mut trie = BinaryTrie::new(0);
        for &value in &raw_values {
            trie.insert(value);
        }

        let expected_searches_right = [true; 3];
        let expected_lcp_right = 0;
        let expected_min_xor_right = 0;
        let expected_max_xor_right = 0;

        let dfs_branch = 0;
        let dfs_node = None;
        let dfs_branch_2 = 1;
        let lcp_query = 42;
        let min_xor_query = 42;
        let max_xor_query = 42;

        // Act
        let actual_searches = query_values.map(|query| trie.search(query));
        let actual_counts = query_values.map(|query| trie.count(query));
        let actual_zero_child = trie.dfs(dfs_branch, dfs_node);
        let actual_one_child = trie.dfs(dfs_branch_2, dfs_node);
        let actual_lcp = trie.lcp(lcp_query);
        let actual_min_xor = trie.min_xor(min_xor_query);
        let actual_max_xor = trie.max_xor(max_xor_query);

        let actual_counts_right = [raw_values.len(); 3];

        // Assert
        assert_eq!(actual_searches, expected_searches_right);
        assert_eq!(actual_counts, actual_counts_right);
        assert!(actual_zero_child.is_none());
        assert!(actual_one_child.is_none());
        assert_eq!(actual_lcp, expected_lcp_right);
        assert_eq!(actual_min_xor, expected_min_xor_right);
        assert_eq!(actual_max_xor, expected_max_xor_right);
    }

    #[test]
    fn zero_bit_trie_remove_and_erase_use_shared_empty_key_count() {
        // Arrange
        let mut trie = create_trie(0, &[1u64, 2u64, 3u64]);

        let expected_count_after_remove_right = 2;
        let expected_erased_count_right = 2;
        let expected_count_after_erase_right = 0;

        let remove_key = u64::MAX;
        let count_key = 99;
        let erase_key = 42;
        let search_key = 0;
        let count_key_2 = u64::MAX;

        // Act
        let actual_removed = trie.remove(remove_key);
        let actual_count_after_remove = trie.count(count_key);
        let actual_erased_count = trie.erase(erase_key);
        let actual_search_after_erase = trie.search(search_key);
        let actual_count_after_erase = trie.count(count_key_2);

        // Assert
        assert!(actual_removed);
        assert_eq!(actual_count_after_remove, expected_count_after_remove_right);
        assert_eq!(actual_erased_count, expected_erased_count_right);
        assert!(!actual_search_after_erase);
        assert_eq!(actual_count_after_erase, expected_count_after_erase_right);
    }

    #[test]
    fn sixty_four_bit_trie_boundary_queries_match_reference() {
        // Arrange
        let stored_values = [0u64, 1u64 << 63, u64::MAX];
        let query_values = [0u64, 1u64 << 63, u64::MAX];
        let trie = create_trie(64, &stored_values);
        let expected_min_xors = query_values.map(|query| {
            stored_values
                .iter()
                .map(|stored| query ^ *stored)
                .min()
                .unwrap_or(0)
        });
        let expected_max_xors = query_values.map(|query| {
            stored_values
                .iter()
                .map(|stored| query ^ *stored)
                .max()
                .unwrap_or(0)
        });

        let expected_searches_right = [true; 3];
        let expected_counts_right = [1; 3];

        // Act
        let actual_searches = query_values.map(|query| trie.search(query));
        let actual_counts = query_values.map(|query| trie.count(query));
        let actual_min_xors = query_values.map(|query| trie.min_xor(query));
        let actual_max_xors = query_values.map(|query| trie.max_xor(query));

        // Assert
        assert_eq!(actual_searches, expected_searches_right);
        assert_eq!(actual_counts, expected_counts_right);
        assert_eq!(actual_min_xors, expected_min_xors);
        assert_eq!(actual_max_xors, expected_max_xors);
    }

    #[test]
    fn sixty_four_bit_trie_lcp_returns_matching_most_significant_prefix() {
        // Arrange
        let stored_value = 0xB000_0000_0000_0000u64;
        let query_value = 0xAFFF_FFFF_FFFF_FFFFu64;
        let expected_lcp = 0xA000_0000_0000_0000u64;
        let trie = create_trie(64, &[stored_value]);

        // Act
        let actual_lcp = trie.lcp(query_value);

        // Assert
        assert_eq!(actual_lcp, expected_lcp);
    }

    #[test]
    fn mixed_remove_and_erase_keeps_total_count_consistent() {
        // Arrange
        let bit_size = 8usize;
        let target_value = 42u64;
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(target_value);
        trie.insert(target_value);
        trie.insert(target_value);

        let expected_erased_rest_right = 2;
        let expected_final_count_right = 0;

        // Act
        let actual_removed_once = trie.remove(target_value);
        let actual_erased_rest = trie.erase(target_value);
        let actual_final_count = trie.count(target_value);

        // Assert
        assert!(actual_removed_once);
        assert_eq!(actual_erased_rest, expected_erased_rest_right);
        assert_eq!(actual_final_count, expected_final_count_right);
    }

    #[test]
    fn absent_remove_and_erase_leave_structure_and_queries_unchanged() {
        // Arrange
        let bit_size = 4usize;
        let values = [0b0010u64, 0b1010u64, 0b1010u64, 0b1011u64];
        let absent_value = 0b1111u64;
        let query_values = [0b0000u64, 0b1010u64, absent_value];
        let mut trie = create_trie(bit_size, &values);
        let expected_snapshot = counter_pruning_tree_snapshot(&trie);
        let expected_lcps = query_values.map(|query| trie.lcp(query));
        let expected_min_xors = query_values.map(|query| trie.min_xor(query));
        let expected_max_xors = query_values.map(|query| trie.max_xor(query));

        let expected_erased_count_right = 0;

        // Act
        let actual_removed = trie.remove(absent_value);
        let actual_erased_count = trie.erase(absent_value);
        let actual_snapshot = counter_pruning_tree_snapshot(&trie);
        let actual_lcps = query_values.map(|query| trie.lcp(query));
        let actual_min_xors = query_values.map(|query| trie.min_xor(query));
        let actual_max_xors = query_values.map(|query| trie.max_xor(query));

        // Assert
        assert!(!actual_removed);
        assert_eq!(actual_erased_count, expected_erased_count_right);
        assert_eq!(actual_snapshot, expected_snapshot);
        assert_eq!(actual_lcps, expected_lcps);
        assert_eq!(actual_min_xors, expected_min_xors);
        assert_eq!(actual_max_xors, expected_max_xors);
    }

    #[test]
    fn fully_pruned_path_can_be_recreated_after_remove_and_erase() {
        // Arrange
        let bit_size = 4usize;
        let stored_value = 0b1011u64;
        let expected_single_path = vec![(1, 0), (1, 0), (1, 0), (1, 1)];
        let expected_single_count = 1usize;
        let mut trie = BinaryTrie::new(bit_size);

        let expected_count_after_remove_right = 0;
        let expected_count_after_erase_right = 0;

        // Act
        trie.insert(stored_value);
        let actual_initial_path =
            counter_pruning_path_counts(&trie, stored_value).expect("initial path must exist");
        let actual_initial_search = trie.search(stored_value);
        let actual_initial_count = trie.count(stored_value);
        let actual_removed = trie.remove(stored_value);
        let actual_path_after_remove = counter_pruning_path_counts(&trie, stored_value);
        let actual_search_after_remove = trie.search(stored_value);
        let actual_count_after_remove = trie.count(stored_value);
        trie.insert(stored_value);
        let actual_path_after_first_reinsert = counter_pruning_path_counts(&trie, stored_value)
            .expect("path after remove must be recreated");
        let actual_search_after_first_reinsert = trie.search(stored_value);
        let actual_count_after_first_reinsert = trie.count(stored_value);
        let actual_erased_count = trie.erase(stored_value);
        let actual_path_after_erase = counter_pruning_path_counts(&trie, stored_value);
        let actual_search_after_erase = trie.search(stored_value);
        let actual_count_after_erase = trie.count(stored_value);
        trie.insert(stored_value);
        let actual_path_after_second_reinsert = counter_pruning_path_counts(&trie, stored_value)
            .expect("path after erase must be recreated");
        let actual_final_search = trie.search(stored_value);
        let actual_final_count = trie.count(stored_value);

        // Assert
        assert_eq!(actual_initial_path, expected_single_path);
        assert!(actual_initial_search);
        assert_eq!(actual_initial_count, expected_single_count);
        assert!(actual_removed);
        assert!(actual_path_after_remove.is_none());
        assert!(!actual_search_after_remove);
        assert_eq!(actual_count_after_remove, expected_count_after_remove_right);
        assert_eq!(actual_path_after_first_reinsert, expected_single_path);
        assert!(actual_search_after_first_reinsert);
        assert_eq!(actual_count_after_first_reinsert, expected_single_count);
        assert_eq!(actual_erased_count, expected_single_count);
        assert!(actual_path_after_erase.is_none());
        assert!(!actual_search_after_erase);
        assert_eq!(actual_count_after_erase, expected_count_after_erase_right);
        assert_eq!(actual_path_after_second_reinsert, expected_single_path);
        assert!(actual_final_search);
        assert_eq!(actual_final_count, expected_single_count);
    }

    #[test]
    fn deterministic_operations_match_reference_for_boundary_bit_sizes() {
        // Arrange
        let bit_sizes = [0usize, 1usize, 5usize, 63usize, 64usize];

        for bit_size in bit_sizes {
            let mut trie = BinaryTrie::new(bit_size);
            let mut reference = HashMap::<u64, usize>::new();
            let (operations, queries) = deterministic_operations(bit_size);
            for operation in operations {
                // Arrange
                let expected_operation = match operation {
                    ReferenceOperation::Insert(value) => {
                        insert_reference_count(&mut reference, normalize_value(value, bit_size));
                        None
                    }
                    ReferenceOperation::Remove(value) => {
                        Some(usize::from(remove_one_reference_count(
                            &mut reference,
                            normalize_value(value, bit_size),
                        )))
                    }
                    ReferenceOperation::Erase(value) => Some(erase_all_reference_count(
                        &mut reference,
                        normalize_value(value, bit_size),
                    )),
                };
                let expected_queries = reference_queries(&reference, bit_size, &queries);
                let expected_total = reference.values().sum::<usize>();

                // Act
                let actual_operation = match operation {
                    ReferenceOperation::Insert(value) => {
                        trie.insert(value);
                        None
                    }
                    ReferenceOperation::Remove(value) => Some(usize::from(trie.remove(value))),
                    ReferenceOperation::Erase(value) => Some(trie.erase(value)),
                };
                let observed_queries = actual_queries(&trie, &queries);
                let actual_total = assert_counter_invariants(&trie.root, true);

                // Assert
                assert_eq!(actual_operation, expected_operation);
                assert_eq!(observed_queries, expected_queries);
                assert_eq!(actual_total, expected_total);
            }
        }
    }

    #[test]
    fn values_differing_only_above_bit_size_are_aggregated_as_same_key() {
        // Arrange
        let bit_size = 3usize;
        let low_value = 0b101u64;
        let high_value = low_value + (1u64 << bit_size);
        let mut trie = BinaryTrie::new(bit_size);
        trie.insert(low_value);
        trie.insert(high_value);

        let expected_low_count_right = 2;
        let expected_high_count_right = 2;

        // Act
        let actual_low_count = trie.count(low_value);
        let actual_high_count = trie.count(high_value);

        // Assert
        assert_eq!(actual_low_count, expected_low_count_right);
        assert_eq!(actual_high_count, expected_high_count_right);
    }
}
