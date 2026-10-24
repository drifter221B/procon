use rand::RngCore;

#[derive(Debug)]
struct Node<T> {
    value: T,
    priority: u64,
    size: usize,
    left: Option<Box<Node<T>>>,
    right: Option<Box<Node<T>>>,
}

impl<T> Node<T> {
    fn new(value: T, priority: u64) -> Box<Self> {
        Box::new(Self {
            value,
            priority,
            size: 1,
            left: None,
            right: None,
        })
    }
}

fn size_of<T>(node: &Option<Box<Node<T>>>) -> usize {
    node.as_ref().map_or(0, |n| n.size)
}

fn update_size<T>(node: &mut Box<Node<T>>) {
    node.size = 1 + size_of(&node.left) + size_of(&node.right);
}

fn merge<T>(left: Option<Box<Node<T>>>, right: Option<Box<Node<T>>>) -> Option<Box<Node<T>>> {
    match (left, right) {
        (None, r) => r,
        (l, None) => l,
        (Some(mut l), Some(mut r)) => {
            if l.priority > r.priority {
                let merged = merge(l.right.take(), Some(r));
                l.right = merged;
                update_size(&mut l);
                Some(l)
            } else {
                let merged = merge(Some(l), r.left.take());
                r.left = merged;
                update_size(&mut r);
                Some(r)
            }
        }
    }
}

fn split<T>(node: Option<Box<Node<T>>>, k: usize) -> (Option<Box<Node<T>>>, Option<Box<Node<T>>>) {
    match node {
        None => (None, None),
        Some(mut root) => {
            let left_size = size_of(&root.left);
            if k <= left_size {
                let (a, b) = split(root.left.take(), k);
                root.left = b;
                update_size(&mut root);
                (a, Some(root))
            } else {
                let (a, b) = split(root.right.take(), k - left_size - 1);
                root.right = a;
                update_size(&mut root);
                (Some(root), b)
            }
        }
    }
}

fn kth<T: Clone>(node: &Option<Box<Node<T>>>, index: usize) -> Option<T> {
    let mut current = node.as_ref()?;
    let mut idx = index;
    loop {
        let left_size = size_of(&current.left);
        if idx < left_size {
            current = current.left.as_ref()?;
        } else if idx == left_size {
            return Some(current.value.clone());
        } else {
            idx -= left_size + 1;
            current = current.right.as_ref()?;
        }
    }
}

#[derive(Debug, Default)]
pub struct DynamicArray<T> {
    root: Option<Box<Node<T>>>,
    rng: rand::rngs::ThreadRng,
}

pub struct Iter<'a, T> {
    node_stack: Vec<&'a Node<T>>,
}

impl<'a, T> Iter<'a, T> {
    fn new(root_node: Option<&'a Node<T>>) -> Self {
        let mut node_stack = Vec::new();
        push_left_nodes(root_node, &mut node_stack);
        Self { node_stack }
    }
}

impl<'a, T> Iterator for Iter<'a, T> {
    type Item = &'a T;

    fn next(&mut self) -> Option<Self::Item> {
        let current_node = self.node_stack.pop()?;
        push_left_nodes(current_node.right.as_deref(), &mut self.node_stack);
        Some(&current_node.value)
    }
}

pub struct IntoIter<T> {
    value_iter: std::vec::IntoIter<T>,
}

impl<T> IntoIter<T> {
    fn new(root_node: Option<Box<Node<T>>>, value_count: usize) -> Self {
        let mut collected_values = Vec::with_capacity(value_count);
        collect_values_in_order(root_node, &mut collected_values);
        Self {
            value_iter: collected_values.into_iter(),
        }
    }
}

impl<T> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.value_iter.next()
    }
}

fn push_left_nodes<'a, T>(root_node: Option<&'a Node<T>>, node_stack: &mut Vec<&'a Node<T>>) {
    let mut current_node = root_node;
    while let Some(next_node) = current_node {
        node_stack.push(next_node);
        current_node = next_node.left.as_deref();
    }
}

fn collect_values_in_order<T>(root_node: Option<Box<Node<T>>>, collected_values: &mut Vec<T>) {
    if let Some(mut current_node) = root_node {
        collect_values_in_order(current_node.left.take(), collected_values);
        collected_values.push(current_node.value);
        collect_values_in_order(current_node.right.take(), collected_values);
    }
}

impl<T> DynamicArray<T> {
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.root.as_deref())
    }
}

impl<'a, T> IntoIterator for &'a DynamicArray<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T> IntoIterator for DynamicArray<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        let DynamicArray { root, .. } = self;
        let value_count = size_of(&root);
        IntoIter::new(root, value_count)
    }
}

impl<T: Clone> DynamicArray<T> {
    pub fn new() -> Self {
        Self {
            root: None,
            rng: rand::rng(),
        }
    }

    pub fn len(&self) -> usize {
        size_of(&self.root)
    }

    pub fn is_empty(&self) -> bool {
        self.root.is_none()
    }

    pub fn push_front(&mut self, value: T) {
        self.insert(0, value);
    }

    pub fn push_back(&mut self, value: T) {
        self.insert(self.len(), value);
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.erase(0)
    }

    pub fn pop_back(&mut self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.erase(self.len() - 1)
        }
    }

    pub fn insert(&mut self, index: usize, value: T) {
        assert!(index <= self.len());
        let priority = self.rng.next_u64();
        let node = Node::new(value, priority);
        let (left, right) = split(self.root.take(), index);
        let merged = merge(left, Some(node));
        self.root = merge(merged, right);
    }

    pub fn erase(&mut self, index: usize) -> Option<T> {
        if index >= self.len() {
            return None;
        }
        let (left, rest) = split(self.root.take(), index);
        let (mid, right) = split(rest, 1);
        let value = mid.map(|n| n.value);
        self.root = merge(left, right);
        value
    }

    pub fn set(&mut self, index: usize, value: T) -> bool {
        if index >= self.len() {
            return false;
        }
        let removed = self.erase(index);
        if removed.is_some() {
            self.insert(index, value);
            true
        } else {
            false
        }
    }

    pub fn get(&self, index: usize) -> Option<T> {
        if index >= self.len() {
            None
        } else {
            kth(&self.root, index)
        }
    }

    pub fn front(&self) -> Option<T> {
        self.get(0)
    }

    pub fn back(&self) -> Option<T> {
        if self.is_empty() {
            None
        } else {
            self.get(self.len() - 1)
        }
    }

    pub fn to_vec(&self) -> Vec<T> {
        fn dfs<T: Clone>(node: &Option<Box<Node<T>>>, acc: &mut Vec<T>) {
            if let Some(n) = node {
                dfs(&n.left, acc);
                acc.push(n.value.clone());
                dfs(&n.right, acc);
            }
        }

        let mut result = Vec::with_capacity(self.len());
        dfs(&self.root, &mut result);
        result
    }
}

