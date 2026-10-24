pub struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Default for Stack<T>
where
    T: Default,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<T> Stack<T> {
    pub fn new() -> Self {
        Stack { stack: vec![] }
    }

    pub fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }

    pub fn len(&self) -> usize {
        self.stack.len()
    }

    pub fn peek(&self) -> Option<&T> {
        self.stack.last()
    }

    pub fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }

    pub fn push(&mut self, data: T) {
        self.stack.push(data);
    }
}

#[cfg(test)]
mod tests {
    use super::Stack;

    #[test]
    fn default_creates_same_empty_state_as_new() {
        // Arrange
        let default_stack: Stack<i32> = Stack::default();
        let new_stack: Stack<i32> = Stack::new();
        let expected_len = new_stack.len();
        let expected_is_empty = new_stack.is_empty();
        let expected_peek_value = new_stack.peek().copied();

        // Act
        let actual_len = default_stack.len();
        let actual_is_empty = default_stack.is_empty();
        let actual_peek_value = default_stack.peek().copied();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_peek_value, expected_peek_value);
    }

    #[test]
    fn empty_stack_boundaries_keep_state_after_repeated_operations() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let expected_len = 0usize;
        let expected_peek_value: Option<i32> = None;
        let expected_pop_value: Option<i32> = None;
        let expected_is_empty = true;

        // Act
        let first_peek_value = stack.peek().copied();
        let first_pop_value = stack.pop();
        let len_after_first_round = stack.len();
        let is_empty_after_first_round = stack.is_empty();

        let second_peek_value = stack.peek().copied();
        let second_pop_value = stack.pop();
        let len_after_second_round = stack.len();
        let is_empty_after_second_round = stack.is_empty();

        // Assert
        assert_eq!(first_peek_value, expected_peek_value);
        assert_eq!(first_pop_value, expected_pop_value);
        assert_eq!(len_after_first_round, expected_len);
        assert_eq!(is_empty_after_first_round, expected_is_empty);
        assert_eq!(second_peek_value, expected_peek_value);
        assert_eq!(second_pop_value, expected_pop_value);
        assert_eq!(len_after_second_round, expected_len);
        assert_eq!(is_empty_after_second_round, expected_is_empty);
    }

    #[test]
    fn len_changes_by_one_for_each_push_and_pop_with_multiple_elements() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_len_initial = 0usize;
        let expected_len_after_first_push = 1usize;
        let expected_len_after_second_push = 2usize;
        let expected_len_after_third_push = 3usize;
        let expected_len_after_first_pop = 2usize;
        let expected_len_after_second_pop = 1usize;
        let expected_len_after_third_pop = 0usize;

        // Act
        let len_initial = stack.len();
        stack.push(first_value);
        let len_after_first_push = stack.len();
        stack.push(second_value);
        let len_after_second_push = stack.len();
        stack.push(third_value);
        let len_after_third_push = stack.len();

        let first_pop_value = stack.pop();
        let len_after_first_pop = stack.len();
        let second_pop_value = stack.pop();
        let len_after_second_pop = stack.len();
        let third_pop_value = stack.pop();
        let len_after_third_pop = stack.len();

        // Assert
        assert_eq!(len_initial, expected_len_initial);
        assert_eq!(len_after_first_push, expected_len_after_first_push);
        assert_eq!(len_after_second_push, expected_len_after_second_push);
        assert_eq!(len_after_third_push, expected_len_after_third_push);
        assert_eq!(first_pop_value, Some(third_value));
        assert_eq!(len_after_first_pop, expected_len_after_first_pop);
        assert_eq!(second_pop_value, Some(second_value));
        assert_eq!(len_after_second_pop, expected_len_after_second_pop);
        assert_eq!(third_pop_value, Some(first_value));
        assert_eq!(len_after_third_pop, expected_len_after_third_pop);
    }

    #[test]
    fn single_element_boundary_transition_after_push_peek_pop() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let pushed_value = 42;
        let expected_len_after_push = 1usize;
        let expected_peek_value = Some(pushed_value);
        let expected_pop_value = Some(pushed_value);
        let expected_len_after_pop = 0usize;
        let expected_is_empty_after_pop = true;

        // Act
        stack.push(pushed_value);
        let len_after_push = stack.len();
        let peek_value_after_push = stack.peek().copied();
        let pop_value = stack.pop();
        let len_after_pop = stack.len();
        let is_empty_after_pop = stack.is_empty();

        // Assert
        assert_eq!(len_after_push, expected_len_after_push);
        assert_eq!(peek_value_after_push, expected_peek_value);
        assert_eq!(pop_value, expected_pop_value);
        assert_eq!(len_after_pop, expected_len_after_pop);
        assert_eq!(is_empty_after_pop, expected_is_empty_after_pop);
    }

    #[test]
    fn peek_is_non_destructive_across_repeated_calls() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_peek_value = Some(third_value);
        let expected_len_after_peek_calls = 3usize;
        let expected_pop_value = Some(third_value);
        let expected_len_after_pop = 2usize;

        // Act
        stack.push(first_value);
        stack.push(second_value);
        stack.push(third_value);
        let first_peek_value = stack.peek().copied();
        let second_peek_value = stack.peek().copied();
        let len_after_peek_calls = stack.len();
        let pop_value_after_peeks = stack.pop();
        let len_after_pop = stack.len();

        // Assert
        assert_eq!(first_peek_value, expected_peek_value);
        assert_eq!(second_peek_value, expected_peek_value);
        assert_eq!(len_after_peek_calls, expected_len_after_peek_calls);
        assert_eq!(pop_value_after_peeks, expected_pop_value);
        assert_eq!(len_after_pop, expected_len_after_pop);
    }

    #[test]
    fn string_stack_supports_push_peek_and_pop() {
        // Arrange
        let mut stack: Stack<String> = Stack::new();
        let first_value = String::from("alpha");
        let second_value = String::from("beta");
        let expected_peek_value = Some(second_value.clone());
        let expected_pop_value = Some(second_value.clone());
        let expected_len_after_pop = 1usize;
        let expected_remaining_peek_value = Some(first_value.clone());

        // Act
        stack.push(first_value.clone());
        stack.push(second_value.clone());
        let peek_value = stack.peek().cloned();
        let pop_value = stack.pop();
        let len_after_pop = stack.len();
        let remaining_peek_value = stack.peek().cloned();

        // Assert
        assert_eq!(peek_value, expected_peek_value);
        assert_eq!(pop_value, expected_pop_value);
        assert_eq!(len_after_pop, expected_len_after_pop);
        assert_eq!(remaining_peek_value, expected_remaining_peek_value);
    }

    #[test]
    fn peek_reference_points_to_last_pushed_value() {
        // Arrange
        let mut stack: Stack<String> = Stack::new();
        let first_value = String::from("first");
        let second_value = String::from("second");
        let expected_value = second_value.clone();

        // Act
        stack.push(first_value);
        stack.push(second_value);
        let actual_value = stack.peek().unwrap();

        // Assert
        assert_eq!(actual_value, &expected_value);
    }

    #[test]
    fn pop_returns_x_plus_one_then_x_then_x_minus_one() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let x_value = 100;
        let x_minus_one = x_value - 1;
        let x_plus_one = x_value + 1;
        let expected_first_pop = Some(x_plus_one);
        let expected_second_pop = Some(x_value);
        let expected_third_pop = Some(x_minus_one);
        let expected_fourth_pop: Option<i32> = None;

        // Act
        stack.push(x_minus_one);
        stack.push(x_value);
        stack.push(x_plus_one);

        let first_pop_value = stack.pop();
        let second_pop_value = stack.pop();
        let third_pop_value = stack.pop();
        let fourth_pop_value = stack.pop();

        // Assert
        assert_eq!(first_pop_value, expected_first_pop);
        assert_eq!(second_pop_value, expected_second_pop);
        assert_eq!(third_pop_value, expected_third_pop);
        assert_eq!(fourth_pop_value, expected_fourth_pop);
    }

    #[test]
    fn lifo_is_kept_when_push_and_pop_are_interleaved() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let first_value = 1;
        let second_value = 2;
        let third_value = 3;
        let expected_first_pop = Some(second_value);
        let expected_second_pop = Some(third_value);
        let expected_third_pop = Some(first_value);
        let expected_fourth_pop: Option<i32> = None;

        // Act
        stack.push(first_value);
        stack.push(second_value);
        let first_pop_value = stack.pop();
        stack.push(third_value);
        let second_pop_value = stack.pop();
        let third_pop_value = stack.pop();
        let fourth_pop_value = stack.pop();

        // Assert
        assert_eq!(first_pop_value, expected_first_pop);
        assert_eq!(second_pop_value, expected_second_pop);
        assert_eq!(third_pop_value, expected_third_pop);
        assert_eq!(fourth_pop_value, expected_fourth_pop);
    }

    #[test]
    fn pop_returns_none_when_called_after_removing_all_elements() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let first_value = 5;
        let second_value = 6;
        let expected_first_pop = Some(second_value);
        let expected_second_pop = Some(first_value);
        let expected_third_pop: Option<i32> = None;
        let expected_len_after_third_pop = 0usize;
        let expected_is_empty_after_third_pop = true;

        // Act
        stack.push(first_value);
        stack.push(second_value);
        let first_pop_value = stack.pop();
        let second_pop_value = stack.pop();
        let third_pop_value = stack.pop();
        let len_after_third_pop = stack.len();
        let is_empty_after_third_pop = stack.is_empty();

        // Assert
        assert_eq!(first_pop_value, expected_first_pop);
        assert_eq!(second_pop_value, expected_second_pop);
        assert_eq!(third_pop_value, expected_third_pop);
        assert_eq!(len_after_third_pop, expected_len_after_third_pop);
        assert_eq!(is_empty_after_third_pop, expected_is_empty_after_third_pop);
    }

    #[test]
    fn push_makes_stack_non_empty() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let pushed_value = 1;
        let expected_is_empty = false;

        // Act
        stack.push(pushed_value);
        let actual_is_empty = stack.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn reuse_after_popping_all_elements_keeps_lifo_order() {
        // Arrange
        let mut stack: Stack<i32> = Stack::new();
        let first_value = 1;
        let second_value = 2;
        let reused_value = 3;
        let expected_pop_after_reuse = Some(reused_value);
        let expected_final_pop = None;

        // Act
        stack.push(first_value);
        stack.push(second_value);
        let first_pop = stack.pop();
        let second_pop = stack.pop();
        stack.push(reused_value);
        let actual_pop_after_reuse = stack.pop();
        let actual_final_pop = stack.pop();

        // Assert
        assert_eq!(first_pop, Some(second_value));
        assert_eq!(second_pop, Some(first_value));
        assert_eq!(actual_pop_after_reuse, expected_pop_after_reuse);
        assert_eq!(actual_final_pop, expected_final_pop);
    }
}
