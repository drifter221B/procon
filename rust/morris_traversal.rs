#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TreeNode {
    pub val: i32,
    pub left: Option<Box<TreeNode>>,
    pub right: Option<Box<TreeNode>>,
}

impl TreeNode {
    pub fn new(val: i32) -> Self {
        Self {
            val,
            left: None,
            right: None,
        }
    }
}

/// 中順走査で全ノードの値を取得する。時間は O(n)、補助メモリは O(h)（h は木の高さ）。
pub fn inorder_traversal(root: &mut Option<Box<TreeNode>>) -> Vec<i32> {
    let mut current = root.as_deref();
    let mut stack = Vec::new();
    let mut result = Vec::new();

    while current.is_some() || !stack.is_empty() {
        while let Some(node) = current {
            stack.push(node);
            current = node.left.as_deref();
        }

        let node = stack.pop().unwrap();
        result.push(node.val);
        current = node.right.as_deref();
    }

    result
}

#[cfg(test)]
mod tests {
    use super::{inorder_traversal, TreeNode};

    fn build_tree() -> Option<Box<TreeNode>> {
        //       1
        //      / \
        //     2   3
        //    / \ /
        //   4  5 6
        let mut n1 = TreeNode::new(1);
        let mut n2 = TreeNode::new(2);
        let n3 = TreeNode::new(3);
        let n4 = TreeNode::new(4);
        let n5 = TreeNode::new(5);
        let n6 = TreeNode::new(6);

        n2.left = Some(Box::new(n4));
        n2.right = Some(Box::new(n5));

        let mut n3 = Box::new(n3);
        n3.left = Some(Box::new(n6));

        n1.left = Some(Box::new(n2));
        n1.right = Some(n3);

        Some(Box::new(n1))
    }

    #[test]
    fn empty_tree_returns_empty_vec() {
        // Arrange
        let mut root: Option<Box<TreeNode>> = None;
        let expected_values = vec![];

        // Act
        let actual_values = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn tree_with_both_children_returns_known_inorder_sequence() {
        // Arrange
        let mut root = build_tree();
        let expected_values = vec![4, 2, 5, 1, 6, 3];

        // Act
        let actual_values = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn inorder_traversal_preserves_tree_structure() {
        // Arrange
        let mut root = build_tree();
        let expected_root = root.clone();
        let expected_values = vec![4, 2, 5, 1, 6, 3];

        // Act
        let actual_values = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(root, expected_root);
    }

    #[test]
    fn single_node_tree_returns_single_element() {
        // Arrange
        let mut root = Some(Box::new(TreeNode::new(10)));
        let expected_values = vec![10];

        // Act
        let actual_values = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn left_skewed_tree_is_traversed_in_order() {
        // Arrange
        let mut root = Some(Box::new(TreeNode {
            val: 3,
            left: Some(Box::new(TreeNode {
                val: 2,
                left: Some(Box::new(TreeNode::new(1))),
                right: None,
            })),
            right: None,
        }));
        let expected_values = vec![1, 2, 3];

        // Act
        let actual_values = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn right_skewed_tree_is_traversed_in_order() {
        // Arrange
        let mut root = Some(Box::new(TreeNode {
            val: 1,
            left: None,
            right: Some(Box::new(TreeNode {
                val: 2,
                left: None,
                right: Some(Box::new(TreeNode::new(3))),
            })),
        }));
        let expected_values = vec![1, 2, 3];

        // Act
        let actual_values = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn repeated_traversal_returns_same_result() {
        // Arrange
        let mut root = Some(Box::new(TreeNode {
            val: 2,
            left: Some(Box::new(TreeNode::new(1))),
            right: Some(Box::new(TreeNode::new(3))),
        }));
        let expected_values = vec![1, 2, 3];

        // Act
        let actual_first = inorder_traversal(&mut root);
        let actual_second = inorder_traversal(&mut root);

        // Assert
        assert_eq!(actual_first, expected_values);
        assert_eq!(actual_second, expected_values);
    }
}
