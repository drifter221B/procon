#[derive(Debug)]
struct Node {
    key: i64,
    count: usize,
    size: usize,
    left: Option<Box<Node>>,
    right: Option<Box<Node>>,
}

impl Node {
    fn new(key: i64) -> Box<Self> {
        Box::new(Self {
            key,
            count: 1,
            size: 1,
            left: None,
            right: None,
        })
    }
}

fn size_of(node: &Option<Box<Node>>) -> usize {
    node.as_ref().map_or(0, |n| n.size)
}

fn update_size(node: &mut Box<Node>) {
    node.size = node.count + size_of(&node.left) + size_of(&node.right);
}

fn rotate_right(mut parent: Box<Node>) -> Box<Node> {
    let mut child = parent
        .left
        .take()
        .expect("rotate_right requires left child");
    parent.left = child.right.take();
    update_size(&mut parent);
    child.right = Some(parent);
    update_size(&mut child);
    child
}

fn rotate_left(mut parent: Box<Node>) -> Box<Node> {
    let mut child = parent
        .right
        .take()
        .expect("rotate_left requires right child");
    parent.right = child.left.take();
    update_size(&mut parent);
    child.left = Some(parent);
    update_size(&mut child);
    child
}

fn splay(root: Option<Box<Node>>, key: i64) -> Option<Box<Node>> {
    let mut root = match root {
        Some(node) => node,
        None => return None,
    };

    if key < root.key {
        if let Some(mut left) = root.left.take() {
            if key < left.key {
                left.left = splay(left.left.take(), key);
                root.left = Some(left);
                root = rotate_right(root);
            } else if key > left.key {
                left.right = splay(left.right.take(), key);
                if left.right.is_some() {
                    left = rotate_left(left);
                }
                root.left = Some(left);
            } else {
                root.left = Some(left);
            }
            if root.left.is_none() {
                return Some(root);
            }
            root = rotate_right(root);
        }
        Some(root)
    } else if key > root.key {
        if let Some(mut right) = root.right.take() {
            if key > right.key {
                right.right = splay(right.right.take(), key);
                root.right = Some(right);
                root = rotate_left(root);
            } else if key < right.key {
                right.left = splay(right.left.take(), key);
                if right.left.is_some() {
                    right = rotate_right(right);
                }
                root.right = Some(right);
            } else {
                root.right = Some(right);
            }
            if root.right.is_none() {
                return Some(root);
            }
            root = rotate_left(root);
        }
        Some(root)
    } else {
        Some(root)
    }
}

#[derive(Debug, Default)]
pub struct SplayTree {
    root: Option<Box<Node>>,
}

