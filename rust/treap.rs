use rand::{self, Rng};
use std::collections::VecDeque;

#[derive(Debug)]
struct Node<T: Ord + Copy + Default + std::fmt::Debug> {
    key: T,
    count: usize,
    subtree_size: usize,
    priority: f64,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T: Ord + Copy + Default + std::fmt::Debug> Node<T> {
    fn new(key: T, count: usize) -> Self {
        Node {
            key,
            count,
            subtree_size: count,
            priority: rand::rng().random(),
            left: None,
            right: None,
        }
    }
}

#[derive(Debug)]
pub struct Treap<T: Ord + Copy + Default + std::fmt::Debug> {
    root: Option<Box<Node<T>>>,
    size: usize,
    min_key: Option<T>,
    max_key: Option<T>,
}

impl<T: Ord + Copy + Default + std::fmt::Debug> Default for Treap<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Ord + Copy + Default + std::fmt::Debug> Treap<T> {
    pub fn new() -> Self {
        Treap {
            root: None,
            size: 0,
            min_key: None,
            max_key: None,
        }
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn len(&self) -> usize {
        self.size
    }

    fn subtree_size_of(node: &Option<Box<Node<T>>>) -> usize {
        node.as_ref()
            .map_or(0, |target_node| target_node.subtree_size)
    }

    fn refresh_subtree_size(node: &mut Box<Node<T>>) {
        node.subtree_size =
            node.count + Self::subtree_size_of(&node.left) + Self::subtree_size_of(&node.right);
    }

    fn find_node(node: &Option<Box<Node<T>>>, key: T) -> Option<&Node<T>> {
        if let Some(current_node) = node {
            match key.cmp(&current_node.key) {
                std::cmp::Ordering::Less => Self::find_node(&current_node.left, key),
                std::cmp::Ordering::Greater => Self::find_node(&current_node.right, key),
                std::cmp::Ordering::Equal => Some(current_node.as_ref()),
            }
        } else {
            None
        }
    }

    fn merge(
        mut left_root: Option<Box<Node<T>>>,
        mut right_root: Option<Box<Node<T>>>,
    ) -> Option<Box<Node<T>>> {
        match (left_root.take(), right_root.take()) {
            (None, right_tree) => right_tree,
            (left_tree, None) => left_tree,
            (Some(mut left_node), Some(mut right_node)) => {
                if right_node.priority <= left_node.priority {
                    left_node.right = Self::merge(left_node.right.take(), Some(right_node));
                    Self::refresh_subtree_size(&mut left_node);
                    Some(left_node)
                } else {
                    right_node.left = Self::merge(Some(left_node), right_node.left.take());
                    Self::refresh_subtree_size(&mut right_node);
                    Some(right_node)
                }
            }
        }
    }

    fn split_less(
        mut node: Option<Box<Node<T>>>,
        key: T,
    ) -> (Option<Box<Node<T>>>, Option<Box<Node<T>>>) {
        if let Some(mut current_node) = node.take() {
            if current_node.key < key {
                let (less_tree, greater_equal_tree) =
                    Self::split_less(current_node.right.take(), key);
                current_node.right = less_tree;
                Self::refresh_subtree_size(&mut current_node);
                (Some(current_node), greater_equal_tree)
            } else {
                let (less_tree, greater_equal_tree) =
                    Self::split_less(current_node.left.take(), key);
                current_node.left = greater_equal_tree;
                Self::refresh_subtree_size(&mut current_node);
                (less_tree, Some(current_node))
            }
        } else {
            (None, None)
        }
    }

    fn split_less_or_equal(
        mut node: Option<Box<Node<T>>>,
        key: T,
    ) -> (Option<Box<Node<T>>>, Option<Box<Node<T>>>) {
        if let Some(mut current_node) = node.take() {
            if key < current_node.key {
                let (less_equal_tree, greater_tree) =
                    Self::split_less_or_equal(current_node.left.take(), key);
                current_node.left = greater_tree;
                Self::refresh_subtree_size(&mut current_node);
                (less_equal_tree, Some(current_node))
            } else {
                let (less_equal_tree, greater_tree) =
                    Self::split_less_or_equal(current_node.right.take(), key);
                current_node.right = less_equal_tree;
                Self::refresh_subtree_size(&mut current_node);
                (Some(current_node), greater_tree)
            }
        } else {
            (None, None)
        }
    }

    fn split_by_key(
        node: Option<Box<Node<T>>>,
        key: T,
    ) -> (
        Option<Box<Node<T>>>,
        Option<Box<Node<T>>>,
        Option<Box<Node<T>>>,
    ) {
        let (less_tree, greater_equal_tree) = Self::split_less(node, key);
        let (equal_tree, greater_tree) = Self::split_less_or_equal(greater_equal_tree, key);
        (less_tree, equal_tree, greater_tree)
    }

    fn merge_three_way(
        left_tree: Option<Box<Node<T>>>,
        middle_tree: Option<Box<Node<T>>>,
        right_tree: Option<Box<Node<T>>>,
    ) -> Option<Box<Node<T>>> {
        let merged_tree = Self::merge(left_tree, middle_tree);
        Self::merge(merged_tree, right_tree)
    }

    fn insert_to_isolated_key_node(
        node: Option<Box<Node<T>>>,
        key: T,
        count: usize,
    ) -> Box<Node<T>> {
        if let Some(mut current_node) = node {
            current_node.count += count;
            Self::refresh_subtree_size(&mut current_node);
            current_node
        } else {
            Box::new(Node::new(key, count))
        }
    }

    fn erase_from_isolated_key_node(
        node: Option<Box<Node<T>>>,
        erase_all: bool,
    ) -> (Option<Box<Node<T>>>, usize) {
        if let Some(mut current_node) = node {
            let removed_count = if erase_all {
                current_node.count
            } else if current_node.count == 0 {
                0
            } else {
                1
            };

            if !erase_all && 1 < current_node.count {
                current_node.count -= 1;
                Self::refresh_subtree_size(&mut current_node);
                (Some(current_node), removed_count)
            } else {
                let merged_tree = Self::merge(current_node.left.take(), current_node.right.take());
                (merged_tree, removed_count)
            }
        } else {
            (None, 0)
        }
    }

    fn refresh_boundary_keys(&mut self) {
        if self.root.is_none() {
            self.min_key = None;
            self.max_key = None;
        } else {
            self.min_key = Self::find_min(&self.root).map(|node| node.key);
            self.max_key = Self::find_max(&self.root).map(|node| node.key);
        }
    }

    pub fn contains(&self, key: T) -> bool {
        Self::find_node(&self.root, key).is_some()
    }

    pub fn count_key(&self, key: T) -> usize {
        Self::find_node(&self.root, key).map_or(0, |node| node.count)
    }

    fn find_kth_node(node: &Option<Box<Node<T>>>, index: usize) -> Option<&Node<T>> {
        if let Some(current_node) = node {
            let left_subtree_size = Self::subtree_size_of(&current_node.left);
            if index < left_subtree_size {
                Self::find_kth_node(&current_node.left, index)
            } else if left_subtree_size + current_node.count <= index {
                Self::find_kth_node(
                    &current_node.right,
                    index - left_subtree_size - current_node.count,
                )
            } else {
                Some(current_node.as_ref())
            }
        } else {
            None
        }
    }

    pub fn insert(&mut self, key: T, count: usize) {
        self.size += count;
        let (less_tree, equal_tree, greater_tree) = Self::split_by_key(self.root.take(), key);
        let inserted_tree = Some(Self::insert_to_isolated_key_node(equal_tree, key, count));
        self.root = Self::merge_three_way(less_tree, inserted_tree, greater_tree);
        self.refresh_boundary_keys();
    }

    pub fn erase(&mut self, key: T) -> usize {
        let (less_tree, equal_tree, greater_tree) = Self::split_by_key(self.root.take(), key);
        let (updated_equal_tree, removed) = Self::erase_from_isolated_key_node(equal_tree, false);
        self.root = Self::merge_three_way(less_tree, updated_equal_tree, greater_tree);
        if 0 < removed {
            self.size -= removed;
        }
        self.refresh_boundary_keys();

        removed
    }

    pub fn all_erase(&mut self, key: T) -> usize {
        let (less_tree, equal_tree, greater_tree) = Self::split_by_key(self.root.take(), key);
        let (updated_equal_tree, removed) = Self::erase_from_isolated_key_node(equal_tree, true);
        self.root = Self::merge_three_way(less_tree, updated_equal_tree, greater_tree);
        if 0 < removed {
            self.size -= removed;
        }
        self.refresh_boundary_keys();

        removed
    }

    fn find_min(node: &Option<Box<Node<T>>>) -> Option<&Node<T>> {
        if let Some(current_node) = node {
            if current_node.left.is_some() {
                Self::find_min(&current_node.left)
            } else {
                Some(current_node.as_ref())
            }
        } else {
            None
        }
    }

    fn find_max(node: &Option<Box<Node<T>>>) -> Option<&Node<T>> {
        if let Some(current_node) = node {
            if current_node.right.is_some() {
                Self::find_max(&current_node.right)
            } else {
                Some(current_node.as_ref())
            }
        } else {
            None
        }
    }

    pub fn first(&self) -> Option<&T> {
        self.min_key.as_ref()
    }

    pub fn last(&self) -> Option<&T> {
        self.max_key.as_ref()
    }

    pub fn pop_first(&mut self) -> Option<T> {
        while let Some(first_key) = self.first().copied() {
            if 0 < self.erase(first_key) {
                return Some(first_key);
            }
        }

        None
    }

    pub fn pop_last(&mut self) -> Option<T> {
        while let Some(last_key) = self.last().copied() {
            if 0 < self.erase(last_key) {
                return Some(last_key);
            }
        }

        None
    }

    pub fn lower_bound(&self, key: T) -> Option<T> {
        fn _lower_bound<T: Ord + Copy + Default + std::fmt::Debug>(
            node: &Option<Box<Node<T>>>,
            key: T,
            candidate: Option<T>,
        ) -> Option<T> {
            if let Some(n) = node {
                if key < n.key {
                    _lower_bound(&n.left, key, Some(n.key))
                } else if n.key < key {
                    _lower_bound(&n.right, key, candidate)
                } else {
                    Some(key)
                }
            } else {
                candidate
            }
        }

        _lower_bound(&self.root, key, None)
    }

    pub fn upper_bound(&self, key: T) -> Option<T> {
        fn _upper_bound<T: Ord + Copy + Default + std::fmt::Debug>(
            node: &Option<Box<Node<T>>>,
            key: T,
            candidate: Option<T>,
        ) -> Option<T> {
            if let Some(n) = node {
                if n.key < key {
                    _upper_bound(&n.right, key, Some(n.key))
                } else {
                    _upper_bound(&n.left, key, candidate)
                }
            } else {
                candidate
            }
        }

        _upper_bound(&self.root, key, None)
    }

    pub fn kth_min(&self, index: usize) -> Option<T> {
        if self.len() <= index {
            None
        } else {
            Self::find_kth_node(&self.root, index).map(|target_node| target_node.key)
        }
    }

    /// in_order traversal (left, root, right)
    pub fn in_order(&self) -> Vec<T> {
        fn _in_order<T: Ord + Copy + Default + std::fmt::Debug>(
            node: &Option<Box<Node<T>>>,
            res: &mut Vec<T>,
        ) {
            if let Some(n) = node {
                _in_order(&n.left, res);

                for _ in 0..n.count {
                    res.push(n.key);
                }
                _in_order(&n.right, res);
            }
        }

        let mut res = Vec::new();
        _in_order(&self.root, &mut res);

        res
    }

    /// pre_order traversal (root, left, right)
    pub fn pre_order(&self) -> Vec<T> {
        fn _pre_order<T: Ord + Copy + Default + std::fmt::Debug>(
            node: &Option<Box<Node<T>>>,
            res: &mut Vec<T>,
        ) {
            if let Some(n) = node {
                for _ in 0..n.count {
                    res.push(n.key);
                }

                _pre_order(&n.left, res);
                _pre_order(&n.right, res);
            }
        }

        let mut res = Vec::new();
        _pre_order(&self.root, &mut res);

        res
    }

    /// post_order traversal (left, right, root)
    pub fn post_order(&self) -> Vec<T> {
        fn _post_order<T: Ord + Copy + Default + std::fmt::Debug>(
            node: &Option<Box<Node<T>>>,
            res: &mut Vec<T>,
        ) {
            if let Some(n) = node {
                _post_order(&n.left, res);
                _post_order(&n.right, res);

                for _ in 0..n.count {
                    res.push(n.key);
                }
            }
        }

        let mut res = Vec::new();
        _post_order(&self.root, &mut res);

        res
    }

    pub fn level_order(&self) -> Vec<T> {
        let mut res = Vec::new();
        if self.root.is_none() {
            return res;
        }

        let mut que = VecDeque::new();
        que.push_back(self.root.as_ref());

        while let Some(Some(node)) = que.pop_front() {
            for _ in 0..node.count {
                res.push(node.key);
            }

            if node.left.is_some() {
                que.push_back(node.left.as_ref());
            }

            if node.right.is_some() {
                que.push_back(node.right.as_ref());
            }
        }
        res
    }
}

#[cfg(test)]
mod tests {
    use super::{Node, Treap};
    use std::collections::BTreeMap;

