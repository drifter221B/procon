use std::collections::{BTreeMap, HashSet};

/// スタックに対して最小値・最大値の取り出しも両立させる。
pub struct MinMaxStack<T>
where
    T: Copy + Ord,
{
    stack: Vec<T>,
    positions: BTreeMap<T, Vec<usize>>,
    deleted_indices: HashSet<usize>,
}

impl<T> MinMaxStack<T>
where
    T: Copy + Ord,
{
    pub fn new() -> Self {
        Self {
            stack: Vec::new(),
            positions: BTreeMap::new(),
            deleted_indices: HashSet::new(),
        }
    }

    pub fn push(&mut self, value: T) {
        let index = self.stack.len();
        self.stack.push(value);
        self.positions.entry(value).or_default().push(index);
    }

    pub fn pop(&mut self) -> Option<T> {
        self.clean();

        if let Some(value) = self.stack.pop() {
            if let Some(indices) = self.positions.get_mut(&value) {
                indices.pop();
                if indices.is_empty() {
                    self.positions.remove(&value);
                }
            }
            Some(value)
        } else {
            None
        }
    }

    pub fn top(&mut self) -> Option<T> {
        self.clean();
        self.stack.last().copied()
    }

    pub fn first(&self) -> Option<&T> {
        self.positions.first_key_value().map(|entry| entry.0)
    }

    pub fn last(&self) -> Option<&T> {
        self.positions.last_key_value().map(|entry| entry.0)
    }

    pub fn pop_first(&mut self) -> Option<T> {
        let (&value, indices) = self.positions.first_key_value()?;
        if let Some(&index) = indices.last() {
            self.deleted_indices.insert(index);
        }
        self.remove_position_entry(value);
        self.clean();
        Some(value)
    }

    pub fn pop_last(&mut self) -> Option<T> {
        let (&value, indices) = self.positions.last_key_value()?;
        if let Some(&index) = indices.last() {
            self.deleted_indices.insert(index);
        }
        self.remove_position_entry(value);
        self.clean();
        Some(value)
    }

    fn remove_position_entry(&mut self, value: T) {
        if let Some(indices) = self.positions.get_mut(&value) {
            indices.pop();
            if indices.is_empty() {
                self.positions.remove(&value);
            }
        }
    }

    fn clean(&mut self) {
        while let Some(index) = self.stack.len().checked_sub(1) {
            if self.deleted_indices.contains(&index) {
                if self.stack.pop().is_some() {
                    self.deleted_indices.remove(&index);
                }
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::MinMaxStack;

    #[test]
    fn pop_first_removes_only_one_of_duplicate_minimums() {
        // Arrange
        let mut stack = MinMaxStack::new();
        stack.push(1);
        stack.push(1);
        stack.push(3);
        let expected_values = (Some(1), Some(&1));

        // Act
        let actual_pop = stack.pop_first();
        let actual_first = stack.first();

        // Assert
        assert_eq!((actual_pop, actual_first), expected_values);
    }

    #[test]
    fn pop_last_removes_only_one_of_duplicate_maximums() {
        // Arrange
        let mut stack = MinMaxStack::new();
        stack.push(1);
        stack.push(3);
        stack.push(3);
        let expected_values = (Some(3), Some(&3));

        // Act
        let actual_pop = stack.pop_last();
        let actual_last = stack.last();

        // Assert
        assert_eq!((actual_pop, actual_last), expected_values);
    }

    #[test]
    fn single_element_pop_last_returns_to_empty_state() {
        // Arrange
        let mut stack = MinMaxStack::new();
        stack.push(7);
        let expected_values = (Some(7), None, None, None);

        // Act
        let actual_pop = stack.pop_last();
        let actual_top = stack.top();
        let actual_first = stack.first();
        let actual_last = stack.last();

        // Assert
        assert_eq!(
            (actual_pop, actual_top, actual_first, actual_last),
            expected_values
        );
    }

    #[test]
    fn pushes_and_pops_while_tracking_first_last() {
        // Arrange
        let mut stack = MinMaxStack::new();
        stack.push(3);
        stack.push(1);
        stack.push(2);
        let expected_values = (Some(1), Some(3), Some(1), Some(2), Some(3), Some(2));

        // Act
        let initial_first = stack.first().copied();
        let initial_last = stack.last().copied();
        let removed_first = stack.pop_first();
        let remaining_first = stack.first().copied();
        let removed_last = stack.pop_last();
        let remaining_top = stack.top();

        // Assert
        assert_eq!(
            (
                initial_first,
                initial_last,
                removed_first,
                remaining_first,
                removed_last,
                remaining_top
            ),
            expected_values
        );
    }

    #[test]
    fn supports_duplicate_values_and_deferred_deletions() {
        // Arrange
        let mut stack = MinMaxStack::new();
        stack.push(2);
        stack.push(2);
        stack.push(1);
        let expected_values = (Some(1), Some(2), Some(2), None);

        // Act
        let removed_first = stack.pop_first();
        let first_pop = stack.pop();
        let second_pop = stack.pop();
        let final_pop = stack.pop();

        // Assert
        assert_eq!(
            (removed_first, first_pop, second_pop, final_pop),
            expected_values
        );
    }

    #[test]
    fn deferred_deletions_keep_top_and_pop_consistent() {
        // Arrange
        let mut stack = MinMaxStack::new();
        stack.push(3);
        stack.push(1);
        stack.push(2);
        stack.pop_first();
        let expected_values = (Some(2), Some(2), Some(3));

        // Act
        let actual_top = stack.top();
        let actual_pop_first = stack.pop();
        let actual_pop_second = stack.pop();

        // Assert
        assert_eq!(
            (actual_top, actual_pop_first, actual_pop_second),
            expected_values
        );
    }

    #[test]
    fn handles_empty_stack_operations() {
        // Arrange
        let mut stack: MinMaxStack<i32> = MinMaxStack::new();
        let expected_values = (None, None, None, None, None, None);

        // Act
        let actual_values = (
            stack.pop(),
            stack.pop_first(),
            stack.pop_last(),
            stack.top(),
            stack.first(),
            stack.last(),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