impl SplayTree {
    pub fn new() -> Self {
        Self { root: None }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn len(&self) -> usize {
        size_of(&self.root)
    }

    pub fn contains(&self, key: i64) -> bool {
        self.count(key) > 0
    }

    pub fn count(&self, key: i64) -> usize {
        let mut current = &self.root;
        while let Some(node) = current {
            if key < node.key {
                current = &node.left;
            } else if key > node.key {
                current = &node.right;
            } else {
                return node.count;
            }
        }
        0
    }

    pub fn insert(&mut self, key: i64) {
        self.root = match self.root.take() {
            None => Some(Node::new(key)),
            Some(root) => {
                let mut root = splay(Some(root), key).unwrap();
                if key == root.key {
                    root.count += 1;
                    update_size(&mut root);
                    Some(root)
                } else if key < root.key {
                    let left_subtree = root.left.take();
                    update_size(&mut root);
                    let mut node = Node::new(key);
                    node.left = left_subtree;
                    node.right = Some(root);
                    update_size(&mut node);
                    Some(node)
                } else {
                    let right_subtree = root.right.take();
                    update_size(&mut root);
                    let mut node = Node::new(key);
                    node.right = right_subtree;
                    node.left = Some(root);
                    update_size(&mut node);
                    Some(node)
                }
            }
        };
    }

    pub fn erase_one(&mut self, key: i64) -> bool {
        if self.root.is_none() {
            return false;
        }

        let mut root = splay(self.root.take(), key).unwrap();
        if key != root.key {
            self.root = Some(root);
            return false;
        }

        if root.count > 1 {
            root.count -= 1;
            update_size(&mut root);
            self.root = Some(root);
            return true;
        }

        let left = root.left.take();
        let right = root.right.take();
        self.root = match left {
            None => right,
            Some(left_root) => {
                let mut left_root = splay(Some(left_root), key).unwrap();
                left_root.right = right;
                update_size(&mut left_root);
                Some(left_root)
            }
        };
        true
    }

    pub fn erase_all(&mut self, key: i64) -> usize {
        if self.root.is_none() {
            return 0;
        }
        let mut root = splay(self.root.take(), key).unwrap();
        if key != root.key {
            self.root = Some(root);
            return 0;
        }

        let removed = root.count;
        let left = root.left.take();
        let right = root.right.take();
        self.root = match left {
            None => right,
            Some(left_root) => {
                let mut left_root = splay(Some(left_root), key).unwrap();
                left_root.right = right;
                update_size(&mut left_root);
                Some(left_root)
            }
        };
        removed
    }

    pub fn kth_min(&mut self, index: usize) -> Option<i64> {
        if index >= self.len() {
            return None;
        }
        let mut idx = index;
        let mut current_node = self.root.as_deref();
        while let Some(node) = current_node {
            let left_size = size_of(&node.left);
            if idx < left_size {
                current_node = node.left.as_deref();
            } else if idx < left_size + node.count {
                return Some(node.key);
            } else {
                idx -= left_size + node.count;
                current_node = node.right.as_deref();
            }
        }
        None
    }

    pub fn first(&mut self) -> Option<i64> {
        self.kth_min(0)
    }

    pub fn last(&mut self) -> Option<i64> {
        let len = self.len();
        if len == 0 {
            None
        } else {
            self.kth_min(len - 1)
        }
    }

    pub fn lower_bound(&mut self, key: i64) -> Option<i64> {
        let mut current = self.root.as_deref();
        let mut candidate = None;
        while let Some(node) = current {
            if key <= node.key {
                candidate = Some(node.key);
                current = node.left.as_deref();
            } else {
                current = node.right.as_deref();
            }
        }
        candidate
    }

    pub fn upper_bound(&mut self, key: i64) -> Option<i64> {
        let mut current = self.root.as_deref();
        let mut candidate = None;
        while let Some(node) = current {
            if node.key < key {
                candidate = Some(node.key);
                current = node.right.as_deref();
            } else {
                current = node.left.as_deref();
            }
        }
        candidate
    }
}

#[cfg(test)]
mod tests {
    use super::SplayTree;
    use std::collections::BTreeMap;

    fn create_tree_from_values(values: &[i64]) -> SplayTree {
        let mut tree = SplayTree::new();
        for &value in values {
            tree.insert(value);
        }
        tree
    }

    fn sorted_values_from_tree(tree: &mut SplayTree) -> Vec<i64> {
        let element_count = tree.len();
        let mut ordered_values = Vec::with_capacity(element_count);
        for index in 0..element_count {
            let key_at_index = tree.kth_min(index).unwrap();
            ordered_values.push(key_at_index);
        }
        ordered_values
    }

    fn sorted_values_from_map(multiset_map: &BTreeMap<i64, usize>) -> Vec<i64> {
        let mut ordered_values = Vec::new();
        for (&key, &count) in multiset_map {
            ordered_values.extend(std::iter::repeat(key).take(count));
        }
        ordered_values
    }

