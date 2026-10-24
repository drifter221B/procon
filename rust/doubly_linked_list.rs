use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct Node<T: Clone + fmt::Debug + fmt::Display> {
    pub data: T,
    pub next: Option<Rc<RefCell<Node<T>>>>,
    pub prev: Option<Weak<RefCell<Node<T>>>>,
}

impl<T: Clone + fmt::Debug + fmt::Display> Node<T> {
    pub fn new(data: T) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self {
            data,
            next: None,
            prev: None,
        }))
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct Iter<'a, T: Clone + fmt::Debug + fmt::Display> {
    front: Option<Weak<RefCell<Node<T>>>>,
    back: Option<Weak<RefCell<Node<T>>>>,
    remaining: usize,
    marker: PhantomData<&'a DoublyLinkedList<T>>,
}

impl<T: Clone + fmt::Debug + fmt::Display> Iterator for Iter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let front_node = self
            .front
            .as_ref()
            .expect("a nonempty iterator has a front node")
            .upgrade()
            .expect("the borrowed list owns the front node");

        let front_node = front_node.borrow();
        let value = front_node.data.clone();
        self.front = front_node.next.as_ref().map(Rc::downgrade);
        self.remaining -= 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }

    fn count(self) -> usize {
        self.remaining
    }

    fn last(mut self) -> Option<Self::Item> {
        self.next_back()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> DoubleEndedIterator for Iter<'_, T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.remaining == 0 {
            return None;
        }

        let back_node = self
            .back
            .as_ref()
            .expect("a nonempty iterator has a back node")
            .upgrade()
            .expect("the borrowed list owns the back node");

        let back_node = back_node.borrow();
        let value = back_node.data.clone();
        self.back = back_node.prev.clone();
        self.remaining -= 1;
        Some(value)
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> ExactSizeIterator for Iter<'_, T> {}

impl<T: Clone + fmt::Debug + fmt::Display> FusedIterator for Iter<'_, T> {}

pub struct IntoIter<T: Clone + fmt::Debug + fmt::Display> {
    doubly_linked_list: DoublyLinkedList<T>,
}

impl<T: Clone + fmt::Debug + fmt::Display> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.doubly_linked_list.pop_front()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining_count = self.doubly_linked_list.len();
        (remaining_count, Some(remaining_count))
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> DoubleEndedIterator for IntoIter<T> {
    fn next_back(&mut self) -> Option<Self::Item> {
        self.doubly_linked_list.pop_back()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> ExactSizeIterator for IntoIter<T> {}

impl<T: Clone + fmt::Debug + fmt::Display> FusedIterator for IntoIter<T> {}

pub struct DoublyLinkedList<T: Clone + fmt::Debug + fmt::Display> {
    size: usize,
    head: Option<Rc<RefCell<Node<T>>>>,
    tail: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: Clone + fmt::Debug + fmt::Display> fmt::Debug for DoublyLinkedList<T> {
    /// Formats the elements in list order without cloning them.
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut entries = formatter.debug_list();
        let mut current = self.head.clone();

        while let Some(current_node) = current {
            let current_node = current_node.borrow();
            entries.entry(&current_node.data);
            current = current_node.next.clone();
        }

        entries.finish()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> Default for DoublyLinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> IntoIterator for DoublyLinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter {
            doubly_linked_list: self,
        }
    }
}

impl<'a, T: Clone + fmt::Debug + fmt::Display> IntoIterator for &'a DoublyLinkedList<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> DoublyLinkedList<T> {
    pub fn new() -> Self {
        Self {
            size: 0,
            head: None,
            tail: None,
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            front: self.head.as_ref().map(Rc::downgrade),
            back: self.tail.as_ref().map(Rc::downgrade),
            remaining: self.size,
            marker: PhantomData,
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }

    pub fn back(&self) -> Option<Ref<'_, T>> {
        self.tail
            .as_ref()
            .map(|tail_vertex| Ref::map(tail_vertex.borrow(), |vertex| vertex.data()))
    }

    pub fn back_mut(&mut self) -> Option<RefMut<'_, T>> {
        self.tail
            .as_ref()
            .map(|tail_vertex| RefMut::map(tail_vertex.borrow_mut(), |vertex| &mut vertex.data))
    }

    pub fn front(&self) -> Option<Ref<'_, T>> {
        self.head
            .as_ref()
            .map(|head_vertex| Ref::map(head_vertex.borrow(), |vertex| vertex.data()))
    }

    pub fn front_mut(&mut self) -> Option<RefMut<'_, T>> {
        self.head
            .as_ref()
            .map(|head_vertex| RefMut::map(head_vertex.borrow_mut(), |vertex| &mut vertex.data))
    }

    pub fn push_back(&mut self, data: T) {
        self.size += 1;
        let new_node = Node::new(data);

        match self.tail.clone() {
            None => {
                self.head = Some(new_node.clone());
            }
            Some(tail_vertex) => {
                tail_vertex.borrow_mut().next = Some(new_node.clone());
                new_node.borrow_mut().prev = Some(Rc::downgrade(&tail_vertex));
            }
        }

        self.tail = Some(new_node);
    }

    pub fn push_front(&mut self, data: T) {
        self.size += 1;
        let new_node = Node::new(data);

        match self.head.clone() {
            None => {
                self.tail = Some(new_node.clone());
            }
            Some(head_vertex) => {
                new_node.borrow_mut().next = Some(head_vertex.clone());
                head_vertex.borrow_mut().prev = Some(Rc::downgrade(&new_node));
            }
        }

        self.head = Some(new_node);
    }

    pub fn pop_back(&mut self) -> Option<T> {
        self.tail.take().map(|tail_vertex| {
            self.size -= 1;

            match tail_vertex.borrow().prev.clone() {
                Some(second_tail_vertex) => {
                    second_tail_vertex.upgrade().unwrap().borrow_mut().next = None;
                    self.tail = Some(second_tail_vertex.upgrade().unwrap());
                }
                None => {
                    self.head = None;
                }
            }

            Rc::into_inner(tail_vertex).unwrap().into_inner().data
        })
    }

    pub fn pop_front(&mut self) -> Option<T> {
        self.head.take().map(|head_vertex| {
            self.size -= 1;

            match head_vertex.borrow().next.clone() {
                Some(second_vertex) => {
                    second_vertex.borrow_mut().prev = None;
                    self.head = Some(second_vertex);
                }
                None => {
                    self.tail = None;
                }
            }

            Rc::into_inner(head_vertex).unwrap().into_inner().data
        })
    }

    pub fn reverse(&mut self) {
        self.tail = self.head.clone();

        let mut previous = None;
        let mut current = self.head.clone();

        while let Some(current_vertex) = current {
            let next = current_vertex.borrow_mut().next.clone();

            current_vertex.borrow_mut().prev = next.clone().map(|v| Rc::downgrade(&v));
            current_vertex.borrow_mut().next = previous.clone();

            previous = Some(current_vertex);
            current = next;
        }

        self.head = previous.clone();
    }
}

