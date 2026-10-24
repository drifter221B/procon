#[derive(Clone, Copy, Debug)]
pub enum Heap {
    Min,
    Max,
}

#[derive(Clone, Debug)]
pub struct Node<T> {
    pub val: T,
    pub left: Option<usize>,
    pub right: Option<usize>,
    pub parent: Option<usize>,
}

impl<T: Ord + Clone> Node<T> {

    pub fn new(val: T, left: Option<usize>, right: Option<usize>, parent: Option<usize>) -> Self {
        Self { val, left, right, parent }
    }

    #[inline]
    pub fn left(&self) -> Option<usize> {
        self.left
    }

    #[inline]
    pub fn right(&self) -> Option<usize> {
        self.right
    }

    #[inline]
    pub fn parent(&self) -> Option<usize> {
        self.parent
    }

}

#[derive(Clone, Debug)]
pub struct CartesianTree<T> {
    kind: Heap,
    root: Option<usize>,
    nodes: Vec<Node<T>>,
}

impl<T: Ord + Clone> CartesianTree<T> {

    pub fn build(values: &[T], kind: Heap) -> Self {

        let n = values.len();
        let mut nodes = values.iter()
            .cloned()
            .map(|v| Node::new(v, None, None, None))
            .collect::<Vec<Node<T>>>();

        let mut stack: Vec<usize> = Vec::with_capacity(n);

        for i in 0..n {

            let mut last = None;

            while let Some(&top) = stack.last() {

                if match kind {
                    Heap::Min => nodes[i].val < nodes[top].val,
                    Heap::Max => nodes[top].val < nodes[i].val,
                } {
                    last = Some(top);
                    stack.pop();
                } else {
                    break;
                }
            }

            if let Some(&parent) = stack.last() {
                nodes[parent].right = Some(i);
                nodes[i].parent = Some(parent);
            }

            if let Some(left) = last {
                nodes[i].left = Some(left);
                nodes[left].parent = Some(i);
            }

            stack.push(i);
        }

        let root = stack.first().copied();

        CartesianTree { kind, root, nodes }
    }

    #[inline]
    pub fn node(&self, index: usize) -> &Node<T> {
        &self.nodes[index]
    }

    pub fn in_order(&self) -> Vec<T> {

        let mut results = Vec::with_capacity(self.nodes.len());

        if let Some(root) = self.root {
            self.dfs_in_order(root, &mut results);
        }

        results
    }

    fn dfs_in_order(&self, vertex: usize, orders: &mut Vec<T>) {

        if let Some(left) = self.nodes[vertex].left {
            self.dfs_in_order(left, orders);
        }

        orders.push(self.nodes[vertex].val.clone());

        if let Some(right) = self.nodes[vertex].right {
            self.dfs_in_order(right, orders);
        }

    }

    pub fn is_heapified(&self) -> bool {

        fn ok<T: Ord>(nodes: &[Node<T>], kind: Heap, vertex: usize) -> bool {

            let me = &nodes[vertex].val;

            if let Some(left) = nodes[vertex].left {

                match kind {
                    Heap::Min if &nodes[left].val < me => return false,
                    Heap::Max if me < &nodes[left].val => return false,
                    _ => {}
                }

                if !ok(nodes, kind, left) { return false; }
            }

            if let Some(right) = nodes[vertex].right {

                match kind {
                    Heap::Min if &nodes[right].val < me => return false,
                    Heap::Max if me < &nodes[right].val => return false,
                    _ => {}
                }

                if !ok(nodes, kind, right) { return false; }
            }

            true
        }

        match self.root {
            Some(root) => ok(&self.nodes, self.kind, root),
            None => true,
        }
    }

}

#[cfg(test)]
mod todo_tests {
    use super::{CartesianTree, Heap, Node};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn assert_right_chain(tree: &CartesianTree<i64>) {
        for index in 0..tree.in_order().len() {
            assert!(tree.node(index).left().is_none());
            if index + 1 < tree.in_order().len() {
                assert_eq!(tree.node(index).right(), Some(index + 1));
            } else {
                assert_eq!(tree.node(index).right(), None);
            }
        }
    }

    fn assert_left_chain(tree: &CartesianTree<i64>) {
        for index in 0..tree.in_order().len() {
            assert!(tree.node(index).right().is_none());
            if index == 0 {
                assert_eq!(tree.node(index).left(), None);
            } else {
                assert_eq!(tree.node(index).left(), Some(index - 1));
            }
        }
    }

