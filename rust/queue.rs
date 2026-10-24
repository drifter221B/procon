mod queue {
    pub struct Queue<T> {
        size: usize,
        queue: Vec<T>,
        pos: usize,
        capacity: usize,
    }

    impl<T: Clone> Queue<T> {

        pub fn new() -> Self {
            Self {
                size: 0,
                queue: vec![],
                pos: 0,
                capacity: (1 << 16) - 1,
            }
        }

        fn shrink(&mut self) {
            self.queue = self.queue.split_off(self.pos);
            self.pos = 0;
        }

        fn resize(&mut self) {
            if self.capacity / 2 < self.pos {
                self.shrink();
            }

            if self.size == self.capacity {
                self.capacity *= 2;
            }
        }

        pub fn is_empty(&self) -> bool {
            self.size == 0
        }

        pub fn len(&self) -> usize {
            self.size
        }

        pub fn front(&self) -> Option<&T> {
            self.queue.get(self.pos)
        }

        pub fn push(&mut self, data: T) {
            self.size += 1;
            self.queue.push(data);
            self.resize();
        }

        pub fn pop(&mut self) -> Option<T> {
            if let Some(data) = self.queue.get(self.pos) {
                self.size -= 1;
                self.pos += 1;
                Some(data.clone())
            } else {
                None
            }
        }

        pub fn reverse(&mut self) {
            self.shrink();
            self.queue.reverse();
        }
    }

}

#[cfg(test)]
mod tests {

    use super::queue::Queue;

    const INITIAL_CAPACITY: usize = (1 << 16) - 1;

    fn create_queue_from_values(values: &[i32]) -> Queue<i32> {
        let mut queue = Queue::new();

        for value in values {
            queue.push(*value);
        }

        queue
    }

    fn create_queue_with_sequential_values(element_count: usize) -> Queue<i32> {
        let mut queue = Queue::new();

        for index in 0..element_count {
            queue.push(index as i32);
        }

        queue
    }

    fn pop_all_values(queue: &mut Queue<i32>) -> Vec<i32> {
        let mut popped_values = vec![];

        loop {
            let popped_value = queue.pop();

            if let Some(value) = popped_value {
                popped_values.push(value);
            } else {
                break;
            }
        }

        popped_values
    }

