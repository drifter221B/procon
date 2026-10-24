pub type NodeLink<T> = Option<Box<T>>;

pub trait ImplicitTreapNode: Sized {
    fn priority(&self) -> u64;
    fn left(&self) -> &NodeLink<Self>;
    fn right(&self) -> &NodeLink<Self>;
    fn left_mut(&mut self) -> &mut NodeLink<Self>;
    fn right_mut(&mut self) -> &mut NodeLink<Self>;
    fn size_of(node: &NodeLink<Self>) -> usize;
    fn push(node: &mut Box<Self>);
    fn pull(node: &mut Box<Self>);
}

pub fn merge<NodeType: ImplicitTreapNode>(
    mut left_root: NodeLink<NodeType>,
    mut right_root: NodeLink<NodeType>,
) -> NodeLink<NodeType> {
    match (left_root.take(), right_root.take()) {
        (None, right_tree) => right_tree,
        (left_tree, None) => left_tree,
        (Some(mut left_node), Some(mut right_node)) => {
            if right_node.priority() <= left_node.priority() {
                NodeType::push(&mut left_node);
                let right_child_tree = left_node.right_mut().take();
                let merged_tree = merge(right_child_tree, Some(right_node));
                *left_node.right_mut() = merged_tree;
                NodeType::pull(&mut left_node);
                Some(left_node)
            } else {
                NodeType::push(&mut right_node);
                let left_child_tree = right_node.left_mut().take();
                let merged_tree = merge(Some(left_node), left_child_tree);
                *right_node.left_mut() = merged_tree;
                NodeType::pull(&mut right_node);
                Some(right_node)
            }
        }
    }
}

pub fn split<NodeType: ImplicitTreapNode>(
    mut root_node: NodeLink<NodeType>,
    split_index: usize,
) -> (NodeLink<NodeType>, NodeLink<NodeType>) {
    if let Some(mut target_node) = root_node.take() {
        NodeType::push(&mut target_node);
        let left_subtree_size = NodeType::size_of(target_node.left());

        if split_index <= left_subtree_size {
            let left_child_tree = target_node.left_mut().take();
            let (left_tree, right_tree) = split(left_child_tree, split_index);
            *target_node.left_mut() = right_tree;
            NodeType::pull(&mut target_node);
            (left_tree, Some(target_node))
        } else {
            let next_split_index = split_index - left_subtree_size - 1;
            let right_child_tree = target_node.right_mut().take();
            let (left_tree, right_tree) = split(right_child_tree, next_split_index);
            *target_node.right_mut() = left_tree;
            NodeType::pull(&mut target_node);
            (Some(target_node), right_tree)
        }
    } else {
        (None, None)
    }
}

#[cfg(test)]
mod tests {
    use super::{merge, split, ImplicitTreapNode, NodeLink};

    #[derive(Debug)]
    struct TestNode {
        value: i32,
        priority: u64,
        size: usize,
        lazy_add: i32,
        left: NodeLink<TestNode>,
        right: NodeLink<TestNode>,
    }

    impl TestNode {
        fn new(value: i32, priority: u64) -> Box<Self> {
            Box::new(Self {
                value,
                priority,
                size: 1,
                lazy_add: 0,
                left: None,
                right: None,
            })
        }

        fn apply_add(node: &mut Box<TestNode>, added_value: i32) {
            node.value += added_value;
            node.lazy_add += added_value;
        }
    }

    impl ImplicitTreapNode for TestNode {
        fn priority(&self) -> u64 {
            self.priority
        }

        fn left(&self) -> &NodeLink<Self> {
            &self.left
        }

        fn right(&self) -> &NodeLink<Self> {
            &self.right
        }

        fn left_mut(&mut self) -> &mut NodeLink<Self> {
            &mut self.left
        }

        fn right_mut(&mut self) -> &mut NodeLink<Self> {
            &mut self.right
        }

        fn size_of(node: &NodeLink<Self>) -> usize {
            node.as_ref().map_or(0, |target_node| target_node.size)
        }

        fn push(node: &mut Box<Self>) {
            if node.lazy_add != 0 {
                let pending_add = node.lazy_add;

                if let Some(left_child_node) = node.left.as_mut() {
                    Self::apply_add(left_child_node, pending_add);
                }
                if let Some(right_child_node) = node.right.as_mut() {
                    Self::apply_add(right_child_node, pending_add);
                }

                node.lazy_add = 0;
            }
        }