    fn create_treap_with_entries(entries: &[(i32, usize)]) -> Treap<i32> {
        let mut treap = Treap::new();
        for (key, count) in entries {
            treap.insert(*key, *count);
        }
        treap
    }

    fn create_manual_treap_with_root_having_two_children(root_count: usize) -> Treap<i32> {
        let left_node = Box::new(Node {
            key: 1,
            count: 1,
            subtree_size: 1,
            priority: 0.8,
            left: None,
            right: None,
        });
        let right_node = Box::new(Node {
            key: 3,
            count: 1,
            subtree_size: 1,
            priority: 0.7,
            left: None,
            right: None,
        });
        let root_node = Box::new(Node {
            key: 2,
            count: root_count,
            subtree_size: root_count + 2,
            priority: 0.9,
            left: Some(left_node),
            right: Some(right_node),
        });

        Treap {
            root: Some(root_node),
            size: root_count + 2,
            min_key: Some(1),
            max_key: Some(3),
        }
    }

    fn manual_subtree_size(node: &Option<Box<Node<i32>>>) -> usize {
        node.as_ref()
            .map_or(0_usize, |current_node| current_node.subtree_size)
    }

    fn create_manual_node(
        key: i32,
        count: usize,
        priority: f64,
        left: Option<Box<Node<i32>>>,
        right: Option<Box<Node<i32>>>,
    ) -> Box<Node<i32>> {
        let subtree_size = count + manual_subtree_size(&left) + manual_subtree_size(&right);

        Box::new(Node {
            key,
            count,
            subtree_size,
            priority,
            left,
            right,
        })
    }

    fn find_manual_min_key(node: &Node<i32>) -> i32 {
        if let Some(left_node) = node.left.as_ref() {
            find_manual_min_key(left_node.as_ref())
        } else {
            node.key
        }
    }

    fn find_manual_max_key(node: &Node<i32>) -> i32 {
        if let Some(right_node) = node.right.as_ref() {
            find_manual_max_key(right_node.as_ref())
        } else {
            node.key
        }
    }

    fn create_manual_treap_from_root(root_node: Box<Node<i32>>) -> Treap<i32> {
        let size = root_node.subtree_size;
        let min_key = Some(find_manual_min_key(root_node.as_ref()));
        let max_key = Some(find_manual_max_key(root_node.as_ref()));

        Treap {
            root: Some(root_node),
            size,
            min_key,
            max_key,
        }
    }

    fn create_manual_treap_for_traversal_orders() -> Treap<i32> {
        let left_right_node = create_manual_node(2_i32, 1_usize, 0.6_f64, None, None);
        let left_node = create_manual_node(1_i32, 1_usize, 0.7_f64, None, Some(left_right_node));
        let right_node = create_manual_node(4_i32, 1_usize, 0.8_f64, None, None);
        let root_node =
            create_manual_node(3_i32, 1_usize, 0.9_f64, Some(left_node), Some(right_node));

        create_manual_treap_from_root(root_node)
    }

