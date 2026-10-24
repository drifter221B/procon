pub struct RingBuffer<T> {
    storage: Vec<Option<T>>,
    head_index: usize,
    element_count: usize,
    capacity: usize,
}

impl<T> RingBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        assert!(0 < capacity, "capacity must be greater than zero");

        let mut storage = Vec::with_capacity(capacity);
        storage.resize_with(capacity, || None);

        Self {
            storage,
            head_index: 0,
            element_count: 0,
            capacity,
        }
    }

    pub fn capacity(&self) -> usize {
        self.capacity
    }

    pub fn len(&self) -> usize {
        self.element_count
    }

    pub fn is_empty(&self) -> bool {
        self.element_count == 0
    }

    pub fn is_full(&self) -> bool {
        self.element_count == self.capacity
    }

    pub fn front(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }

        self.storage[self.head_index].as_ref()
    }

    pub fn back(&self) -> Option<&T> {
        if self.is_empty() {
            return None;
        }

        let back_index = self.calculate_back_index();

        self.storage[back_index].as_ref()
    }

    pub fn push_back(&mut self, value: T) -> Result<(), T> {
        if self.is_full() {
            return Err(value);
        }

        let tail_index = self.calculate_tail_index();
        self.storage[tail_index] = Some(value);
        self.element_count += 1;

        Ok(())
    }

    pub fn pop_front(&mut self) -> Option<T> {
        if self.is_empty() {
            return None;
        }

        let popped_value = self.storage[self.head_index].take();
        self.head_index = (self.head_index + 1) % self.capacity;
        self.element_count -= 1;

        popped_value
    }

    pub fn clear(&mut self) {
        for slot in self.storage.iter_mut() {
            *slot = None;
        }

        self.head_index = 0;
        self.element_count = 0;
    }

    fn calculate_tail_index(&self) -> usize {
        (self.head_index + self.element_count) % self.capacity
    }

    fn calculate_back_index(&self) -> usize {
        (self.head_index + self.element_count - 1) % self.capacity
    }
}

#[cfg(test)]
mod tests {
    use super::RingBuffer;