#[cfg(test)]
mod tests {
    use super::{kth, merge, size_of, split, update_size, DynamicArray, Node};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    type TestNodeLink = Option<Box<Node<i32>>>;

    fn build_test_treap(entries: &[(i32, u64)]) -> TestNodeLink {
        let mut root_node = None;
        for &(value, priority) in entries {
            root_node = merge(root_node, Some(Node::new(value, priority)));
        }
        root_node
    }

    fn collect_test_node_values(root_node: &TestNodeLink, values: &mut Vec<i32>) {
        if let Some(node) = root_node {
            collect_test_node_values(&node.left, values);
            values.push(node.value);
            collect_test_node_values(&node.right, values);
        }
    }

    fn test_node_values(root_node: &TestNodeLink) -> Vec<i32> {
        let mut values = Vec::new();
        collect_test_node_values(root_node, &mut values);
        values
    }

    fn root_value(root_node: &TestNodeLink) -> Option<i32> {
        root_node.as_ref().map(|node| node.value)
    }

    fn treap_invariants(root_node: &TestNodeLink) -> (bool, usize) {
        if let Some(node) = root_node {
            let (left_is_valid, left_size) = treap_invariants(&node.left);
            let (right_is_valid, right_size) = treap_invariants(&node.right);
            let actual_size = 1 + left_size + right_size;
            let left_priority_is_valid = node
                .left
                .as_ref()
                .is_none_or(|left_node| node.priority >= left_node.priority);
            let right_priority_is_valid = node
                .right
                .as_ref()
                .is_none_or(|right_node| node.priority >= right_node.priority);
            let is_valid = left_is_valid
                && right_is_valid
                && left_priority_is_valid
                && right_priority_is_valid
                && node.size == actual_size;
            (is_valid, actual_size)
        } else {
            (true, 0)
        }
    }

    #[derive(Clone, Copy, Debug)]
    enum TestOperation {
        PushFront(i32),
        PushBack(i32),
        Insert(usize, i32),
        Erase(usize),
        Set(usize, i32),
        PopFront,
        PopBack,
    }

    #[derive(Debug, PartialEq)]
    enum TestOperationResult {
        Unit,
        Value(Option<i32>),
        Updated(bool),
    }

    fn apply_array_operation(
        array: &mut DynamicArray<i32>,
        operation: TestOperation,
    ) -> TestOperationResult {
        match operation {
            TestOperation::PushFront(value) => {
                array.push_front(value);
                TestOperationResult::Unit
            }
            TestOperation::PushBack(value) => {
                array.push_back(value);
                TestOperationResult::Unit
            }
            TestOperation::Insert(index, value) => {
                array.insert(index, value);
                TestOperationResult::Unit
            }
            TestOperation::Erase(index) => TestOperationResult::Value(array.erase(index)),
            TestOperation::Set(index, value) => {
                TestOperationResult::Updated(array.set(index, value))
            }
            TestOperation::PopFront => TestOperationResult::Value(array.pop_front()),
            TestOperation::PopBack => TestOperationResult::Value(array.pop_back()),
        }
    }

    fn apply_model_operation(
        model: &mut Vec<i32>,
        operation: TestOperation,
    ) -> TestOperationResult {
        match operation {
            TestOperation::PushFront(value) => {
                model.insert(0, value);
                TestOperationResult::Unit
            }
            TestOperation::PushBack(value) => {
                model.push(value);
                TestOperationResult::Unit
            }
            TestOperation::Insert(index, value) => {
                model.insert(index, value);
                TestOperationResult::Unit
            }
            TestOperation::Erase(index) => {
                let removed_value = if index < model.len() {
                    Some(model.remove(index))
                } else {
                    None
                };
                TestOperationResult::Value(removed_value)
            }
            TestOperation::Set(index, value) => {
                let was_updated = if index < model.len() {
                    model[index] = value;
                    true
                } else {
                    false
                };
                TestOperationResult::Updated(was_updated)
            }
            TestOperation::PopFront => {
                let popped_value = if model.is_empty() {
                    None
                } else {
                    Some(model.remove(0))
                };
                TestOperationResult::Value(popped_value)
            }
            TestOperation::PopBack => TestOperationResult::Value(model.pop()),
        }
    }

    fn mixed_test_operations() -> Vec<TestOperation> {
        vec![
            TestOperation::PushBack(10),
            TestOperation::PushFront(5),
            TestOperation::Insert(1, 7),
            TestOperation::PushBack(10),
            TestOperation::Set(2, 11),
            TestOperation::Erase(1),
            TestOperation::Insert(0, 3),
            TestOperation::Insert(4, 13),
            TestOperation::PopFront,
            TestOperation::PopBack,
            TestOperation::PushFront(8),
            TestOperation::PushBack(14),
            TestOperation::Insert(2, 9),
            TestOperation::Set(1, 6),
            TestOperation::Erase(3),
            TestOperation::PushBack(14),
            TestOperation::Insert(3, 11),
            TestOperation::PopFront,
            TestOperation::PushFront(4),
            TestOperation::Set(0, 2),
            TestOperation::Erase(2),
            TestOperation::PushBack(16),
            TestOperation::Insert(6, 15),
            TestOperation::Erase(7),
            TestOperation::Set(5, 13),
            TestOperation::PopBack,
            TestOperation::PopFront,
            TestOperation::Erase(0),
            TestOperation::Set(3, 12),
            TestOperation::Insert(2, 10),
            TestOperation::PopBack,
        ]
    }