        fn pull(node: &mut Box<Self>) {
            node.size = 1 + Self::size_of(&node.left) + Self::size_of(&node.right);
        }
    }

    fn build_treap(entries: &[(i32, u64)]) -> NodeLink<TestNode> {
        let mut root_node: NodeLink<TestNode> = None;
        for (value, priority) in entries {
            let new_node = Some(TestNode::new(*value, *priority));
            root_node = merge(root_node, new_node);
        }
        root_node
    }

    fn collect_values_in_order(node: &mut NodeLink<TestNode>, result_values: &mut Vec<i32>) {
        if let Some(target_node) = node.as_mut() {
            TestNode::push(target_node);
            collect_values_in_order(&mut target_node.left, result_values);
            result_values.push(target_node.value);
            collect_values_in_order(&mut target_node.right, result_values);
            TestNode::pull(target_node);
        }
    }

    fn to_values(root_node: &mut NodeLink<TestNode>) -> Vec<i32> {
        let mut result_values = Vec::new();
        collect_values_in_order(root_node, &mut result_values);
        result_values
    }

    fn apply_add_to_root(root_node: &mut NodeLink<TestNode>, added_value: i32) {
        if let Some(target_node) = root_node.as_mut() {
            TestNode::apply_add(target_node, added_value);
        }
    }

    fn create_single_node(value: i32, priority: u64) -> NodeLink<TestNode> {
        Some(TestNode::new(value, priority))
    }

    fn build_root_with_left_child(
        root_value: i32,
        root_priority: u64,
        left_value: i32,
        left_priority: u64,
    ) -> NodeLink<TestNode> {
        let mut root_node = TestNode::new(root_value, root_priority);
        root_node.left = Some(TestNode::new(left_value, left_priority));
        TestNode::pull(&mut root_node);
        Some(root_node)
    }

    fn build_root_with_right_child(
        root_value: i32,
        root_priority: u64,
        right_value: i32,
        right_priority: u64,
    ) -> NodeLink<TestNode> {
        let mut root_node = TestNode::new(root_value, root_priority);
        root_node.right = Some(TestNode::new(right_value, right_priority));
        TestNode::pull(&mut root_node);
        Some(root_node)
    }

    fn build_tree_with_left_subtree_size_two() -> NodeLink<TestNode> {
        let mut left_child_root = TestNode::new(10, 80);
        left_child_root.right = Some(TestNode::new(20, 70));
        TestNode::pull(&mut left_child_root);

        let mut root_node = TestNode::new(30, 100);
        root_node.left = Some(left_child_root);
        root_node.right = Some(TestNode::new(40, 60));
        TestNode::pull(&mut root_node);

        Some(root_node)
    }

    fn root_value(root_node: &NodeLink<TestNode>) -> Option<i32> {
        root_node.as_ref().map(|target_node| target_node.value)
    }

    fn root_priority(root_node: &NodeLink<TestNode>) -> Option<u64> {
        root_node.as_ref().map(|target_node| target_node.priority)
    }

    fn root_size(root_node: &NodeLink<TestNode>) -> usize {
        root_node.as_ref().map_or(0, |target_node| target_node.size)
    }

    fn count_nodes(root_node: &NodeLink<TestNode>) -> usize {
        if let Some(target_node) = root_node.as_ref() {
            1 + count_nodes(&target_node.left) + count_nodes(&target_node.right)
        } else {
            0
        }
    }

    #[test]
    fn merge_returns_none_when_both_inputs_are_none() {
        // Arrange
        let left_tree: NodeLink<TestNode> = None;
        let right_tree: NodeLink<TestNode> = None;
        let expected_is_none = true;

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_is_none = merged_tree.is_none();

        // Assert
        assert_eq!(actual_is_none, expected_is_none);
    }