    #[test]
    fn new_after_creation_reports_empty_and_len_zero() {
        // Arrange
        let tree = SplayTree::new();
        let expected_is_empty = true;
        let expected_len = 0;

        // Act
        let actual_is_empty = tree.is_empty();
        let actual_len = tree.len();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn contains_when_tree_is_empty_returns_false() {
        // Arrange
        let tree = SplayTree::new();
        let target_key = 0;
        let expected_contains = false;

        // Act
        let actual_contains = tree.contains(target_key);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn count_when_tree_is_empty_returns_zero() {
        // Arrange
        let tree = SplayTree::new();
        let target_key = 0;
        let expected_count = 0;

        // Act
        let actual_count = tree.count(target_key);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn insert_when_tree_is_empty_makes_key_searchable_with_count_one() {
        // Arrange
        let mut tree = SplayTree::new();
        let inserted_key = 5;
        let expected_contains = true;
        let expected_count = 1;

        // Act
        tree.insert(inserted_key);
        let actual_contains = tree.contains(inserted_key);
        let actual_count = tree.count(inserted_key);

        // Assert
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn insert_multiple_distinct_keys_makes_len_equal_to_total_occurrence_count() {
        // Arrange
        let mut tree = SplayTree::new();
        let inserted_values = vec![1, 2, 3, 4];
        let expected_len = inserted_values.len();

        // Act
        for &value in &inserted_values {
            tree.insert(value);
        }
        let actual_len = tree.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_one_on_empty_tree_returns_false_and_keeps_len_unchanged() {
        // Arrange
        let mut tree = SplayTree::new();
        let erased_key = 10;
        let expected_result = false;
        let expected_len = 0;

        // Act
        let actual_result = tree.erase_one(erased_key);
        let actual_len = tree.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_one_successfully_decreases_len_by_one() {
        // Arrange
        let mut tree = create_tree_from_values(&[4, 5, 6]);
        let target_key = 5;
        let expected_len_difference = 1;

        let expected_boolean_1 = true;

        // Act
        let len_before = tree.len();
        let erase_result = tree.erase_one(target_key);
        let len_after = tree.len();
        let actual_len_difference = len_before - len_after;

        // Assert
        assert_eq!(erase_result, expected_boolean_1);
        assert_eq!(actual_len_difference, expected_len_difference);
    }

    #[test]
    fn erase_all_on_empty_tree_returns_zero_and_keeps_len_unchanged() {
        // Arrange
        let mut tree = SplayTree::new();
        let erased_key = 10;
        let expected_removed = 0;
        let expected_len = 0;

        // Act
        let actual_removed = tree.erase_all(erased_key);
        let actual_len = tree.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_all_successfully_decreases_len_by_removed_count() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 5, 5, 7]);
        let target_key = 5;
        let expected_len_difference = 3;

        // Act
        let len_before = tree.len();
        let removed_count = tree.erase_all(target_key);
        let len_after = tree.len();
        let actual_len_difference = len_before - len_after;

        // Assert
        assert_eq!(removed_count, expected_len_difference);
        assert_eq!(actual_len_difference, expected_len_difference);
    }

    #[test]
    fn kth_min_with_index_equal_to_len_returns_none() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 4, 9, 16]);
        let query_index = tree.len();
        let expected_value = None;

        // Act
        let actual_value = tree.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_with_index_greater_than_len_returns_none() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 4, 9, 16]);
        let query_index = tree.len() + 1;
        let expected_value = None;

