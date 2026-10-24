use std::cell::{Ref, RefCell, RefMut};
use std::fmt;
use std::iter::FusedIterator;
use std::marker::PhantomData;
use std::rc::{Rc, Weak};

#[derive(Debug)]
pub struct Node<T: Clone + fmt::Debug + fmt::Display> {
    pub data: T,
    pub next: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: Clone + fmt::Debug + fmt::Display> Node<T> {

    pub fn new(data: T) -> Rc<RefCell<Self>> {
        Rc::new(
            RefCell::new(
                Self {
                    data,
                    next: None,
                }
            )
        )
    }

    pub fn data(&self) -> &T {
        &self.data
    }
}

pub struct Iter<'a, T: Clone + fmt::Debug + fmt::Display> {
    current: Option<Weak<RefCell<Node<T>>>>,
    remaining: usize,
    marker: PhantomData<&'a LinkedList<T>>,
}

impl<T: Clone + fmt::Debug + fmt::Display> Iterator for Iter<'_, T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        let current_node = self
            .current
            .as_ref()?
            .upgrade()
            .expect("the borrowed list owns the current node");
        let current_node = current_node.borrow();
        let value = current_node.data.clone();
        self.current = current_node.next.as_ref().map(Rc::downgrade);
        self.remaining -= 1;
        Some(value)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.remaining, Some(self.remaining))
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> ExactSizeIterator for Iter<'_, T> {}

impl<T: Clone + fmt::Debug + fmt::Display> FusedIterator for Iter<'_, T> {}

pub struct IntoIter<T: Clone + fmt::Debug + fmt::Display> {
    list: LinkedList<T>,
}

impl<T: Clone + fmt::Debug + fmt::Display> Iterator for IntoIter<T> {
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        self.list.pop()
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.list.len();
        (remaining, Some(remaining))
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> ExactSizeIterator for IntoIter<T> {}

impl<T: Clone + fmt::Debug + fmt::Display> FusedIterator for IntoIter<T> {}

pub struct LinkedList<T: Clone + fmt::Debug + fmt::Display> {
    size: usize,
    head: Option<Rc<RefCell<Node<T>>>>,
}

impl<T: Clone + fmt::Debug + fmt::Display> fmt::Debug for LinkedList<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut debug_list = f.debug_list();
        let mut current = self.head.clone();

        while let Some(node) = current {
            let node = node.borrow();
            debug_list.entry(&node.data);
            current = node.next.clone();
        }