    #[test]
    fn node_new_initializes_all_fields() {
        // Arrange
        let value = 42;
        let priority = 99;
        let expected_size = 1;
        let expected_left_is_none = true;
        let expected_right_is_none = true;

        // Act
        let node = Node::new(value, priority);
        let actual_value = node.value;
        let actual_priority = node.priority;
        let actual_size = node.size;
        let actual_left_is_none = node.left.is_none();
        let actual_right_is_none = node.right.is_none();

        // Assert
        assert_eq!(actual_value, value);
        assert_eq!(actual_priority, priority);
        assert_eq!(actual_size, expected_size);
        assert_eq!(actual_left_is_none, expected_left_is_none);
        assert_eq!(actual_right_is_none, expected_right_is_none);
    }

    #[test]
    fn size_of_returns_zero_for_none() {
        // Arrange
        let root_node: TestNodeLink = None;
        let expected_size = 0;

        // Act
        let actual_size = size_of(&root_node);

        // Assert
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn size_of_returns_stored_node_size() {
        // Arrange
        let value = 10;
        let priority = 50;
        let stored_size = 7;
        let mut node = Node::new(value, priority);
        node.size = stored_size;
        let root_node = Some(node);

        // Act
        let actual_size = size_of(&root_node);

        // Assert
        assert_eq!(actual_size, stored_size);
    }

    #[test]
    fn update_size_recalculates_size_from_both_subtrees() {
        // Arrange
        let expected_size = 5;
        let mut left_node = Node::new(10, 80);
        left_node.left = Some(Node::new(5, 70));
        left_node.size = 2;
        let mut right_node = Node::new(30, 60);
        right_node.right = Some(Node::new(40, 50));
        right_node.size = 2;
        let mut root_node = Node::new(20, 100);
        root_node.left = Some(left_node);
        root_node.right = Some(right_node);

        // Act
        update_size(&mut root_node);
        let actual_size = root_node.size;

        // Assert
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_returns_none_when_both_trees_are_none() {
        // Arrange
        let left_tree: TestNodeLink = None;
        let right_tree: TestNodeLink = None;
        let expected_is_none = true;

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_is_none = merged_tree.is_none();

        // Assert
        assert_eq!(actual_is_none, expected_is_none);
    }

    #[test]
    fn merge_returns_right_tree_when_left_tree_is_none() {
        // Arrange
        let left_tree: TestNodeLink = None;
        let entries = [(10, 100), (20, 80), (30, 60)];
        let right_tree = build_test_treap(&entries);
        let expected_values = vec![10, 20, 30];
        let expected_size = expected_values.len();

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_returns_left_tree_when_right_tree_is_none() {
        // Arrange
        let entries = [(1, 100), (2, 80), (3, 60)];
        let left_tree = build_test_treap(&entries);
        let right_tree: TestNodeLink = None;
        let expected_values = vec![1, 2, 3];
        let expected_size = expected_values.len();

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_uses_left_root_when_left_priority_is_higher() {
        // Arrange
        let left_value = 10;
        let right_value = 20;
        let left_priority = 100;
        let right_priority = 50;
        let left_tree = Some(Node::new(left_value, left_priority));
        let right_tree = Some(Node::new(right_value, right_priority));
        let expected_root_value = Some(left_value);
        let expected_values = vec![left_value, right_value];
        let expected_size = expected_values.len();

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_root_value = root_value(&merged_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_root_value, expected_root_value);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_uses_right_root_when_right_priority_is_higher() {
        // Arrange
        let left_value = 10;
        let right_value = 20;
        let left_priority = 50;
        let right_priority = 100;
        let left_tree = Some(Node::new(left_value, left_priority));
        let right_tree = Some(Node::new(right_value, right_priority));
        let expected_root_value = Some(right_value);
        let expected_values = vec![left_value, right_value];
        let expected_size = expected_values.len();

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_root_value = root_value(&merged_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_root_value, expected_root_value);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_uses_right_root_when_priorities_are_equal() {
        // Arrange
        let left_value = 10;
        let right_value = 20;
        let equal_priority = 100;
        let left_tree = Some(Node::new(left_value, equal_priority));
        let right_tree = Some(Node::new(right_value, equal_priority));
        let expected_root_value = Some(right_value);
        let expected_values = vec![left_value, right_value];
        let expected_size = expected_values.len();

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_root_value = root_value(&merged_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_root_value, expected_root_value);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_after_split_at_zero_restores_original_tree_sequence() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let split_index = 0;
        let expected_values = vec![10, 20, 30, 40];
        let expected_size = expected_values.len();

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let merged_tree = merge(left_tree, right_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_after_split_in_middle_restores_original_tree_sequence() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let split_index = 2;
        let expected_values = vec![10, 20, 30, 40];
        let expected_size = expected_values.len();

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let merged_tree = merge(left_tree, right_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn merge_after_split_at_len_restores_original_tree_sequence() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let split_index = entries.len();
        let expected_values = vec![10, 20, 30, 40];
        let expected_size = expected_values.len();

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let merged_tree = merge(left_tree, right_tree);
        let actual_values = test_node_values(&merged_tree);
        let actual_size = size_of(&merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_size, expected_size);
    }

    #[test]
    fn split_none_at_zero_returns_two_none_trees() {
        // Arrange
        let root_node: TestNodeLink = None;
        let split_index = 0;
        let expected_left_is_none = true;
        let expected_right_is_none = true;

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_is_none = left_tree.is_none();
        let actual_right_is_none = right_tree.is_none();

        // Assert
        assert_eq!(actual_left_is_none, expected_left_is_none);
        assert_eq!(actual_right_is_none, expected_right_is_none);
    }

    #[test]
    fn split_single_node_at_zero_places_node_on_right() {
        // Arrange
        let node_value = 10;
        let node_priority = 100;
        let root_node = Some(Node::new(node_value, node_priority));
        let split_index = 0;
        let expected_left_values: Vec<i32> = vec![];
        let expected_right_values = vec![node_value];

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_values = test_node_values(&left_tree);
        let actual_right_values = test_node_values(&right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_single_node_at_one_places_node_on_left() {
        // Arrange
        let node_value = 10;
        let node_priority = 100;
        let root_node = Some(Node::new(node_value, node_priority));
        let split_index = 1;
        let expected_left_values = vec![node_value];
        let expected_right_values: Vec<i32> = vec![];

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_values = test_node_values(&left_tree);
        let actual_right_values = test_node_values(&right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_inside_fixed_left_subtree_preserves_values_and_sizes() {
        // Arrange
        let entries = [(10, 80), (20, 70), (30, 100), (40, 60)];
        let root_node = build_test_treap(&entries);
        let split_index = 1;
        let expected_left_values = vec![10];
        let expected_right_values = vec![20, 30, 40];
        let expected_left_size = expected_left_values.len();
        let expected_right_size = expected_right_values.len();

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_values = test_node_values(&left_tree);
        let actual_right_values = test_node_values(&right_tree);
        let actual_left_size = size_of(&left_tree);
        let actual_right_size = size_of(&right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
        assert_eq!(actual_left_size, expected_left_size);
        assert_eq!(actual_right_size, expected_right_size);
    }

    #[test]
    fn split_inside_fixed_right_subtree_preserves_values_and_sizes() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let split_index = 3;
        let expected_left_values = vec![10, 20, 30];
        let expected_right_values = vec![40];
        let expected_left_size = expected_left_values.len();
        let expected_right_size = expected_right_values.len();

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_values = test_node_values(&left_tree);
        let actual_right_values = test_node_values(&right_tree);
        let actual_left_size = size_of(&left_tree);
        let actual_right_size = size_of(&right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
        assert_eq!(actual_left_size, expected_left_size);
        assert_eq!(actual_right_size, expected_right_size);
    }

    #[test]
    fn split_multiple_nodes_at_zero_places_all_nodes_on_right() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let split_index = 0;
        let expected_left_values: Vec<i32> = vec![];
        let expected_right_values = vec![10, 20, 30, 40];

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_values = test_node_values(&left_tree);
        let actual_right_values = test_node_values(&right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_multiple_nodes_at_len_places_all_nodes_on_left() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let split_index = entries.len();
        let expected_left_values = vec![10, 20, 30, 40];
        let expected_right_values: Vec<i32> = vec![];

        // Act
        let (left_tree, right_tree) = split(root_node, split_index);
        let actual_left_values = test_node_values(&left_tree);
        let actual_right_values = test_node_values(&right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn kth_returns_none_for_empty_tree_at_zero() {
        // Arrange
        let root_node: TestNodeLink = None;
        let index = 0;
        let expected_value = None;

        // Act
        let actual_value = kth(&root_node, index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_returns_value_from_fixed_left_subtree() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let index = 0;
        let expected_value = Some(10);

        // Act
        let actual_value = kth(&root_node, index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_returns_value_from_fixed_root() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let index = 1;
        let expected_value = Some(20);

        // Act
        let actual_value = kth(&root_node, index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_returns_value_from_fixed_right_subtree() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let index = 3;
        let expected_value = Some(40);

        // Act
        let actual_value = kth(&root_node, index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn kth_returns_none_for_index_equal_to_tree_size() {
        // Arrange
        let entries = [(10, 60), (20, 100), (30, 80), (40, 70)];
        let root_node = build_test_treap(&entries);
        let index = entries.len();
        let expected_value = None;

        // Act
        let actual_value = kth(&root_node, index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn default_array_supports_same_state_transitions_as_new_array() {
        // Arrange
        let mut default_array: DynamicArray<i32> = DynamicArray::default();
        let mut new_array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let get_index = 1;
        let erase_index = 0;

        // Act
        default_array.push_back(first_value);
        default_array.push_back(second_value);
        new_array.push_back(first_value);
        new_array.push_back(second_value);
        let actual_default_value = default_array.get(get_index);
        let actual_new_value = new_array.get(get_index);
        let actual_default_removed = default_array.erase(erase_index);
        let actual_new_removed = new_array.erase(erase_index);
        let actual_default_values = default_array.to_vec();
        let actual_new_values = new_array.to_vec();

        // Assert
        assert_eq!(actual_default_value, actual_new_value);
        assert_eq!(actual_default_removed, actual_new_removed);
        assert_eq!(actual_default_values, actual_new_values);
    }

    #[test]
    fn new_array_is_empty_and_len_is_zero() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_is_empty = true;
        let expected_len = 0;

        // Act
        let actual_is_empty = array.is_empty();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn default_array_matches_new_initial_state() {
        // Arrange
        let default_array: DynamicArray<i32> = DynamicArray::default();
        let new_array: DynamicArray<i32> = DynamicArray::new();
        let expected_len = 0;
        let expected_is_empty = true;
        let expected_front = None;
        let expected_back = None;

        // Act
        let actual_default_len = default_array.len();
        let actual_default_is_empty = default_array.is_empty();
        let actual_default_front = default_array.front();
        let actual_default_back = default_array.back();
        let actual_new_len = new_array.len();
        let actual_new_is_empty = new_array.is_empty();
        let actual_new_front = new_array.front();
        let actual_new_back = new_array.back();

        // Assert
        assert_eq!(actual_default_len, expected_len);
        assert_eq!(actual_default_is_empty, expected_is_empty);
        assert_eq!(actual_default_front, expected_front);
        assert_eq!(actual_default_back, expected_back);
        assert_eq!(actual_new_len, expected_len);
        assert_eq!(actual_new_is_empty, expected_is_empty);
        assert_eq!(actual_new_front, expected_front);
        assert_eq!(actual_new_back, expected_back);
    }

    #[test]
    fn push_front_on_empty_array_sets_both_ends_and_non_empty_state() {
        // Arrange
        let mut array = DynamicArray::new();
        let pushed_value = 42;
        let expected_front = Some(pushed_value);
        let expected_back = Some(pushed_value);
        let expected_len = 1;
        let expected_is_empty = false;

        // Act
        array.push_front(pushed_value);
        let actual_front = array.front();
        let actual_back = array.back();
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();

        // Assert
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_back, expected_back);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn repeated_push_front_reverses_insertion_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_values = vec![third_value, second_value, first_value];
        let expected_len = expected_values.len();

        // Act
        array.push_front(first_value);
        array.push_front(second_value);
        array.push_front(third_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn push_front_places_value_at_front_and_preserves_existing_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let pushed_value = 0;
        let expected_values = vec![pushed_value, first_value, second_value];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        array.push_front(pushed_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn push_back_increases_len_from_zero_to_one() {
        // Arrange
        let mut array = DynamicArray::new();
        let pushed_value = 1;
        let expected_len = 1;

        // Act
        array.push_back(pushed_value);
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn push_back_updates_back_value() {
        // Arrange
        let mut array = DynamicArray::new();
        let pushed_value = 7;
        let expected_back = Some(pushed_value);

        // Act
        array.push_back(pushed_value);
        let actual_back = array.back();

        // Assert
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn repeated_pop_front_returns_values_in_order_and_empties_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_first_pop = Some(first_value);
        let expected_second_pop = Some(second_value);
        let expected_third_pop = Some(third_value);
        let expected_len = 0;
        let expected_is_empty = true;
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_first_pop = array.pop_front();
        let actual_second_pop = array.pop_front();
        let actual_third_pop = array.pop_front();
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();

        // Assert
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn pop_front_returns_none_for_empty_array() {
        // Arrange
        let mut array: DynamicArray<i32> = DynamicArray::new();
        let expected_popped = None;
        let expected_len = 0;

        // Act
        let actual_popped = array.pop_front();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_front_returns_value_for_single_element_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let only_value = 1;
        let expected_popped = Some(only_value);
        let expected_is_empty = true;
        array.push_front(only_value);

        // Act
        let actual_popped = array.pop_front();
        let actual_is_empty = array.is_empty();

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn pop_front_returns_first_value_and_preserves_remaining_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_popped = Some(first_value);
        let expected_values = vec![second_value, third_value];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_popped = array.pop_front();
        let actual_values = array.to_vec();

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pop_back_returns_none_for_empty_array() {
        // Arrange
        let mut array: DynamicArray<i32> = DynamicArray::new();
        let expected_popped = None;
        let expected_len = 0;

        // Act
        let actual_popped = array.pop_back();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_back_returns_value_for_single_element_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let expected_popped = Some(first_value);
        let expected_len = 0;
        array.push_back(first_value);

        // Act
        let actual_popped = array.pop_back();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_back_returns_last_value_for_two_element_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 2;
        let last_value = 3;
        let expected_popped = Some(last_value);
        let expected_len = 1;
        let expected_front = Some(first_value);
        array.push_back(first_value);
        array.push_back(last_value);

        // Act
        let actual_popped = array.pop_back();
        let actual_len = array.len();
        let actual_front = array.front();

        // Assert
        assert_eq!(actual_popped, expected_popped);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_front, expected_front);
    }

    #[test]
    fn pop_back_repeatedly_until_element_count_returns_array_to_empty_state() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_first_pop = Some(third_value);
        let expected_second_pop = Some(second_value);
        let expected_third_pop = Some(first_value);
        let expected_len = 0;
        let expected_is_empty = true;
        let expected_front = None;
        let expected_back = None;
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_first_pop = array.pop_back();
        let actual_second_pop = array.pop_back();
        let actual_third_pop = array.pop_back();
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();
        let actual_front = array.front();
        let actual_back = array.back();

        // Assert
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn out_of_bounds_insert_panic_preserves_non_empty_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let inserted_value = 30;
        let expected_values = vec![first_value, second_value];
        let expected_len = expected_values.len();
        let invalid_index = expected_len + 1;
        let expected_panicked = true;
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            array.insert(invalid_index, inserted_value);
        }));
        let actual_panicked = panic_result.is_err();
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn repeated_duplicate_insertions_and_removals_keep_all_views_in_sync() {
        // Arrange
        let mut array = DynamicArray::new();
        let duplicate_value = 7;
        let operations = vec![
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::PushBack(duplicate_value),
            TestOperation::Insert(0, duplicate_value),
            TestOperation::Insert(5, duplicate_value),
            TestOperation::Insert(10, duplicate_value),
            TestOperation::Erase(0),
            TestOperation::Erase(5),
            TestOperation::Erase(8),
        ];
        let mut expected_model = Vec::new();
        let expected_results: Vec<TestOperationResult> = operations
            .iter()
            .map(|operation| apply_model_operation(&mut expected_model, *operation))
            .collect();
        let expected_values = expected_model;

        // Act
        let actual_results: Vec<TestOperationResult> = operations
            .iter()
            .map(|operation| apply_array_operation(&mut array, *operation))
            .collect();
        let actual_get_values: Vec<i32> = (0..array.len())
            .filter_map(|index| array.get(index))
            .collect();
        let actual_iter_values: Vec<i32> = array.iter().copied().collect();
        let actual_to_vec_values = array.to_vec();

        // Assert
        assert_eq!(actual_results, expected_results);
        assert_eq!(actual_get_values, expected_values);
        assert_eq!(actual_iter_values, expected_values);
        assert_eq!(actual_to_vec_values, expected_values);
    }

    #[test]
    fn insert_at_zero_places_value_at_front() {
        // Arrange
        let mut array = DynamicArray::new();
        let existing_value = 2;
        let inserted_value = 1;
        let insert_index = 0;
        let expected_front = Some(inserted_value);

        // Act
        array.push_back(existing_value);
        array.insert(insert_index, inserted_value);
        let actual_front = array.front();

        // Assert
        assert_eq!(actual_front, expected_front);
    }

    #[test]
    fn insert_at_len_places_value_at_back() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let inserted_value = 2;
        array.push_back(first_value);
        let insert_index = array.len();
        let expected_back = Some(inserted_value);

        // Act
        array.insert(insert_index, inserted_value);
        let actual_back = array.back();

        // Assert
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn insert_in_middle_preserves_order_and_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let middle_value = 2;
        let last_value = 3;
        let middle_index = 1;
        let expected_values = vec![1, 2, 3];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(last_value);

        // Act
        array.insert(middle_index, middle_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn insert_panics_when_index_is_len_plus_one() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let inserted_value = 20;
        array.push_back(first_value);
        let invalid_index = array.len() + 1;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            array.insert(invalid_index, inserted_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn insert_at_zero_for_empty_array_sets_front_and_back_to_inserted_value() {
        // Arrange
        let mut array = DynamicArray::new();
        let insert_index = 0;
        let inserted_value = 42;
        let expected_front = Some(inserted_value);
        let expected_back = Some(inserted_value);
        let expected_len = 1;

        // Act
        array.insert(insert_index, inserted_value);
        let actual_front = array.front();
        let actual_back = array.back();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_back, expected_back);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn insert_in_middle_with_duplicate_values_preserves_multiplicity_and_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let duplicate_value = 1;
        let tail_value = 3;
        let insert_index = 2;
        let expected_values = vec![1, 1, 1, 3];
        let expected_len = expected_values.len();
        array.push_back(duplicate_value);
        array.push_back(duplicate_value);
        array.push_back(tail_value);

        // Act
        array.insert(insert_index, duplicate_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn insert_panics_when_index_is_usize_max() {
        // Arrange
        let mut array = DynamicArray::new();
        let inserted_value = 10;
        let invalid_index = usize::MAX;

        // Act
        let actual_result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            array.insert(invalid_index, inserted_value)
        }));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn iter_after_mixed_insert_and_erase_operations_matches_to_vec_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let third_value = 3;
        let inserted_middle_value = 2;
        let duplicated_middle_value = 2;
        let first_insert_index = 1;
        let erase_index = 0;
        let second_insert_index = 1;
        let expected_values = vec![2, 2, 3];
        array.push_back(first_value);
        array.push_back(third_value);

        // Act
        array.insert(first_insert_index, inserted_middle_value);
        let _removed_value = array.erase(erase_index);
        array.insert(second_insert_index, duplicated_middle_value);
        let actual_iter_values: Vec<i32> = array.iter().copied().collect();
        let actual_to_vec_values = array.to_vec();

        // Assert
        assert_eq!(actual_iter_values, expected_values);
        assert_eq!(actual_to_vec_values, expected_values);
    }

    #[test]
    fn to_vec_after_insert_erase_and_set_preserves_order_with_duplicates() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let duplicate_value = 2;
        let tail_value = 4;
        let insert_index = 2;
        let erase_index = 0;
        let set_index = 3;
        let expected_removed = Some(first_value);
        let expected_set_result = true;
        let expected_values = vec![2, 2, 2, 2];
        array.push_back(first_value);
        array.push_back(duplicate_value);
        array.push_back(duplicate_value);
        array.push_back(tail_value);

        // Act
        array.insert(insert_index, duplicate_value);
        let actual_removed = array.erase(erase_index);
        let actual_set_result = array.set(set_index, duplicate_value);
        let actual_values = array.to_vec();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_set_result, expected_set_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn array_can_be_reused_after_all_values_are_erased() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let reused_value = 99;
        let erase_index = 0;
        let expected_first_removed = Some(first_value);
        let expected_second_removed = Some(second_value);
        let expected_values = vec![reused_value];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        let actual_first_removed = array.erase(erase_index);
        let actual_second_removed = array.erase(erase_index);
        array.push_back(reused_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_first_removed, expected_first_removed);
        assert_eq!(actual_second_removed, expected_second_removed);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_removes_first_value_for_index_zero() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        array.push_back(first_value);
        array.push_back(second_value);
        let erase_index = 0;
        let expected_removed = Some(first_value);

        // Act
        let actual_removed = array.erase(erase_index);

        // Assert
        assert_eq!(actual_removed, expected_removed);
    }

    #[test]
    fn erase_removes_last_value_for_index_len_minus_one() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        array.push_back(first_value);
        array.push_back(second_value);
        let erase_index = array.len() - 1;
        let expected_removed = Some(second_value);

        // Act
        let actual_removed = array.erase(erase_index);

        // Assert
        assert_eq!(actual_removed, expected_removed);
    }

    #[test]
    fn erase_returns_none_when_index_is_equal_to_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        array.push_back(first_value);
        array.push_back(second_value);
        let expected_values = vec![1, 2];
        let expected_len = expected_values.len();
        let index_equal_to_len = expected_len;
        let expected_removed = None;

        // Act
        let actual_removed = array.erase(index_equal_to_len);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_returns_none_when_index_is_len_plus_one() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        array.push_back(first_value);
        array.push_back(second_value);
        let expected_values = vec![1, 2];
        let expected_len = expected_values.len();
        let index_len_plus_one = expected_len + 1;
        let expected_removed = None;

        // Act
        let actual_removed = array.erase(index_len_plus_one);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_middle_returns_value_and_updates_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let middle_value = 2;
        let last_value = 3;
        let middle_index = 1;
        let expected_removed = Some(middle_value);
        let expected_values = vec![1, 3];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(middle_value);
        array.push_back(last_value);

        // Act
        let removed_value = array.erase(middle_index);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(removed_value, expected_removed);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn erase_for_empty_array_returns_none_and_preserves_empty_state() {
        // Arrange
        let mut array: DynamicArray<i32> = DynamicArray::new();
        let erase_index = 0;
        let expected_removed = None;
        let expected_len = 0;
        let expected_is_empty = true;

        // Act
        let actual_removed = array.erase(erase_index);
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn erase_for_single_element_array_returns_value_and_makes_array_empty() {
        // Arrange
        let mut array = DynamicArray::new();
        let only_value = 9;
        let erase_index = 0;
        let expected_removed = Some(only_value);
        let expected_len = 0;
        let expected_is_empty = true;
        let expected_front = None;
        let expected_back = None;
        array.push_back(only_value);

        // Act
        let actual_removed = array.erase(erase_index);
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();
        let actual_front = array.front();
        let actual_back = array.back();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn erase_for_usize_max_index_returns_none_and_preserves_values() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let erase_index = usize::MAX;
        let expected_removed = None;
        let expected_values = vec![1, 2, 3];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_removed = array.erase(erase_index);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_removed, expected_removed);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn set_returns_false_when_array_is_empty() {
        // Arrange
        let mut array = DynamicArray::new();
        let set_index = 0;
        let replacement_value = 5;
        let expected_result = false;
        let expected_values: Vec<i32> = vec![];
        let expected_len = expected_values.len();

        // Act
        let actual_result = array.set(set_index, replacement_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn set_returns_false_when_index_is_equal_to_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        array.push_back(first_value);
        let set_index = array.len();
        let replacement_value = 9;
        let expected_result = false;
        let expected_values = vec![first_value];

        // Act
        let actual_result = array.set(set_index, replacement_value);
        let actual_values = array.to_vec();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn set_updates_head_without_changing_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let middle_value = 2;
        let last_value = 3;
        let head_index = 0;
        let replacement_head_value = 10;
        let expected_result = true;
        let expected_values = vec![10, 2, 3];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(middle_value);
        array.push_back(last_value);

        // Act
        let actual_result = array.set(head_index, replacement_head_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn set_updates_tail_without_changing_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let middle_value = 2;
        let last_value = 3;
        let tail_index = 2;
        let replacement_tail_value = 30;
        let expected_result = true;
        let expected_values = vec![1, 2, 30];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(middle_value);
        array.push_back(last_value);

        // Act
        let actual_result = array.set(tail_index, replacement_tail_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn set_for_middle_index_updates_only_target_position_without_changing_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let middle_value = 2;
        let last_value = 3;
        let set_index = 1;
        let replacement_value = 20;
        let expected_result = true;
        let expected_values = vec![1, 20, 3];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(middle_value);
        array.push_back(last_value);

        // Act
        let actual_result = array.set(set_index, replacement_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn set_returns_false_when_index_is_len_plus_one_for_non_empty_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        array.push_back(first_value);
        array.push_back(second_value);
        let set_index = array.len() + 1;
        let replacement_value = 9;
        let expected_result = false;
        let expected_values = vec![1, 2];
        let expected_len = expected_values.len();

        // Act
        let actual_result = array.set(set_index, replacement_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn set_returns_false_for_usize_max_index_without_panicking() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let set_index = usize::MAX;
        let replacement_value = 9;
        let expected_result = false;
        let expected_values = vec![1, 2];
        let expected_len = expected_values.len();
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        let actual_result = array.set(set_index, replacement_value);
        let actual_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn modifying_string_returned_by_get_does_not_change_stored_value() {
        // Arrange
        let mut array = DynamicArray::new();
        let original_value = String::from("alpha");
        let suffix = "-changed";
        let index = 0;
        let expected_returned_value = Some(String::from("alpha-changed"));
        let expected_stored_value = Some(original_value.clone());
        array.push_back(original_value);

        // Act
        let mut actual_returned_value = array.get(index);
        if let Some(value) = actual_returned_value.as_mut() {
            value.push_str(suffix);
        }
        let actual_stored_value = array.get(index);

        // Assert
        assert_eq!(actual_returned_value, expected_returned_value);
        assert_eq!(actual_stored_value, expected_stored_value);
    }

    #[test]
    fn get_returns_none_for_empty_array_at_index_zero() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let index = 0;
        let expected_value = None;

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn get_returns_value_for_index_len_minus_one() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        array.push_back(first_value);
        array.push_back(second_value);
        let index = array.len() - 1;
        let expected_value = Some(second_value);

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn get_returns_none_for_index_equal_to_len() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        array.push_back(first_value);
        let index = array.len();
        let expected_value = None;

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn get_returns_none_for_index_len_plus_one() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        array.push_back(first_value);
        let index = array.len() + 1;
        let expected_value = None;

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn get_returns_first_value_for_index_zero_in_three_element_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 11;
        let second_value = 22;
        let third_value = 33;
        let index = 0;
        let expected_value = Some(first_value);
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn get_returns_middle_value_for_index_one_in_three_element_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 11;
        let second_value = 22;
        let third_value = 33;
        let index = 1;
        let expected_value = Some(second_value);
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn get_returns_none_for_usize_max_index() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let index = usize::MAX;
        let expected_value = None;
        array.push_back(first_value);

        // Act
        let actual_value = array.get(index);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn front_returns_none_for_empty_array() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_front = None;

        // Act
        let actual_front = array.front();

        // Assert
        assert_eq!(actual_front, expected_front);
    }

    #[test]
    fn front_returns_first_value_for_non_empty_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 7;
        let second_value = 9;
        let expected_front = Some(first_value);
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        let actual_front = array.front();

        // Assert
        assert_eq!(actual_front, expected_front);
    }

    #[test]
    fn back_returns_none_for_empty_array() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_back = None;

        // Act
        let actual_back = array.back();

        // Assert
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn back_returns_last_value_for_non_empty_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 7;
        let last_value = 9;
        let expected_back = Some(last_value);
        array.push_back(first_value);
        array.push_back(last_value);

        // Act
        let actual_back = array.back();

        // Assert
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn modifying_to_vec_result_does_not_change_source_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let replacement_value = 99;
        let expected_snapshot = vec![replacement_value, second_value];
        let expected_source_values = vec![first_value, second_value];
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        let mut actual_snapshot = array.to_vec();
        if let Some(value) = actual_snapshot.first_mut() {
            *value = replacement_value;
        }
        let actual_source_values = array.to_vec();

        // Assert
        assert_eq!(actual_snapshot, expected_snapshot);
        assert_eq!(actual_source_values, expected_source_values);
    }

    #[test]
    fn to_vec_returns_empty_vec_for_empty_array() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_values: Vec<i32> = vec![];

        // Act
        let actual_values = array.to_vec();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn to_vec_returns_values_in_order_for_non_empty_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_values = vec![first_value, second_value, third_value];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_values = array.to_vec();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn mixed_operations_keep_to_vec_len_and_is_empty_consistent() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let third_value = 30;
        let second_value = 20;
        let second_index = 1;
        let removed_index = 1;
        let updated_first_index = 0;
        let updated_first_value = 5;
        let expected_removed = Some(second_value);
        let expected_set_result = true;
        let expected_popped = Some(third_value);
        let expected_values = vec![updated_first_value];
        let expected_len = expected_values.len();
        let expected_is_empty = false;

        // Act
        array.push_back(first_value);
        array.push_back(third_value);
        array.insert(second_index, second_value);
        let removed_value = array.erase(removed_index);
        let set_result = array.set(updated_first_index, updated_first_value);
        let popped_value = array.pop_back();
        let actual_values = array.to_vec();
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();

        // Assert
        assert_eq!(removed_value, expected_removed);
        assert_eq!(set_result, expected_set_result);
        assert_eq!(popped_value, expected_popped);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn into_iter_for_empty_reference_returns_none_and_preserves_source() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_value = None;
        let expected_len = 0;
        let expected_is_empty = true;

        // Act
        let mut value_iter = (&array).into_iter();
        let actual_value = value_iter.next().copied();
        drop(value_iter);
        let actual_len = array.len();
        let actual_is_empty = array.is_empty();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn into_iter_for_non_empty_reference_preserves_source_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_iter_values = vec![first_value, second_value, third_value];
        let expected_source_values = expected_iter_values.clone();
        let expected_len = expected_source_values.len();
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_iter_values: Vec<i32> = (&array).into_iter().copied().collect();
        let actual_source_values = array.to_vec();
        let actual_len = array.len();

        // Assert
        assert_eq!(actual_iter_values, expected_iter_values);
        assert_eq!(actual_source_values, expected_source_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn owned_into_iter_for_single_value_is_exhausted_permanently() {
        // Arrange
        let mut array = DynamicArray::new();
        let only_value = 10;
        let expected_first_value = Some(only_value);
        let expected_first_none = None;
        let expected_second_none = None;
        array.push_back(only_value);

        // Act
        let mut value_iter = array.into_iter();
        let actual_first_value = value_iter.next();
        let actual_first_none = value_iter.next();
        let actual_second_none = value_iter.next();

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_first_none, expected_first_none);
        assert_eq!(actual_second_none, expected_second_none);
    }

    #[test]
    fn partially_consumed_owned_into_iter_returns_remaining_values_in_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_first_value = Some(first_value);
        let expected_remaining_values = vec![second_value, third_value];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let mut value_iter = array.into_iter();
        let actual_first_value = value_iter.next();
        let actual_remaining_values: Vec<i32> = value_iter.collect();

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    fn iter_returns_none_for_empty_array() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_value = None;

        // Act
        let mut value_iter = array.iter();
        let actual_value = value_iter.next().copied();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn iter_returns_value_for_single_element_array() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let expected_values = vec![first_value];
        array.push_back(first_value);

        // Act
        let actual_values: Vec<i32> = array.iter().copied().collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_returns_values_in_order_for_multiple_elements() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_values = vec![first_value, second_value, third_value];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_values: Vec<i32> = array.iter().copied().collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_for_reference_returns_values_in_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_values = vec![first_value, second_value, third_value];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_values: Vec<i32> = (&array).into_iter().copied().collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_for_owned_array_returns_empty_vec_for_empty_array() {
        // Arrange
        let array: DynamicArray<i32> = DynamicArray::new();
        let expected_values: Vec<i32> = vec![];

        // Act
        let actual_values: Vec<i32> = array.into_iter().collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_for_owned_array_returns_values_in_order_for_multiple_elements() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_values = vec![first_value, second_value, third_value];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_values: Vec<i32> = array.into_iter().collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_next_returns_none_even_when_called_multiple_times_after_exhaustion() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = 1;
        let second_value = 2;
        let expected_first_none = None;
        let expected_second_none = None;
        array.push_back(first_value);
        array.push_back(second_value);

        // Act
        let mut value_iter = array.iter();
        let _first_item = value_iter.next().copied();
        let _second_item = value_iter.next().copied();
        let actual_first_none = value_iter.next().copied();
        let actual_second_none = value_iter.next().copied();

        // Assert
        assert_eq!(actual_first_none, expected_first_none);
        assert_eq!(actual_second_none, expected_second_none);
    }

    #[test]
    fn into_iter_for_owned_string_array_returns_strings_in_order() {
        // Arrange
        let mut array = DynamicArray::new();
        let first_value = String::from("alpha");
        let second_value = String::from("beta");
        let third_value = String::from("gamma");
        let expected_values = vec![
            first_value.clone(),
            second_value.clone(),
            third_value.clone(),
        ];
        array.push_back(first_value);
        array.push_back(second_value);
        array.push_back(third_value);

        // Act
        let actual_values: Vec<String> = array.into_iter().collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn long_mixed_operation_sequence_matches_vec_after_every_operation() {
        // Arrange
        let mut array = DynamicArray::new();
        let operations = mixed_test_operations();
        let mut expected_model = Vec::new();
        let expected_snapshots: Vec<(TestOperationResult, Vec<i32>)> = operations
            .iter()
            .map(|operation| {
                let expected_result = apply_model_operation(&mut expected_model, *operation);
                (expected_result, expected_model.clone())
            })
            .collect();

        // Act
        let mut actual_snapshots = Vec::with_capacity(operations.len());
        for operation in operations {
            let actual_result = apply_array_operation(&mut array, operation);
            actual_snapshots.push((actual_result, array.to_vec()));
        }

        // Assert
        assert_eq!(actual_snapshots, expected_snapshots);
    }

    #[test]
    fn long_mixed_operation_sequence_preserves_all_treap_invariants() {
        // Arrange
        let mut array = DynamicArray::new();
        let operations = mixed_test_operations();
        let expected_invariants = vec![true; operations.len()];

        // Act
        let mut actual_invariants = Vec::with_capacity(operations.len());
        for operation in operations {
            let _actual_operation_result = apply_array_operation(&mut array, operation);
            let (tree_is_valid, counted_size) = treap_invariants(&array.root);
            let size_is_valid = counted_size == array.len();
            actual_invariants.push(tree_is_valid && size_is_valid);
        }

        // Assert
        assert_eq!(actual_invariants, expected_invariants);
    }
}
