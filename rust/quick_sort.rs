fn partition<T: Ord>(vec: &mut Vec<T>, low: usize, high: usize) -> usize {

    let mut index = low;

    for i in low..high {

        if vec[i] <= vec[high] {
            vec.swap(i, index);
            index += 1;
        }
    }

    vec.swap(index, high);

    index
}

fn _quick_sort<T: Ord>(vec: &mut Vec<T>, low: usize, high: usize) {

    if high <= low {
        return;
    }

    let partition_index = partition(vec, low, high);
    if 0 < partition_index {
        _quick_sort(vec, low, partition_index - 1);
    }

    _quick_sort(vec, partition_index + 1, high);
}

fn quick_sort<T: Ord>(vec: &mut Vec<T>) {
    if let Some(high) = vec.len().checked_sub(1) {
        _quick_sort(vec, 0, high);
    }
}

#[cfg(test)]
mod tests {
    use super::quick_sort;

    #[test]
    fn unordered_values_are_sorted_in_ascending_order() {
        // Arrange
        let mut values = vec![3, 1, 2];
        let expected_values = vec![1, 2, 3];

        // Act
        quick_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn duplicate_values_keep_their_counts_and_are_sorted() {
        // Arrange
        let mut values = vec![2, 2, 1, 1];
        let expected_values = vec![1, 1, 2, 2];

        // Act
        quick_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn empty_vector_is_unchanged() {
        // Arrange
        let mut values: Vec<i32> = vec![];
        let expected_values: Vec<i32> = vec![];

        // Act
        quick_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn single_element_vector_is_unchanged() {
        // Arrange
        let mut values = vec![1];
        let expected_values = vec![1];

        // Act
        quick_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn descending_input_is_sorted() {
        // Arrange
        let mut values = vec![5, 4, 3, 2, 1];
        let expected_values = vec![1, 2, 3, 4, 5];

        // Act
        quick_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn all_equal_values_keep_length_and_values() {
        // Arrange
        let mut values = vec![7, 7, 7, 7];
        let expected_values = vec![7, 7, 7, 7];

        // Act
        quick_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }
}