        debug_list.finish()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> Default for LinkedList<T> {
    fn default() -> Self {
        Self::new()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> IntoIterator for LinkedList<T> {
    type Item = T;
    type IntoIter = IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        IntoIter { list: self }
    }
}

impl<'a, T: Clone + fmt::Debug + fmt::Display> IntoIterator for &'a LinkedList<T> {
    type Item = T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<T: Clone + fmt::Debug + fmt::Display> LinkedList<T> {

    pub fn new() -> Self {
        Self {
            size: 0,
            head: None,
        }
    }

    pub fn iter(&self) -> Iter<'_, T> {
        Iter {
            current: self.head.as_ref().map(Rc::downgrade),
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

    pub fn push(&mut self, data: T) {

        self.size += 1;
        let new_node = Node::new(data);

        if self.head.is_none() {
            self.head = Some(new_node);
            return;
        }

        let mut current = self.head.clone();

        while let Some(current_vertex) = current.as_ref() {

            let next = current_vertex.borrow().next.clone();
            match next {
                Some(_) => {
                    current = next;
                },
                None => {
                    current_vertex.borrow_mut().next = Some(new_node);
                    break;
                },
            }
        }
    }

    pub fn pop(&mut self) -> Option<T> {

        self.head.take().map(|head_vertex| {

            self.size -= 1;

            if let Some(second_head_vertex) = head_vertex.borrow().next.clone() {
                self.head = Some(second_head_vertex);
            }

            Rc::into_inner(head_vertex).unwrap().into_inner().data
        })
    }

    pub fn reverse(&mut self) {

        let mut previous = None;
        let mut current = self.head.clone();

        while let Some(current_vertex) = current {

            let next = current_vertex.borrow_mut().next.take();
            current_vertex.borrow_mut().next = previous;

            previous = Some(current_vertex);
            current = next;
        }

        self.head = previous;
    }

    pub fn print(&self) {

        let mut current = self.head.clone();

        print!("Head");
        while let Some(current_vertex) = current {
            print!(" <=> {}", current_vertex.borrow().data());
            current = current_vertex.borrow().next.clone();
        }
        println!(" <=> tail");
    }
}

#[cfg(doctest)]
mod iterator_borrow_tests {
    /// ```
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let values = [1, 2, 3];
    /// let mut list = LinkedList::new();
    /// for value in values {
    ///     list.push(value);
    /// }
    /// let expected_iterated = [Some(1), Some(2)];
    /// let expected_popped = [Some(1), Some(2), Some(3), None];
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// let actual_iterated = [iterator.next(), iterator.next()];
    /// let actual_popped = [list.pop(), list.pop(), list.pop(), list.pop()];
    ///
    /// // Assert
    /// assert_eq!(actual_iterated, expected_iterated);
    /// assert_eq!(actual_popped, expected_popped);
    /// ```
    fn pop_after_iter_last_use_is_allowed() {}

    /// ```compile_fail,E0502
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let value = 1;
    /// let additional_value = 2;
    /// let mut list = LinkedList::new();
    /// list.push(value);
    /// let expected = Some(value);
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// list.push(additional_value);
    /// let actual = iterator.next();
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn push_while_iter_is_used_is_rejected() {}

    /// ```compile_fail,E0502
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let value = 1;
    /// let mut list = LinkedList::new();
    /// list.push(value);
    /// let expected = Some(value);
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// let _popped = list.pop();
    /// let actual = iterator.next();
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn pop_while_iter_is_used_is_rejected() {}

    /// ```compile_fail,E0502
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let values = [1, 2];
    /// let mut list = LinkedList::new();
    /// for value in values {
    ///     list.push(value);
    /// }
    /// let expected = Some(1);
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// list.reverse();
    /// let actual = iterator.next();
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn reverse_while_iter_is_used_is_rejected() {}

    /// ```compile_fail,E0502
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let value = 1;
    /// let mut list = LinkedList::new();
    /// list.push(value);
    /// let expected = Some(value);
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// let _front = list.front_mut();
    /// let actual = iterator.next();
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn front_mut_while_iter_is_used_is_rejected() {}

    /// ```compile_fail,E0505
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let value = 1;
    /// let mut list = LinkedList::new();
    /// list.push(value);
    /// let expected = Some(value);
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// let _owned_iterator = list.into_iter();
    /// let actual = iterator.next();
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn into_iter_while_iter_is_used_is_rejected() {}

    /// ```compile_fail,E0505
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let value = 1;
    /// let mut list = LinkedList::new();
    /// list.push(value);
    /// let expected = Some(value);
    ///
    /// // Act
    /// let mut iterator = list.iter();
    /// drop(list);
    /// let actual = iterator.next();
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn drop_while_iter_is_used_is_rejected() {}

    /// ```compile_fail,E0502
    /// use linked_list::LinkedList;
    ///
    /// // Arrange
    /// let values = [1, 2];
    /// let additional_value = 3;
    /// let mut list = LinkedList::new();
    /// for value in values {
    ///     list.push(value);
    /// }
    /// let expected = values.to_vec();
    ///
    /// // Act
    /// let mut actual = Vec::new();
    /// for value in &list {
    ///     list.push(additional_value);
    ///     actual.push(value);
    /// }
    ///
    /// // Assert
    /// assert_eq!(actual, expected);
    /// ```
    fn push_during_shared_for_loop_is_rejected() {}
}

#[cfg(test)]
mod tests {
    use super::LinkedList;
    use std::cell::Cell;
    use std::fmt::{self, Write};
    use std::iter::FusedIterator;
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::rc::Rc;

    #[derive(Debug)]
    struct ObservedClone {
        value: String,
        clone_count: Rc<Cell<usize>>,
        panic_on_clone: Rc<Cell<bool>>,
    }

    impl Clone for ObservedClone {
        fn clone(&self) -> Self {
            self.clone_count.set(self.clone_count.get() + 1);
            assert!(!self.panic_on_clone.get(), "clone requested to panic");
            Self {
                value: self.value.clone(),
                clone_count: Rc::clone(&self.clone_count),
                panic_on_clone: Rc::clone(&self.panic_on_clone),
            }
        }
    }

    impl fmt::Display for ObservedClone {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    #[derive(Clone, Debug)]
    struct ObservedDrop {
        value: i32,
        drop_count: Rc<Cell<usize>>,
    }

    impl Drop for ObservedDrop {
        fn drop(&mut self) {
            self.drop_count.set(self.drop_count.get() + 1);
        }
    }

    impl fmt::Display for ObservedDrop {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}", self.value)
        }
    }

    struct FailingWriter;

    impl fmt::Write for FailingWriter {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            Err(fmt::Error)
        }
    }

    type IteratorObservation = (usize, (usize, Option<usize>), Option<i32>);

    fn observe_exact_fused_iterator<I>(mut iterator: I) -> Vec<IteratorObservation>
    where
        I: ExactSizeIterator<Item = i32> + FusedIterator,
    {
        let attempts = iterator.len() + 2;
        (0..attempts)
            .map(|_| {
                let remaining = iterator.len();
                let hint = iterator.size_hint();
                let value = iterator.next();
                (remaining, hint, value)
            })
            .collect()
    }

    #[test]
    fn debug_formats_empty_single_and_duplicate_values_as_a_list() {
        // Arrange
        let inputs = [vec![], vec![7], vec![2, 2, 1]];
        let lists: Vec<_> = inputs
            .into_iter()
            .map(|values| {
                let mut list = LinkedList::new();
                for value in values {
                    list.push(value);
                }
                list
            })
            .collect();
        let expected = vec!["[]", "[7]", "[2, 2, 1]"];

        // Act
        let actual = lists
            .iter()
            .map(|list| format!("{list:?}"))
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn debug_formats_strings_with_quotes_and_escaped_characters() {
        // Arrange
        let values = ["plain", "line\n\"quote\"\\end"];
        let mut list = LinkedList::new();
        for value in values {
            list.push(String::from(value));
        }
        let expected = r#"["plain", "line\n\"quote\"\\end"]"#;

        // Act
        let actual = format!("{list:?}");

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn debug_with_alternate_flag_uses_pretty_list_format() {
        // Arrange
        let values = [1, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let expected = "[\n    1,\n    2,\n]";

        // Act
        let actual = format!("{list:#?}");

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn debug_after_pop_and_reverse_formats_current_order() {
        // Arrange
        let values = [4, 1, 3, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let _ = list.pop();
        list.reverse();
        let expected = "[2, 3, 1]";

        // Act
        let actual = format!("{list:?}");

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn debug_preserves_values_without_cloning() {
        // Arrange
        let values = ["first", "second"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(true));
        let mut list = LinkedList::new();
        for value in values {
            list.push(ObservedClone {
                value: String::from(value),
                clone_count: Rc::clone(&clone_count),
                panic_on_clone: Rc::clone(&panic_on_clone),
            });
        }
        let mut output = String::new();
        let expected_result = Ok(());
        let expected_clone_count = 0;
        let expected_len = values.len();
        let expected_values = values.map(String::from).to_vec();

        // Act
        let actual_result = write!(&mut output, "{list:?}");
        let actual_clone_count = clone_count.get();
        let actual_len = list.len();
        let actual_values = list
            .into_iter()
            .map(|value| value.value)
            .collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_clone_count, expected_clone_count);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn debug_propagates_writer_errors() {
        // Arrange
        let values = [1, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let mut writer = FailingWriter;
        let expected = Err(fmt::Error);

        // Act
        let actual = write!(&mut writer, "{list:?}");

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn iter_reports_exact_remaining_length_through_repeated_exhaustion() {
        // Arrange
        let inputs = [vec![], vec![7], vec![2, 2, 1]];
        let lists: Vec<_> = inputs
            .into_iter()
            .map(|values| {
                let mut list = LinkedList::new();
                for value in values {
                    list.push(value);
                }
                list
            })
            .collect();
        let expected = vec![
            vec![(0, (0, Some(0)), None), (0, (0, Some(0)), None)],
            vec![
                (1, (1, Some(1)), Some(7)),
                (0, (0, Some(0)), None),
                (0, (0, Some(0)), None),
            ],
            vec![
                (3, (3, Some(3)), Some(2)),
                (2, (2, Some(2)), Some(2)),
                (1, (1, Some(1)), Some(1)),
                (0, (0, Some(0)), None),
                (0, (0, Some(0)), None),
            ],
        ];

        // Act
        let actual: Vec<_> = lists
            .iter()
            .map(|list| observe_exact_fused_iterator(list.iter()))
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn into_iter_reports_exact_remaining_length_through_repeated_exhaustion() {
        // Arrange
        let inputs = [vec![], vec![7], vec![2, 2, 1]];
        let lists: Vec<_> = inputs
            .into_iter()
            .map(|values| {
                let mut list = LinkedList::new();
                for value in values {
                    list.push(value);
                }
                list
            })
            .collect();
        let expected = vec![
            vec![(0, (0, Some(0)), None), (0, (0, Some(0)), None)],
            vec![
                (1, (1, Some(1)), Some(7)),
                (0, (0, Some(0)), None),
                (0, (0, Some(0)), None),
            ],
            vec![
                (3, (3, Some(3)), Some(2)),
                (2, (2, Some(2)), Some(2)),
                (1, (1, Some(1)), Some(1)),
                (0, (0, Some(0)), None),
                (0, (0, Some(0)), None),
            ],
        ];

        // Act
        let actual: Vec<_> = lists
            .into_iter()
            .map(|list| observe_exact_fused_iterator(list.into_iter()))
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn iter_after_pop_and_reverse_reports_current_order_and_remaining_length() {
        // Arrange
        let values = [4, 1, 3, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let _ = list.pop();
        list.reverse();
        let expected = vec![
            (3, (3, Some(3)), Some(2)),
            (2, (2, Some(2)), Some(3)),
            (1, (1, Some(1)), Some(1)),
            (0, (0, Some(0)), None),
            (0, (0, Some(0)), None),
        ];

        // Act
        let actual = observe_exact_fused_iterator(list.iter());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn into_iter_after_pop_and_reverse_reports_current_order_and_remaining_length() {
        // Arrange
        let values = [4, 1, 3, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let _ = list.pop();
        list.reverse();
        let expected = vec![
            (3, (3, Some(3)), Some(2)),
            (2, (2, Some(2)), Some(3)),
            (1, (1, Some(1)), Some(1)),
            (0, (0, Some(0)), None),
            (0, (0, Some(0)), None),
        ];

        // Act
        let actual = observe_exact_fused_iterator(list.into_iter());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn for_shared_list_yields_owned_values_in_order_and_preserves_list() {
        // Arrange
        let values = ["first", "second", "first"];
        let mut list = LinkedList::new();
        for value in values {
            list.push(String::from(value));
        }
        let expected_values = values.map(String::from).to_vec();
        let expected_len = values.len();

        // Act
        let mut actual_values = Vec::new();
        for value in &list {
            actual_values.push(value);
        }
        let remaining_values = list.iter().collect::<Vec<_>>();
        let actual_len = list.len();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(remaining_values, expected_values);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn into_iter_moves_non_copy_values_without_cloning() {
        // Arrange
        let values = ["first", "second"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(true));
        let mut list = LinkedList::new();
        for value in values {
            list.push(ObservedClone {
                value: String::from(value),
                clone_count: Rc::clone(&clone_count),
                panic_on_clone: Rc::clone(&panic_on_clone),
            });
        }
        let expected_values = values.map(String::from).to_vec();
        let expected_clone_count = 0;

        // Act
        let actual_values = list
            .into_iter()
            .map(|value| value.value)
            .collect::<Vec<_>>();
        let actual_clone_count = clone_count.get();

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_clone_count, expected_clone_count);
    }

    #[test]
    fn dropping_partial_into_iter_releases_only_unconsumed_values() {
        // Arrange
        let values = [10, 20, 30];
        let drop_counts = [
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
            Rc::new(Cell::new(0)),
        ];
        let mut list = LinkedList::new();
        for (value, drop_count) in values.into_iter().zip(&drop_counts) {
            list.push(ObservedDrop {
                value,
                drop_count: Rc::clone(drop_count),
            });
        }
        let expected_first = Some(10);
        let expected_after_next = [0, 0, 0];
        let expected_after_iterator_drop = [0, 1, 1];
        let expected_after_value_drop = [1, 1, 1];

        // Act
        let mut iterator = list.into_iter();
        let first = iterator.next();
        let actual_after_next = drop_counts.each_ref().map(|count| count.get());
        drop(iterator);
        let actual_after_iterator_drop = drop_counts.each_ref().map(|count| count.get());
        let actual_first = first.as_ref().map(|value| value.value);
        drop(first);
        let actual_after_value_drop = drop_counts.each_ref().map(|count| count.get());

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_after_next, expected_after_next);
        assert_eq!(actual_after_iterator_drop, expected_after_iterator_drop);
        assert_eq!(actual_after_value_drop, expected_after_value_drop);
    }

    #[test]
    fn iter_clone_panic_preserves_position_and_remaining_length() {
        // Arrange
        let values = ["first", "second"];
        let clone_count = Rc::new(Cell::new(0));
        let panic_on_clone = Rc::new(Cell::new(true));
        let allow_cloning = false;
        let mut list = LinkedList::new();
        for value in values {
            list.push(ObservedClone {
                value: String::from(value),
                clone_count: Rc::clone(&clone_count),
                panic_on_clone: Rc::clone(&panic_on_clone),
            });
        }
        let expected_panicked = true;
        let expected_remaining = (2, (2, Some(2)));
        let expected_values = values.map(String::from).to_vec();
        let expected_clone_count = 3;

        // Act
        let mut iterator = list.iter();
        let result = catch_unwind(AssertUnwindSafe(|| iterator.next()));
        let actual_panicked = result.is_err();
        let actual_remaining = (iterator.len(), iterator.size_hint());
        panic_on_clone.set(allow_cloning);
        let actual_values = iterator.map(|value| value.value).collect::<Vec<_>>();
        let actual_clone_count = clone_count.get();

        // Assert
        assert_eq!(actual_panicked, expected_panicked);
        assert_eq!(actual_remaining, expected_remaining);
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_clone_count, expected_clone_count);
    }

    #[test]
    fn pop_after_iter_last_use_returns_all_values_without_explicit_drop() {
        // Arrange
        let values = [1, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let expected_first = Some(1);
        let expected_popped = [Some(1), Some(2), None];

        // Act
        let mut iterator = list.iter();
        let actual_first = iterator.next();
        let actual_popped = [list.pop(), list.pop(), list.pop()];

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_popped, expected_popped);
    }

    #[test]
    fn into_iter_after_iter_last_use_moves_all_values_without_explicit_drop() {
        // Arrange
        let values = [1, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let expected_first = Some(1);
        let expected_values = values.to_vec();

        // Act
        let mut iterator = list.iter();
        let actual_first = iterator.next();
        let actual_values = list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn pop_after_forgetting_iter_returns_all_values() {
        // Arrange
        let values = [1, 2];
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }
        let expected_first = Some(1);
        let expected_popped = [Some(1), Some(2), None];

        // Act
        let mut iterator = list.iter();
        let actual_first = iterator.next();
        std::mem::forget(iterator);
        let actual_popped = [list.pop(), list.pop(), list.pop()];

        // Assert
        assert_eq!(actual_first, expected_first);
        assert_eq!(actual_popped, expected_popped);
    }

    #[test]
    fn default_matches_new_empty_state() {
        // Arrange
        let from_new: LinkedList<i32> = LinkedList::new();
        let expected_states = (true, 0, true, 0);

        // Act
        let from_default: LinkedList<i32> = LinkedList::default();
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
    fn into_iterator_consumes_values_in_order() {
        // Arrange
        let mut linked_list = LinkedList::new();
        linked_list.push(1);
        linked_list.push(2);
        linked_list.push(3);
        let expected_values = vec![1, 2, 3];

        // Act
        let actual_values = linked_list.into_iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn iter_on_empty_list_returns_none() {
        // Arrange
        let list = LinkedList::<i32>::new();
        let expected = None;

        // Act
        let actual = list.iter().next();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn iter_with_duplicates_returns_complete_insertion_order() {
        // Arrange
        let values = [1, 1, 2, 3, 4, 5, 0];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let expected = values.to_vec();
        let expected_len = values.len();

        // Act
        let actual = list.iter().collect::<Vec<_>>();
        let actual_len = list.len();

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn iter_repeated_calls_return_complete_sequence_without_consuming_list() {
        // Arrange
        let values = [1, 1, 2, 3, 4, 5, 0];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let expected = values.to_vec();
        let expected_len = values.len();

        // Act
        let actual = list.iter().collect::<Vec<_>>();
        let actual_len = list.len();
        let repeated_actual = list.iter().collect::<Vec<_>>();
        let repeated_len = list.len();

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_len, expected_len);
        assert_eq!(repeated_actual, expected);
        assert_eq!(repeated_len, expected_len);
    }

    #[test]
    fn iter_with_single_element_returns_value_then_none() {
        // Arrange
        let values = [0];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let expected = [Some(0), None];
        let expected_len = 1;

        // Act
        let mut iterator = list.iter();
        let actual = [iterator.next(), iterator.next()];
        let actual_len = list.len();

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn len_on_empty_list_returns_zero() {
        // Arrange
        let list = LinkedList::<i32>::new();
        let expected = 0;

        // Act
        let actual = list.len();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn len_after_each_push_increases_by_one() {
        // Arrange
        let values = [0, 1, 2];
        let expected = vec![1, 2, 3];
        let mut list = LinkedList::new();

        // Act
        let actual: Vec<_> = values
            .into_iter()
            .map(|value| {
                list.push(value);
                list.len()
            })
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn len_after_each_push_and_pop_returns_to_zero() {
        // Arrange
        let values = [0, 1, 2];
        let expected = vec![(Some(0), 0), (Some(1), 0), (Some(2), 0)];
        let mut list = LinkedList::new();

        // Act
        let actual: Vec<_> = values
            .into_iter()
            .map(|value| {
                list.push(value);
                let popped = list.pop();
                (popped, list.len())
            })
            .collect();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_empty_on_new_list_returns_true() {
        // Arrange
        let list = LinkedList::<i32>::new();
        let expected = true;

        // Act
        let actual = list.is_empty();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn is_empty_after_multiple_pushes_returns_false() {
        // Arrange
        let values = [0, 1, 2];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let expected = false;

        // Act
        let actual = list.is_empty();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn pop_repeatedly_on_empty_list_keeps_empty_state() {
        // Arrange
        let mut list = LinkedList::<i32>::new();
        let pop_count = 2000;
        let expected_pops = vec![None; pop_count];
        let expected_empty = true;

        // Act
        let actual_pops: Vec<_> = (0..pop_count).map(|_| list.pop()).collect();
        let actual_empty = list.is_empty();

        // Assert
        assert_eq!(actual_pops, expected_pops);
        assert_eq!(actual_empty, expected_empty);
    }

    #[test]
    fn front_after_each_push_keeps_first_inserted_value() {
        // Arrange
        let values = [0, 1, 2];
        let expected = vec![Some(0); values.len()];
        let expected_len = values.len();
        let mut list = LinkedList::new();

        // Act
        let actual: Vec<_> = values
            .into_iter()
            .map(|value| {
                list.push(value);
                list.front().map(|value| *value)
            })
            .collect();
        let actual_len = list.len();

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_len, expected_len);
    }

    #[test]
    fn front_on_empty_list_returns_none() {
        // Arrange
        let values = [];
        let mut list = LinkedList::<i32>::new();
        for &value in &values {
            list.push(value);
        }
        let expected = (None, 0);

        // Act
        let actual_front = list.front().map(|value| *value);
        let actual = (actual_front, list.len());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn front_on_single_element_returns_that_value() {
        // Arrange
        let values = [0];
        let mut list = LinkedList::<i32>::new();
        for &value in &values {
            list.push(value);
        }
        let expected = (Some(0), 1);

        // Act
        let actual_front = list.front().map(|value| *value);
        let actual = (actual_front, list.len());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn front_mut_repeated_increments_update_head_and_preserve_next_value() {
        // Arrange
        let values = [0, -1];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let increments = [1, 1, 1];
        let expected_observed = vec![Some(0), Some(1), Some(2)];
        let expected_len = values.len();
        let expected_popped = [Some(3), Some(-1)];

        // Act
        let mut actual_observed = Vec::new();
        for increment in increments {
            let mut front = list.front_mut().unwrap();
            actual_observed.push(Some(*front));
            *front += increment;
        }
        let actual_len = list.len();
        let actual_popped = [list.pop(), list.pop()];

        // Assert
        assert_eq!(actual_observed, expected_observed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_popped, expected_popped);
    }

    #[test]
    fn front_mut_on_empty_list_returns_none() {
        // Arrange
        let values = [];
        let mut list = LinkedList::<i32>::new();
        for &value in &values {
            list.push(value);
        }
        let expected = (None, 0);

        // Act
        let actual_front = list.front_mut().map(|value| *value);
        let actual = (actual_front, list.len());

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn front_mut_repeated_increments_update_single_element() {
        // Arrange
        let values = [0];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let increments = [1, 1, 1];
        let expected_observed = vec![Some(0), Some(1), Some(2)];
        let expected_len = values.len();
        let expected_popped = [Some(3), None];

        // Act
        let mut actual_observed = Vec::new();
        for increment in increments {
            let mut front = list.front_mut().unwrap();
            actual_observed.push(Some(*front));
            *front += increment;
        }
        let actual_len = list.len();
        let actual_popped = [list.pop(), list.pop()];

        // Assert
        assert_eq!(actual_observed, expected_observed);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_popped, expected_popped);
    }

    #[test]
    fn front_mut_after_guard_drop_changes_front_iter_and_pop_results() {
        // Arrange
        let values = [1, 2];
        let mut linked_list = LinkedList::new();
        for value in values {
            linked_list.push(value);
        }
        let updated_value = 10;
        let expected_values = (Some(10), Some(10), Some(10));

        // Act
        let mut front = linked_list.front_mut().unwrap();
        *front = updated_value;
        drop(front);
        let actual_front = linked_list.front().map(|value| *value);
        let actual_iter_first = linked_list.iter().next();
        let actual_pop_first = linked_list.pop();
        let actual_values = (actual_front, actual_iter_first, actual_pop_first);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn front_with_multiple_live_guards_returns_same_value() {
        // Arrange
        let value = 1;
        let mut list = LinkedList::new();
        list.push(value);
        let expected = (value, value);

        // Act
        let first_front = list.front().unwrap();
        let second_front = list.front().unwrap();
        let actual = (*first_front, *second_front);

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn front_guard_blocks_mutable_borrow_until_dropped() {
        // Arrange
        let value = 1;
        let mut list = LinkedList::new();
        list.push(value);
        let head = list.head.as_ref().unwrap().clone();
        let expected_borrow_states = (true, true);

        // Act
        let front = list.front().unwrap();
        let blocked_while_held = head.try_borrow_mut().is_err();
        drop(front);
        let allowed_after_drop = head.try_borrow_mut().is_ok();
        let actual_borrow_states = (blocked_while_held, allowed_after_drop);

        // Assert
        assert_eq!(actual_borrow_states, expected_borrow_states);
    }

    #[test]
    fn front_mut_guard_blocks_other_borrows_until_dropped() {
        // Arrange
        let value = 1;
        let mut list = LinkedList::new();
        list.push(value);
        let head = list.head.as_ref().unwrap().clone();
        let expected_borrow_states = (true, true, true, true);

        // Act
        let front = list.front_mut().unwrap();
        let shared_blocked_while_held = head.try_borrow().is_err();
        let mutable_blocked_while_held = head.try_borrow_mut().is_err();
        drop(front);
        let shared_allowed_after_drop = head.try_borrow().is_ok();
        let mutable_allowed_after_drop = head.try_borrow_mut().is_ok();
        let actual_borrow_states = (
            shared_blocked_while_held,
            mutable_blocked_while_held,
            shared_allowed_after_drop,
            mutable_allowed_after_drop,
        );

        // Assert
        assert_eq!(actual_borrow_states, expected_borrow_states);
    }

    #[test]
    fn front_mut_string_append_updates_owned_value() {
        // Arrange
        let value = String::from("head");
        let suffix = " updated";
        let mut list = LinkedList::new();
        list.push(value);
        let expected = Some(String::from("head updated"));

        // Act
        let mut front = list.front_mut().unwrap();
        front.push_str(suffix);
        drop(front);
        let actual = list.pop();

        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn push_multiple_values_preserves_fifo_order_and_exhausts_list() {
        // Arrange
        let values = [0, 1, 2];
        let expected = [Some(0), Some(1), Some(2), None];
        let expected_len = values.len();
        let expected_empty = true;
        let mut list = LinkedList::new();

        // Act
        for value in values {
            list.push(value);
        }
        let actual_len = list.len();
        let actual = [list.pop(), list.pop(), list.pop(), list.pop()];
        let actual_empty = list.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual, expected);
        assert_eq!(actual_empty, expected_empty);
    }

    #[test]
    fn pop_multiple_values_preserves_fifo_order_and_exhausts_list() {
        // Arrange
        let values = [0, 1, 2];
        let expected = [Some(0), Some(1), Some(2), None];
        let expected_len = values.len();
        let expected_empty = true;
        let mut list = LinkedList::new();
        for value in values {
            list.push(value);
        }

        // Act
        let actual_len = list.len();
        let actual = [list.pop(), list.pop(), list.pop(), list.pop()];
        let actual_empty = list.is_empty();

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual, expected);
        assert_eq!(actual_empty, expected_empty);
    }

    #[test]
    fn pop_on_empty_list_returns_none_and_keeps_empty() {
        // Arrange
        let values = [];
        let mut list = LinkedList::<i32>::new();
        for &value in &values {
            list.push(value);
        }
        let expected = [None, None];
        let expected_len = 0;
        let expected_empty = true;

        // Act
        let actual = [list.pop(), list.pop()];
        let actual_len = list.len();
        let actual_empty = list.is_empty();

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_empty, expected_empty);
    }

    #[test]
    fn pop_on_single_element_returns_value_then_none() {
        // Arrange
        let values = [1];
        let mut list = LinkedList::<i32>::new();
        for &value in &values {
            list.push(value);
        }
        let expected = [Some(1), None];
        let expected_len = 0;
        let expected_empty = true;

        // Act
        let actual = [list.pop(), list.pop()];
        let actual_len = list.len();
        let actual_empty = list.is_empty();

        // Assert
        assert_eq!(actual, expected);
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual_empty, expected_empty);
    }

    #[test]
    fn reverse_multiple_values_reverses_pop_order_and_preserves_length() {
        // Arrange
        let values = [0, 1, 2];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let expected = [Some(2), Some(1), Some(0)];
        let expected_len = values.len();

        // Act
        list.reverse();
        let actual_len = list.len();
        let actual = [list.pop(), list.pop(), list.pop()];

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual, expected);
    }

    #[test]
    fn reverse_empty_list_keeps_zero_length_and_pops_none() {
        // Arrange
        let mut list = LinkedList::<i32>::new();
        let expected_len = 0;
        let expected = [None, None];

        // Act
        list.reverse();
        let actual_len = list.len();
        let actual = [list.pop(), list.pop()];

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual, expected);
    }

    #[test]
    fn reverse_single_element_preserves_front_and_length() {
        // Arrange
        let values = [1];
        let mut list = LinkedList::new();
        for &value in &values {
            list.push(value);
        }
        let expected_len = 1;
        let expected = [Some(1), Some(1)];

        // Act
        list.reverse();
        let actual_len = list.len();
        let actual = [
            list.front().map(|value| *value),
            list.front().map(|value| *value),
        ];

        // Assert
        assert_eq!(actual_len, expected_len);
        assert_eq!(actual, expected);
    }

    #[test]
    fn reverse_after_partial_pop_keeps_remaining_order_reversed() {
        // Arrange
        let mut linked_list = LinkedList::new();
        linked_list.push(1);
        linked_list.push(2);
        linked_list.push(3);
        linked_list.push(4);
        linked_list.pop();
        let expected_values = vec![4, 3, 2];

        // Act
        linked_list.reverse();
        let actual_values = linked_list.iter().collect::<Vec<_>>();

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
