fn shell_sort<T: Ord>(vec: &mut Vec<T>) {

    let mut gap = vec.len() / 2;

    while 0 < gap {

        for i in 0..vec.len() {

            for j in (gap..=i).rev() {
                if vec[j] < vec[j - gap] {
                    vec.swap(j, j - gap);
                } else {
                    break;
                }
            }
        }

        gap /= 2;
    }
}

#[cfg(test)]
mod tests {
    use super::shell_sort;

    #[test]
    fn sorts_even_and_odd_lengths() {
        // Arrange
        let mut even = vec![4, 2, 3, 1];
        let expected_even = vec![1, 2, 3, 4];
        let mut odd = vec![3, 1, 2];
        let expected_odd = vec![1, 2, 3];

        // Act
        shell_sort(&mut even);
        shell_sort(&mut odd);

        // Assert
        assert_eq!(even, expected_even);
        assert_eq!(odd, expected_odd);
    }

    #[test]
    fn empty_vector_finishes_without_changes() {
        // Arrange
        let mut values: Vec<i32> = vec![];
        let expected_values: Vec<i32> = vec![];

        // Act
        shell_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn single_element_vector_is_preserved() {
        // Arrange
        let mut values = vec![1];
        let expected_values = vec![1];

        // Act
        shell_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }

    #[test]
    fn descending_input_with_duplicates_is_sorted() {
        // Arrange
        let mut values = vec![5, 4, 4, 3, 2, 1];
        let expected_values = vec![1, 2, 3, 4, 4, 5];

        // Act
        shell_sort(&mut values);

        // Assert
        assert_eq!(values, expected_values);
    }
}