    #[test]
    fn new_creates_empty_buffer_with_requested_capacity() {
        // Arrange
        let capacity_value = 3usize;
        let expected_len = 0usize;
        let expected_capacity = capacity_value;
        let expected_is_empty = true;
        let expected_is_full = false;
        let expected_front_value: Option<i32> = None;
        let expected_back_value: Option<i32> = None;

        // Act
        let ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let actual_len = ring_buffer.len();
        let actual_capacity = ring_buffer.capacity();
        let actual_is_empty = ring_buffer.is_empty();
        let actual_is_full = ring_buffer.is_full();
        let actual_front_value = ring_buffer.front().copied();
        let actual_back_value = ring_buffer.back().copied();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_capacity, expected_capacity);
        assert_eq!(actual_is_empty, expected_is_empty);
        assert_eq!(actual_is_full, expected_is_full);
        assert_eq!(actual_front_value, expected_front_value);
        assert_eq!(actual_back_value, expected_back_value);
    }

    #[test]
    fn new_panics_when_capacity_is_lower_bound_zero() {
        // Arrange
        let invalid_capacity = 0usize;

        // Act
        let actual_result = std::panic::catch_unwind(|| RingBuffer::<i32>::new(invalid_capacity));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn front_and_back_are_correct_after_wraparound() {
        // Arrange
        let capacity_value = 3usize;
        let first_value = 1i32;
        let second_value = 2i32;
        let third_value = 3i32;
        let wrapped_value = 4i32;
        let expected_front_value = Some(second_value);
        let expected_back_value = Some(wrapped_value);

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        ring_buffer.push_back(first_value).unwrap();
        ring_buffer.push_back(second_value).unwrap();
        ring_buffer.push_back(third_value).unwrap();
        let actual_pop = ring_buffer.pop_front();
        ring_buffer.push_back(wrapped_value).unwrap();
        let actual_front_value = ring_buffer.front().copied();
        let actual_back_value = ring_buffer.back().copied();

        // Assert
        assert_eq!(actual_pop, Some(first_value));
        assert_eq!(actual_front_value, expected_front_value);
        assert_eq!(actual_back_value, expected_back_value);
    }

    #[test]
    fn capacity_boundary_x_minus_one_x_x_plus_one_for_push_back() {
        // Arrange
        let x_capacity = 3usize;
        let first_value = 10i32;
        let second_value = 20i32;
        let third_value = 30i32;
        let fourth_value = 40i32;
        let expected_first_push = Ok(());
        let expected_second_push = Ok(());
        let expected_third_push = Ok(());
        let expected_fourth_push = Err(fourth_value);
        let expected_len_after_pushes = x_capacity;
        let expected_is_full_after_pushes = true;

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(x_capacity);
        let actual_first_push = ring_buffer.push_back(first_value);
        let actual_second_push = ring_buffer.push_back(second_value);
        let actual_third_push = ring_buffer.push_back(third_value);
        let actual_fourth_push = ring_buffer.push_back(fourth_value);
        let actual_len_after_pushes = ring_buffer.len();
        let actual_is_full_after_pushes = ring_buffer.is_full();

        // Assert
        assert_eq!(actual_first_push, expected_first_push);
        assert_eq!(actual_second_push, expected_second_push);
        assert_eq!(actual_third_push, expected_third_push);
        assert_eq!(actual_fourth_push, expected_fourth_push);
        assert_eq!(actual_len_after_pushes, expected_len_after_pushes);
        assert_eq!(actual_is_full_after_pushes, expected_is_full_after_pushes);
    }

    #[test]
    fn push_back_works_after_wraparound_when_one_slot_is_freed() {
        // Arrange
        let capacity_value = 3usize;
        let first_value = 1i32;
        let second_value = 2i32;
        let third_value = 3i32;
        let wrapped_value = 4i32;
        let expected_first_pop = Some(first_value);
        let expected_second_pop = Some(second_value);
        let expected_third_pop = Some(third_value);
        let expected_fourth_pop = Some(wrapped_value);
        let expected_fifth_pop: Option<i32> = None;
        let expected_push_result: Result<(), i32> = Ok(());

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let first_push_result = ring_buffer.push_back(first_value);
        let second_push_result = ring_buffer.push_back(second_value);
        let third_push_result = ring_buffer.push_back(third_value);
        let actual_first_pop = ring_buffer.pop_front();
        let wrapped_push_result = ring_buffer.push_back(wrapped_value);
        let actual_second_pop = ring_buffer.pop_front();
        let actual_third_pop = ring_buffer.pop_front();
        let actual_fourth_pop = ring_buffer.pop_front();
        let actual_fifth_pop = ring_buffer.pop_front();

        // Assert
        assert_eq!(first_push_result, expected_push_result);
        assert_eq!(second_push_result, expected_push_result);
        assert_eq!(third_push_result, expected_push_result);
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(wrapped_push_result, expected_push_result);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
        assert_eq!(actual_fourth_pop, expected_fourth_pop);
        assert_eq!(actual_fifth_pop, expected_fifth_pop);
    }

    #[test]
    fn push_back_on_full_buffer_returns_err_and_keeps_existing_state() {
        // Arrange
        let capacity_value = 2usize;
        let first_value = 1i32;
        let second_value = 2i32;
        let rejected_value = 3i32;
        let expected_push_result = Err(rejected_value);
        let expected_front_value = Some(first_value);
        let expected_back_value = Some(second_value);
        let expected_len = capacity_value;

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let first_push_result = ring_buffer.push_back(first_value);
        let second_push_result = ring_buffer.push_back(second_value);
        let actual_push_result = ring_buffer.push_back(rejected_value);
        let actual_front_value = ring_buffer.front().copied();
        let actual_back_value = ring_buffer.back().copied();
        let actual_len = ring_buffer.len();

        // Assert
        assert_eq!(first_push_result, Ok(()));
        assert_eq!(second_push_result, Ok(()));
        assert_eq!(actual_push_result, expected_push_result);
        assert_eq!(actual_front_value, expected_front_value);
        assert_eq!(actual_back_value, expected_back_value);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn pop_front_returns_fifo_order_for_multiple_elements() {
        // Arrange
        let capacity_value = 5usize;
        let x_value = 100i32;
        let x_minus_one = x_value - 1;
        let x_plus_one = x_value + 1;
        let expected_first_pop = Some(x_minus_one);
        let expected_second_pop = Some(x_value);
        let expected_third_pop = Some(x_plus_one);
        let expected_fourth_pop: Option<i32> = None;

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let first_push_result = ring_buffer.push_back(x_minus_one);
        let second_push_result = ring_buffer.push_back(x_value);
        let third_push_result = ring_buffer.push_back(x_plus_one);
        let actual_first_pop = ring_buffer.pop_front();
        let actual_second_pop = ring_buffer.pop_front();
        let actual_third_pop = ring_buffer.pop_front();
        let actual_fourth_pop = ring_buffer.pop_front();

        // Assert
        assert_eq!(first_push_result, Ok(()));
        assert_eq!(second_push_result, Ok(()));
        assert_eq!(third_push_result, Ok(()));
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
        assert_eq!(actual_third_pop, expected_third_pop);
        assert_eq!(actual_fourth_pop, expected_fourth_pop);
    }

    #[test]
    fn clear_removes_all_elements_and_allows_reuse() {
        // Arrange
        let capacity_value = 4usize;
        let first_value = 11i32;
        let second_value = 22i32;
        let reused_value = 33i32;
        let expected_len_after_clear = 0usize;
        let expected_is_empty_after_clear = true;
        let expected_front_after_clear: Option<i32> = None;
        let expected_back_after_clear: Option<i32> = None;
        let expected_pop_after_clear: Option<i32> = None;
        let expected_pop_after_reuse = Some(reused_value);
        let expected_push_result: Result<(), i32> = Ok(());

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let first_push_result = ring_buffer.push_back(first_value);
        let second_push_result = ring_buffer.push_back(second_value);
        ring_buffer.clear();
        let actual_len_after_clear = ring_buffer.len();
        let actual_is_empty_after_clear = ring_buffer.is_empty();
        let actual_front_after_clear = ring_buffer.front().copied();
        let actual_back_after_clear = ring_buffer.back().copied();
        let actual_pop_after_clear = ring_buffer.pop_front();
        let reuse_push_result = ring_buffer.push_back(reused_value);
        let actual_pop_after_reuse = ring_buffer.pop_front();

        // Assert
        assert_eq!(first_push_result, expected_push_result);
        assert_eq!(second_push_result, expected_push_result);
        assert_eq!(actual_len_after_clear, expected_len_after_clear);
        assert_eq!(actual_is_empty_after_clear, expected_is_empty_after_clear);
        assert_eq!(actual_front_after_clear, expected_front_after_clear);
        assert_eq!(actual_back_after_clear, expected_back_after_clear);
        assert_eq!(actual_pop_after_clear, expected_pop_after_clear);
        assert_eq!(reuse_push_result, expected_push_result);
        assert_eq!(actual_pop_after_reuse, expected_pop_after_reuse);
    }

    #[test]
    fn clear_on_empty_buffer_keeps_empty_state() {
        // Arrange
        let capacity_value = 2usize;
        let expected_len = 0usize;
        let expected_is_empty = true;

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        ring_buffer.clear();
        let actual_len = ring_buffer.len();
        let actual_is_empty = ring_buffer.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn clear_keeps_capacity_unchanged() {
        // Arrange
        let capacity_value = 4usize;
        let pushed_value = 1i32;
        let expected_capacity = capacity_value;
        let expected_push_result: Result<(), i32> = Ok(());

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let actual_push_result = ring_buffer.push_back(pushed_value);
        ring_buffer.clear();
        let actual_capacity = ring_buffer.capacity();

        // Assert
        assert_eq!(actual_capacity, expected_capacity);
        assert_eq!(actual_push_result, expected_push_result);
    }

    #[test]
    fn capacity_one_buffer_can_be_reused_after_pop() {
        // Arrange
        let capacity_value = 1usize;
        let first_value = 10i32;
        let second_value = 20i32;
        let expected_first_pop = Some(first_value);
        let expected_second_pop = Some(second_value);

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let first_push_result = ring_buffer.push_back(first_value);
        let actual_first_pop = ring_buffer.pop_front();
        let second_push_result = ring_buffer.push_back(second_value);
        let actual_second_pop = ring_buffer.pop_front();

        // Assert
        assert_eq!(first_push_result, Ok(()));
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(second_push_result, Ok(()));
        assert_eq!(actual_second_pop, expected_second_pop);
    }

    #[test]
    fn string_buffer_supports_push_front_back_and_pop() {
        // Arrange
        let capacity_value = 3usize;
        let first_value = String::from("alpha");
        let second_value = String::from("beta");
        let expected_front_value = Some(first_value.clone());
        let expected_back_value = Some(second_value.clone());
        let expected_first_pop = Some(first_value.clone());
        let expected_second_pop = Some(second_value.clone());

        // Act
        let mut ring_buffer: RingBuffer<String> = RingBuffer::new(capacity_value);
        let first_push_result = ring_buffer.push_back(first_value.clone());
        let second_push_result = ring_buffer.push_back(second_value.clone());
        let actual_front_value = ring_buffer.front().cloned();
        let actual_back_value = ring_buffer.back().cloned();
        let actual_first_pop = ring_buffer.pop_front();
        let actual_second_pop = ring_buffer.pop_front();

        // Assert
        assert_eq!(first_push_result, Ok(()));
        assert_eq!(second_push_result, Ok(()));
        assert_eq!(actual_front_value, expected_front_value);
        assert_eq!(actual_back_value, expected_back_value);
        assert_eq!(actual_first_pop, expected_first_pop);
        assert_eq!(actual_second_pop, expected_second_pop);
    }

    #[test]
    fn single_element_boundary_transition_empty_to_non_empty_to_empty() {
        // Arrange
        let capacity_value = 2usize;
        let pushed_value = 7i32;
        let expected_front_after_push = Some(pushed_value);
        let expected_back_after_push = Some(pushed_value);
        let expected_len_after_push = 1usize;
        let expected_popped_value = Some(pushed_value);
        let expected_len_after_pop = 0usize;
        let expected_is_empty_after_pop = true;

        // Act
        let mut ring_buffer: RingBuffer<i32> = RingBuffer::new(capacity_value);
        let push_result = ring_buffer.push_back(pushed_value);
        let actual_front_after_push = ring_buffer.front().copied();
        let actual_back_after_push = ring_buffer.back().copied();
        let actual_len_after_push = ring_buffer.len();
        let actual_popped_value = ring_buffer.pop_front();
        let actual_len_after_pop = ring_buffer.len();
        let actual_is_empty_after_pop = ring_buffer.is_empty();

        // Assert
        assert_eq!(push_result, Ok(()));
        assert_eq!(actual_front_after_push, expected_front_after_push);
        assert_eq!(actual_back_after_push, expected_back_after_push);
        assert_eq!(actual_len_after_push, expected_len_after_push);
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_len_after_pop, expected_len_after_pop);
        assert_eq!(actual_is_empty_after_pop, expected_is_empty_after_pop);
    }
}