        // Act
        let actual_value = tree.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn insert_same_key_multiple_times_increases_count_by_insertion_times() {
        // Arrange
        let mut tree = SplayTree::new();
        let duplicated_key = 7;
        let insert_count = 3;
        let expected_count = 3;

        // Act
        for _ in 0..insert_count {
            tree.insert(duplicated_key);
        }
        let actual_count = tree.count(duplicated_key);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn insert_i64_min_and_i64_max_keeps_contains_and_count_correct() {
        // Arrange
        let mut tree = SplayTree::new();
        let smallest_key = i64::MIN;
        let largest_key = i64::MAX;
        let expected_contains = true;
        let expected_count = 1;

        // Act
        tree.insert(smallest_key);
        tree.insert(largest_key);
        let actual_min_contains = tree.contains(smallest_key);
        let actual_max_contains = tree.contains(largest_key);
        let actual_min_count = tree.count(smallest_key);
        let actual_max_count = tree.count(largest_key);

        // Assert
        assert_eq!(actual_min_contains, expected_contains);
        assert_eq!(actual_max_contains, expected_contains);
        assert_eq!(actual_min_count, expected_count);
        assert_eq!(actual_max_count, expected_count);
    }

    #[test]
    fn contains_for_non_inserted_key_returns_false() {
        // Arrange
        let tree = create_tree_from_values(&[2, 4, 6]);
        let non_existing_key = 5;
        let expected_contains = false;

        // Act
        let actual_contains = tree.contains(non_existing_key);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn count_for_non_inserted_key_returns_zero() {
        // Arrange
        let tree = create_tree_from_values(&[2, 4, 6]);
        let non_existing_key = 5;
        let expected_count = 0;

        // Act
        let actual_count = tree.count(non_existing_key);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn mixed_insert_erase_one_erase_all_matches_btreemap_reference() {
        // Arrange
        let input_value_1 = 0;
        let input_value_2 = 0;
        let input_value_3 = 0;
        let inserted_values = vec![5, 1, 5, 3, 8, 3, 3, 10, -2];
        let erase_one_targets = vec![3, 42, 5, -2];
        let erase_all_targets = vec![5, 1, 99];
        let query_keys = vec![-2, 1, 3, 5, 8, 10, 42, 99];
        let mut reference = BTreeMap::<i64, usize>::new();
        let mut expected = Vec::new();
        for &value in &inserted_values {
            *reference.entry(value).or_default() += 1;
        }
        for &target in &erase_one_targets {
            let removed = if let Some(count) = reference.get_mut(&target) {
                *count -= 1;
                if *count == 0 {
                    reference.remove(&target);
                }
                1
            } else {
                0
            };
            let values = sorted_values_from_map(&reference);
            let queries = query_keys
                .iter()
                .map(|key| {
                    (
                        reference.get(key).copied().unwrap_or(input_value_1),
                        reference.contains_key(key),
                    )
                })
                .collect::<Vec<_>>();
            expected.push((removed, values.len(), values, queries));
        }
        for &target in &erase_all_targets {
            let removed = reference.remove(&target).unwrap_or(input_value_2);
            let values = sorted_values_from_map(&reference);
            let queries = query_keys
                .iter()
                .map(|key| {
                    (
                        reference.get(key).copied().unwrap_or(input_value_3),
                        reference.contains_key(key),
                    )
                })
                .collect::<Vec<_>>();
            expected.push((removed, values.len(), values, queries));
        }
        let mut tree = SplayTree::new();

        // Act
        for value in inserted_values {
            tree.insert(value);
        }
        let mut actual = Vec::new();
        for target in erase_one_targets {
            let removed = usize::from(tree.erase_one(target));
            let len = tree.len();
            let values = sorted_values_from_tree(&mut tree);
            let queries = query_keys
                .iter()
                .map(|&key| (tree.count(key), tree.contains(key)))
                .collect::<Vec<_>>();
            actual.push((removed, len, values, queries));
        }
        for target in erase_all_targets {
            let removed = tree.erase_all(target);
            let len = tree.len();
            let values = sorted_values_from_tree(&mut tree);
            let queries = query_keys
                .iter()
                .map(|&key| (tree.count(key), tree.contains(key)))
                .collect::<Vec<_>>();
            actual.push((removed, len, values, queries));
        }

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn insert_singleton_makes_first_last_and_kth_min_return_same_value() {
        // Arrange
        let input_value_1 = 0;
        let mut tree = SplayTree::new();
        let value = 7i64;
        let expected_value = Some(value);

        // Act
        tree.insert(value);
        let actual_first = tree.first();
        let actual_last = tree.last();
        let actual_kth_min = tree.kth_min(input_value_1);

        // Assert
        assert_eq!(actual_first, expected_value);
        assert_eq!(actual_last, expected_value);
        assert_eq!(actual_kth_min, expected_value);
    }

    #[test]
    fn erase_one_for_single_count_key_removes_key_and_returns_true() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 2, 3]);
        let target_key = 2;
        let expected_result = true;
        let expected_contains = false;

        // Act
        let actual_result = tree.erase_one(target_key);
        let actual_contains = tree.contains(target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn erase_all_for_single_count_key_returns_one_and_removes_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 2, 3]);
        let target_key = 2;
        let expected_removed = 1;
        let expected_contains = false;

        // Act
        let actual_removed = tree.erase_all(target_key);
        let actual_contains = tree.contains(target_key);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn erase_all_for_duplicated_key_returns_removed_count_and_zero_count_afterward() {
        // Arrange
        let mut tree = create_tree_from_values(&[2, 2, 2, 4]);
        let target_key = 2;
        let expected_removed = 3;
        let expected_count = 0;

        // Act
        let actual_removed = tree.erase_all(target_key);
        let actual_count = tree.count(target_key);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn erase_one_for_non_existing_key_returns_false_and_keeps_existing_values() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 3, 5]);
        let non_existing_key = 4;
        let expected_result = false;
        let expected_values = vec![1, 3, 5];

