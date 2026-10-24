use std::rc::Rc;

#[derive(Debug, Clone)]
struct Node<T> {
    value: T,
    prev: Option<Rc<Node<T>>>,
    len: usize,
}

pub struct PersistentStack<T> {
    head: Option<Rc<Node<T>>>,
}

impl<T> Clone for PersistentStack<T> {
    fn clone(&self) -> Self {
        Self {
            head: self.head.clone(),
        }
    }
}

impl<T> Default for PersistentStack<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T> PersistentStack<T> {
    pub fn new() -> Self {
        Self { head: None }
    }

    pub fn is_empty(&self) -> bool {
        self.head.is_none()
    }

    pub fn len(&self) -> usize {
        self.head.as_ref().map_or(0, |node| node.len)
    }

    pub fn top(&self) -> Option<&T> {
        self.head.as_ref().map(|node| &node.value)
    }

    pub fn push(&self, value: T) -> Self {
        let new_head = Rc::new(Node {
            value,
            prev: self.head.clone(),
            len: self.len() + 1,
        });
        Self {
            head: Some(new_head),
        }
    }

    pub fn pop(&self) -> Option<Self> {
        self.head.as_ref().map(|node| Self {
            head: node.prev.clone(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::PersistentStack;
    use std::{cell::Cell, rc::Rc};

    #[test]
    fn new_stack_is_empty() {
        // Arrange
        let expected_len = 0;

        // Act
        let stack = PersistentStack::<i32>::new();
        let actual_len = stack.len();
        let actual_is_empty = stack.is_empty();
        let actual_top = stack.top();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert!(actual_is_empty);
        assert!(actual_top.is_none());
    }

    #[test]
    fn default_stack_matches_new_stack_initial_state() {
        // Arrange
        let new_stack: PersistentStack<i32> = PersistentStack::new();
        let default_stack: PersistentStack<i32> = PersistentStack::default();

        let expected_new_len_right = 0;

        // Act
        let new_len = new_stack.len();
        let new_is_empty = new_stack.is_empty();
        let new_top = new_stack.top();
        let default_len = default_stack.len();
        let default_is_empty = default_stack.is_empty();
        let default_top = default_stack.top();

        // Assert
        assert_eq!(new_len, expected_new_len_right);
        assert!(new_is_empty);
        assert!(new_top.is_none());
        assert_eq!(default_len, new_len);
        assert_eq!(default_is_empty, new_is_empty);
        assert_eq!(default_top, new_top);
    }

    #[test]
    fn len_and_is_empty_are_consistent_across_boundary_transitions() {
        // Arrange
        let first_value = 11;
        let second_value = 22;
        let stack_0: PersistentStack<i32> = PersistentStack::new();

        let expected_stack_0_len_right = 0;
        let expected_stack_1_len_right = 1;
        let expected_stack_2_len_right = 2;
        let expected_stack_1_again_len_right = 1;
        let expected_stack_0_again_len_right = 0;

        // Act
        let stack_1 = stack_0.push(first_value);
        let stack_2 = stack_1.push(second_value);
        let stack_1_again = stack_2.pop().expect("stack with two elements should pop");
        let stack_0_again = stack_1_again
            .pop()
            .expect("stack with one element should pop");
        let stack_0_len = stack_0.len();
        let stack_0_is_empty = stack_0.is_empty();
        let stack_1_len = stack_1.len();
        let stack_1_is_empty = stack_1.is_empty();
        let stack_2_len = stack_2.len();
        let stack_2_is_empty = stack_2.is_empty();
        let stack_1_again_len = stack_1_again.len();
        let stack_1_again_is_empty = stack_1_again.is_empty();
        let stack_0_again_len = stack_0_again.len();
        let stack_0_again_is_empty = stack_0_again.is_empty();

        // Assert
        assert_eq!(stack_0_len, expected_stack_0_len_right);
        assert_eq!(stack_0_is_empty, stack_0_len == 0);
        assert_eq!(stack_1_len, expected_stack_1_len_right);
        assert_eq!(stack_1_is_empty, stack_1_len == 0);
        assert_eq!(stack_2_len, expected_stack_2_len_right);
        assert_eq!(stack_2_is_empty, stack_2_len == 0);
        assert_eq!(stack_1_again_len, expected_stack_1_again_len_right);
        assert_eq!(stack_1_again_is_empty, stack_1_again_len == 0);
        assert_eq!(stack_0_again_len, expected_stack_0_again_len_right);
        assert_eq!(stack_0_again_is_empty, stack_0_again_len == 0);
    }

    #[test]
    fn push_increases_length_and_updates_top() {
        // Arrange
        let values = [1, 2];
        let stack = PersistentStack::new();
        let expected_states = [(1, Some(1)), (2, Some(2))];

        // Act
        let first_stack = stack.push(values[0]);
        let second_stack = first_stack.push(values[1]);
        let actual_states = [
            (first_stack.len(), first_stack.top().copied()),
            (second_stack.len(), second_stack.top().copied()),
        ];

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn push_top_pop_work_with_non_copy_string_values() {
        // Arrange
        let bottom_value = String::from("bottom");
        let top_value = String::from("top");
        let empty_stack: PersistentStack<String> = PersistentStack::new();

        let expected_popped_len_right = 1;
        let expected_top = Some(top_value.clone());
        let expected_after_pop = Some(bottom_value.clone());

        // Act
        let stack_with_bottom = empty_stack.push(bottom_value.clone());
        let stack_with_top = stack_with_bottom.push(top_value.clone());
        let top_on_stack_with_top = stack_with_top.top().cloned();
        let popped_stack = stack_with_top.pop().expect("stack should pop");
        let top_after_pop = popped_stack.top().cloned();
        let popped_len = popped_stack.len();

        // Assert
        assert_eq!(top_on_stack_with_top, expected_top);
        assert_eq!(top_after_pop, expected_after_pop);
        assert_eq!(popped_len, expected_popped_len_right);
    }

    #[test]
    fn clone_shares_the_same_top_element() {
        // Arrange
        let original_stack = PersistentStack::new().push(String::from("shared"));

        // Act
        let cloned_stack = original_stack.clone();
        let original_top = original_stack.top().expect("original should have a top");
        let cloned_top = cloned_stack.top().expect("clone should have a top");

        // Assert
        assert!(std::ptr::eq(original_top, cloned_top));
    }

    #[test]
    fn pop_after_push_restores_the_same_top_element() {
        // Arrange
        let original_stack = PersistentStack::new().push(String::from("shared"));

        // Act
        let pushed_stack = original_stack.push(String::from("temporary"));
        let restored_stack = pushed_stack.pop().expect("pushed stack should pop");
        let original_top = original_stack.top().expect("original should have a top");
        let restored_top = restored_stack
            .top()
            .expect("restored stack should have a top");

        // Assert
        assert!(std::ptr::eq(original_top, restored_top));
    }

    #[test]
    fn push_on_empty_stack_keeps_original_stack_unchanged() {
        // Arrange
        let pushed_value = 5;
        let original_empty_stack: PersistentStack<i32> = PersistentStack::new();

        let expected_original_len_after_push_right = 0;
        let expected_pushed_len_right = 1;

        // Act
        let pushed_stack = original_empty_stack.push(pushed_value);
        let original_len_after_push = original_empty_stack.len();
        let original_is_empty_after_push = original_empty_stack.is_empty();
        let original_top_after_push = original_empty_stack.top();
        let pushed_len = pushed_stack.len();
        let pushed_top = pushed_stack.top();

        let actual_pushed_top_right = Some(&pushed_value);

        // Assert
        assert_eq!(
            original_len_after_push,
            expected_original_len_after_push_right
        );
        assert!(original_is_empty_after_push);
        assert!(original_top_after_push.is_none());
        assert_eq!(pushed_len, expected_pushed_len_right);
        assert_eq!(pushed_top, actual_pushed_top_right);
    }

    #[test]
    fn push_on_same_version_creates_independent_branches() {
        // Arrange
        let base_stack: PersistentStack<i32> = PersistentStack::new();
        let branch_a_value = 1;
        let branch_b_value = 2;

        let expected_base_len_right = 0;
        let expected_branch_a_len_right = 1;
        let expected_branch_b_len_right = 1;

        // Act
        let branch_a_stack = base_stack.push(branch_a_value);
        let branch_b_stack = base_stack.push(branch_b_value);
        let branch_a_len = branch_a_stack.len();
        let branch_a_top = branch_a_stack.top();
        let branch_b_len = branch_b_stack.len();
        let branch_b_top = branch_b_stack.top();
        let base_len = base_stack.len();
        let base_top = base_stack.top();

        let actual_branch_a_top_right = Some(&branch_a_value);
        let actual_branch_b_top_right = Some(&branch_b_value);

        // Assert
        assert_eq!(base_len, expected_base_len_right);
        assert!(base_top.is_none());
        assert_eq!(branch_a_len, expected_branch_a_len_right);
        assert_eq!(branch_a_top, actual_branch_a_top_right);
        assert_eq!(branch_b_len, expected_branch_b_len_right);
        assert_eq!(branch_b_top, actual_branch_b_top_right);
    }

    #[test]
    fn clone_stack_remains_independent_after_push_and_pop() {
        // Arrange
        let first_value = 1;
        let second_value = 2;
        let pushed_value = 3;
        let original_stack = PersistentStack::new().push(first_value).push(second_value);
        let cloned_stack = original_stack.clone();

        let expected_original_len_right = 2;
        let expected_clone_len_right = 2;
        let expected_pushed_len_right = 3;
        let expected_popped_len_right = 1;

        // Act
        let pushed_from_original = original_stack.push(pushed_value);
        let popped_from_clone = cloned_stack.pop().expect("cloned stack should pop");
        let original_len = original_stack.len();
        let original_top = original_stack.top();
        let clone_len = cloned_stack.len();
        let clone_top = cloned_stack.top();
        let pushed_len = pushed_from_original.len();
        let pushed_top = pushed_from_original.top();
        let popped_len = popped_from_clone.len();
        let popped_top = popped_from_clone.top();

        let actual_original_top_right = Some(&second_value);
        let actual_clone_top_right = Some(&second_value);
        let actual_pushed_top_right = Some(&pushed_value);
        let actual_popped_top_right = Some(&first_value);

        // Assert
        assert_eq!(original_len, expected_original_len_right);
        assert_eq!(original_top, actual_original_top_right);
        assert_eq!(clone_len, expected_clone_len_right);
        assert_eq!(clone_top, actual_clone_top_right);
        assert_eq!(pushed_len, expected_pushed_len_right);
        assert_eq!(pushed_top, actual_pushed_top_right);
        assert_eq!(popped_len, expected_popped_len_right);
        assert_eq!(popped_top, actual_popped_top_right);
    }

    #[test]
    fn push_branches_from_non_empty_common_ancestor_remain_independent() {
        // Arrange
        let common_ancestor = PersistentStack::new().push(10).push(20);

        let expected_common_ancestor_len_right = 2;
        let expected_common_ancestor_top_right = Some(&20);
        let expected_branch_a_len_right = 3;
        let expected_branch_a_top_right = Some(&30);
        let expected_branch_b_len_right = 3;
        let expected_branch_b_top_right = Some(&40);
        let expected_extended_branch_a_len_right = 4;
        let expected_extended_branch_a_top_right = Some(&31);

        // Act
        let branch_a = common_ancestor.push(30);
        let branch_b = common_ancestor.push(40);
        let extended_branch_a = branch_a.push(31);
        let restored_from_branch_a = branch_a.pop().expect("branch A should pop");
        let restored_from_branch_b = branch_b.pop().expect("branch B should pop");

        let actual_common_ancestor_len = common_ancestor.len();
        let actual_common_ancestor_top = common_ancestor.top();
        let actual_branch_a_len = branch_a.len();
        let actual_branch_a_top = branch_a.top();
        let actual_branch_b_len = branch_b.len();
        let actual_branch_b_top = branch_b.top();
        let actual_extended_branch_a_len = extended_branch_a.len();
        let actual_extended_branch_a_top = extended_branch_a.top();
        let actual_restored_from_branch_a_len = restored_from_branch_a.len();
        let actual_restored_from_branch_a_len_right = common_ancestor.len();
        let actual_restored_from_branch_a_top = restored_from_branch_a.top();
        let actual_restored_from_branch_a_top_right = common_ancestor.top();
        let actual_restored_from_branch_b_len = restored_from_branch_b.len();
        let actual_restored_from_branch_b_len_right = common_ancestor.len();
        let actual_restored_from_branch_b_top = restored_from_branch_b.top();
        let actual_restored_from_branch_b_top_right = common_ancestor.top();

        // Assert
        assert_eq!(
            actual_common_ancestor_len,
            expected_common_ancestor_len_right
        );
        assert_eq!(
            actual_common_ancestor_top,
            expected_common_ancestor_top_right
        );
        assert_eq!(actual_branch_a_len, expected_branch_a_len_right);
        assert_eq!(actual_branch_a_top, expected_branch_a_top_right);
        assert_eq!(actual_branch_b_len, expected_branch_b_len_right);
        assert_eq!(actual_branch_b_top, expected_branch_b_top_right);
        assert_eq!(
            actual_extended_branch_a_len,
            expected_extended_branch_a_len_right
        );
        assert_eq!(
            actual_extended_branch_a_top,
            expected_extended_branch_a_top_right
        );
        assert_eq!(
            actual_restored_from_branch_a_len,
            actual_restored_from_branch_a_len_right
        );
        assert_eq!(
            actual_restored_from_branch_a_top,
            actual_restored_from_branch_a_top_right
        );
        assert_eq!(
            actual_restored_from_branch_b_len,
            actual_restored_from_branch_b_len_right
        );
        assert_eq!(
            actual_restored_from_branch_b_top,
            actual_restored_from_branch_b_top_right
        );
    }

    #[test]
    fn long_push_and_pop_sequence_matches_reference_vec() {
        // Arrange
        let element_count = 1_024usize;
        let values = (0..element_count).collect::<Vec<_>>();
        let expected_push_states = values
            .iter()
            .enumerate()
            .map(|(index, &value)| (index + 1, Some(value)))
            .collect::<Vec<_>>();
        let expected_pop_states = (0..element_count)
            .rev()
            .map(|length| (length, length.checked_sub(1)))
            .collect::<Vec<_>>();
        let mut stack = PersistentStack::new();

        // Act
        let mut actual_push_states = Vec::new();
        for value in values {
            stack = stack.push(value);
            actual_push_states.push((stack.len(), stack.top().copied()));
        }
        let mut actual_pop_states = Vec::new();
        for _ in 0..element_count {
            stack = stack.pop().unwrap();
            actual_pop_states.push((stack.len(), stack.top().copied()));
        }
        let actual_final_is_empty = stack.is_empty();

        // Assert
        assert_eq!(actual_push_states, expected_push_states);
        assert_eq!(actual_pop_states, expected_pop_states);
        assert!(actual_final_is_empty);
    }

    #[test]
    fn pop_returns_previous_version() {
        // Arrange
        let values = [1, 2];
        let stack = PersistentStack::new().push(values[0]).push(values[1]);
        let expected_previous_state = (1, Some(values[0]));
        let expected_empty_state = (0, true, None);

        // Act
        let previous = stack.pop().unwrap();
        let actual_previous_state = (previous.len(), previous.top().copied());
        let empty = previous.pop().unwrap();
        let actual_empty_state = (empty.len(), empty.is_empty(), empty.top().copied());

        // Assert
        assert_eq!(actual_previous_state, expected_previous_state);
        assert_eq!(actual_empty_state, expected_empty_state);
    }

    #[test]
    fn pop_on_empty_stack_returns_none() {
        // Arrange
        let empty_stack: PersistentStack<i32> = PersistentStack::new();

        // Act
        let popped_stack = empty_stack.pop();

        // Assert
        assert!(popped_stack.is_none());
    }

    #[test]
    fn pop_on_single_element_stack_returns_empty_stack() {
        // Arrange
        let pushed_value = 10;
        let one_element_stack = PersistentStack::new().push(pushed_value);

        let expected_popped_len_right = 0;

        // Act
        let popped_stack = one_element_stack
            .pop()
            .expect("single element stack should pop");
        let popped_len = popped_stack.len();
        let popped_is_empty = popped_stack.is_empty();
        let popped_top = popped_stack.top();

        // Assert
        assert_eq!(popped_len, expected_popped_len_right);
        assert!(popped_is_empty);
        assert!(popped_top.is_none());
    }

    #[test]
    fn versions_are_independent() {
        // Arrange
        let values = [1, 2, 3];
        let first = PersistentStack::new().push(values[0]);
        let second = first.push(values[1]);
        let third = second.push(values[2]);
        let expected_second = (second.len(), second.top().copied());
        let expected_first = (1, Some(values[0]));
        let expected_third = (3, Some(values[2]));

        // Act
        let second_again = third.pop().unwrap();
        let first_again = second_again.pop().unwrap();
        let actual_second = (second_again.len(), second_again.top().copied());
        let actual_first = (first_again.len(), first_again.top().copied());
        let actual_third = (third.len(), third.top().copied());

        // Assert
        assert_eq!(actual_second, expected_second);
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_third, expected_third);
    }

    #[test]
    fn clone_non_empty_stack_does_not_require_element_to_implement_clone() {
        // Arrange
        #[derive(Debug, PartialEq, Eq)]
        struct NonCloneValue(i32);

        let original_stack = PersistentStack::new().push(NonCloneValue(42));

        let expected_original_len_right = 1;
        let expected_original_top_right = Some(42);

        // Act
        let cloned_stack = original_stack.clone();
        let original_len = original_stack.len();
        let original_top = original_stack.top().map(|value| value.0);
        let cloned_len = cloned_stack.len();
        let cloned_top = cloned_stack.top().map(|value| value.0);

        // Assert
        assert_eq!(original_len, expected_original_len_right);
        assert_eq!(cloned_len, original_len);
        assert_eq!(original_top, expected_original_top_right);
        assert_eq!(cloned_top, original_top);
    }

    #[test]
    fn cloned_version_survives_after_original_is_dropped() {
        // Arrange
        let original_stack = PersistentStack::new().push(10).push(20);
        let cloned_stack = original_stack.clone();

        let expected_cloned_stack_len_right = 2;
        let expected_cloned_stack_top_right = Some(&20);
        let expected_pushed_stack_len_right = 3;
        let expected_pushed_stack_top_right = Some(&30);

        // Act
        drop(original_stack);
        let pushed_stack = cloned_stack.push(30);
        let restored_clone = pushed_stack.pop().expect("pushed clone should pop");

        let actual_cloned_stack_len = cloned_stack.len();
        let actual_cloned_stack_top = cloned_stack.top();
        let actual_pushed_stack_len = pushed_stack.len();
        let actual_pushed_stack_top = pushed_stack.top();
        let actual_restored_clone_len = restored_clone.len();
        let actual_restored_clone_len_right = cloned_stack.len();
        let actual_restored_clone_top = restored_clone.top();
        let actual_restored_clone_top_right = cloned_stack.top();

        // Assert
        assert_eq!(actual_cloned_stack_len, expected_cloned_stack_len_right);
        assert_eq!(actual_cloned_stack_top, expected_cloned_stack_top_right);
        assert_eq!(actual_pushed_stack_len, expected_pushed_stack_len_right);
        assert_eq!(actual_pushed_stack_top, expected_pushed_stack_top_right);
        assert_eq!(actual_restored_clone_len, actual_restored_clone_len_right);
        assert_eq!(actual_restored_clone_top, actual_restored_clone_top_right);
    }

    #[test]
    fn shared_elements_are_dropped_once_after_last_referencing_version() {
        // Arrange
        struct DropTrackedValue {
            index: usize,
            drop_counts: Rc<Vec<Cell<usize>>>,
        }

        impl Drop for DropTrackedValue {
            fn drop(&mut self) {
                let count = &self.drop_counts[self.index];
                count.set(count.get() + 1);
            }
        }

        let drop_counts = Rc::new((0..4).map(|_| Cell::new(0)).collect::<Vec<_>>());
        let common_ancestor = PersistentStack::new()
            .push(DropTrackedValue {
                index: 0,
                drop_counts: drop_counts.clone(),
            })
            .push(DropTrackedValue {
                index: 1,
                drop_counts: drop_counts.clone(),
            });
        let branch_a = common_ancestor.push(DropTrackedValue {
            index: 2,
            drop_counts: drop_counts.clone(),
        });
        let branch_b = common_ancestor.push(DropTrackedValue {
            index: 3,
            drop_counts: drop_counts.clone(),
        });
        let cloned_branch_a = branch_a.clone();
        let observed_drop_counts = || drop_counts.iter().map(Cell::get).collect::<Vec<_>>();

        let expected_snapshots = vec![
            vec![0, 0, 0, 0],
            vec![0, 0, 0, 0],
            vec![0, 0, 1, 0],
            vec![1, 1, 1, 1],
        ];

        // Act
        drop(common_ancestor);
        let after_ancestor = observed_drop_counts();
        drop(branch_a);
        let after_branch_a = observed_drop_counts();
        drop(cloned_branch_a);
        let after_last_branch_a = observed_drop_counts();
        drop(branch_b);
        let after_last_branch = observed_drop_counts();
        let actual_snapshots = vec![
            after_ancestor,
            after_branch_a,
            after_last_branch_a,
            after_last_branch,
        ];

        // Assert
        assert_eq!(actual_snapshots, expected_snapshots);
    }

    #[test]
    fn child_version_survives_after_parent_is_dropped() {
        // Arrange
        let parent_stack = PersistentStack::new().push(10);
        let child_stack = parent_stack.push(20);

        let expected_child_len_right = 2;
        let expected_child_top_right = Some(20);
        let expected_restored_ancestor_len_right = 1;
        let expected_restored_ancestor_top_right = Some(&10);

        // Act
        drop(parent_stack);
        let child_len = child_stack.len();
        let child_top = child_stack.top().copied();
        let restored_ancestor = child_stack.pop().expect("child stack should pop");

        let actual_restored_ancestor_len = restored_ancestor.len();
        let actual_restored_ancestor_top = restored_ancestor.top();

        // Assert
        assert_eq!(child_len, expected_child_len_right);
        assert_eq!(child_top, expected_child_top_right);
        assert_eq!(
            actual_restored_ancestor_len,
            expected_restored_ancestor_len_right
        );
        assert_eq!(
            actual_restored_ancestor_top,
            expected_restored_ancestor_top_right
        );
    }
}