    #[test]
    fn merge_returns_right_tree_when_left_is_none() {
        // Arrange
        let left_tree: NodeLink<TestNode> = None;
        let right_entries = [(10, 30), (20, 20), (30, 10)];
        let right_tree = build_treap(&right_entries);
        let expected_values = vec![10, 20, 30];

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_returns_left_tree_when_right_is_none() {
        // Arrange
        let left_entries = [(1, 30), (2, 20), (3, 10)];
        let left_tree = build_treap(&left_entries);
        let right_tree: NodeLink<TestNode> = None;
        let expected_values = vec![1, 2, 3];

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_keeps_left_root_when_left_priority_is_higher() {
        // Arrange
        let left_value = 10;
        let right_value = 20;
        let left_priority = 100;
        let right_priority = 50;
        let left_tree = create_single_node(left_value, left_priority);
        let right_tree = create_single_node(right_value, right_priority);
        let expected_root_value = Some(left_value);
        let expected_root_priority = Some(left_priority);
        let expected_values = vec![left_value, right_value];

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let actual_root_value = root_value(&merged_tree);
        let actual_root_priority = root_priority(&merged_tree);
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_root_value, expected_root_value);
        assert_eq!(actual_root_priority, expected_root_priority);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_keeps_right_root_when_right_priority_is_higher() {
        // Arrange
        let left_value = 10;
        let right_value = 20;
        let left_priority = 50;
        let right_priority = 100;
        let left_tree = create_single_node(left_value, left_priority);
        let right_tree = create_single_node(right_value, right_priority);
        let expected_root_value = Some(right_value);
        let expected_root_priority = Some(right_priority);
        let expected_values = vec![left_value, right_value];

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let actual_root_value = root_value(&merged_tree);
        let actual_root_priority = root_priority(&merged_tree);
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_root_value, expected_root_value);
        assert_eq!(actual_root_priority, expected_root_priority);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_prefers_left_root_when_priorities_are_equal() {
        // Arrange
        let left_value = 10;
        let right_value = 20;
        let equal_priority = 100;
        let left_tree = create_single_node(left_value, equal_priority);
        let right_tree = create_single_node(right_value, equal_priority);
        let expected_root_value = Some(left_value);
        let expected_root_priority = Some(equal_priority);
        let expected_values = vec![left_value, right_value];

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let actual_root_value = root_value(&merged_tree);
        let actual_root_priority = root_priority(&merged_tree);
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_root_value, expected_root_value);
        assert_eq!(actual_root_priority, expected_root_priority);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_preserves_concatenated_in_order_for_multi_node_trees() {
        // Arrange
        let left_entries = [(1, 100), (2, 90)];
        let right_entries = [(3, 80), (4, 70)];
        let left_tree = build_treap(&left_entries);
        let right_tree = build_treap(&right_entries);
        let expected_values = vec![1, 2, 3, 4];

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_updates_root_size_to_total_node_count_for_multi_node_trees() {
        // Arrange
        let left_entries = [(1, 100), (2, 90), (3, 80)];
        let right_entries = [(4, 70), (5, 60)];
        let left_tree = build_treap(&left_entries);
        let right_tree = build_treap(&right_entries);
        let expected_total_node_count = left_entries.len() + right_entries.len();

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let actual_root_size = root_size(&merged_tree);
        let actual_node_count = count_nodes(&merged_tree);

        // Assert
        assert_eq!(actual_root_size, expected_total_node_count);
        assert_eq!(actual_node_count, expected_total_node_count);
    }

    #[test]
    fn merge_pushes_lazy_from_left_root_before_attaching_right_tree() {
        // Arrange
        let root_value = 30;
        let root_priority = 100;
        let left_child_value = 10;
        let left_child_priority = 50;
        let right_value = 40;
        let right_priority = 10;
        let added_value = 5;
        let mut left_tree =
            build_root_with_left_child(root_value, root_priority, left_child_value, left_child_priority);
        let right_tree = create_single_node(right_value, right_priority);
        let expected_root_lazy = 0;
        let expected_left_child_value = Some(left_child_value + added_value);
        let expected_left_child_lazy = Some(added_value);
        let expected_values = vec![left_child_value + added_value, root_value + added_value, right_value];
        apply_add_to_root(&mut left_tree, added_value);

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let (actual_root_lazy, actual_left_child_value, actual_left_child_lazy) = {
            let root_node = merged_tree
                .as_ref()
                .expect("merged tree should contain a root node");
            (
                root_node.lazy_add,
                root_node.left.as_ref().map(|left_node| left_node.value),
                root_node.left.as_ref().map(|left_node| left_node.lazy_add),
            )
        };
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_root_lazy, expected_root_lazy);
        assert_eq!(actual_left_child_value, expected_left_child_value);
        assert_eq!(actual_left_child_lazy, expected_left_child_lazy);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_pushes_lazy_from_right_root_before_attaching_left_tree() {
        // Arrange
        let left_value = 10;
        let left_priority = 10;
        let root_value = 30;
        let root_priority = 100;
        let right_child_value = 40;
        let right_child_priority = 60;
        let added_value = 7;
        let left_tree = create_single_node(left_value, left_priority);
        let mut right_tree =
            build_root_with_right_child(root_value, root_priority, right_child_value, right_child_priority);
        let expected_root_lazy = 0;
        let expected_right_child_value = Some(right_child_value + added_value);
        let expected_right_child_lazy = Some(added_value);
        let expected_values = vec![left_value, root_value + added_value, right_child_value + added_value];
        apply_add_to_root(&mut right_tree, added_value);

        // Act
        let mut merged_tree = merge(left_tree, right_tree);
        let (actual_root_lazy, actual_right_child_value, actual_right_child_lazy) = {
            let root_node = merged_tree
                .as_ref()
                .expect("merged tree should contain a root node");
            (
                root_node.lazy_add,
                root_node.right.as_ref().map(|right_node| right_node.value),
                root_node.right.as_ref().map(|right_node| right_node.lazy_add),
            )
        };
        let actual_values = to_values(&mut merged_tree);

        // Assert
        assert_eq!(actual_root_lazy, expected_root_lazy);
        assert_eq!(actual_right_child_value, expected_right_child_value);
        assert_eq!(actual_right_child_lazy, expected_right_child_lazy);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn merge_after_split_restores_original_in_order_for_multiple_k() {
        // Arrange
        let entries = [(1, 100), (2, 90), (3, 80), (4, 70)];
        let split_indexes = vec![0usize, 1, 2, 3, 4, 5];
        let expected_values = vec![1, 2, 3, 4];
        let mut actual_values_by_split_index: Vec<(usize, Vec<i32>)> = Vec::new();

        // Act
        for split_index in split_indexes {
            let source_tree = build_treap(&entries);
            let (left_tree, right_tree) = split(source_tree, split_index);
            let mut merged_tree = merge(left_tree, right_tree);
            let merged_values = to_values(&mut merged_tree);
            actual_values_by_split_index.push((split_index, merged_values));
        }

        // Assert
        for (split_index, actual_values) in actual_values_by_split_index {
            assert_eq!(
                actual_values, expected_values,
                "split_index={} should restore the original sequence",
                split_index
            );
        }
    }

    #[test]
    fn split_returns_none_pair_when_root_is_none() {
        // Arrange
        let root_tree: NodeLink<TestNode> = None;
        let split_index = 0usize;
        let expected_left_is_none = true;
        let expected_right_is_none = true;

        // Act
        let (left_tree, right_tree) = split(root_tree, split_index);
        let actual_left_is_none = left_tree.is_none();
        let actual_right_is_none = right_tree.is_none();

        // Assert
        assert_eq!(actual_left_is_none, expected_left_is_none);
        assert_eq!(actual_right_is_none, expected_right_is_none);
    }

    #[test]
    fn split_single_node_at_index_zero_returns_empty_left_and_full_right() {
        // Arrange
        let node_value = 10;
        let node_priority = 100;
        let root_tree = create_single_node(node_value, node_priority);
        let split_index = 0usize;
        let expected_left_values: Vec<i32> = vec![];
        let expected_right_values = vec![node_value];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_single_node_at_index_one_returns_full_left_and_empty_right() {
        // Arrange
        let node_value = 10;
        let node_priority = 100;
        let root_tree = create_single_node(node_value, node_priority);
        let split_index = 1usize;
        let expected_left_values = vec![node_value];
        let expected_right_values: Vec<i32> = vec![];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_single_node_at_index_size_plus_one_returns_full_left_and_empty_right() {
        // Arrange
        let node_value = 10;
        let node_priority = 100;
        let root_tree = create_single_node(node_value, node_priority);
        let split_index = 2usize;
        let expected_left_values = vec![node_value];
        let expected_right_values: Vec<i32> = vec![];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_multi_node_tree_at_zero_returns_empty_left_and_full_right() {
        // Arrange
        let entries = [(1, 100), (2, 90), (3, 80), (4, 70)];
        let root_tree = build_treap(&entries);
        let split_index = 0usize;
        let expected_left_values: Vec<i32> = vec![];
        let expected_right_values = vec![1, 2, 3, 4];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_multi_node_tree_at_size_returns_full_left_and_empty_right() {
        // Arrange
        let entries = [(1, 100), (2, 90), (3, 80), (4, 70)];
        let root_tree = build_treap(&entries);
        let split_index = entries.len();
        let expected_left_values = vec![1, 2, 3, 4];
        let expected_right_values: Vec<i32> = vec![];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_multi_node_tree_at_size_plus_one_matches_split_at_size() {
        // Arrange
        let entries = [(1, 100), (2, 90), (3, 80), (4, 70)];
        let split_index_at_size = entries.len();
        let split_index_at_size_plus_one = entries.len() + 1;

        // Act
        let (mut left_tree_at_size, mut right_tree_at_size) =
            split(build_treap(&entries), split_index_at_size);
        let (mut left_tree_at_size_plus_one, mut right_tree_at_size_plus_one) =
            split(build_treap(&entries), split_index_at_size_plus_one);
        let actual_left_values_at_size = to_values(&mut left_tree_at_size);
        let actual_right_values_at_size = to_values(&mut right_tree_at_size);
        let actual_left_values_at_size_plus_one = to_values(&mut left_tree_at_size_plus_one);
        let actual_right_values_at_size_plus_one = to_values(&mut right_tree_at_size_plus_one);

        // Assert
        assert_eq!(actual_left_values_at_size_plus_one, actual_left_values_at_size);
        assert_eq!(actual_right_values_at_size_plus_one, actual_right_values_at_size);
    }

    #[test]
    fn split_around_left_subtree_size_minus_one_takes_left_branch() {
        // Arrange
        let root_tree = build_tree_with_left_subtree_size_two();
        let left_subtree_size = 2usize;
        let split_index = left_subtree_size - 1;
        let expected_left_values = vec![10];
        let expected_right_values = vec![20, 30, 40];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_around_left_subtree_size_takes_left_branch() {
        // Arrange
        let root_tree = build_tree_with_left_subtree_size_two();
        let split_index = 2usize;
        let expected_left_values = vec![10, 20];
        let expected_right_values = vec![30, 40];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_around_left_subtree_size_plus_one_takes_right_branch() {
        // Arrange
        let root_tree = build_tree_with_left_subtree_size_two();
        let left_subtree_size = 2usize;
        let split_index = left_subtree_size + 1;
        let expected_left_values = vec![10, 20, 30];
        let expected_right_values = vec![40];

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_pushes_lazy_from_root_before_partitioning_children() {
        // Arrange
        let mut root_tree = build_tree_with_left_subtree_size_two();
        let added_value = 7;
        let split_index = 2usize;
        let expected_right_root_lazy = Some(0);
        let expected_left_values = vec![17, 27];
        let expected_right_values = vec![37, 47];
        apply_add_to_root(&mut root_tree, added_value);

        // Act
        let (mut left_tree, mut right_tree) = split(root_tree, split_index);
        let actual_right_root_lazy = right_tree.as_ref().map(|right_node| right_node.lazy_add);
        let actual_left_values = to_values(&mut left_tree);
        let actual_right_values = to_values(&mut right_tree);

        // Assert
        assert_eq!(actual_right_root_lazy, expected_right_root_lazy);
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }

    #[test]
    fn split_result_subtrees_keep_correct_size_metadata() {
        // Arrange
        let entries = [(1, 100), (2, 90), (3, 80), (4, 70), (5, 60)];
        let root_tree = build_treap(&entries);
        let split_index = 2usize;
        let expected_left_node_count = split_index;
        let expected_right_node_count = entries.len() - split_index;

        // Act
        let (left_tree, right_tree) = split(root_tree, split_index);
        let actual_left_root_size = root_size(&left_tree);
        let actual_right_root_size = root_size(&right_tree);
        let actual_left_node_count = count_nodes(&left_tree);
        let actual_right_node_count = count_nodes(&right_tree);

        // Assert
        assert_eq!(actual_left_root_size, expected_left_node_count);
        assert_eq!(actual_right_root_size, expected_right_node_count);
        assert_eq!(actual_left_node_count, expected_left_node_count);
        assert_eq!(actual_right_node_count, expected_right_node_count);
    }

    #[test]
    fn split_after_merge_restores_each_side_in_order() {
        // Arrange
        let left_entries = [(1, 100), (2, 90)];
        let right_entries = [(3, 80), (4, 70), (5, 60)];
        let left_tree = build_treap(&left_entries);
        let right_tree = build_treap(&right_entries);
        let split_index = left_entries.len();
        let expected_left_values = vec![1, 2];
        let expected_right_values = vec![3, 4, 5];

        // Act
        let merged_tree = merge(left_tree, right_tree);
        let (mut split_left_tree, mut split_right_tree) = split(merged_tree, split_index);
        let actual_left_values = to_values(&mut split_left_tree);
        let actual_right_values = to_values(&mut split_right_tree);

        // Assert
        assert_eq!(actual_left_values, expected_left_values);
        assert_eq!(actual_right_values, expected_right_values);
    }
}