        // Act
        let actual_result = tree.erase_one(non_existing_key);
        let actual_values = sorted_values_from_tree(&mut tree);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn erase_one_for_duplicated_key_decrements_count_by_one() {
        // Arrange
        let mut tree = create_tree_from_values(&[2, 2, 2]);
        let target_key = 2;
        let expected_result = true;
        let expected_count = 2;

        // Act
        let actual_result = tree.erase_one(target_key);
        let actual_count = tree.count(target_key);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn erase_all_for_non_existing_key_returns_zero_and_keeps_existing_values() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 3, 5]);
        let non_existing_key = 4;
        let expected_removed = 0;
        let expected_values = vec![1, 3, 5];

        // Act
        let actual_removed = tree.erase_all(non_existing_key);
        let actual_values = sorted_values_from_tree(&mut tree);

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn erase_one_minimum_updates_first_to_next_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 2, 3]);
        let erase_value = 1i64;
        let expected_first = Some(2i64);

        let expected_boolean_1 = true;

        // Act
        let erase_result = tree.erase_one(erase_value);
        let actual_first = tree.first();

        // Assert
        assert_eq!(erase_result, expected_boolean_1);
        assert_eq!(actual_first, expected_first);
    }

    #[test]
    fn erase_one_maximum_updates_last_to_previous_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 2, 3]);
        let erase_value = 3i64;
        let expected_last = Some(2i64);

        let expected_boolean_1 = true;

        // Act
        let erase_result = tree.erase_one(erase_value);
        let actual_last = tree.last();

        // Assert
        assert_eq!(erase_result, expected_boolean_1);
        assert_eq!(actual_last, expected_last);
    }

    #[test]
    fn erase_all_minimum_updates_lower_bound_and_first_to_next_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 1, 2, 3]);
        let erase_value = 1i64;
        let expected_removed_count = 2usize;
        let expected_lower_bound = Some(2i64);
        let expected_first = Some(2i64);

        // Act
        let actual_removed_count = tree.erase_all(erase_value);
        let actual_lower_bound = tree.lower_bound(erase_value);
        let actual_first = tree.first();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_lower_bound, expected_lower_bound);
        assert_eq!(actual_first, expected_first);
    }

    #[test]
    fn erase_all_maximum_updates_upper_bound_and_last_to_previous_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 2, 3, 3]);
        let erase_value = 3i64;
        let expected_removed_count = 2usize;
        let expected_upper_bound = Some(2i64);
        let expected_last = Some(2i64);

        // Act
        let actual_removed_count = tree.erase_all(erase_value);
        let actual_upper_bound = tree.upper_bound(erase_value);
        let actual_last = tree.last();

        // Assert
        assert_eq!(actual_removed_count, expected_removed_count);
        assert_eq!(actual_upper_bound, expected_upper_bound);
        assert_eq!(actual_last, expected_last);
    }

    #[test]
    fn lower_bound_on_empty_tree_returns_none() {
        // Arrange
        let mut tree = SplayTree::new();
        let query_key = 10;
        let expected_value = None;

        // Act
        let actual_value = tree.lower_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_for_value_smaller_than_minimum_returns_minimum_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let query_key = 4;
        let expected_value = Some(5);

        // Act
        let actual_value = tree.lower_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_for_existing_key_returns_same_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let existing_key = 10;
        let expected_value = Some(10);

        // Act
        let actual_value = tree.lower_bound(existing_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_for_missing_key_returns_smallest_key_greater_or_equal_to_query() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let query_key = 11;
        let expected_value = Some(15);

        // Act
        let actual_value = tree.lower_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn lower_bound_for_value_greater_than_maximum_returns_none() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let query_key = 16;
        let expected_value = None;

        // Act
        let actual_value = tree.lower_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_on_empty_tree_returns_none() {
        // Arrange
        let mut tree = SplayTree::new();
        let query_key = 10;
        let expected_value = None;

        // Act
        let actual_value = tree.upper_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_for_minimum_existing_key_returns_none() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let minimum_existing_key = 5;
        let expected_value = None;

        // Act
        let actual_value = tree.upper_bound(minimum_existing_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_for_existing_key_returns_largest_key_strictly_smaller_than_query() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let existing_key = 10;
        let expected_value = Some(5);

        // Act
        let actual_value = tree.upper_bound(existing_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_for_missing_key_returns_largest_key_strictly_smaller_than_query() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let query_key = 11;
        let expected_value = Some(10);

        // Act
        let actual_value = tree.upper_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn upper_bound_for_value_greater_than_maximum_returns_maximum_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[5, 10, 15]);
        let query_key = 16;
        let expected_value = Some(15);

        // Act
        let actual_value = tree.upper_bound(query_key);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_on_empty_tree_returns_none() {
        // Arrange
        let mut tree = SplayTree::new();
        let query_index = 0;
        let expected_value = None;

        // Act
        let actual_value = tree.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_with_last_valid_index_returns_maximum_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 4, 9, 16]);
        let element_count = tree.len();
        let target_index = element_count - 1;
        let expected_value = Some(16);

        // Act
        let actual_value = tree.kth_min(target_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_with_duplicate_keys_counts_duplicates_as_individual_positions() {
        // Arrange
        let mut tree = create_tree_from_values(&[1, 2, 2, 2, 3]);
        let query_index = 2;
        let expected_value = Some(2);

        // Act
        let actual_value = tree.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_on_single_element_tree_returns_that_element_for_index_zero() {
        // Arrange
        let mut tree = create_tree_from_values(&[42]);
        let query_index = 0;
        let expected_value = Some(42);

        // Act
        let actual_value = tree.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_order_is_preserved_after_contains_and_count_reads() {
        // Arrange
        let input_value_1 = 4;
        let input_value_2 = 1;
        let mut tree = create_tree_from_values(&[3, 1, 4, 1, 5]);
        let expected_values = vec![1, 1, 3, 4, 5];
        let expected_check_2 = 2;

        let expected_boolean_1 = true;

        // Act
        let contains_result = tree.contains(input_value_1);
        let count_result = tree.count(input_value_2);
        let actual_values = sorted_values_from_tree(&mut tree);

        // Assert
        assert_eq!(contains_result, expected_boolean_1);
        assert_eq!(count_result, expected_check_2);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn first_on_empty_tree_returns_none() {
        // Arrange
        let mut tree = SplayTree::new();
        let expected_value = None;

        // Act
        let actual_value = tree.first();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn first_on_single_element_tree_returns_that_element() {
        // Arrange
        let mut tree = create_tree_from_values(&[42]);
        let expected_value = Some(42);

        // Act
        let actual_value = tree.first();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn first_on_multiple_elements_returns_smallest_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[10, -3, 7, 5]);
        let expected_value = Some(-3);

        // Act
        let actual_value = tree.first();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn last_on_empty_tree_returns_none() {
        // Arrange
        let mut tree = SplayTree::new();
        let expected_value = None;

        // Act
        let actual_value = tree.last();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn last_on_single_element_tree_returns_that_element() {
        // Arrange
        let mut tree = create_tree_from_values(&[42]);
        let expected_value = Some(42);

        // Act
        let actual_value = tree.last();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn last_on_multiple_elements_returns_largest_key() {
        // Arrange
        let mut tree = create_tree_from_values(&[10, -3, 7, 5]);
        let expected_value = Some(10);

        // Act
        let actual_value = tree.last();

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