    fn to_sorted_values_from_key_count_map(key_count_map: &BTreeMap<i32, usize>) -> Vec<i32> {
        let mut sorted_values = Vec::new();
        for (key, count) in key_count_map {
            for _ in 0..*count {
                sorted_values.push(*key);
            }
        }
        sorted_values
    }

    fn key_count_total_size(key_count_map: &BTreeMap<i32, usize>) -> usize {
        key_count_map.values().copied().sum::<usize>()
    }

    fn key_count_min_key(key_count_map: &BTreeMap<i32, usize>) -> Option<i32> {
        key_count_map.keys().next().copied()
    }

    fn key_count_max_key(key_count_map: &BTreeMap<i32, usize>) -> Option<i32> {
        key_count_map.keys().next_back().copied()
    }

    fn key_count_lower_bound(key_count_map: &BTreeMap<i32, usize>, key: i32) -> Option<i32> {
        key_count_map
            .range(key..)
            .next()
            .map(|(target_key, _)| *target_key)
    }

    fn key_count_upper_bound(key_count_map: &BTreeMap<i32, usize>, key: i32) -> Option<i32> {
        key_count_map
            .range(..key)
            .next_back()
            .map(|(target_key, _)| *target_key)
    }

    fn key_count_kth_min(key_count_map: &BTreeMap<i32, usize>, index: usize) -> Option<i32> {
        let mut traversed_count = 0_usize;
        for (key, count) in key_count_map {
            if index < traversed_count + count {
                return Some(*key);
            }
            traversed_count += count;
        }
        None
    }

    fn collect_kth_values(treap: &Treap<i32>) -> Vec<i32> {
        (0..treap.len())
            .map(|index| treap.kth_min(index).expect("index in range"))
            .collect()
    }

    fn collect_values_in_order_from_node(node: &Option<Box<Node<i32>>>) -> Vec<i32> {
        fn collect_node_values(node: &Option<Box<Node<i32>>>, values: &mut Vec<i32>) {
            if let Some(current_node) = node {
                collect_node_values(&current_node.left, values);
                for _ in 0..current_node.count {
                    values.push(current_node.key);
                }
                collect_node_values(&current_node.right, values);
            }
        }

        let mut values = Vec::new();
        collect_node_values(node, &mut values);
        values
    }

    fn key_count_insert(key_count_map: &mut BTreeMap<i32, usize>, key: i32, count: usize) {
        let current_count = key_count_map.get(&key).copied().unwrap_or(0);
        let next_count = current_count + count;
        key_count_map.insert(key, next_count);
    }

    fn key_count_erase_one(key_count_map: &mut BTreeMap<i32, usize>, key: i32) -> usize {
        let existing_count = key_count_map.get(&key).copied();

        if let Some(count) = existing_count {
            if count == 1 {
                key_count_map.remove(&key);
            } else {
                key_count_map.insert(key, count - 1);
            }
            1
        } else {
            0
        }
    }

    fn key_count_all_erase(key_count_map: &mut BTreeMap<i32, usize>, key: i32) -> usize {
        key_count_map.remove(&key).unwrap_or(0)
    }

    fn next_lcg_state(current_state: u64) -> u64 {
        current_state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407)
    }