    #[test]
    fn new_returns_empty_queue_with_no_front_and_no_pop_value() {
        // Arrange
        let queue: Queue<i32> = Queue::new();
        let expected_len = 0usize;
        let expected_is_empty = true;
        let expected_front = None;
        let expected_pop = None;

        // Act
        let actual_len = queue.len();
        let actual_is_empty = queue.is_empty();
        let actual_front = queue.front().cloned();
        let mut queue_for_pop = queue;
        let actual_pop = queue_for_pop.pop();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_pop, expected_pop);
    }

    #[test]
    fn front_on_empty_queue_returns_none_repeatedly_without_state_change() {
        // Arrange
        let queue: Queue<i32> = Queue::new();
        let expected_first_front = None;
        let expected_second_front = None;
        let expected_len = 0usize;

        // Act
        let actual_first_front = queue.front().cloned();
        let actual_second_front = queue.front().cloned();
        let actual_len = queue.len();

        // Assert
        assert_eq!(actual_first_front, expected_first_front);
        assert_eq!(actual_second_front, expected_second_front);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn front_called_multiple_times_keeps_head_and_len_unchanged() {
        // Arrange
        let pushed_values = vec![8, 9];
        let queue = create_queue_from_values(&pushed_values);
        let expected_first_front = Some(8);
        let expected_second_front = Some(8);
        let expected_len = 2usize;

        // Act
        let actual_first_front = queue.front().cloned();
        let actual_second_front = queue.front().cloned();
        let actual_len = queue.len();

        // Assert
        assert_eq!(actual_first_front, expected_first_front);
        assert_eq!(actual_second_front, expected_second_front);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn front_points_to_next_value_after_single_pop() {
        // Arrange
        let pushed_values = vec![11, 12];
        let mut queue = create_queue_from_values(&pushed_values);
        let expected_popped_value = Some(11);
        let expected_front = Some(12);

        // Act
        let actual_popped_value = queue.pop();
        let actual_front = queue.front().cloned();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_front, expected_front);
    }

    #[test]
    fn push_single_value_sets_front_len_and_non_empty_state() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let pushed_value = 10;
        let expected_front = Some(pushed_value);
        let expected_len = 1usize;
        let expected_is_empty = false;

        // Act
        queue.push(pushed_value);
        let actual_front = queue.front().cloned();
        let actual_len = queue.len();
        let actual_is_empty = queue.is_empty();

        // Assert
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn push_after_reverse_appends_new_value_to_tail() {
        // Arrange
        let pushed_values = vec![1, 2, 3];
        let mut queue = create_queue_from_values(&pushed_values);
        let pushed_after_reverse = 4;
        let expected_popped_values = vec![3, 2, 1, pushed_after_reverse];

        // Act
        queue.reverse();
        queue.push(pushed_after_reverse);
        let actual_popped_values = pop_all_values(&mut queue);

        // Assert
        assert_eq!(actual_popped_values, expected_popped_values);
    }

    #[test]
    fn push_after_popping_up_to_half_capacity_keeps_front_value_consistent() {
        // Arrange
        let pop_count = INITIAL_CAPACITY / 2;
        let initial_element_count = pop_count + 3;
        let pushed_after_pop = 1_000_000;
        let mut queue = create_queue_with_sequential_values(initial_element_count);
        let expected_front = Some(pop_count as i32);
        let expected_remaining_values = vec![pop_count as i32, (pop_count + 1) as i32, (pop_count + 2) as i32, pushed_after_pop];

        // Act
        for _ in 0..pop_count {
            let _ = queue.pop();
        }
        queue.push(pushed_after_pop);
        let actual_front = queue.front().cloned();
        let actual_remaining_values = pop_all_values(&mut queue);

        // Assert
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    fn push_after_popping_beyond_half_capacity_keeps_order_and_length_consistent() {
        // Arrange
        let pop_count = (INITIAL_CAPACITY / 2) + 1;
        let initial_element_count = pop_count + 3;
        let pushed_after_pop = 2_000_000;
        let mut queue = create_queue_with_sequential_values(initial_element_count);
        let expected_remaining_values = vec![pop_count as i32, (pop_count + 1) as i32, (pop_count + 2) as i32, pushed_after_pop];
        let expected_len_after_push = expected_remaining_values.len();
        let expected_len_after_all_pop = 0usize;

        // Act
        for _ in 0..pop_count {
            let _ = queue.pop();
        }
        queue.push(pushed_after_pop);
        let actual_len_after_push = queue.len();
        let actual_remaining_values = pop_all_values(&mut queue);
        let actual_len_after_all_pop = queue.len();

        // Assert
        assert_eq!(actual_remaining_values, expected_remaining_values);
        assert_eq!(actual_len_after_push, expected_len_after_push);
        assert_eq!(actual_len_after_all_pop, expected_len_after_all_pop);
    }

    #[test]
    fn push_beyond_capacity_boundary_keeps_order_without_panic() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let expected_popped_values: Vec<i32> = (0..=INITIAL_CAPACITY as i32).collect();

        // Act
        for value in 0..INITIAL_CAPACITY {
            queue.push(value as i32);
        }
        queue.push(INITIAL_CAPACITY as i32);
        let actual_popped_values = pop_all_values(&mut queue);

        // Assert
        assert_eq!(actual_popped_values, expected_popped_values);
    }

    #[test]
    fn push_and_pop_large_number_of_values_preserves_order_and_final_empty_state() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let element_count = 100_000usize;
        let values = (0..element_count)
            .map(|value| value as i32)
            .collect::<Vec<_>>();
        let expected_popped_values = values.iter().copied().map(Some).collect::<Vec<_>>();
        let expected_final_pop = None;
        let expected_final_len = 0usize;
        let expected_final_is_empty = true;

        // Act
        for value in values {
            queue.push(value);
        }
        let actual_popped_values = (0..element_count).map(|_| queue.pop()).collect::<Vec<_>>();
        let actual_final_pop = queue.pop();
        let actual_final_len = queue.len();
        let actual_final_is_empty = queue.is_empty();

        // Assert
        assert_eq!(actual_popped_values, expected_popped_values);
        assert_eq!(actual_final_pop, expected_final_pop);
        assert_eq!(actual_final_len, expected_final_len);
        assert_eq!(actual_final_is_empty, expected_final_is_empty);
    }

    #[test]
    fn pop_on_empty_queue_returns_none_repeatedly_and_keeps_len_zero() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let expected_first_pop = None;
        let expected_second_pop = None;
        let expected_len = 0usize;

        // Act
        let actual_first_pop = queue.pop();
        let actual_second_pop = queue.pop();
        let actual_len = queue.len();

        // Assert
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_after_single_push_returns_value_and_restores_empty_state() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let pushed_value = 20;
        let expected_popped_value = Some(pushed_value);
        let expected_len = 0usize;
        let expected_is_empty = true;

        // Act
        queue.push(pushed_value);
        let actual_popped_value = queue.pop();
        let actual_len = queue.len();
        let actual_is_empty = queue.is_empty();

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn pop_returns_values_in_fifo_order_after_multiple_pushes() {
        // Arrange
        let pushed_values = vec![1, 2, 3];
        let mut queue = create_queue_from_values(&pushed_values);
        let expected_first_pop = Some(1);
        let expected_second_pop = Some(2);
        let expected_third_pop = Some(3);
        let expected_fourth_pop = None;

        // Act
        let actual_first_pop = queue.pop();
        let actual_second_pop = queue.pop();
        let actual_third_pop = queue.pop();
        let actual_fourth_pop = queue.pop();

        // Assert
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
        assert_eq!(actual_fourth_pop, expected_fourth_pop);
    }

    #[test]
    fn pop_preserves_duplicate_counts_after_duplicate_pushes() {
        // Arrange
        let pushed_values = vec![4, 4, 5];
        let mut queue = create_queue_from_values(&pushed_values);
        let expected_first_pop = Some(4);
        let expected_second_pop = Some(4);
        let expected_third_pop = Some(5);

        // Act
        let actual_first_pop = queue.pop();
        let actual_second_pop = queue.pop();
        let actual_third_pop = queue.pop();

        // Assert
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
    }

    #[test]
    fn reverse_on_empty_queue_keeps_queue_empty_without_panic() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let expected_len = 0usize;
        let expected_front = None;
        let expected_pop = None;

        // Act
        queue.reverse();
        let actual_len = queue.len();
        let actual_front = queue.front().cloned();
        let actual_pop = queue.pop();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_pop, expected_pop);
    }

    #[test]
    fn reverse_on_single_element_queue_keeps_same_order() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let inserted_value = 40;
        let expected_front = Some(inserted_value);
        let expected_pop = Some(inserted_value);

        // Act
        queue.push(inserted_value);
        queue.reverse();
        let actual_front = queue.front().cloned();
        let actual_pop = queue.pop();

        // Assert
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_pop, expected_pop);
    }

    #[test]
    fn reverse_on_multiple_elements_reverses_remaining_order() {
        // Arrange
        let pushed_values = vec![1, 2, 3];
        let mut queue = create_queue_from_values(&pushed_values);
        let expected_popped_values = vec![3, 2, 1];

        // Act
        queue.reverse();
        let actual_popped_values = pop_all_values(&mut queue);

        // Assert
        assert_eq!(actual_popped_values, expected_popped_values);
    }

    #[test]
    fn reverse_after_partial_pop_does_not_restore_already_popped_values() {
        // Arrange
        let pushed_values = vec![1, 2, 3, 4];
        let mut queue = create_queue_from_values(&pushed_values);
        let expected_popped_before_reverse = Some(1);
        let expected_popped_after_reverse = vec![4, 3, 2];

        // Act
        let actual_popped_before_reverse = queue.pop();
        queue.reverse();
        let actual_popped_after_reverse = pop_all_values(&mut queue);

        // Assert
        assert_eq!(actual_popped_before_reverse, expected_popped_before_reverse);
        assert_eq!(actual_popped_after_reverse, expected_popped_after_reverse);
    }

    #[test]
    fn reverse_called_twice_restores_original_order() {
        // Arrange
        let pushed_values = vec![5, 6, 7];
        let mut queue = create_queue_from_values(&pushed_values);
        let expected_popped_values = vec![5, 6, 7];

        // Act
        queue.reverse();
        queue.reverse();
        let actual_popped_values = pop_all_values(&mut queue);

        // Assert
        assert_eq!(actual_popped_values, expected_popped_values);
    }

    #[test]
    fn alternating_push_and_pop_operations_keep_fifo_order() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let expected_first_pop = Some(1);
        let expected_second_pop = Some(2);
        let expected_third_pop = Some(3);
        let expected_fourth_pop = Some(4);
        let expected_final_pop = None;

        let push_value = 1;
        let push_value_2 = 2;
        let push_value_3 = 3;
        let push_value_4 = 4;

        // Act
        queue.push(push_value);
        queue.push(push_value_2);
        let actual_first_pop = queue.pop();
        queue.push(push_value_3);
        let actual_second_pop = queue.pop();
        queue.push(push_value_4);
        let actual_third_pop = queue.pop();
        let actual_fourth_pop = queue.pop();
        let actual_final_pop = queue.pop();

        // Assert
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
        assert_eq!(actual_fourth_pop, expected_fourth_pop);
        assert_eq!(actual_final_pop, expected_final_pop);
    }

    #[test]
    fn queue_can_be_reused_after_all_values_are_popped() {
        // Arrange
        let mut queue: Queue<i32> = Queue::new();
        let first_inserted_value = 30;
        let second_inserted_value = 31;
        let reinserted_value = 99;
        let expected_front_after_reuse = Some(reinserted_value);
        let expected_pop_after_reuse = Some(reinserted_value);
        let expected_len_after_reuse_pop = 0usize;
        let expected_is_empty_after_reuse_pop = true;

        // Act
        queue.push(first_inserted_value);
        queue.push(second_inserted_value);
        let _ = queue.pop();
        let _ = queue.pop();
        queue.push(reinserted_value);
        let actual_front_after_reuse = queue.front().cloned();
        let actual_pop_after_reuse = queue.pop();
        let actual_len_after_reuse_pop = queue.len();
        let actual_is_empty_after_reuse_pop = queue.is_empty();

        // Assert
        assert_eq!(actual_front_after_reuse, expected_front_after_reuse);
        assert_eq!(actual_pop_after_reuse, expected_pop_after_reuse);
        assert_eq!(actual_len_after_reuse_pop, expected_len_after_reuse_pop);
        assert_eq!(actual_is_empty_after_reuse_pop, expected_is_empty_after_reuse_pop);
    }

    #[test]
    fn queue_supports_clone_required_type_string_for_front_and_pop() {
        // Arrange
        let mut queue: Queue<String> = Queue::new();
        let first_value = String::from("alpha");
        let second_value = String::from("beta");
        let expected_front_before_pop = Some(first_value.clone());
        let expected_first_pop = Some(first_value.clone());
        let expected_second_pop = Some(second_value.clone());

        // Act
        queue.push(first_value);
        queue.push(second_value);
        let actual_front_before_pop = queue.front().cloned();
        let actual_first_pop = queue.pop();
        let actual_second_pop = queue.pop();

        // Assert
        assert_eq!(actual_front_before_pop, expected_front_before_pop);
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
    }
}