#[cfg(test)]
mod tests {
    use super::DoublyLinkedList;
    use std::cell::Cell;
    use std::fmt;
    use std::iter::FusedIterator;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::rc::Rc;

    #[derive(Debug)]
    struct CloneTracked {
        value: String,
        clone_count: Rc<Cell<usize>>,
        panic_on_clone: Rc<Cell<bool>>,
    }

    impl Clone for CloneTracked {
        fn clone(&self) -> Self {
            assert!(!self.panic_on_clone.get(), "requested clone panic");
            self.clone_count.set(self.clone_count.get() + 1);
            Self {
                value: self.value.clone(),
                clone_count: Rc::clone(&self.clone_count),
                panic_on_clone: Rc::clone(&self.panic_on_clone),
            }
        }
    }

    impl fmt::Display for CloneTracked {
        fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            self.value.fmt(formatter)
        }
    }

    fn create_clone_tracked_list(
        values: &[&str],
        clone_count: &Rc<Cell<usize>>,
        panic_on_clone: &Rc<Cell<bool>>,
    ) -> DoublyLinkedList<CloneTracked> {
        let mut doubly_linked_list = DoublyLinkedList::new();
        for value in values {
            doubly_linked_list.push_back(CloneTracked {
                value: (*value).to_owned(),
                clone_count: Rc::clone(clone_count),
                panic_on_clone: Rc::clone(panic_on_clone),
            });
        }
        doubly_linked_list
    }

    fn assert_iterator_traits<I: DoubleEndedIterator + ExactSizeIterator + FusedIterator>(_: &I) {}

    fn create_doubly_linked_list(values: &[i32]) -> DoublyLinkedList<i32> {
        let mut doubly_linked_list = DoublyLinkedList::new();
        for value in values.iter() {
            doubly_linked_list.push_back(*value);
        }

        doubly_linked_list
    }

    #[test]
    fn debug_when_list_has_values_formats_them_in_list_order() {
        // Arrange
        let cases = [
            (vec![], "[]"),
            (vec![10], "[10]"),
            (vec![30, 10, 20, 10], "[30, 10, 20, 10]"),
        ];

        for (inserted_values, expected_output) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let actual_output = format!("{:?}", doubly_linked_list);

            // Assert
            assert_eq!(actual_output, expected_output);
        }
    }

    #[test]
    fn debug_when_elements_are_strings_preserves_debug_quotes_and_escaping() {
        // Arrange
        let inserted_values = [r#"quote"here"#, "line\nbreak", r"path\name"];
        let expected_output = r#"["quote\"here", "line\nbreak", "path\\name"]"#;
        let mut doubly_linked_list = DoublyLinkedList::new();
        for value in inserted_values {
            doubly_linked_list.push_back(value.to_owned());
        }

        // Act
        let actual_output = format!("{:?}", doubly_linked_list);

        // Assert
        assert_eq!(actual_output, expected_output);
    }

    #[test]
    fn debug_when_alternate_format_is_requested_indents_elements() {
        // Arrange
        let cases = [
            (vec![], "[]"),
            (vec![10], "[\n    10,\n]"),
            (vec![30, 10, 20], "[\n    30,\n    10,\n    20,\n]"),
        ];

        for (inserted_values, expected_output) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let actual_output = format!("{:#?}", doubly_linked_list);

            // Assert
            assert_eq!(actual_output, expected_output);
        }
    }

    #[test]
    fn debug_when_element_clone_panics_formats_without_cloning() {
        // Arrange
        let inserted_values = ["a", "b", "c"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(true));
        let expected_clone_count = 0;
        let doubly_linked_list =
            create_clone_tracked_list(&inserted_values, &clone_count, &panic_on_clone);

        // Act
        let _actual_output = format!("{:?}", doubly_linked_list);
        let actual_clone_count = clone_count.get();

        // Assert
        assert_eq!(actual_clone_count, expected_clone_count);
    }

    #[test]
    fn debug_after_formatting_preserves_contents_and_allows_removing_nodes() {
        // Arrange
        let inserted_values = [30, 10, 20, 10];
        let expected_len = inserted_values.len();
        let expected_original_values = inserted_values.to_vec();
        let expected_popped_values = (Some(30), Some(10));
        let expected_remaining_values = vec![10, 20];
        let mut doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let _actual_output = format!("{:?}", doubly_linked_list);
        let actual_len = doubly_linked_list.len();
        let actual_original_values = doubly_linked_list.iter().collect::<Vec<_>>();
        let actual_popped_values = (
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_back(),
        );
        let actual_remaining_values = doubly_linked_list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_original_values, expected_original_values);
        assert_eq!(actual_popped_values, expected_popped_values);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    fn debug_when_writer_fails_propagates_formatting_error() {
        // Arrange
        struct FailingWriter;

        impl fmt::Write for FailingWriter {
            fn write_str(&mut self, _: &str) -> fmt::Result {
                Err(fmt::Error)
            }
        }

        let inserted_values = [10, 20];
        let expected_result = Err(fmt::Error);
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);
        let mut writer = FailingWriter;

        // Act
        let actual_result = fmt::write(&mut writer, format_args!("{:?}", doubly_linked_list));

        // Assert
        assert_eq!(actual_result, expected_result);
    }

    #[test]
    fn default_when_called_matches_new_empty_state() {
        // Arrange
        let from_new: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_states = (true, 0, true, 0);

        // Act
        let from_default: DoublyLinkedList<i32> = DoublyLinkedList::default();
        let actual_states = (
            from_new.is_empty(),
            from_new.len(),
            from_default.is_empty(),
            from_default.len(),
        );

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn into_iter_when_list_is_empty_returns_no_values() {
        // Arrange
        let doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_values: Vec<i32> = Vec::new();

        // Act
        let actual_values = doubly_linked_list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_when_list_has_one_element_returns_single_value() {
        // Arrange
        let only_value = 10;
        let expected_values = vec![only_value];
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let actual_values = doubly_linked_list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_when_list_has_multiple_elements_returns_values_from_front_to_back() {
        // Arrange
        let inserted_values = [1, 1, 2, 3, 4, 5, 0];
        let expected_values = inserted_values.to_vec();
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let actual_values = doubly_linked_list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_rev_when_list_has_three_elements_returns_values_from_back_to_front() {
        // Arrange
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_values = vec![third_value, second_value, first_value];
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(first_value);
        doubly_linked_list.push_back(second_value);
        doubly_linked_list.push_back(third_value);

        // Act
        let mut actual_values = Vec::new();
        for value in doubly_linked_list.into_iter().rev() {
            actual_values.push(value);
        }

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_next_back_when_list_has_one_element_returns_only_value() {
        // Arrange
        let only_value = 10;
        let expected_first_value = Some(only_value);
        let expected_second_value = None;
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let mut into_iter = doubly_linked_list.into_iter();
        let actual_first_value = into_iter.next_back();
        let actual_second_value = into_iter.next_back();

        // Assert
        assert_eq!(actual_first_value, expected_first_value);
        assert_eq!(actual_second_value, expected_second_value);
    }

    #[test]
    fn into_iter_when_next_and_next_back_are_called_returns_each_value_once() {
        // Arrange
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_front_value = Some(first_value);
        let expected_back_value = Some(third_value);
        let expected_middle_value = Some(second_value);
        let expected_end_value = None;
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(first_value);
        doubly_linked_list.push_back(second_value);
        doubly_linked_list.push_back(third_value);

        // Act
        let mut into_iter = doubly_linked_list.into_iter();
        let actual_front_value = into_iter.next();
        let actual_back_value = into_iter.next_back();
        let actual_middle_value = into_iter.next();
        let actual_end_value = into_iter.next_back();

        // Assert
        assert_eq!(actual_front_value, expected_front_value);
        assert_eq!(actual_back_value, expected_back_value);
        assert_eq!(actual_middle_value, expected_middle_value);
        assert_eq!(actual_end_value, expected_end_value);
    }

    #[test]
    fn iter_when_list_has_multiple_elements_returns_values_from_front_to_back() {
        // Arrange
        let inserted_values = [1, 1, 2, 3, 4, 5, 0];
        let expected_values = inserted_values.to_vec();
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let actual_values = doubly_linked_list.iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_when_list_is_empty_returns_no_values() {
        // Arrange
        let doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_next_value = None;

        // Act
        let actual_next_value = doubly_linked_list.iter().next();

        // Assert
        assert_eq!(actual_next_value, expected_next_value);
    }

    #[test]
    fn iter_when_list_has_one_element_returns_single_value() {
        // Arrange
        let only_value = 10;
        let expected_next_value = Some(only_value);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let actual_next_value = doubly_linked_list.iter().next();

        // Assert
        assert_eq!(actual_next_value, expected_next_value);
    }

    #[test]
    fn iter_when_called_repeatedly_returns_same_values_and_keeps_len() {
        // Arrange
        let inserted_values = [1, 1, 2, 3, 4, 5, 0];
        let expected_state = (
            inserted_values.to_vec(),
            inserted_values.to_vec(),
            inserted_values.len(),
        );
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let actual_state = (
            doubly_linked_list.iter().collect::<Vec<_>>(),
            doubly_linked_list.iter().collect::<Vec<_>>(),
            doubly_linked_list.len(),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn iter_rev_when_list_contains_duplicates_returns_values_from_back_to_front() {
        // Arrange
        let inserted_values = [10, 20, 20, 30];
        let expected_values = vec![30, 20, 20, 10];
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let actual_values = doubly_linked_list.iter().rev().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_when_ends_cross_returns_each_value_once_and_stays_exhausted() {
        // Arrange
        let cases = [
            (
                vec![],
                vec![None, None, None, None],
                vec![None, None, None, None],
            ),
            (
                vec![10],
                vec![Some(10), None, None, None],
                vec![Some(10), None, None, None],
            ),
            (
                vec![10, 20, 30],
                vec![Some(10), Some(30), Some(20), None, None],
                vec![Some(30), Some(10), Some(20), None, None],
            ),
            (
                vec![10, 20, 30, 40],
                vec![Some(10), Some(40), Some(20), Some(30), None, None],
                vec![Some(40), Some(10), Some(30), Some(20), None, None],
            ),
        ];

        for (inserted_values, expected_front_first, expected_back_first) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);
            for (start_at_back, expected_values) in
                [(false, expected_front_first), (true, expected_back_first)]
            {
                // Act
                let mut iter = doubly_linked_list.iter();
                let actual_values = (0..expected_values.len())
                    .map(|index| {
                        if (index % 2 == 0) == start_at_back {
                            iter.next_back()
                        } else {
                            iter.next()
                        }
                    })
                    .collect::<Vec<_>>();

                // Assert
                assert_eq!(actual_values, expected_values);
            }
        }
    }

    #[test]
    fn iter_when_both_ends_are_consumed_reports_exact_remaining_lengths() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_states = [
            ((3, Some(3)), 3),
            ((2, Some(2)), 2),
            ((1, Some(1)), 1),
            ((0, Some(0)), 0),
            ((0, Some(0)), 0),
        ];
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut iter = doubly_linked_list.iter();
        assert_iterator_traits(&iter);
        let initial_state = (iter.size_hint(), iter.len());
        let _front_value = iter.next();
        let after_front = (iter.size_hint(), iter.len());
        let _back_value = iter.next_back();
        let after_back = (iter.size_hint(), iter.len());
        let _middle_value = iter.next_back();
        let after_middle = (iter.size_hint(), iter.len());
        let _end_value = iter.next();
        let actual_states = [
            initial_state,
            after_front,
            after_back,
            after_middle,
            (iter.size_hint(), iter.len()),
        ];

        // Assert
        assert_eq!(actual_states, expected_states);
    }

    #[test]
    fn into_iter_when_both_ends_are_consumed_reports_exact_remaining_lengths() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_states = [
            ((3, Some(3)), 3),
            ((2, Some(2)), 2),
            ((1, Some(1)), 1),
            ((0, Some(0)), 0),
            ((0, Some(0)), 0),
        ];
        let expected_end_values = (None, None, None);
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut iter = doubly_linked_list.into_iter();
        assert_iterator_traits(&iter);
        let initial_state = (iter.size_hint(), iter.len());
        let _front_value = iter.next();
        let after_front = (iter.size_hint(), iter.len());
        let _back_value = iter.next_back();
        let after_back = (iter.size_hint(), iter.len());
        let _middle_value = iter.next_back();
        let after_middle = (iter.size_hint(), iter.len());
        let actual_end_values = (iter.next(), iter.next_back(), iter.next());
        let actual_states = [
            initial_state,
            after_front,
            after_back,
            after_middle,
            (iter.size_hint(), iter.len()),
        ];

        // Assert
        assert_eq!(actual_states, expected_states);
        assert_eq!(actual_end_values, expected_end_values);
    }

    #[test]
    fn borrowed_into_iter_when_used_in_for_preserves_values_and_original_list() {
        // Arrange
        let inserted_values = [10, 20, 20, 30];
        let expected_values = inserted_values.to_vec();
        let expected_len = inserted_values.len();
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut actual_values = Vec::new();
        for value in &doubly_linked_list {
            actual_values.push(value);
        }
        let actual_original_values = doubly_linked_list.iter().collect::<Vec<_>>();
        let actual_len = doubly_linked_list.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_original_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn iter_count_when_both_ends_have_been_consumed_returns_remaining_count() {
        // Arrange
        let cases = [
            (vec![], 0),
            (vec![10], 0),
            (vec![10, 20], 0),
            (vec![10, 20, 30], 1),
            (vec![10, 20, 30, 40], 2),
        ];

        for (inserted_values, expected_count) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let mut iter = doubly_linked_list.iter();
            let _front_value = iter.next();
            let _back_value = iter.next_back();
            let actual_count = iter.count();

            // Assert
            assert_eq!(actual_count, expected_count);
        }
    }

    #[test]
    fn iter_last_when_both_ends_have_been_consumed_returns_last_remaining_value() {
        // Arrange
        let cases = [
            (vec![], None),
            (vec![10], None),
            (vec![10, 20], None),
            (vec![10, 20, 30], Some(20)),
            (vec![10, 20, 30, 40], Some(30)),
        ];

        for (inserted_values, expected_value) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let mut iter = doubly_linked_list.iter();
            let _front_value = iter.next();
            let _back_value = iter.next_back();
            let actual_value = iter.last();

            // Assert
            assert_eq!(actual_value, expected_value);
        }
    }

    #[test]
    fn iter_count_when_values_remain_does_not_clone_them() {
        // Arrange
        let inserted_values = ["a", "b", "c", "d", "e"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(false));
        let expected_count = 3;
        let expected_clone_count = 2;
        let doubly_linked_list =
            create_clone_tracked_list(&inserted_values, &clone_count, &panic_on_clone);

        // Act
        let mut iter = doubly_linked_list.iter();
        let _front_value = iter.next();
        let _back_value = iter.next_back();
        let actual_count = iter.count();
        let actual_clone_count = clone_count.get();

        // Assert
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_clone_count, expected_clone_count);
    }

    #[test]
    fn iter_last_when_values_remain_clones_only_last_remaining_value() {
        // Arrange
        let inserted_values = ["a", "b", "c", "d", "e"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(false));
        let expected_value = Some("d".to_owned());
        let expected_clone_count = 3;
        let doubly_linked_list =
            create_clone_tracked_list(&inserted_values, &clone_count, &panic_on_clone);

        // Act
        let mut iter = doubly_linked_list.iter();
        let _front_value = iter.next();
        let _back_value = iter.next_back();
        let actual_value = iter.last().map(|value| value.value);
        let actual_clone_count = clone_count.get();

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_clone_count, expected_clone_count);
    }

    #[test]
    fn iter_when_clone_panics_at_either_end_preserves_position_and_remaining_length() {
        // Arrange
        let inserted_values = ["a", "b", "c"];
        let cases = [(false, vec!["a", "b", "c"]), (true, vec!["c", "b", "a"])];
        let resume_cloning = false;
        let expected_panic = true;
        let expected_state = ((3, Some(3)), 3);

        for (from_back, expected_values) in cases {
            let clone_count = Rc::new(Cell::new(0));
            let panic_on_clone = Rc::new(Cell::new(true));
            let doubly_linked_list =
                create_clone_tracked_list(&inserted_values, &clone_count, &panic_on_clone);

            // Act
            let mut iter = doubly_linked_list.iter();
            let actual_result = catch_unwind(AssertUnwindSafe(|| {
                if from_back {
                    iter.next_back()
                } else {
                    iter.next()
                }
            }));
            let actual_panic = actual_result.is_err();
            let actual_state = (iter.size_hint(), iter.len());
            panic_on_clone.set(resume_cloning);
            let actual_values = if from_back {
                iter.rev().map(|value| value.value).collect::<Vec<_>>()
            } else {
                iter.map(|value| value.value).collect::<Vec<_>>()
            };

            // Assert
            assert_eq!(actual_panic, expected_panic);
            assert_eq!(actual_state, expected_state);
            assert_eq!(actual_values, expected_values);
        }
    }

    #[test]
    fn iter_after_last_use_allows_pop_front_without_explicit_drop() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_values = (Some(30), Some(10), vec![20, 30]);
        let mut doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut iter = doubly_linked_list.iter();
        let actual_iter_value = iter.next_back();
        let actual_popped_value = doubly_linked_list.pop_front();
        let actual_values = (
            actual_iter_value,
            actual_popped_value,
            doubly_linked_list.iter().collect::<Vec<_>>(),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_after_last_use_allows_pop_back_without_explicit_drop() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_values = (Some(10), Some(30), vec![10, 20]);
        let mut doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut iter = doubly_linked_list.iter();
        let actual_iter_value = iter.next();
        let actual_popped_value = doubly_linked_list.pop_back();
        let actual_values = (
            actual_iter_value,
            actual_popped_value,
            doubly_linked_list.iter().collect::<Vec<_>>(),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_after_last_use_allows_into_iter_without_explicit_drop() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_iter_value = Some(10);
        let expected_values = inserted_values.to_vec();
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut iter = doubly_linked_list.iter();
        let actual_iter_value = iter.next();
        let actual_values = doubly_linked_list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_iter_value, expected_iter_value);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn into_iter_when_values_are_non_copy_moves_them_without_cloning() {
        // Arrange
        let inserted_values = ["a", "b", "c"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(true));
        let expected_values = [
            Some("a".to_owned()),
            Some("c".to_owned()),
            Some("b".to_owned()),
            None,
        ];
        let expected_clone_count = 0;
        let doubly_linked_list =
            create_clone_tracked_list(&inserted_values, &clone_count, &panic_on_clone);

        // Act
        let mut iter = doubly_linked_list.into_iter();
        let actual_values = [
            iter.next().map(|value| value.value),
            iter.next_back().map(|value| value.value),
            iter.next().map(|value| value.value),
            iter.next_back().map(|value| value.value),
        ];
        let actual_clone_count = clone_count.get();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_clone_count, expected_clone_count);
    }

    #[test]
    fn iter_nth_after_mixed_consumption_respects_remaining_boundaries() {
        // Arrange
        let inserted_values = [10, 20, 30, 40, 50];
        let cases = [
            (0, (Some(20), 2, Some(40))),
            (2, (Some(40), 0, None)),
            (3, (None, 0, None)),
            (usize::MAX, (None, 0, None)),
        ];

        for (skip_count, expected_state) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let mut iter = doubly_linked_list.iter();
            let _front_value = iter.next();
            let _back_value = iter.next_back();
            let actual_state = (iter.nth(skip_count), iter.len(), iter.next_back());

            // Assert
            assert_eq!(actual_state, expected_state);
        }
    }

    #[test]
    fn iter_nth_back_after_mixed_consumption_respects_remaining_boundaries() {
        // Arrange
        let inserted_values = [10, 20, 30, 40, 50];
        let cases = [
            (0, (Some(40), 2, Some(20))),
            (2, (Some(20), 0, None)),
            (3, (None, 0, None)),
            (usize::MAX, (None, 0, None)),
        ];

        for (skip_count, expected_state) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let mut iter = doubly_linked_list.iter();
            let _front_value = iter.next();
            let _back_value = iter.next_back();
            let actual_state = (iter.nth_back(skip_count), iter.len(), iter.next());

            // Assert
            assert_eq!(actual_state, expected_state);
        }
    }

    #[test]
    fn iter_count_before_consumption_returns_full_length() {
        // Arrange
        let cases = [(vec![], 0), (vec![10], 1), (vec![30, 10, 20, 10], 4)];

        for (inserted_values, expected_count) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let actual_count = doubly_linked_list.iter().count();

            // Assert
            assert_eq!(actual_count, expected_count);
        }
    }

    #[test]
    fn iter_last_before_consumption_returns_tail_value() {
        // Arrange
        let cases = [
            (vec![], None),
            (vec![10], Some(10)),
            (vec![30, 10, 20, 10], Some(10)),
        ];

        for (inserted_values, expected_value) in cases {
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let actual_value = doubly_linked_list.iter().last();

            // Assert
            assert_eq!(actual_value, expected_value);
        }
    }

    #[test]
    fn iter_when_empty_or_singleton_reports_exact_lengths_through_exhaustion() {
        // Arrange
        let cases = [
            (vec![], false, None),
            (vec![], true, None),
            (vec![10], false, Some(10)),
            (vec![10], true, Some(10)),
        ];
        let expected_end_values = [None, None, None];
        let expected_exhausted_state = ((0, Some(0)), 0);

        for (inserted_values, from_back, expected_first_value) in cases {
            let expected_initial_state = (
                (inserted_values.len(), Some(inserted_values.len())),
                inserted_values.len(),
            );
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let mut iter = doubly_linked_list.iter();
            assert_iterator_traits(&iter);
            let actual_initial_state = (iter.size_hint(), iter.len());
            let actual_first_value = if from_back {
                iter.next_back()
            } else {
                iter.next()
            };
            let actual_after_first_state = (iter.size_hint(), iter.len());
            let actual_end_values = [iter.next(), iter.next_back(), iter.next()];
            let actual_final_state = (iter.size_hint(), iter.len());

            // Assert
            assert_eq!(actual_initial_state, expected_initial_state);
            assert_eq!(actual_first_value, expected_first_value);
            assert_eq!(actual_after_first_state, expected_exhausted_state);
            assert_eq!(actual_end_values, expected_end_values);
            assert_eq!(actual_final_state, expected_exhausted_state);
        }
    }

    #[test]
    fn into_iter_when_empty_or_singleton_reports_exact_lengths_through_exhaustion() {
        // Arrange
        let cases = [
            (vec![], false, None),
            (vec![], true, None),
            (vec![10], false, Some(10)),
            (vec![10], true, Some(10)),
        ];
        let expected_end_values = [None, None, None];
        let expected_exhausted_state = ((0, Some(0)), 0);

        for (inserted_values, from_back, expected_first_value) in cases {
            let expected_initial_state = (
                (inserted_values.len(), Some(inserted_values.len())),
                inserted_values.len(),
            );
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let mut iter = doubly_linked_list.into_iter();
            assert_iterator_traits(&iter);
            let actual_initial_state = (iter.size_hint(), iter.len());
            let actual_first_value = if from_back {
                iter.next_back()
            } else {
                iter.next()
            };
            let actual_after_first_state = (iter.size_hint(), iter.len());
            let actual_end_values = [iter.next(), iter.next_back(), iter.next()];
            let actual_final_state = (iter.size_hint(), iter.len());

            // Assert
            assert_eq!(actual_initial_state, expected_initial_state);
            assert_eq!(actual_first_value, expected_first_value);
            assert_eq!(actual_after_first_state, expected_exhausted_state);
            assert_eq!(actual_end_values, expected_end_values);
            assert_eq!(actual_final_state, expected_exhausted_state);
        }
    }

    #[test]
    fn borrowed_into_iter_rev_returns_reverse_order_and_preserves_list() {
        // Arrange
        let cases = [
            (vec![], vec![]),
            (vec![10], vec![10]),
            (vec![30, 10, 20, 10], vec![10, 20, 10, 30]),
        ];

        for (inserted_values, expected_values) in cases {
            let expected_len = inserted_values.len();
            let doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            let actual_values = (&doubly_linked_list).into_iter().rev().collect::<Vec<_>>();
            let actual_original_values = doubly_linked_list.iter().collect::<Vec<_>>();
            let actual_len = doubly_linked_list.len();

            // Assert
            assert_eq!(actual_values, expected_values);
            assert_eq!(actual_original_values, inserted_values);
            assert_eq!(actual_len, expected_len);
        }
    }

    #[test]
    fn borrowed_iterators_when_consumed_together_keep_independent_positions() {
        // Arrange
        let inserted_values = [10, 20, 20, 30, 40];
        let expected_consumed_values = (Some(10), Some(40), Some(40));
        let expected_lengths = (3, 4);
        let expected_remaining_values = (vec![20, 20, 30], vec![10, 20, 20, 30]);
        let doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let mut first_iter = doubly_linked_list.iter();
        let mut second_iter = (&doubly_linked_list).into_iter();
        let actual_consumed_values = (
            first_iter.next(),
            first_iter.next_back(),
            second_iter.next_back(),
        );
        let actual_lengths = (first_iter.len(), second_iter.len());
        let actual_remaining_values = (
            first_iter.collect::<Vec<_>>(),
            second_iter.collect::<Vec<_>>(),
        );

        // Assert
        assert_eq!(actual_consumed_values, expected_consumed_values);
        assert_eq!(actual_lengths, expected_lengths);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    fn iter_when_returned_string_is_modified_keeps_original_values_at_both_ends() {
        // Arrange
        let inserted_values = ["left", "middle", "right"];
        let cases = [(false, "left!"), (true, "right!")];
        let suffix = "!";
        let expected_original_values = inserted_values.to_vec();

        for (from_back, expected_value) in cases {
            let mut doubly_linked_list = DoublyLinkedList::new();
            for value in inserted_values {
                doubly_linked_list.push_back(value.to_owned());
            }

            // Act
            let mut iter = doubly_linked_list.iter();
            let mut actual_value = if from_back {
                iter.next_back()
            } else {
                iter.next()
            }
            .expect("the list contains values");
            actual_value.push_str(suffix);
            let actual_original_values = doubly_linked_list.iter().collect::<Vec<_>>();

            // Assert
            assert_eq!(actual_value, expected_value);
            assert_eq!(actual_original_values, expected_original_values);
        }
    }

    #[test]
    fn iter_rev_after_list_reverse_returns_original_insertion_order() {
        // Arrange
        let cases = [vec![], vec![10], vec![30, 10, 20, 10]];

        for inserted_values in cases {
            let expected_values = inserted_values.clone();
            let mut doubly_linked_list = create_doubly_linked_list(&inserted_values);

            // Act
            doubly_linked_list.reverse();
            let actual_values = doubly_linked_list.iter().rev().collect::<Vec<_>>();

            // Assert
            assert_eq!(actual_values, expected_values);
        }
    }

    #[test]
    fn len_when_list_is_empty_returns_zero() {
        // Arrange
        let doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_len = 0;

        // Act
        let actual_len = doubly_linked_list.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn len_when_push_front_and_push_back_are_called_updates_element_count() {
        // Arrange
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_lengths = [1, 2, 3];
        let mut doubly_linked_list = DoublyLinkedList::new();

        // Act
        doubly_linked_list.push_back(first_value);
        let actual_len_after_first_push = doubly_linked_list.len();
        doubly_linked_list.push_front(second_value);
        let actual_len_after_second_push = doubly_linked_list.len();
        doubly_linked_list.push_back(third_value);
        let actual_len_after_third_push = doubly_linked_list.len();
        let actual_lengths = [
            actual_len_after_first_push,
            actual_len_after_second_push,
            actual_len_after_third_push,
        ];

        // Assert
        assert_eq!(actual_lengths, expected_lengths);
    }

    #[test]
    fn len_when_push_and_pop_are_balanced_returns_zero() {
        // Arrange
        let first_value = 10;
        let second_value = 20;
        let third_value = 30;
        let expected_len = 0;
        let mut doubly_linked_list = DoublyLinkedList::new();

        // Act
        doubly_linked_list.push_back(first_value);
        doubly_linked_list.pop_front();
        doubly_linked_list.push_front(second_value);
        doubly_linked_list.pop_back();
        doubly_linked_list.push_back(third_value);
        doubly_linked_list.pop_front();
        let actual_len = doubly_linked_list.len();

        // Assert
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn is_empty_when_list_is_new_returns_true() {
        // Arrange
        let doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_is_empty = true;

        // Act
        let actual_is_empty = doubly_linked_list.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn is_empty_when_list_has_elements_returns_false() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_is_empty = false;
        let mut doubly_linked_list = DoublyLinkedList::new();
        for value in inserted_values.iter() {
            doubly_linked_list.push_back(*value);
        }

        // Act
        let actual_is_empty = doubly_linked_list.is_empty();

        // Assert
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn is_empty_when_pop_front_and_pop_back_are_repeated_on_empty_list_remains_true() {
        // Arrange
        let repeat_count = 100;
        let expected_state = (true, 0);
        let mut doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();

        // Act
        for _ in 0..repeat_count {
            doubly_linked_list.pop_front();
            doubly_linked_list.pop_back();
        }
        let actual_state = (doubly_linked_list.is_empty(), doubly_linked_list.len());

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn back_when_list_is_empty_returns_none() {
        // Arrange
        let doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_back_value = None;

        // Act
        let actual_back_value = doubly_linked_list.back().map(|back_value| *back_value);

        // Assert
        assert_eq!(actual_back_value, expected_back_value);
    }

    #[test]
    fn back_when_one_element_exists_returns_that_element() {
        // Arrange
        let only_value = 10;
        let expected_back_value = Some(only_value);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let actual_back_value = doubly_linked_list.back().map(|back_value| *back_value);

        // Assert
        assert_eq!(actual_back_value, expected_back_value);
    }

    #[test]
    fn back_when_values_are_pushed_to_both_ends_returns_tail_value() {
        // Arrange
        let head_value = 5;
        let first_tail_value = 10;
        let second_tail_value = 20;
        let expected_back_value = Some(second_tail_value);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_front(head_value);
        doubly_linked_list.push_back(first_tail_value);
        doubly_linked_list.push_back(second_tail_value);

        // Act
        let actual_back_value = doubly_linked_list.back().map(|back_value| *back_value);

        // Assert
        assert_eq!(actual_back_value, expected_back_value);
    }

    #[test]
    fn back_mut_when_list_is_empty_returns_none() {
        // Arrange
        let mut doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_back_value = None;

        // Act
        let actual_back_value = doubly_linked_list.back_mut().map(|back_value| *back_value);

        // Assert
        assert_eq!(actual_back_value, expected_back_value);
    }

    #[test]
    fn back_mut_when_tail_value_is_updated_changes_back_and_pop_back_results() {
        // Arrange
        let original_values = [10, 20];
        let updated_back_value = 99;
        let expected_values = (Some(updated_back_value), Some(updated_back_value));
        let mut doubly_linked_list = create_doubly_linked_list(&original_values);

        // Act
        {
            let mut actual_back_value = doubly_linked_list.back_mut().unwrap();
            *actual_back_value = updated_back_value;
        }
        let actual_values = (
            doubly_linked_list.back().map(|back_value| *back_value),
            doubly_linked_list.pop_back(),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn front_when_list_is_empty_returns_none() {
        // Arrange
        let doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_front_value = None;

        // Act
        let actual_front_value = doubly_linked_list.front().map(|front_value| *front_value);

        // Assert
        assert_eq!(actual_front_value, expected_front_value);
    }

    #[test]
    fn front_when_one_element_exists_returns_that_element() {
        // Arrange
        let only_value = 10;
        let expected_front_value = Some(only_value);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let actual_front_value = doubly_linked_list.front().map(|front_value| *front_value);

        // Assert
        assert_eq!(actual_front_value, expected_front_value);
    }

    #[test]
    fn front_when_values_are_pushed_to_both_ends_returns_head_value() {
        // Arrange
        let head_value = 5;
        let first_tail_value = 10;
        let second_tail_value = 20;
        let expected_front_value = Some(head_value);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(first_tail_value);
        doubly_linked_list.push_back(second_tail_value);
        doubly_linked_list.push_front(head_value);

        // Act
        let actual_front_value = doubly_linked_list.front().map(|front_value| *front_value);

        // Assert
        assert_eq!(actual_front_value, expected_front_value);
    }

    #[test]
    fn front_mut_when_list_is_empty_returns_none() {
        // Arrange
        let mut doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_front_value = None;

        // Act
        let actual_front_value = doubly_linked_list
            .front_mut()
            .map(|front_value| *front_value);

        // Assert
        assert_eq!(actual_front_value, expected_front_value);
    }

    #[test]
    fn front_mut_when_head_value_is_updated_changes_front_and_pop_front_results() {
        // Arrange
        let original_values = [10, 20];
        let updated_front_value = 99;
        let expected_values = (Some(updated_front_value), Some(updated_front_value));
        let mut doubly_linked_list = create_doubly_linked_list(&original_values);

        // Act
        {
            let mut actual_front_value = doubly_linked_list.front_mut().unwrap();
            *actual_front_value = updated_front_value;
        }
        let actual_values = (
            doubly_linked_list.front().map(|front_value| *front_value),
            doubly_linked_list.pop_front(),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn push_front_when_values_are_popped_from_front_returns_reverse_insertion_order() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_values = [Some(30), Some(20), Some(10), None];
        let mut doubly_linked_list = DoublyLinkedList::new();

        // Act
        for value in inserted_values.iter() {
            doubly_linked_list.push_front(*value);
        }
        let actual_values = [
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_front(),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pop_back_when_list_is_empty_returns_none() {
        // Arrange
        let mut doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_value = None;

        // Act
        let actual_value = doubly_linked_list.pop_back();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pop_back_when_list_has_one_element_returns_element_and_becomes_empty() {
        // Arrange
        let only_value = 10;
        let expected_state = (Some(only_value), None, None, 0, true);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let actual_state = (
            doubly_linked_list.pop_back(),
            doubly_linked_list.front().map(|front_value| *front_value),
            doubly_linked_list.back().map(|back_value| *back_value),
            doubly_linked_list.len(),
            doubly_linked_list.is_empty(),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn pop_back_when_values_are_pushed_back_returns_values_in_reverse_order() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_values = [Some(30), Some(20), Some(10), None];
        let mut doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let actual_values = [
            doubly_linked_list.pop_back(),
            doubly_linked_list.pop_back(),
            doubly_linked_list.pop_back(),
            doubly_linked_list.pop_back(),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pop_front_when_list_is_empty_returns_none() {
        // Arrange
        let mut doubly_linked_list: DoublyLinkedList<i32> = DoublyLinkedList::new();
        let expected_value = None;

        // Act
        let actual_value = doubly_linked_list.pop_front();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn pop_front_when_list_has_one_element_returns_element_and_becomes_empty() {
        // Arrange
        let only_value = 10;
        let expected_state = (Some(only_value), None, None, 0, true);
        let mut doubly_linked_list = DoublyLinkedList::new();
        doubly_linked_list.push_back(only_value);

        // Act
        let actual_state = (
            doubly_linked_list.pop_front(),
            doubly_linked_list.front().map(|front_value| *front_value),
            doubly_linked_list.back().map(|back_value| *back_value),
            doubly_linked_list.len(),
            doubly_linked_list.is_empty(),
        );

        // Assert
        assert_eq!(actual_state, expected_state);
    }

    #[test]
    fn pop_front_when_values_are_pushed_back_returns_values_in_insertion_order() {
        // Arrange
        let inserted_values = [10, 20, 30];
        let expected_values = [Some(10), Some(20), Some(30), None];
        let mut doubly_linked_list = create_doubly_linked_list(&inserted_values);

        // Act
        let actual_values = [
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_front(),
            doubly_linked_list.pop_front(),
        ];

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    fn create_list_from_values(values: &[i32]) -> DoublyLinkedList<i32> {
        let mut doubly_linked_list = DoublyLinkedList::new();

        for value in values {
            doubly_linked_list.push_back(*value);
        }

        doubly_linked_list
    }

    fn collect_list_values(doubly_linked_list: &DoublyLinkedList<i32>) -> Vec<i32> {
        doubly_linked_list.iter().collect()
    }

    #[test]
    fn reverse_empty_list_keeps_empty_state() {
        // Arrange
        let input_values: Vec<i32> = vec![];
        let expected_values: Vec<i32> = vec![];
        let expected_len = 0;
        let expected_is_empty = true;
        let mut doubly_linked_list = create_list_from_values(&input_values);

        // Act
        doubly_linked_list.reverse();
        let actual_values = collect_list_values(&doubly_linked_list);
        let actual_len = doubly_linked_list.len();
        let actual_is_empty = doubly_linked_list.is_empty();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_is_empty, expected_is_empty);
    }

    #[test]
    fn reverse_single_element_list_keeps_same_order() {
        // Arrange
        let input_values = vec![1];
        let expected_values = vec![1];
        let expected_front = Some(1);
        let expected_back = Some(1);
        let mut doubly_linked_list = create_list_from_values(&input_values);

        // Act
        doubly_linked_list.reverse();
        let actual_values = collect_list_values(&doubly_linked_list);
        let actual_front = doubly_linked_list.front().map(|value| *value);
        let actual_back = doubly_linked_list.back().map(|value| *value);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_front, expected_front);
        assert_eq!(actual_back, expected_back);
    }

    #[test]
    fn reverse_multiple_elements_reverses_iteration_order() {
        // Arrange
        let input_values = vec![1, 2, 3, 4];
        let expected_values = vec![4, 3, 2, 1];
        let expected_len = 4;
        let mut doubly_linked_list = create_list_from_values(&input_values);

        // Act
        doubly_linked_list.reverse();
        let actual_values = collect_list_values(&doubly_linked_list);
        let actual_len = doubly_linked_list.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn reverse_then_pop_front_returns_original_tail() {
        // Arrange
        let input_values = vec![1, 2, 3, 4];
        let expected_popped_value = Some(4);
        let expected_remaining_values = vec![3, 2, 1];
        let mut doubly_linked_list = create_list_from_values(&input_values);

        // Act
        doubly_linked_list.reverse();
        let actual_popped_value = doubly_linked_list.pop_front();
        let actual_remaining_values = collect_list_values(&doubly_linked_list);

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    fn reverse_then_pop_back_returns_original_head() {
        // Arrange
        let input_values = vec![1, 2, 3, 4];
        let expected_popped_value = Some(1);
        let expected_remaining_values = vec![4, 3, 2];
        let mut doubly_linked_list = create_list_from_values(&input_values);

        // Act
        doubly_linked_list.reverse();
        let actual_popped_value = doubly_linked_list.pop_back();
        let actual_remaining_values = collect_list_values(&doubly_linked_list);

        // Assert
        assert_eq!(actual_popped_value, expected_popped_value);
        assert_eq!(actual_remaining_values, expected_remaining_values);
    }

    #[test]
    fn reverse_twice_restores_original_order() {
        // Arrange
        let input_values = vec![1, 2, 3, 4];
        let expected_values = vec![1, 2, 3, 4];
        let mut doubly_linked_list = create_list_from_values(&input_values);

        // Act
        doubly_linked_list.reverse();
        doubly_linked_list.reverse();
        let actual_values = collect_list_values(&doubly_linked_list);

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