    #[test]
    fn new_returns_empty_state() {
        // Arrange
        let expected_is_empty = true;
        let expected_len = 0_usize;
        let expected_min = None;
        let expected_max = None;
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_is_empty = treap.is_empty();
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn default_returns_same_initial_state_as_new() {
        // Arrange
        let expected_is_empty = true;
        let expected_len = 0_usize;
        let expected_min = None;
        let expected_max = None;
        let treap: Treap<i32> = Treap::default();

        // Act
        let actual_is_empty = treap.is_empty();
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn contains_returns_false_for_empty_set() {
        // Arrange
        let query_key = 10_i32;
        let expected_contains = false;
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_contains = treap.contains(query_key);

        // Assert
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn insert_single_count_updates_contains_count_and_len() {
        // Arrange
        let inserted_key = 7_i32;
        let inserted_count = 1_usize;
        let expected_contains = true;
        let expected_count = 1_usize;
        let expected_len = 1_usize;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(inserted_key, inserted_count);
        let actual_contains = treap.contains(inserted_key);
        let actual_count = treap.count_key(inserted_key);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn insert_single_then_contains_handles_x_minus_1_x_x_plus_1() {
        // Arrange
        let x = 10_i32;
        let x_minus_1 = x - 1;
        let x_plus_1 = x + 1;
        let inserted_count = 1_usize;
        let expected_contains_x_minus_1 = false;
        let expected_contains_x = true;
        let expected_contains_x_plus_1 = false;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(x, inserted_count);
        let actual_contains_x_minus_1 = treap.contains(x_minus_1);
        let actual_contains_x = treap.contains(x);
        let actual_contains_x_plus_1 = treap.contains(x_plus_1);

        // Assert
        assert_eq!(actual_contains_x_minus_1, expected_contains_x_minus_1);
        assert_eq!(actual_contains_x, expected_contains_x);
        assert_eq!(actual_contains_x_plus_1, expected_contains_x_plus_1);
    }

    #[test]
    fn count_key_returns_zero_for_empty_set() {
        // Arrange
        let query_key = 10_i32;
        let expected_count = 0_usize;
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_count = treap.count_key(query_key);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn split_by_key_isolates_target_key_from_smaller_and_larger_keys() {
        // Arrange
        let target_key = 2_i32;
        let expected_less_values = vec![1_i32];
        let expected_equal_values = vec![2_i32, 2_i32, 2_i32];
        let expected_greater_values = vec![3_i32];
        let treap =
            create_treap_with_entries(&[(1_i32, 1_usize), (target_key, 3_usize), (3_i32, 1_usize)]);

        // Act
        let (less_tree, equal_tree, greater_tree) = Treap::split_by_key(treap.root, target_key);
        let actual_less_values = collect_values_in_order_from_node(&less_tree);
        let actual_equal_values = collect_values_in_order_from_node(&equal_tree);
        let actual_greater_values = collect_values_in_order_from_node(&greater_tree);

        // Assert
        assert_eq!(actual_less_values, expected_less_values);
        assert_eq!(actual_equal_values, expected_equal_values);
        assert_eq!(actual_greater_values, expected_greater_values);
    }

    #[test]
    fn merge_three_way_after_split_by_key_restores_original_in_order_values() {
        // Arrange
        let target_key = 2_i32;
        let expected_values = vec![1_i32, 2_i32, 2_i32, 2_i32, 3_i32];
        let treap =
            create_treap_with_entries(&[(1_i32, 1_usize), (target_key, 3_usize), (3_i32, 1_usize)]);

        // Act
        let (less_tree, equal_tree, greater_tree) = Treap::split_by_key(treap.root, target_key);
        let merged_tree = Treap::merge_three_way(less_tree, equal_tree, greater_tree);
        let actual_values = collect_values_in_order_from_node(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn split_by_key_with_missing_smaller_key_puts_all_values_in_greater_tree() {
        // Arrange
        let split_key = 9_i32;
        let expected_less_values: Vec<i32> = vec![];
        let expected_equal_values: Vec<i32> = vec![];
        let expected_greater_values = vec![10_i32, 20_i32, 30_i32];
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let (less_tree, equal_tree, greater_tree) = Treap::split_by_key(treap.root, split_key);
        let actual_less_values = collect_values_in_order_from_node(&less_tree);
        let actual_equal_values = collect_values_in_order_from_node(&equal_tree);
        let actual_greater_values = collect_values_in_order_from_node(&greater_tree);

        // Assert
        assert_eq!(actual_less_values, expected_less_values);
        assert_eq!(actual_equal_values, expected_equal_values);
        assert_eq!(actual_greater_values, expected_greater_values);
    }

    #[test]
    fn split_by_key_with_missing_middle_key_returns_empty_equal_tree() {
        // Arrange
        let split_key = 25_i32;
        let expected_less_values = vec![10_i32, 20_i32];
        let expected_equal_values: Vec<i32> = vec![];
        let expected_greater_values = vec![30_i32];
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let (less_tree, equal_tree, greater_tree) = Treap::split_by_key(treap.root, split_key);
        let actual_less_values = collect_values_in_order_from_node(&less_tree);
        let actual_equal_values = collect_values_in_order_from_node(&equal_tree);
        let actual_greater_values = collect_values_in_order_from_node(&greater_tree);

        // Assert
        assert_eq!(actual_less_values, expected_less_values);
        assert_eq!(actual_equal_values, expected_equal_values);
        assert_eq!(actual_greater_values, expected_greater_values);
    }

    #[test]
    fn split_by_key_with_missing_greater_key_puts_all_values_in_less_tree() {
        // Arrange
        let split_key = 31_i32;
        let expected_less_values = vec![10_i32, 20_i32, 30_i32];
        let expected_equal_values: Vec<i32> = vec![];
        let expected_greater_values: Vec<i32> = vec![];
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let (less_tree, equal_tree, greater_tree) = Treap::split_by_key(treap.root, split_key);
        let actual_less_values = collect_values_in_order_from_node(&less_tree);
        let actual_equal_values = collect_values_in_order_from_node(&equal_tree);
        let actual_greater_values = collect_values_in_order_from_node(&greater_tree);

        // Assert
        assert_eq!(actual_less_values, expected_less_values);
        assert_eq!(actual_equal_values, expected_equal_values);
        assert_eq!(actual_greater_values, expected_greater_values);
    }

    #[test]
    fn merge_three_way_after_split_by_missing_key_restores_original_values() {
        // Arrange
        let split_key = 25_i32;
        let expected_values = vec![10_i32, 20_i32, 30_i32];
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let (less_tree, equal_tree, greater_tree) = Treap::split_by_key(treap.root, split_key);
        let merged_tree = Treap::merge_three_way(less_tree, equal_tree, greater_tree);
        let actual_values = collect_values_in_order_from_node(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn insert_same_key_multiple_times_accumulates_count_and_len() {
        // Arrange
        let target_key = 4_i32;
        let first_count = 2_usize;
        let second_count = 3_usize;
        let expected_count = 5_usize;
        let expected_len = 5_usize;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(target_key, first_count);
        treap.insert(target_key, second_count);
        let actual_count = treap.count_key(target_key);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn insert_multiple_unsorted_keys_keeps_in_order_sorted_with_multiplicity() {
        // Arrange
        let entries = vec![(5_i32, 1_usize), (3_i32, 2_usize), (4_i32, 1_usize)];
        let expected_in_order = vec![3_i32, 3_i32, 4_i32, 5_i32];
        let mut treap: Treap<i32> = Treap::new();

        // Act
        for (key, count) in entries {
            treap.insert(key, count);
        }
        let actual_in_order = treap.in_order();

        // Assert
        assert_eq!(actual_in_order, expected_in_order);
    }

    #[test]
    fn insert_min_and_max_i32_updates_min_and_max() {
        // Arrange
        let min_key = i32::MIN;
        let max_key = i32::MAX;
        let inserted_count = 1_usize;
        let expected_min = Some(min_key);
        let expected_max = Some(max_key);
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(max_key, inserted_count);
        treap.insert(min_key, inserted_count);
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn insert_zero_count_keeps_len_zero_and_creates_searchable_node_as_current_behavior() {
        // Arrange
        let inserted_key = 12_i32;
        let inserted_count = 0_usize;
        let expected_len = 0_usize;
        let expected_is_empty = false;
        let expected_contains = true;
        let expected_count = 0_usize;
        let expected_in_order: Vec<i32> = vec![];
        let expected_pre_order: Vec<i32> = vec![];
        let expected_post_order: Vec<i32> = vec![];
        let expected_level_order: Vec<i32> = vec![];
        let expected_min = Some(inserted_key);
        let expected_max = Some(inserted_key);
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(inserted_key, inserted_count);
        let actual_len = treap.len();
        let actual_is_empty = treap.is_empty();
        let actual_contains = treap.contains(inserted_key);
        let actual_count = treap.count_key(inserted_key);
        let actual_in_order = treap.in_order();
        let actual_pre_order = treap.pre_order();
        let actual_post_order = treap.post_order();
        let actual_level_order = treap.level_order();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_in_order, expected_in_order);
        assert_eq!(actual_pre_order, expected_pre_order);
        assert_eq!(actual_post_order, expected_post_order);
        assert_eq!(actual_level_order, expected_level_order);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn insert_zero_count_for_existing_key_keeps_existing_count_and_sorted_values() {
        // Arrange
        let inserted_key = 2_i32;
        let inserted_count = 0_usize;
        let expected_count = 2_usize;
        let expected_len = 4_usize;
        let expected_in_order = vec![1_i32, 2_i32, 2_i32, 3_i32];
        let mut treap = create_treap_with_entries(&[
            (1_i32, 1_usize),
            (inserted_key, 2_usize),
            (3_i32, 1_usize),
        ]);

        // Act
        treap.insert(inserted_key, inserted_count);
        let actual_count = treap.count_key(inserted_key);
        let actual_len = treap.len();
        let actual_in_order = treap.in_order();

        // Assert
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_in_order, expected_in_order);
    }

    #[test]
    fn insert_zero_count_for_new_key_exposes_current_search_and_bound_behavior() {
        // Arrange
        let inserted_key = 2_i32;
        let inserted_count = 0_usize;
        let expected_contains = true;
        let expected_len = 2_usize;
        let expected_min = Some(1_i32);
        let expected_max = Some(3_i32);
        let expected_lower_bound = Some(inserted_key);
        let expected_upper_bound = Some(1_i32);
        let mut treap = create_treap_with_entries(&[(1_i32, 1_usize), (3_i32, 1_usize)]);

        // Act
        treap.insert(inserted_key, inserted_count);
        let actual_contains = treap.contains(inserted_key);
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();
        let actual_lower_bound = treap.lower_bound(inserted_key);
        let actual_upper_bound = treap.upper_bound(inserted_key);

        // Assert
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
        assert_eq!(actual_lower_bound, expected_lower_bound);
        assert_eq!(actual_upper_bound, expected_upper_bound);
    }

    #[test]
    fn erase_on_empty_set_returns_zero_and_keeps_state() {
        // Arrange
        let erase_key = 3_i32;
        let expected_removed = 0_usize;
        let expected_len = 0_usize;
        let expected_min = None;
        let expected_max = None;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        let actual_removed = treap.erase(erase_key);
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn erase_on_missing_key_returns_zero_and_keeps_existing_counts() {
        // Arrange
        let erase_key = 3_i32;
        let expected_removed = 0_usize;
        let expected_count_of_two = 1_usize;
        let expected_count_of_four = 2_usize;
        let expected_len = 3_usize;
        let mut treap = create_treap_with_entries(&[(2_i32, 1_usize), (4_i32, 2_usize)]);

        let key_2 = 2_i32;
        let key_4 = 4_i32;
        // Act
        let actual_removed = treap.erase(erase_key);
        let actual_count_of_two = treap.count_key(key_2);
        let actual_count_of_four = treap.count_key(key_4);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count_of_two, expected_count_of_two);
        assert_eq!(actual_count_of_four, expected_count_of_four);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_on_duplicate_key_once_decrements_count_only_by_one() {
        // Arrange
        let target_key = 9_i32;
        let expected_removed = 1_usize;
        let expected_count = 2_usize;
        let expected_contains = true;
        let expected_len = 2_usize;
        let mut treap = create_treap_with_entries(&[(target_key, 3_usize)]);

        // Act
        let actual_removed = treap.erase(target_key);
        let actual_count = treap.count_key(target_key);
        let actual_contains = treap.contains(target_key);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_on_last_occurrence_of_duplicate_key_removes_node() {
        // Arrange
        let target_key = 9_i32;
        let expected_removed = 1_usize;
        let expected_count = 0_usize;
        let expected_contains = false;
        let mut treap = create_treap_with_entries(&[(target_key, 2_usize)]);

        let prepared_removed = treap.erase(target_key);
        // Act
        let actual_removed = treap.erase(target_key);
        let actual_count = treap.count_key(target_key);
        let actual_contains = treap.contains(target_key);

        // Assert
        assert_eq!(prepared_removed, expected_removed);
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_contains, expected_contains);
    }

    #[test]
    fn erase_min_key_updates_min_to_next_key() {
        // Arrange
        let erase_key = 1_i32;
        let expected_min = Some(2_i32);
        let mut treap =
            create_treap_with_entries(&[(1_i32, 1_usize), (2_i32, 1_usize), (3_i32, 1_usize)]);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_min = treap.first().copied();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_min, expected_min);
    }

    #[test]
    fn erase_max_key_updates_max_to_previous_key() {
        // Arrange
        let erase_key = 3_i32;
        let expected_max = Some(2_i32);
        let mut treap =
            create_treap_with_entries(&[(1_i32, 1_usize), (2_i32, 1_usize), (3_i32, 1_usize)]);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn erase_on_single_element_tree_resets_empty_state() {
        // Arrange
        let erase_key = 5_i32;
        let expected_is_empty = true;
        let expected_len = 0_usize;
        let expected_min = None;
        let expected_max = None;
        let mut treap = create_treap_with_entries(&[(erase_key, 1_usize)]);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_is_empty = treap.is_empty();
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn erase_key_with_two_children_keeps_in_order_sorted() {
        // Arrange
        let erase_key = 2_i32;
        let expected_in_order = vec![1_i32, 3_i32];
        let mut treap = create_manual_treap_with_root_having_two_children(1_usize);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_in_order = treap.in_order();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_in_order, expected_in_order);
    }

    #[test]
    fn erase_key_with_two_children_returns_one_removed() {
        // Arrange
        let erase_key = 2_i32;
        let expected_removed = 1_usize;
        let mut treap = create_manual_treap_with_root_having_two_children(1_usize);

        // Act
        let actual_removed = treap.erase(erase_key);

        // Assert
        assert_eq!(actual_removed, expected_removed);
    }

    #[test]
    fn erase_zero_count_node_returns_zero_and_restores_empty_state() {
        // Arrange
        let target_key = 6_i32;
        let inserted_count = 0_usize;
        let expected_removed = 0_usize;
        let expected_is_empty = true;
        let expected_len = 0_usize;
        let expected_contains = false;
        let expected_min = None;
        let expected_max = None;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(target_key, inserted_count);
        let actual_removed = treap.erase(target_key);
        let actual_is_empty = treap.is_empty();
        let actual_len = treap.len();
        let actual_contains = treap.contains(target_key);
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn erase_duplicate_root_with_two_children_keeps_children_and_decrements_count() {
        // Arrange
        let erase_key = 2_i32;
        let expected_root_key = Some(2_i32);
        let expected_root_count = Some(1_usize);
        let expected_left_key = Some(1_i32);
        let expected_right_key = Some(3_i32);
        let expected_len = 3_usize;
        let expected_in_order = vec![1_i32, 2_i32, 3_i32];
        let mut treap = create_manual_treap_with_root_having_two_children(2_usize);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_root_key = treap.root.as_ref().map(|root_node| root_node.key);
        let actual_root_count = treap.root.as_ref().map(|root_node| root_node.count);
        let actual_left_key = treap
            .root
            .as_ref()
            .and_then(|root_node| root_node.left.as_ref().map(|left_node| left_node.key));
        let actual_right_key = treap
            .root
            .as_ref()
            .and_then(|root_node| root_node.right.as_ref().map(|right_node| right_node.key));
        let actual_len = treap.len();
        let actual_in_order = treap.in_order();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_root_key, expected_root_key);
        assert_eq!(actual_root_count, expected_root_count);
        assert_eq!(actual_left_key, expected_left_key);
        assert_eq!(actual_right_key, expected_right_key);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_in_order, expected_in_order);
    }

    #[test]
    fn erase_on_duplicated_min_key_keeps_same_min_key() {
        // Arrange
        let erase_key = 1_i32;
        let expected_min = Some(1_i32);
        let expected_count = 2_usize;
        let expected_len = 4_usize;
        let mut treap =
            create_treap_with_entries(&[(erase_key, 3_usize), (2_i32, 1_usize), (3_i32, 1_usize)]);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_min = treap.first().copied();
        let actual_count = treap.count_key(erase_key);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_on_duplicated_max_key_keeps_same_max_key() {
        // Arrange
        let erase_key = 3_i32;
        let expected_max = Some(3_i32);
        let expected_count = 2_usize;
        let expected_len = 4_usize;
        let mut treap =
            create_treap_with_entries(&[(1_i32, 1_usize), (2_i32, 1_usize), (erase_key, 3_usize)]);

        let expected_removed_1 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(erase_key);
        let actual_max = treap.last().copied();
        let actual_count = treap.count_key(erase_key);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_max, expected_max);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn all_erase_on_empty_set_returns_zero_and_keeps_state() {
        // Arrange
        let erase_key = 3_i32;
        let expected_removed = 0_usize;
        let expected_len = 0_usize;
        let expected_min = None;
        let expected_max = None;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        let actual_removed = treap.all_erase(erase_key);
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn all_erase_on_missing_key_returns_zero_and_keeps_existing_counts() {
        // Arrange
        let erase_key = 3_i32;
        let expected_removed = 0_usize;
        let expected_count_of_two = 1_usize;
        let expected_count_of_four = 2_usize;
        let expected_len = 3_usize;
        let mut treap = create_treap_with_entries(&[(2_i32, 1_usize), (4_i32, 2_usize)]);

        let key_2 = 2_i32;
        let key_4 = 4_i32;
        // Act
        let actual_removed = treap.all_erase(erase_key);
        let actual_count_of_two = treap.count_key(key_2);
        let actual_count_of_four = treap.count_key(key_4);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_count_of_two, expected_count_of_two);
        assert_eq!(actual_count_of_four, expected_count_of_four);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn all_erase_on_duplicate_key_removes_all_counts() {
        // Arrange
        let target_key = 7_i32;
        let expected_removed = 4_usize;
        let expected_contains = false;
        let expected_len = 1_usize;
        let mut treap = create_treap_with_entries(&[(target_key, 4_usize), (8_i32, 1_usize)]);

        // Act
        let actual_removed = treap.all_erase(target_key);
        let actual_contains = treap.contains(target_key);
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn all_erase_min_key_updates_min_to_next_key() {
        // Arrange
        let erase_key = 1_i32;
        let expected_min = Some(2_i32);
        let mut treap =
            create_treap_with_entries(&[(1_i32, 2_usize), (2_i32, 1_usize), (3_i32, 1_usize)]);

        let expected_removed_1 = 2_usize;
        // Act
        let actual_removed_1 = treap.all_erase(erase_key);
        let actual_min = treap.first().copied();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_min, expected_min);
    }

    #[test]
    fn all_erase_max_key_updates_max_to_previous_key() {
        // Arrange
        let erase_key = 3_i32;
        let expected_max = Some(2_i32);
        let mut treap =
            create_treap_with_entries(&[(1_i32, 1_usize), (2_i32, 1_usize), (3_i32, 2_usize)]);

        let expected_removed_1 = 2_usize;
        // Act
        let actual_removed_1 = treap.all_erase(erase_key);
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn all_erase_on_single_element_tree_resets_empty_state() {
        // Arrange
        let erase_key = 5_i32;
        let expected_is_empty = true;
        let expected_len = 0_usize;
        let expected_min = None;
        let expected_max = None;
        let mut treap = create_treap_with_entries(&[(erase_key, 3_usize)]);

        let expected_removed_1 = 3_usize;
        // Act
        let actual_removed_1 = treap.all_erase(erase_key);
        let actual_is_empty = treap.is_empty();
        let actual_len = treap.len();
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn all_erase_key_with_two_children_keeps_in_order_sorted() {
        // Arrange
        let erase_key = 2_i32;
        let expected_in_order = vec![1_i32, 3_i32];
        let mut treap = create_manual_treap_with_root_having_two_children(2_usize);

        let expected_removed_1 = 2_usize;
        // Act
        let actual_removed_1 = treap.all_erase(erase_key);
        let actual_in_order = treap.in_order();

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);

        assert_eq!(actual_in_order, expected_in_order);
    }

    #[test]
    fn all_erase_key_with_two_children_returns_root_count() {
        // Arrange
        let erase_key = 2_i32;
        let expected_removed = 2_usize;
        let mut treap = create_manual_treap_with_root_having_two_children(2_usize);

        // Act
        let actual_removed = treap.all_erase(erase_key);

        // Assert
        assert_eq!(actual_removed, expected_removed);
    }

    #[test]
    fn all_erase_zero_count_node_returns_zero_and_restores_empty_state() {
        // Arrange
        let target_key = 6_i32;
        let inserted_count = 0_usize;
        let expected_removed = 0_usize;
        let expected_is_empty = true;
        let expected_len = 0_usize;
        let expected_contains = false;
        let expected_min = None;
        let expected_max = None;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(target_key, inserted_count);
        let actual_removed = treap.all_erase(target_key);
        let actual_is_empty = treap.is_empty();
        let actual_len = treap.len();
        let actual_contains = treap.contains(target_key);
        let actual_min = treap.first().copied();
        let actual_max = treap.last().copied();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_contains, expected_contains);
        assert_eq!(actual_min, expected_min);
        assert_eq!(actual_max, expected_max);
    }

    #[test]
    fn kth_min_updates_after_erase_and_all_erase() {
        // Arrange
        let expected_values = vec![1_i32, 2_i32, 4_i32];
        let mut treap = create_treap_with_entries(&[
            (1_i32, 1_usize),
            (2_i32, 2_usize),
            (3_i32, 1_usize),
            (4_i32, 1_usize),
        ]);

        let key_2 = 2_i32;
        let key_3 = 3_i32;
        let expected_removed_1 = 1_usize;
        let expected_removed_2 = 1_usize;
        // Act
        let actual_removed_1 = treap.erase(key_2);
        let actual_removed_2 = treap.all_erase(key_3);
        let actual_values = collect_kth_values(&treap);

        // Assert
        assert_eq!(actual_removed_1, expected_removed_1);
        assert_eq!(actual_removed_2, expected_removed_2);

        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pop_first_returns_none_for_empty_set() {
        // Arrange
        let expected_popped_value = None;
        let expected_len = 0_usize;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        let actual_popped_value = treap.pop_first();
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_first_on_duplicate_minimum_decrements_count_and_keeps_same_first() {
        // Arrange
        let expected_popped_value = Some(1_i32);
        let expected_count = 1_usize;
        let expected_len = 2_usize;
        let expected_first = Some(1_i32);
        let mut treap = create_treap_with_entries(&[(1_i32, 2_usize), (3_i32, 1_usize)]);

        let key_1 = 1_i32;
        // Act
        let actual_popped_value = treap.pop_first();
        let actual_count = treap.count_key(key_1);
        let actual_len = treap.len();
        let actual_first = treap.first().copied();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_first, expected_first);
    }

    #[test]
    fn pop_first_skips_zero_count_boundary_node_and_returns_first_counted_key() {
        // Arrange
        let zero_count_key = 1_i32;
        let expected_popped_value = Some(2_i32);
        let expected_contains_zero_count_key = false;
        let expected_len = 1_usize;
        let expected_first = Some(3_i32);
        let mut treap = create_treap_with_entries(&[(2_i32, 1_usize), (3_i32, 1_usize)]);

        let count_0 = 0_usize;
        // Act
        treap.insert(zero_count_key, count_0);
        let actual_popped_value = treap.pop_first();
        let actual_contains_zero_count_key = treap.contains(zero_count_key);
        let actual_len = treap.len();
        let actual_first = treap.first().copied();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(
            actual_contains_zero_count_key,
            expected_contains_zero_count_key
        );
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_first, expected_first);
    }

    #[test]
    fn pop_last_returns_none_for_empty_set() {
        // Arrange
        let expected_popped_value = None;
        let expected_len = 0_usize;
        let mut treap: Treap<i32> = Treap::new();

        // Act
        let actual_popped_value = treap.pop_last();
        let actual_len = treap.len();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_last_on_duplicate_maximum_decrements_count_and_keeps_same_last() {
        // Arrange
        let expected_popped_value = Some(3_i32);
        let expected_count = 1_usize;
        let expected_len = 2_usize;
        let expected_last = Some(3_i32);
        let mut treap = create_treap_with_entries(&[(1_i32, 1_usize), (3_i32, 2_usize)]);

        let key_3 = 3_i32;
        // Act
        let actual_popped_value = treap.pop_last();
        let actual_count = treap.count_key(key_3);
        let actual_len = treap.len();
        let actual_last = treap.last().copied();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_last, expected_last);
    }

    #[test]
    fn pop_last_skips_zero_count_boundary_node_and_returns_last_counted_key() {
        // Arrange
        let zero_count_key = 4_i32;
        let expected_popped_value = Some(3_i32);
        let expected_contains_zero_count_key = false;
        let expected_len = 1_usize;
        let expected_last = Some(2_i32);
        let mut treap = create_treap_with_entries(&[(2_i32, 1_usize), (3_i32, 1_usize)]);

        let count_0 = 0_usize;
        // Act
        treap.insert(zero_count_key, count_0);
        let actual_popped_value = treap.pop_last();
        let actual_contains_zero_count_key = treap.contains(zero_count_key);
        let actual_len = treap.len();
        let actual_last = treap.last().copied();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(
            actual_contains_zero_count_key,
            expected_contains_zero_count_key
        );
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_last, expected_last);
    }

    #[test]
    fn lower_bound_on_empty_set_returns_none() {
        // Arrange
        let query_key = 10_i32;
        let expected_bound = None;
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_bound = treap.lower_bound(query_key);

        // Assert
        assert_eq!(actual_bound, expected_bound);
    }

    #[test]
    fn lower_and_upper_bound_when_key_is_less_than_min_key() {
        // Arrange
        let query_key = 5_i32;
        let expected_lower_bound = Some(10_i32);
        let expected_upper_bound = None;
        let treap = create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize)]);

        // Act
        let actual_lower_bound = treap.lower_bound(query_key);
        let actual_upper_bound = treap.upper_bound(query_key);

        // Assert
        assert_eq!(actual_lower_bound, expected_lower_bound);
        assert_eq!(actual_upper_bound, expected_upper_bound);
    }

    #[test]
    fn lower_bound_handles_x_minus_1_x_x_plus_1_around_existing_key() {
        // Arrange
        let x = 20_i32;
        let x_minus_1 = x - 1;
        let x_plus_1 = x + 1;
        let expected_for_x_minus_1 = Some(20_i32);
        let expected_for_x = Some(20_i32);
        let expected_for_x_plus_1 = Some(30_i32);
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let actual_for_x_minus_1 = treap.lower_bound(x_minus_1);
        let actual_for_x = treap.lower_bound(x);
        let actual_for_x_plus_1 = treap.lower_bound(x_plus_1);

        // Assert
        assert_eq!(actual_for_x_minus_1, expected_for_x_minus_1);
        assert_eq!(actual_for_x, expected_for_x);
        assert_eq!(actual_for_x_plus_1, expected_for_x_plus_1);
    }

    #[test]
    fn lower_and_upper_bound_when_key_is_greater_than_max_key() {
        // Arrange
        let query_key = 31_i32;
        let expected_lower_bound = None;
        let expected_upper_bound = Some(30_i32);
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let actual_lower_bound = treap.lower_bound(query_key);
        let actual_upper_bound = treap.upper_bound(query_key);

        // Assert
        assert_eq!(actual_lower_bound, expected_lower_bound);
        assert_eq!(actual_upper_bound, expected_upper_bound);
    }

    #[test]
    fn lower_and_upper_bound_are_independent_from_duplicate_counts() {
        // Arrange
        let lower_query_key = 20_i32;
        let upper_query_key = 21_i32;
        let expected_lower_bound = Some(20_i32);
        let expected_upper_bound = Some(20_i32);
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 3_usize), (30_i32, 1_usize)]);

        // Act
        let actual_lower_bound = treap.lower_bound(lower_query_key);
        let actual_upper_bound = treap.upper_bound(upper_query_key);

        // Assert
        assert_eq!(actual_lower_bound, expected_lower_bound);
        assert_eq!(actual_upper_bound, expected_upper_bound);
    }

    #[test]
    fn lower_and_upper_bound_on_boundary_keys_return_expected_values() {
        // Arrange
        let min_key = 10_i32;
        let max_key = 30_i32;
        let expected_lower_bound_for_min = Some(min_key);
        let expected_upper_bound_for_min = None;
        let expected_lower_bound_for_max = Some(max_key);
        let expected_upper_bound_for_max = Some(20_i32);
        let treap =
            create_treap_with_entries(&[(min_key, 1_usize), (20_i32, 1_usize), (max_key, 1_usize)]);

        // Act
        let actual_lower_bound_for_min = treap.lower_bound(min_key);
        let actual_upper_bound_for_min = treap.upper_bound(min_key);
        let actual_lower_bound_for_max = treap.lower_bound(max_key);
        let actual_upper_bound_for_max = treap.upper_bound(max_key);

        // Assert
        assert_eq!(actual_lower_bound_for_min, expected_lower_bound_for_min);
        assert_eq!(actual_upper_bound_for_min, expected_upper_bound_for_min);
        assert_eq!(actual_lower_bound_for_max, expected_lower_bound_for_max);
        assert_eq!(actual_upper_bound_for_max, expected_upper_bound_for_max);
    }

    #[test]
    fn upper_bound_on_empty_set_returns_none() {
        // Arrange
        let query_key = 10_i32;
        let expected_bound = None;
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_bound = treap.upper_bound(query_key);

        // Assert
        assert_eq!(actual_bound, expected_bound);
    }

    #[test]
    fn upper_bound_handles_x_minus_1_x_x_plus_1_around_existing_key() {
        // Arrange
        let x = 20_i32;
        let x_minus_1 = x - 1;
        let x_plus_1 = x + 1;
        let expected_for_x_minus_1 = Some(10_i32);
        let expected_for_x = Some(10_i32);
        let expected_for_x_plus_1 = Some(20_i32);
        let treap =
            create_treap_with_entries(&[(10_i32, 1_usize), (20_i32, 1_usize), (30_i32, 1_usize)]);

        // Act
        let actual_for_x_minus_1 = treap.upper_bound(x_minus_1);
        let actual_for_x = treap.upper_bound(x);
        let actual_for_x_plus_1 = treap.upper_bound(x_plus_1);

        // Assert
        assert_eq!(actual_for_x_minus_1, expected_for_x_minus_1);
        assert_eq!(actual_for_x, expected_for_x);
        assert_eq!(actual_for_x_plus_1, expected_for_x_plus_1);
    }

    #[test]
    fn kth_min_returns_none_for_empty_set() {
        // Arrange
        let query_index = 0_usize;
        let expected_value = None;
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_value = treap.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_returns_only_element_for_single_key_tree() {
        // Arrange
        let query_index = 0_usize;
        let expected_value = Some(7_i32);
        let treap = create_treap_with_entries(&[(7_i32, 1_usize)]);

        // Act
        let actual_value = treap.kth_min(query_index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_min_returns_expected_values_for_boundary_indices() {
        // Arrange
        let first_index = 0_usize;
        let treap = create_treap_with_entries(&[
            (7_i32, 1_usize),
            (3_i32, 1_usize),
            (9_i32, 1_usize),
            (5_i32, 1_usize),
        ]);
        let last_index = treap.len() - 1;
        let out_of_range_index = treap.len();
        let expected_first_value = Some(3_i32);
        let expected_last_value = Some(9_i32);
        let expected_out_of_range_value = None;

        // Act
        let actual_first_value = treap.kth_min(first_index);
        let actual_last_value = treap.kth_min(last_index);
        let actual_out_of_range_value = treap.kth_min(out_of_range_index);

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_last_value, expected_last_value);
        assert_eq!(actual_out_of_range_value, expected_out_of_range_value);
    }

    #[test]
    fn kth_min_counts_duplicate_keys_as_separate_positions() {
        // Arrange
        let expected_values = vec![1_i32, 2_i32, 2_i32, 3_i32];
        let treap =
            create_treap_with_entries(&[(2_i32, 2_usize), (1_i32, 1_usize), (3_i32, 1_usize)]);

        // Act
        let actual_values = collect_kth_values(&treap);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn kth_min_skips_zero_count_node_between_existing_keys() {
        // Arrange
        let zero_count_key = 2_i32;
        let zero_count = 0_usize;
        let first_index = 0_usize;
        let second_index = 1_usize;
        let out_of_range_index = 2_usize;
        let expected_first_value = Some(1_i32);
        let expected_second_value = Some(3_i32);
        let expected_out_of_range_value = None;
        let mut treap = create_treap_with_entries(&[(1_i32, 1_usize), (3_i32, 1_usize)]);

        // Act
        treap.insert(zero_count_key, zero_count);
        let actual_first_value = treap.kth_min(first_index);
        let actual_second_value = treap.kth_min(second_index);
        let actual_out_of_range_value = treap.kth_min(out_of_range_index);

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_second_value, expected_second_value);
        assert_eq!(actual_out_of_range_value, expected_out_of_range_value);
    }

    #[test]
    fn in_order_on_multi_key_with_duplicates_returns_sorted_multiset() {
        // Arrange
        let expected_in_order = vec![1_i32, 3_i32, 3_i32, 3_i32, 4_i32, 4_i32];
        let treap =
            create_treap_with_entries(&[(4_i32, 2_usize), (1_i32, 1_usize), (3_i32, 3_usize)]);

        // Act
        let actual_in_order = treap.in_order();

        // Assert
        assert_eq!(actual_in_order, expected_in_order);
    }

    #[test]
    fn sorted_traversal_results_match_in_order_multiset_for_multi_key_with_duplicates() {
        // Arrange
        let treap =
            create_treap_with_entries(&[(4_i32, 2_usize), (1_i32, 1_usize), (3_i32, 3_usize)]);

        // Act
        let actual_in_order = treap.in_order();
        let mut actual_pre_order_sorted = treap.pre_order();
        let mut actual_post_order_sorted = treap.post_order();
        let mut actual_level_order_sorted = treap.level_order();
        actual_pre_order_sorted.sort();
        actual_post_order_sorted.sort();
        actual_level_order_sorted.sort();

        // Assert
        assert_eq!(actual_pre_order_sorted, actual_in_order);
        assert_eq!(actual_post_order_sorted, actual_in_order);
        assert_eq!(actual_level_order_sorted, actual_in_order);
    }

    #[test]
    fn mixed_operations_keep_len_equal_to_in_order_length_on_each_step() {
        // Arrange
        // Code 0 inserts, 1 erases one occurrence, and 2 erases every occurrence.
        let operations = [
            (0, 3, 2),
            (0, 1, 1),
            (0, 5, 1),
            (1, 3, 0),
            (2, 1, 0),
            (0, 4, 3),
            (1, 5, 0),
            (2, 3, 0),
            (0, 2, 1),
        ];
        let expected_removed = vec![
            None,
            None,
            None,
            Some(1),
            Some(1),
            None,
            Some(1),
            Some(1),
            None,
        ];
        let expected_lengths = vec![2, 3, 4, 3, 2, 5, 4, 3, 4];
        let mut treap: Treap<i32> = Treap::new();

        // Act
        let mut actual_removed = Vec::new();
        let mut actual_lengths = Vec::new();
        let mut actual_traversal_lengths = Vec::new();
        for (code, key, count) in operations {
            let removed = match code {
                0 => {
                    treap.insert(key, count);
                    None
                }
                1 => Some(treap.erase(key)),
                _ => Some(treap.all_erase(key)),
            };
            actual_removed.push(removed);
            actual_lengths.push(treap.len());
            actual_traversal_lengths.push(treap.in_order().len());
        }

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_lengths, expected_lengths);
        assert_eq!(actual_lengths, actual_traversal_lengths);
    }

    #[test]
    fn manual_tree_returns_expected_pre_post_and_level_orders() {
        // Arrange
        let expected_pre_order = vec![3_i32, 1_i32, 2_i32, 4_i32];
        let expected_post_order = vec![2_i32, 1_i32, 4_i32, 3_i32];
        let expected_level_order = vec![3_i32, 1_i32, 4_i32, 2_i32];
        let treap = create_manual_treap_for_traversal_orders();

        // Act
        let actual_pre_order = treap.pre_order();
        let actual_post_order = treap.post_order();
        let actual_level_order = treap.level_order();

        // Assert
        assert_eq!(actual_pre_order, expected_pre_order);
        assert_eq!(actual_post_order, expected_post_order);
        assert_eq!(actual_level_order, expected_level_order);
    }

    #[test]
    fn traversals_on_empty_set_return_empty_vectors() {
        // Arrange
        let expected_values: Vec<i32> = vec![];
        let treap: Treap<i32> = Treap::new();

        // Act
        let actual_in_order = treap.in_order();
        let actual_pre_order = treap.pre_order();
        let actual_post_order = treap.post_order();
        let actual_level_order = treap.level_order();

        // Assert
        assert_eq!(actual_in_order, expected_values);
        assert_eq!(actual_pre_order, expected_values);
        assert_eq!(actual_post_order, expected_values);
        assert_eq!(actual_level_order, expected_values);
    }

    #[test]
    fn traversals_on_single_key_with_count_c_return_c_same_values() {
        // Arrange
        let key = 7_i32;
        let count = 3_usize;
        let expected_values = vec![7_i32, 7_i32, 7_i32];
        let mut treap: Treap<i32> = Treap::new();

        // Act
        treap.insert(key, count);
        let actual_in_order = treap.in_order();
        let actual_pre_order = treap.pre_order();
        let actual_post_order = treap.post_order();
        let actual_level_order = treap.level_order();

        // Assert
        assert_eq!(actual_in_order, expected_values);
        assert_eq!(actual_pre_order, expected_values);
        assert_eq!(actual_post_order, expected_values);
        assert_eq!(actual_level_order, expected_values);
    }

    #[test]
    fn traversal_lengths_match_len_for_multi_key_with_duplicates() {
        // Arrange
        let expected_len = 6_usize;
        let treap =
            create_treap_with_entries(&[(4_i32, 2_usize), (1_i32, 1_usize), (3_i32, 3_usize)]);

        // Act
        let actual_len = treap.len();
        let actual_pre_order_len = treap.pre_order().len();
        let actual_post_order_len = treap.post_order().len();
        let actual_level_order_len = treap.level_order().len();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_pre_order_len, expected_len);
        assert_eq!(actual_post_order_len, expected_len);
        assert_eq!(actual_level_order_len, expected_len);
    }

    #[test]
    fn repeated_traversal_calls_do_not_change_results() {
        // Arrange
        let treap =
            create_treap_with_entries(&[(4_i32, 2_usize), (1_i32, 1_usize), (3_i32, 3_usize)]);

        // Act
        let first_in_order = treap.in_order();
        let second_in_order = treap.in_order();
        let first_pre_order = treap.pre_order();
        let second_pre_order = treap.pre_order();
        let first_post_order = treap.post_order();
        let second_post_order = treap.post_order();
        let first_level_order = treap.level_order();
        let second_level_order = treap.level_order();

        // Assert
        assert_eq!(first_in_order, second_in_order);
        assert_eq!(first_pre_order, second_pre_order);
        assert_eq!(first_post_order, second_post_order);
        assert_eq!(first_level_order, second_level_order);
    }

    #[test]
    fn merge_prefers_left_root_when_root_priorities_are_equal() {
        // Arrange
        let left_root_priority = 0.5_f64;
        let right_root_priority = 0.5_f64;
        let expected_root_key = Some(1_i32);
        let expected_values = vec![1_i32, 2_i32];
        let left_tree = Some(create_manual_node(
            1_i32,
            1_usize,
            left_root_priority,
            None,
            None,
        ));
        let right_tree = Some(create_manual_node(
            2_i32,
            1_usize,
            right_root_priority,
            None,
            None,
        ));

        // Act
        let actual_merged_tree = Treap::merge(left_tree, right_tree);
        let actual_root_key = actual_merged_tree.as_ref().map(|root_node| root_node.key);
        let actual_values = collect_values_in_order_from_node(&actual_merged_tree);

        // Assert
        assert_eq!(actual_root_key, expected_root_key);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn long_pseudo_random_operations_match_btreemap_reference_results() {
        // Arrange
        let seed = 123_456_789_u64;
        let step_count = 400_usize;
        let key_mod = 41_u64;
        let key_offset = 20_i32;
        let mut treap: Treap<i32> = Treap::new();
        let mut key_count_map: BTreeMap<i32, usize> = BTreeMap::new();
        let mut lcg_state = seed;
        let mut cases = Vec::with_capacity(step_count);
        for _ in 0..step_count {
            lcg_state = next_lcg_state(lcg_state);
            let operation_code = lcg_state % 3_u64;
            lcg_state = next_lcg_state(lcg_state);
            let operation_key = (lcg_state % key_mod) as i32 - key_offset;
            let insert_count = if operation_code == 0 {
                lcg_state = next_lcg_state(lcg_state);
                (lcg_state % 3_u64 + 1_u64) as usize
            } else {
                0
            };
            let expected_removed = match operation_code {
                0 => {
                    key_count_insert(&mut key_count_map, operation_key, insert_count);
                    None
                }
                1 => Some(key_count_erase_one(&mut key_count_map, operation_key)),
                _ => Some(key_count_all_erase(&mut key_count_map, operation_key)),
            };
            lcg_state = next_lcg_state(lcg_state);
            let query_key = (lcg_state % key_mod) as i32 - key_offset;
            let expected_len = key_count_total_size(&key_count_map);
            let query_indices = (0..=expected_len).collect::<Vec<_>>();
            let expected_state = (
                expected_removed,
                key_count_map.contains_key(&query_key),
                key_count_map.get(&query_key).copied().unwrap_or(0),
                expected_len,
                key_count_min_key(&key_count_map),
                key_count_max_key(&key_count_map),
                key_count_lower_bound(&key_count_map, query_key),
                key_count_upper_bound(&key_count_map, query_key),
                query_indices
                    .iter()
                    .map(|&index| key_count_kth_min(&key_count_map, index))
                    .collect::<Vec<_>>(),
                to_sorted_values_from_key_count_map(&key_count_map),
            );
            cases.push((
                operation_code,
                operation_key,
                insert_count,
                query_key,
                query_indices,
                expected_state,
            ));
        }
        let expected_states: Vec<_> = cases.iter().map(|case| case.5.clone()).collect();

        // Act
        let actual_states: Vec<_> = cases
            .iter()
            .map(
                |&(
                    operation_code,
                    operation_key,
                    insert_count,
                    query_key,
                    ref query_indices,
                    _,
                )| {
                    let actual_removed = match operation_code {
                        0 => {
                            treap.insert(operation_key, insert_count);
                            None
                        }
                        1 => Some(treap.erase(operation_key)),
                        _ => Some(treap.all_erase(operation_key)),
                    };
                    (
                        actual_removed,
                        treap.contains(query_key),
                        treap.count_key(query_key),
                        treap.len(),
                        treap.first().copied(),
                        treap.last().copied(),
                        treap.lower_bound(query_key),
                        treap.upper_bound(query_key),
                        query_indices
                            .iter()
                            .map(|&index| treap.kth_min(index))
                            .collect::<Vec<_>>(),
                        treap.in_order(),
                    )
                },
            )
            .collect();

        // Assert
        for (step, (actual, expected)) in actual_states.iter().zip(&expected_states).enumerate() {
            assert_eq!(actual, expected, "state mismatch after step {step}");
        }
    }
}
