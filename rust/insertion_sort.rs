fn insertion_sort<T: Ord>(vec: &mut Vec<T>) {

    for i in 1..vec.len() {
        let mut j = i;
        while 0 < j && vec[j] < vec[j - 1] {
            vec.swap(j, j - 1);
            j -= 1;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::insertion_sort;

    #[test]
    fn insertion_sort_empty_vec_remains_empty() {
        // Arrange
        let mut values: Vec<i32> = vec![];
        let expected_values: Vec<i32> = vec![];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn insertion_sort_single_element_remains_unchanged() {
        // Arrange
        let mut values = vec![42];
        let expected_values = vec![42];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn insertion_sort_sorts_two_elements_in_reverse_order() {
        // Arrange
        let mut values = vec![2, 1];
        let expected_values = vec![1, 2];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn insertion_sort_sorts_multiple_elements_with_duplicates() {
        // Arrange
        let mut values = vec![3, 1, 2, 1, 4];
        let expected_values = vec![1, 1, 2, 3, 4];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn already_sorted_input_is_preserved() {
        // Arrange
        let mut values = vec![1, 2, 3, 4];
        let expected_values = vec![1, 2, 3, 4];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn descending_input_of_three_or_more_elements_is_sorted() {
        // Arrange
        let mut values = vec![5, 4, 3, 2, 1];
        let expected_values = vec![1, 2, 3, 4, 5];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn non_integer_ord_types_can_be_sorted() {
        // Arrange
        let mut values = vec!["bb".to_string(), "aa".to_string(), "cc".to_string()];
        let expected_values = vec!["aa".to_string(), "bb".to_string(), "cc".to_string()];

        // Act
        insertion_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }
}
