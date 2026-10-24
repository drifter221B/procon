fn gnome_sort<T: Ord>(vec: &mut Vec<T>) {

    for i in 0..vec.len() {
        for j in (0..i).rev() {
            if vec[j + 1] < vec[j] {
                vec.swap(j, j + 1);
            } else {
                break;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::gnome_sort;

    #[test]
    fn empty_input_remains_empty() {
        // Arrange
        let mut empty: Vec<i32> = vec![];

        // Act
        gnome_sort(&mut empty);
        let actual_values = empty;

        // Assert
        assert_eq!(actual_values, Vec::<i32>::new());
    }

    #[test]
    fn duplicate_values_are_sorted_in_ascending_order() {
        // Arrange
        let mut dup = vec![2, 1, 2, 1];
        let expected_values = vec![1, 1, 2, 2];

        // Act
        gnome_sort(&mut dup);

        // Assert
        assert_eq!(dup, expected_values);
    }

    #[test]
    fn single_element_vector_is_unchanged() {
        // Arrange
        let mut values = vec![1];
        let expected_values = vec![1];

        // Act
        gnome_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn already_sorted_input_stays_sorted() {
        // Arrange
        let mut values = vec![1, 2, 3, 4];
        let expected_values = vec![1, 2, 3, 4];

        // Act
        gnome_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn descending_input_becomes_ascending() {
        // Arrange
        let mut values = vec![4, 3, 2, 1];
        let expected_values = vec![1, 2, 3, 4];

        // Act
        gnome_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }
}