    #[test]
    fn node_constructor_and_accessors_return_same_values() {
        // Arrange
        let expected_value = (10, Some(1), Some(2), Some(3));

        // Act
        let node = Node::new(10, Some(1), Some(2), Some(3));
        let actual_value = (node.val, node.left(), node.right(), node.parent());

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn empty_trees_are_heapified_for_both_modes() {
        // Arrange
        let empty_values: Vec<i64> = vec![];

        // Act
        let min_tree = CartesianTree::build(&empty_values, Heap::Min);
        let max_tree = CartesianTree::build(&empty_values, Heap::Max);
        let actual_values = (
            min_tree.in_order(),
            min_tree.is_heapified(),
            max_tree.in_order(),
            max_tree.is_heapified(),
        );
        let expected_values = (Vec::<i64>::new(), true, Vec::<i64>::new(), true);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn singleton_trees_have_no_relations_for_both_modes() {
        // Arrange
        let values = vec![7];

        // Act
        let min_tree = CartesianTree::build(&values, Heap::Min);
        let max_tree = CartesianTree::build(&values, Heap::Max);

        // Assert
        for tree in [min_tree, max_tree] {
            let node = tree.node(0);
            assert_eq!(node.left(), None);
            assert_eq!(node.right(), None);
            assert_eq!(node.parent(), None);
        }
    }

    #[test]
    fn boundary_triples_keep_input_order_and_heap_property() {
        // Arrange
        let values = vec![4, 5, 6];

        // Act
        let min_tree = CartesianTree::build(&values, Heap::Min);
        let max_tree = CartesianTree::build(&values, Heap::Max);
        let actual_values = (
            min_tree.in_order(),
            min_tree.is_heapified(),
            max_tree.in_order(),
            max_tree.is_heapified(),
        );
        let expected_values = (values.clone(), true, values, true);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn monotonic_inputs_create_expected_chain_shapes() {
        // Arrange
        let increasing = vec![1, 2, 3, 4];
        let decreasing = vec![4, 3, 2, 1];

        // Act
        let inc_min = CartesianTree::build(&increasing, Heap::Min);
        let dec_min = CartesianTree::build(&decreasing, Heap::Min);
        let inc_max = CartesianTree::build(&increasing, Heap::Max);
        let dec_max = CartesianTree::build(&decreasing, Heap::Max);

        // Assert
        assert_right_chain(&inc_min);
        assert_left_chain(&dec_min);
        assert_left_chain(&inc_max);
        assert_right_chain(&dec_max);
    }

    #[test]
    fn duplicate_values_create_right_chains_for_both_heap_modes() {
        // Arrange
        let duplicates = vec![2, 2, 2];

        // Act
        let dup_min = CartesianTree::build(&duplicates, Heap::Min);
        let dup_max = CartesianTree::build(&duplicates, Heap::Max);
        let actual_values = (
            dup_min.in_order(),
            dup_min.is_heapified(),
            dup_max.in_order(),
            dup_max.is_heapified(),
        );
        let expected_values = (duplicates.clone(), true, duplicates, true);

        // Assert
        assert_right_chain(&dup_min);
        assert_right_chain(&dup_max);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn extreme_values_preserve_in_order_sequence_for_both_heap_modes() {
        // Arrange
        let values = vec![i64::MIN, -1, 0, 1, i64::MAX];

        // Act
        let min_tree = CartesianTree::build(&values, Heap::Min);
        let max_tree = CartesianTree::build(&values, Heap::Max);
        let actual_values = (min_tree.in_order(), max_tree.in_order());
        let expected_values = (values.clone(), values);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn parent_child_links_and_lengths_are_consistent() {
        // Arrange
        let values = vec![3, 1, 4, 2];

        // Act
        let tree = CartesianTree::build(&values, Heap::Min);

        // Assert
        assert_eq!(tree.in_order().len(), values.len());
        let mut root_count = 0;
        for index in 0..values.len() {
            let node = tree.node(index);
            if node.parent().is_none() {
                root_count += 1;
            }
            if let Some(left) = node.left() {
                assert_eq!(tree.node(left).parent(), Some(index));
            }
            if let Some(right) = node.right() {
                assert_eq!(tree.node(right).parent(), Some(index));
            }
        }
        assert_eq!(root_count, 1);
    }

    #[test]
    fn node_out_of_bounds_panics() {
        // Arrange
        let tree = CartesianTree::build(&[1, 2], Heap::Min);

        // Act
        let actual_result = catch_unwind(AssertUnwindSafe(|| tree.node(2)));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn manual_heap_break_is_detected_for_both_modes() {
        // Arrange
        let mut min_tree = CartesianTree::build(&[1, 2, 3], Heap::Min);
        let mut max_tree = CartesianTree::build(&[3, 2, 1], Heap::Max);

        // Act
        min_tree.nodes[0].val = 10;
        max_tree.nodes[0].val = -10;
        let actual_values = (min_tree.is_heapified(), max_tree.is_heapified());
        let expected_values = (false, false);

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
