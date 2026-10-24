use std::fmt::Display;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Matrix<T> {
    row: usize,
    col: usize,
    data: Vec<Vec<T>>,
}

impl<T: Clone> Matrix<T> {

    pub fn new(row: usize, col: usize, initial_value: T) -> Self {

        let data = (0..row).map(|_| vec![initial_value.clone(); col]).collect();

        let col_len = if row == 0 { 0 } else { col };

        Self { row, col: col_len, data }
    }

    pub fn from_vec(data: Vec<Vec<T>>) -> Self {

        if data.is_empty() {
            return Self { row: 0, col: 0, data };
        }

        let col = data[0].len();
        assert!(data.iter().all(|line| line.len() == col), "matrix is jagged");

        let row = data.len();

        Self { row, col, data }
    }

    pub fn rows(&self) -> usize {
        self.row
    }

    pub fn cols(&self) -> usize {
        self.col
    }

    pub fn as_slice(&self) -> &[Vec<T>] {
        &self.data
    }

    pub fn transpose_and_reverse_rows(&self) -> Self {

        if self.row == 0 {
            return Self::from_vec(vec![]);
        }

        let mut transposed = (0..self.col)
            .map(|c| (0..self.row).map(|r| self.data[r][c].clone()).collect::<Vec<T>>())
            .collect::<Vec<Vec<T>>>();

        for line in transposed.iter_mut() {
            line.reverse();
        }

        Self::from_vec(transposed)
    }

    pub fn rotate_left(&self) -> Self {

        if self.row == 0 {
            return Self::from_vec(vec![]);
        }

        let mut rotated: Vec<Vec<T>> = (0..self.col).map(|_| Vec::with_capacity(self.row)).collect();

        for r in 0..self.row {

            for c in 0..self.col {
                let target_row = self.col - 1 - c;
                rotated[target_row].push(self.data[r][c].clone());
            }
        }

        Self::from_vec(rotated)
    }

    pub fn rotate_right(&self) -> Self {

        if self.row == 0 {
            return Self::from_vec(vec![]);
        }

        let rotated = (0..self.col)
            .map(|c| {
                let mut new_row = Vec::with_capacity(self.row);

                for r in (0..self.row).rev() {
                    new_row.push(self.data[r][c].clone());
                }

                new_row
            })
            .collect::<Vec<Vec<T>>>();

        Self::from_vec(rotated)
    }

    pub fn bottom_trim(&self, empty_value: &T) -> Self
    where
        T: PartialEq,
    {

        let mut trimmed = self.data.clone();

        while let Some(last_row) = trimmed.last() {

            if last_row.iter().all(|value| value == empty_value) {
                trimmed.pop();
            } else {
                break;
            }
        }

        Self::from_vec(trimmed)
    }

    pub fn copy(&self) -> Self {
        self.clone()
    }

    pub fn print(&self)
    where
        T: Display,
    {
        for line in self.data.iter() {
            let joined = line.iter().map(|v| v.to_string()).collect::<Vec<String>>().join(" ");
            println!("{}", joined);
        }
    }

    pub fn is_mergeable_into(&self, larger: &Matrix<T>, sx: usize, sy: usize, non_empty_signs: &[T]) -> bool
    where
        T: PartialEq,
    {

        if self.row == 0 {
            return true;
        }

        if larger.row < sx + self.row {
            return false;
        }

        if larger.col < sy + self.col {
            return false;
        }

        for i in 0..self.row {

            for j in 0..self.col {

                let value = &self.data[i][j];
                if !non_empty_signs.iter().any(|sign| sign == value) {
                    continue;
                }

                let target = &larger.data[i + sx][j + sy];
                if non_empty_signs.iter().any(|sign| sign == target) {
                    return false;
                }
            }
        }

        true
    }

    pub fn merge_into(&self, larger: &Matrix<T>, sx: usize, sy: usize, non_empty_signs: &[T], assume_mergeable: bool) -> Matrix<T>
    where
        T: Clone + PartialEq,
    {

        assert!(assume_mergeable || self.is_mergeable_into(larger, sx, sy, non_empty_signs), "matrices are not mergeable");

        let mut merged = larger.data.clone();

        for i in 0..self.row {

            for j in 0..self.col {

                if non_empty_signs.iter().any(|sign| sign == &self.data[i][j]) {
                    merged[i + sx][j + sy] = self.data[i][j].clone();
                }
            }
        }

        Matrix::from_vec(merged)
    }
}

impl Matrix<i64> {

    pub fn multiply_mod(&self, other: &Matrix<i64>, modulo: i64) -> Matrix<i64> {

        assert!(1 <= modulo, "modulo must be positive");
        assert!(self.col == other.row, "matrix dimensions do not match for multiplication");

        let mut result = vec![vec![0_i64; other.col]; self.row];
        let modulo_i128 = modulo as i128;

        for i in 0..self.row {

            for k in 0..self.col {

                let left_value = self.data[i][k] as i128;

                for j in 0..other.col {

                    let product = left_value * other.data[k][j] as i128;
                    let next = ((result[i][j] as i128 + product) % modulo_i128 + modulo_i128) % modulo_i128;
                    result[i][j] = next as i64;
                }
            }
        }

        Matrix::from_vec(result)
    }

    pub fn power_mod(&self, mut power: u64, modulo: i64) -> Matrix<i64> {

        assert!(1 <= modulo, "modulo must be positive");
        assert!(self.row == self.col, "matrix is not square");

        if power == 0 {
            return Matrix::identity(self.row);
        }

        let mut result = Matrix::identity(self.row);
        let mut base = self.clone();

        while 0 < power {

            if power & 1 == 1 {
                result = result.multiply_mod(&base, modulo);
            }

            base = base.multiply_mod(&base, modulo);
            power >>= 1;
        }

        result
    }

    pub fn sum_row(&self, row: usize, modulo: Option<i64>) -> i64 {

        assert!(row < self.row, "row index is out of range");

        let modulo_checked = modulo.map(|m| {
            assert!(1 <= m, "modulo must be positive");
            m
        });

        let sum = self.data[row].iter().fold(0_i128, |acc, &v| acc + v as i128);

        modulo_checked
            .map(|m| ((sum % m as i128 + m as i128) % m as i128) as i64)
            .unwrap_or_else(|| sum as i64)
    }

    pub fn sum_all_rows(&self, modulo: Option<i64>) -> i64 {

        let modulo_checked = modulo.map(|m| {
            assert!(1 <= m, "modulo must be positive");
            m
        });

        let total = self.data.iter().fold(0_i128, |outer_acc, row| {

            let row_sum = row.iter().fold(0_i128, |inner_acc, &v| inner_acc + v as i128);

            match modulo_checked {
                Some(m) => outer_acc + (row_sum % m as i128 + m as i128) % m as i128,
                None => outer_acc + row_sum,
            }
        });

        match modulo_checked {
            Some(m) => ((total % m as i128 + m as i128) % m as i128) as i64,
            None => total as i64,
        }
    }

    fn identity(size: usize) -> Matrix<i64> {

        let data = (0..size)
            .map(|i| {
                (0..size)
                    .map(|j| if i == j { 1_i64 } else { 0_i64 })
                    .collect::<Vec<i64>>()
            })
            .collect::<Vec<Vec<i64>>>();

        Matrix::from_vec(data)
    }
}

pub fn is_matrix_2d<T>(matrix: &[Vec<T>]) -> bool {

    if matrix.is_empty() {
        return false;
    }

    let col_len = matrix[0].len();

    matrix.iter().all(|row| row.len() == col_len)
}

#[cfg(test)]
mod tests {
    use super::{is_matrix_2d, Matrix};
    use std::panic::{catch_unwind, AssertUnwindSafe};
    use std::process::Command;

    fn run_print_probe(mode: &str) -> std::process::Output {
        let current_executable_path =
            std::env::current_exe().expect("failed to resolve current executable path");

        Command::new(current_executable_path)
            .arg("--nocapture")
            .arg("--exact")
            .arg("tests::print_probe_matrix_output")
            .env("MATRIX_PRINT_PROBE_MODE", mode)
            .output()
            .expect("failed to execute print probe subprocess")
    }

    fn extract_probe_payload(stdout: &str) -> String {
        let payload_lines = stdout
            .lines()
            .filter(|line| {
                let trimmed_line = line.trim();
                !trimmed_line.is_empty()
                    && !line.starts_with("running ")
                    && !line.starts_with("test ")
                    && !line.starts_with("test result:")
            })
            .collect::<Vec<&str>>();

        if payload_lines.is_empty() {
            return String::new();
        }

        format!("{}\n", payload_lines.join("\n"))
    }

    fn matrix_i32(data: Vec<Vec<i32>>) -> Matrix<i32> {
        Matrix::from_vec(data)
    }

    fn matrix_i64(data: Vec<Vec<i64>>) -> Matrix<i64> {
        Matrix::from_vec(data)
    }

    #[test]
    fn matrix_new_when_zero_zero_returns_empty_dimensions() {
        // Arrange
        let row_value = 0_usize;
        let col_value = 0_usize;
        let initial_value = 7_i32;

        let expected_matrix_rows_right = 0;
        let expected_matrix_cols_right = 0;

        // Act
        let actual_matrix = Matrix::new(row_value, col_value, initial_value);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();

        let actual_condition_1 = actual_matrix.as_slice().is_empty();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
        assert!(actual_condition_1);
    }

    #[test]
    fn matrix_new_when_zero_row_and_positive_col_sets_col_to_zero() {
        // Arrange
        let row_value = 0_usize;
        let col_value = 3_usize;
        let initial_value = 9_i32;

        let expected_matrix_rows_right = 0;
        let expected_matrix_cols_right = 0;

        // Act
        let actual_matrix = Matrix::new(row_value, col_value, initial_value);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
    }

    #[test]
    fn matrix_new_when_positive_row_and_zero_col_creates_empty_each_row() {
        // Arrange
        let row_value = 3_usize;
        let col_value = 0_usize;
        let initial_value = 5_i32;

        let expected_matrix_rows_right = 3;
        let expected_matrix_cols_right = 0;
        let expected_matrix_as_slice_right = &vec![vec![], vec![], vec![]];

        // Act
        let actual_matrix = Matrix::new(row_value, col_value, initial_value);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();
        let actual_matrix_as_slice = actual_matrix.as_slice();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
        assert_eq!(actual_matrix_as_slice, expected_matrix_as_slice_right);
    }

    #[test]
    fn matrix_new_when_non_zero_initializes_all_cells() {
        // Arrange
        let row_value = 2_usize;
        let col_value = 3_usize;
        let initial_value = 7_i32;

        let expected_matrix_rows_right = 2;
        let expected_matrix_cols_right = 3;
        let expected_matrix_as_slice_right = &vec![vec![7, 7, 7], vec![7, 7, 7]];

        // Act
        let actual_matrix = Matrix::new(row_value, col_value, initial_value);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();
        let actual_matrix_as_slice = actual_matrix.as_slice();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
        assert_eq!(actual_matrix_as_slice, expected_matrix_as_slice_right);
    }

    #[test]
    fn matrix_from_vec_when_empty_returns_zero_dimensions() {
        // Arrange
        let input_data: Vec<Vec<i32>> = vec![];

        let expected_matrix_rows_right = 0;
        let expected_matrix_cols_right = 0;

        // Act
        let actual_matrix = Matrix::from_vec(input_data);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();

        let actual_condition_1 = actual_matrix.as_slice().is_empty();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
        assert!(actual_condition_1);
    }

    #[test]
    fn matrix_from_vec_when_single_row_sets_dimensions() {
        // Arrange
        let input_data = vec![vec![1, 2, 3]];

        let expected_matrix_rows_right = 1;
        let expected_matrix_cols_right = 3;

        // Act
        let actual_matrix = Matrix::from_vec(input_data);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
    }

    #[test]
    fn matrix_from_vec_when_multi_row_preserves_data_order() {
        // Arrange
        let input_data = vec![vec![1, 2], vec![3, 4]];

        // Act
        let actual_matrix = Matrix::from_vec(input_data.clone());

        let actual_matrix_as_slice = actual_matrix.as_slice();

        // Assert
        assert_eq!(actual_matrix_as_slice, &input_data);
    }

    #[test]
    fn matrix_from_vec_when_all_rows_empty_sets_col_zero() {
        // Arrange
        let input_data: Vec<Vec<i32>> = vec![vec![], vec![]];

        let expected_matrix_rows_right = 2;
        let expected_matrix_cols_right = 0;

        // Act
        let actual_matrix = Matrix::from_vec(input_data);

        let actual_matrix_rows = actual_matrix.rows();
        let actual_matrix_cols = actual_matrix.cols();

        // Assert
        assert_eq!(actual_matrix_rows, expected_matrix_rows_right);
        assert_eq!(actual_matrix_cols, expected_matrix_cols_right);
    }

    #[test]
    fn matrix_from_vec_when_jagged_panics() {
        // Arrange
        let input_data = vec![vec![1], vec![2, 3]];

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = Matrix::from_vec(input_data);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn transpose_and_reverse_rows_when_empty_returns_empty() {
        // Arrange
        let source_matrix: Matrix<i32> = Matrix::from_vec(vec![]);

        // Act
        let actual_matrix = source_matrix.transpose_and_reverse_rows();

        let actual_matrix_right = Matrix::from_vec(vec![]);

        // Assert
        assert_eq!(actual_matrix, actual_matrix_right);
    }

    #[test]
    fn transpose_and_reverse_rows_when_single_cell_preserves_value() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![5]]);

        // Act
        let actual_matrix = source_matrix.transpose_and_reverse_rows();

        let actual_matrix_right = matrix_i32(vec![vec![5]]);

        // Assert
        assert_eq!(actual_matrix, actual_matrix_right);
    }

    #[test]
    fn transpose_and_reverse_rows_when_two_by_three_returns_expected_three_by_two() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2, 3], vec![4, 5, 6]]);

        // Act
        let actual_matrix = source_matrix.transpose_and_reverse_rows();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![4, 1], vec![5, 2], vec![6, 3]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn transpose_and_reverse_rows_when_one_by_n_returns_n_by_one() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![10, 20, 30]]);

        // Act
        let actual_matrix = source_matrix.transpose_and_reverse_rows();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![10], vec![20], vec![30]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn transpose_and_reverse_rows_when_n_by_one_returns_one_by_n() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1], vec![2], vec![3]]);

        // Act
        let actual_matrix = source_matrix.transpose_and_reverse_rows();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![3, 2, 1]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn transpose_and_reverse_rows_when_rows_positive_and_cols_zero_returns_zero_by_zero_current_behavior() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![], vec![]]);

        // Act
        let actual_matrix = source_matrix.transpose_and_reverse_rows();

        // Assert
        let expected_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_left_when_rows_positive_and_cols_zero_returns_zero_by_zero_current_behavior() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![], vec![]]);

        // Act
        let actual_matrix = source_matrix.rotate_left();

        // Assert
        let expected_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_left_when_empty_returns_empty() {
        // Arrange
        let source_matrix: Matrix<i32> = Matrix::from_vec(vec![]);

        // Act
        let actual_matrix = source_matrix.rotate_left();

        let actual_matrix_right = Matrix::from_vec(vec![]);

        // Assert
        assert_eq!(actual_matrix, actual_matrix_right);
    }

    #[test]
    fn rotate_left_when_two_by_three_returns_counter_clockwise_rotation() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2, 3], vec![4, 5, 6]]);

        // Act
        let actual_matrix = source_matrix.rotate_left();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![3, 6], vec![2, 5], vec![1, 4]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_left_when_one_by_n_returns_n_by_one() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2, 3]]);

        // Act
        let actual_matrix = source_matrix.rotate_left();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![3], vec![2], vec![1]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_left_when_n_by_one_returns_one_by_n() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1], vec![2], vec![3]]);

        // Act
        let actual_matrix = source_matrix.rotate_left();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![1, 2, 3]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_right_when_rows_positive_and_cols_zero_returns_zero_by_zero_current_behavior() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![], vec![]]);

        // Act
        let actual_matrix = source_matrix.rotate_right();

        // Assert
        let expected_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_right_when_empty_returns_empty() {
        // Arrange
        let source_matrix: Matrix<i32> = Matrix::from_vec(vec![]);

        // Act
        let actual_matrix = source_matrix.rotate_right();

        let actual_matrix_right = Matrix::from_vec(vec![]);

        // Assert
        assert_eq!(actual_matrix, actual_matrix_right);
    }

    #[test]
    fn rotate_right_when_two_by_three_returns_clockwise_rotation() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2, 3], vec![4, 5, 6]]);

        // Act
        let actual_matrix = source_matrix.rotate_right();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![4, 1], vec![5, 2], vec![6, 3]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_right_when_one_by_n_returns_n_by_one() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2, 3]]);

        // Act
        let actual_matrix = source_matrix.rotate_right();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![1], vec![2], vec![3]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn rotate_right_when_n_by_one_returns_one_by_n() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1], vec![2], vec![3]]);

        // Act
        let actual_matrix = source_matrix.rotate_right();

        // Assert
        let expected_matrix = matrix_i32(vec![vec![3, 2, 1]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn bottom_trim_when_multiple_trailing_empty_rows_removes_all_trailing_empty_rows() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 0], vec![0, 0], vec![0, 0]]);
        let empty_value = 0_i32;

        // Act
        let actual_matrix = source_matrix.bottom_trim(&empty_value);

        // Assert
        let expected_matrix = matrix_i32(vec![vec![1, 0]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn bottom_trim_when_all_rows_empty_returns_zero_by_zero() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![0, 0], vec![0, 0]]);
        let empty_value = 0_i32;

        // Act
        let actual_matrix = source_matrix.bottom_trim(&empty_value);

        // Assert
        let expected_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn bottom_trim_when_empty_returns_empty() {
        // Arrange
        let source_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        let empty_value = 0_i32;

        // Act
        let actual_matrix = source_matrix.bottom_trim(&empty_value);

        let actual_matrix_right = Matrix::from_vec(vec![]);

        // Assert
        assert_eq!(actual_matrix, actual_matrix_right);
    }

    #[test]
    fn bottom_trim_when_one_trailing_empty_row_removes_single_row() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 0], vec![0, 0]]);
        let empty_value = 0_i32;

        // Act
        let actual_matrix = source_matrix.bottom_trim(&empty_value);

        // Assert
        let expected_matrix = matrix_i32(vec![vec![1, 0]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn bottom_trim_when_middle_empty_row_keeps_non_trailing_row() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![0, 0], vec![1, 0], vec![0, 0]]);
        let empty_value = 0_i32;

        // Act
        let actual_matrix = source_matrix.bottom_trim(&empty_value);

        // Assert
        let expected_matrix = matrix_i32(vec![vec![0, 0], vec![1, 0]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn bottom_trim_when_last_row_contains_non_empty_value_keeps_last_row() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 0], vec![0, 2]]);
        let empty_value = 0_i32;

        // Act
        let actual_matrix = source_matrix.bottom_trim(&empty_value);

        // Assert
        assert_eq!(actual_matrix, source_matrix);
    }

    #[test]
    fn matrix_copy_returns_equal_matrix() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2], vec![3, 4]]);

        // Act
        let copied_matrix = source_matrix.copy();

        // Assert
        assert_eq!(copied_matrix, source_matrix);
    }

    #[test]
    fn matrix_copy_creates_deep_copy() {
        // Arrange
        let source_matrix = matrix_i32(vec![vec![1, 2], vec![3, 4]]);

        // Act
        let copied_matrix = source_matrix.copy();
        let source_outer_pointer = source_matrix.as_slice().as_ptr();
        let copied_outer_pointer = copied_matrix.as_slice().as_ptr();
        let source_inner_pointer = source_matrix.as_slice()[0].as_ptr();
        let copied_inner_pointer = copied_matrix.as_slice()[0].as_ptr();

        // Assert
        assert_ne!(source_outer_pointer, copied_outer_pointer);
        assert_ne!(source_inner_pointer, copied_inner_pointer);
    }

    #[test]
    fn print_when_multi_rows_outputs_space_and_newline_delimited_values() {
        // Arrange
        let probe_mode = "multi_rows";
        let expected_payload = "1 2\n3 4\n".to_string();

        // Act
        let process_output = run_print_probe(probe_mode);
        let process_stdout = String::from_utf8(process_output.stdout)
            .expect("print probe stdout should be utf-8");
        let actual_payload = extract_probe_payload(&process_stdout);

        // Assert
        assert!(process_output.status.success());
        assert_eq!(actual_payload, expected_payload);
    }

    #[test]
    fn print_when_empty_outputs_nothing() {
        // Arrange
        let probe_mode = "empty";

        let expected_payload_right = "";

        // Act
        let process_output = run_print_probe(probe_mode);
        let process_stdout =
            String::from_utf8(process_output.stdout).expect("print probe stdout should be utf-8");
        let actual_payload = extract_probe_payload(&process_stdout);

        // Assert
        assert!(process_output.status.success());
        assert_eq!(actual_payload, expected_payload_right);
    }

    #[test]
    fn print_probe_matrix_output() {
        // Arrange
        let probe_mode = std::env::var("MATRIX_PRINT_PROBE_MODE").unwrap_or_default();

        // Act
        match probe_mode.as_str() {
            "empty" => {
                let source_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
                source_matrix.print();
            }
            "multi_rows" => {
                let source_matrix = matrix_i32(vec![vec![1, 2], vec![3, 4]]);
                source_matrix.print();
            }
            "" => {}
            _ => panic!("unexpected print probe mode"),
        }

        // Assert
        assert!(true);
    }

    #[test]
    fn is_mergeable_into_when_self_empty_returns_true_even_out_of_bounds_offsets_current_behavior() {
        // Arrange
        let self_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        let larger_matrix = matrix_i32(vec![vec![0, 0], vec![0, 0]]);
        let offset_row = 10_usize;
        let offset_col = 10_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_mergeable_into_when_row_boundary_equals_limit_returns_true() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 1], vec![1, 1]]);
        let larger_matrix = matrix_i32(vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]);
        let offset_row = 1_usize;
        let offset_col = 1_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_mergeable_into_when_row_boundary_x_minus_1_returns_false() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 1], vec![1, 1]]);
        let larger_matrix = matrix_i32(vec![vec![0, 0], vec![0, 0]]);
        let offset_row = 1_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(!actual_result);
    }

    #[test]
    fn is_mergeable_into_when_col_boundary_equals_limit_returns_true() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 1], vec![1, 1]]);
        let larger_matrix = matrix_i32(vec![vec![0, 0, 0], vec![0, 0, 0], vec![0, 0, 0]]);
        let offset_row = 0_usize;
        let offset_col = 1_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_mergeable_into_when_col_boundary_x_minus_1_returns_false() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 1], vec![1, 1]]);
        let larger_matrix = matrix_i32(vec![vec![0, 0], vec![0, 0], vec![0, 0]]);
        let offset_row = 0_usize;
        let offset_col = 1_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(!actual_result);
    }

    #[test]
    fn is_mergeable_into_when_non_empty_signs_empty_returns_true_within_bounds() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![9, 9], vec![9, 9]]);
        let larger_matrix = matrix_i32(vec![vec![9, 9], vec![9, 9]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs: Vec<i32> = vec![];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_mergeable_into_when_self_cells_all_empty_signs_returns_true_even_if_overlap() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![0, 0], vec![0, 0]]);
        let larger_matrix = matrix_i32(vec![vec![1, 1], vec![1, 1]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_mergeable_into_when_non_empty_cells_overlap_returns_false() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 0], vec![0, 0]]);
        let larger_matrix = matrix_i32(vec![vec![1, 0], vec![0, 0]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(!actual_result);
    }

    #[test]
    fn is_mergeable_into_when_larger_target_not_non_empty_sign_returns_true() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1]]);
        let larger_matrix = matrix_i32(vec![vec![2]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];

        // Act
        let actual_result =
            self_matrix.is_mergeable_into(&larger_matrix, offset_row, offset_col, &non_empty_signs);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn merge_into_when_assume_mergeable_false_and_not_mergeable_panics() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1]]);
        let larger_matrix = matrix_i32(vec![vec![1]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];
        let assume_mergeable = false;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = self_matrix.merge_into(
                &larger_matrix,
                offset_row,
                offset_col,
                &non_empty_signs,
                assume_mergeable,
            );
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn merge_into_when_assume_mergeable_true_and_overlap_overwrites_without_validation() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![2]]);
        let larger_matrix = matrix_i32(vec![vec![1]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32, 2_i32];
        let assume_mergeable = true;

        // Act
        let actual_matrix = self_matrix.merge_into(
            &larger_matrix,
            offset_row,
            offset_col,
            &non_empty_signs,
            assume_mergeable,
        );

        // Assert
        let expected_matrix = matrix_i32(vec![vec![2]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn merge_into_when_self_has_empty_cells_keeps_larger_values_on_empty_cells() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 0], vec![0, 1]]);
        let larger_matrix = matrix_i32(vec![vec![9, 9], vec![9, 9]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];
        let assume_mergeable = true;

        // Act
        let actual_matrix = self_matrix.merge_into(
            &larger_matrix,
            offset_row,
            offset_col,
            &non_empty_signs,
            assume_mergeable,
        );

        // Assert
        let expected_matrix = matrix_i32(vec![vec![1, 9], vec![9, 1]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn merge_into_when_non_empty_signs_empty_returns_same_as_larger() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1, 2], vec![3, 4]]);
        let larger_matrix = matrix_i32(vec![vec![9, 9], vec![9, 9]]);
        let offset_row = 0_usize;
        let offset_col = 0_usize;
        let non_empty_signs: Vec<i32> = vec![];
        let assume_mergeable = true;

        // Act
        let actual_matrix = self_matrix.merge_into(
            &larger_matrix,
            offset_row,
            offset_col,
            &non_empty_signs,
            assume_mergeable,
        );

        // Assert
        assert_eq!(actual_matrix, larger_matrix);
    }

    #[test]
    fn merge_into_when_self_empty_returns_same_as_larger() {
        // Arrange
        let self_matrix: Matrix<i32> = Matrix::from_vec(vec![]);
        let larger_matrix = matrix_i32(vec![vec![9, 9], vec![9, 9]]);
        let offset_row = 100_usize;
        let offset_col = 100_usize;
        let non_empty_signs = vec![1_i32];
        let assume_mergeable = true;

        // Act
        let actual_matrix = self_matrix.merge_into(
            &larger_matrix,
            offset_row,
            offset_col,
            &non_empty_signs,
            assume_mergeable,
        );

        // Assert
        assert_eq!(actual_matrix, larger_matrix);
    }

    #[test]
    fn merge_into_when_assume_mergeable_true_and_out_of_bounds_panics_current_behavior() {
        // Arrange
        let self_matrix = matrix_i32(vec![vec![1]]);
        let larger_matrix = matrix_i32(vec![vec![0]]);
        let offset_row = 1_usize;
        let offset_col = 0_usize;
        let non_empty_signs = vec![1_i32];
        let assume_mergeable = true;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = self_matrix.merge_into(
                &larger_matrix,
                offset_row,
                offset_col,
                &non_empty_signs,
                assume_mergeable,
            );
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn multiply_mod_when_modulo_zero_panics() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![1]]);
        let right_matrix = matrix_i64(vec![vec![1]]);
        let modulo_value = 0_i64;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = left_matrix.multiply_mod(&right_matrix, modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn multiply_mod_when_modulo_one_returns_all_zero() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![1, 2], vec![3, 4]]);
        let right_matrix = matrix_i64(vec![vec![5, 6], vec![7, 8]]);
        let modulo_value = 1_i64;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![0, 0], vec![0, 0]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn multiply_mod_when_modulo_two_returns_expected_product() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let right_matrix = matrix_i64(vec![vec![7, 8], vec![9, 10], vec![11, 12]]);
        let modulo_value = 2_i64;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![0, 0], vec![1, 0]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn multiply_mod_when_dimension_mismatch_panics() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![1, 2]]);
        let right_matrix = matrix_i64(vec![vec![1, 2]]);
        let modulo_value = 5_i64;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = left_matrix.multiply_mod(&right_matrix, modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn multiply_mod_when_one_by_one_returns_product_modulo() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![7]]);
        let right_matrix = matrix_i64(vec![vec![8]]);
        let modulo_value = 10_i64;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![6]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn multiply_mod_when_two_by_three_and_three_by_two_returns_expected() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let right_matrix = matrix_i64(vec![vec![7, 8], vec![9, 10], vec![11, 12]]);
        let modulo_value = 1_000_000_007_i64;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![58, 64], vec![139, 154]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn multiply_mod_when_negative_values_normalizes_each_cell_to_non_negative_mod_range() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![-1, 2]]);
        let right_matrix = matrix_i64(vec![vec![3], vec![-4]]);
        let modulo_value = 5_i64;

        let expected_value_right = 4;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);
        let actual_value = actual_matrix.as_slice()[0][0];

        // Assert
        assert_eq!(actual_value, expected_value_right);
        assert!(0 <= actual_value && actual_value < modulo_value);
    }

    #[test]
    fn multiply_mod_when_values_near_i64_max_avoids_overflow_and_matches_expected() {
        // Arrange
        let left_matrix = matrix_i64(vec![vec![i64::MAX]]);
        let right_matrix = matrix_i64(vec![vec![i64::MAX]]);
        let modulo_value = 1_000_000_007_i64;
        let reduced_value = ((i64::MAX as i128 % modulo_value as i128) + modulo_value as i128)
            % modulo_value as i128;
        let expected_value = ((reduced_value * reduced_value) % modulo_value as i128) as i64;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![expected_value]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn multiply_mod_when_zero_by_zero_times_zero_by_zero_returns_zero_by_zero() {
        // Arrange
        let left_matrix: Matrix<i64> = Matrix::from_vec(vec![]);
        let right_matrix: Matrix<i64> = Matrix::from_vec(vec![]);
        let modulo_value = 7_i64;

        // Act
        let actual_matrix = left_matrix.multiply_mod(&right_matrix, modulo_value);

        // Assert
        let expected_matrix: Matrix<i64> = Matrix::from_vec(vec![]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn power_mod_when_non_square_panics() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2, 3]]);
        let power_value = 2_u64;
        let modulo_value = 7_i64;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = source_matrix.power_mod(power_value, modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn power_mod_when_modulo_zero_panics() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2], vec![3, 4]]);
        let power_value = 2_u64;
        let modulo_value = 0_i64;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = source_matrix.power_mod(power_value, modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn power_mod_when_power_zero_returns_identity_matrix() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2], vec![3, 4]]);
        let power_value = 0_u64;
        let modulo_value = 7_i64;

        // Act
        let actual_matrix = source_matrix.power_mod(power_value, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![1, 0], vec![0, 1]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn power_mod_when_power_one_returns_matrix_modulo() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![9, -1], vec![15, 4]]);
        let power_value = 1_u64;
        let modulo_value = 5_i64;
        let expected_matrix = matrix_i64(vec![vec![4, 4], vec![0, 4]]);

        // Act
        let actual_matrix = source_matrix.power_mod(power_value, modulo_value);

        // Assert
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn power_mod_when_power_two_matches_self_multiplication() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2], vec![3, 4]]);
        let power_value = 2_u64;
        let modulo_value = 1_000_000_007_i64;
        let expected_matrix = source_matrix.multiply_mod(&source_matrix, modulo_value);

        // Act
        let actual_matrix = source_matrix.power_mod(power_value, modulo_value);

        // Assert
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn power_mod_when_power_boundaries_zero_one_two_are_consistent() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 1], vec![1, 0]]);
        let modulo_value = 1_000_000_007_i64;
        let power_zero = 0_u64;
        let power_one = 1_u64;
        let power_two = 2_u64;
        let expected_zero = matrix_i64(vec![vec![1, 0], vec![0, 1]]);
        let expected_one = source_matrix.clone();
        let expected_two = matrix_i64(vec![vec![2, 1], vec![1, 1]]);

        // Act
        let actual_zero = source_matrix.power_mod(power_zero, modulo_value);
        let actual_one = source_matrix.power_mod(power_one, modulo_value);
        let actual_two = source_matrix.power_mod(power_two, modulo_value);

        // Assert
        assert_eq!(actual_zero, expected_zero);
        assert_eq!(actual_one, expected_one);
        assert_eq!(actual_two, expected_two);
    }

    #[test]
    fn power_mod_when_negative_values_normalizes_each_cell_to_non_negative_mod_range() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-1, 0], vec![0, 1]]);
        let power_value = 3_u64;
        let modulo_value = 5_i64;

        // Act
        let actual_matrix = source_matrix.power_mod(power_value, modulo_value);
        let values = actual_matrix.as_slice();

        let actual_matrix_right = matrix_i64(vec![vec![4, 0], vec![0, 1]]);

        // Assert
        assert_eq!(actual_matrix, actual_matrix_right);
        assert!(values
            .iter()
            .flatten()
            .all(|&value| 0 <= value && value < modulo_value));
    }

    #[test]
    fn power_mod_when_modulo_one_and_power_positive_returns_zero_matrix() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![2, 3], vec![4, 5]]);
        let power_value = 3_u64;
        let modulo_value = 1_i64;

        // Act
        let actual_matrix = source_matrix.power_mod(power_value, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![0, 0], vec![0, 0]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn power_mod_when_modulo_one_and_power_zero_returns_identity_current_behavior() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![2, 3], vec![4, 5]]);
        let power_value = 0_u64;
        let modulo_value = 1_i64;

        // Act
        let actual_matrix = source_matrix.power_mod(power_value, modulo_value);

        // Assert
        let expected_matrix = matrix_i64(vec![vec![1, 0], vec![0, 1]]);
        assert_eq!(actual_matrix, expected_matrix);
    }

    #[test]
    fn sum_all_rows_when_modulo_some_m_matches_sum_row_based_computation() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-1, 2], vec![3, -4]]);
        let modulo_value = 5_i64;
        let first_row_sum = source_matrix.sum_row(0, Some(modulo_value));
        let second_row_sum = source_matrix.sum_row(1, Some(modulo_value));
        let expected_sum =
            ((first_row_sum + second_row_sum) % modulo_value + modulo_value) % modulo_value;

        // Act
        let actual_sum = source_matrix.sum_all_rows(Some(modulo_value));

        // Assert
        assert_eq!(actual_sum, expected_sum);
    }

    #[test]
    fn sum_row_when_row_is_lower_bound_zero_returns_correct_sum() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let target_row = 0_usize;
        let modulo_value = None;

        let expected_sum_right = 6;

        // Act
        let actual_sum = source_matrix.sum_row(target_row, modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_row_when_row_is_upper_bound_last_returns_correct_sum() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let target_row = 1_usize;
        let modulo_value = None;

        let expected_sum_right = 15;

        // Act
        let actual_sum = source_matrix.sum_row(target_row, modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_row_when_row_is_upper_bound_plus_one_panics() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2, 3], vec![4, 5, 6]]);
        let target_row = 2_usize;
        let modulo_value = None;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = source_matrix.sum_row(target_row, modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn sum_row_when_modulo_none_returns_raw_sum_with_negative_values() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-1, 3, -5]]);
        let target_row = 0_usize;
        let modulo_value = None;

        let expected_sum_right = -3;

        // Act
        let actual_sum = source_matrix.sum_row(target_row, modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_row_when_modulo_some_one_returns_zero() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-100, 45, 9]]);
        let target_row = 0_usize;
        let modulo_value = Some(1_i64);

        let expected_sum_right = 0;

        // Act
        let actual_sum = source_matrix.sum_row(target_row, modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_row_when_modulo_some_two_returns_value_in_mod_range() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-3, 2]]);
        let target_row = 0_usize;
        let modulo_value = Some(2_i64);

        let expected_sum_right = 1;

        // Act
        let actual_sum = source_matrix.sum_row(target_row, modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
        assert!(0 <= actual_sum && actual_sum < 2);
    }

    #[test]
    fn sum_row_when_modulo_some_zero_panics() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2, 3]]);
        let target_row = 0_usize;
        let modulo_value = Some(0_i64);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = source_matrix.sum_row(target_row, modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn sum_row_when_target_row_has_zero_columns_returns_zero() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![]]);
        let target_row = 0_usize;
        let modulo_value = None;

        let expected_sum_right = 0;

        // Act
        let actual_sum = source_matrix.sum_row(target_row, modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_all_rows_when_empty_and_modulo_none_returns_zero() {
        // Arrange
        let source_matrix: Matrix<i64> = Matrix::from_vec(vec![]);
        let modulo_value = None;

        let expected_sum_right = 0;

        // Act
        let actual_sum = source_matrix.sum_all_rows(modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_all_rows_when_multiple_rows_and_modulo_none_returns_total_sum() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2], vec![3, 4]]);
        let modulo_value = None;

        let expected_sum_right = 10;

        // Act
        let actual_sum = source_matrix.sum_all_rows(modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_all_rows_when_modulo_some_one_with_mixed_signs_returns_zero() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-1, 2], vec![3, -4]]);
        let modulo_value = Some(1_i64);

        let expected_sum_right = 0;

        // Act
        let actual_sum = source_matrix.sum_all_rows(modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
    }

    #[test]
    fn sum_all_rows_when_modulo_some_two_with_mixed_signs_returns_value_in_mod_range() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![-3, 2], vec![7, -4]]);
        let modulo_value = Some(2_i64);

        let expected_sum_right = 0;

        // Act
        let actual_sum = source_matrix.sum_all_rows(modulo_value);

        // Assert
        assert_eq!(actual_sum, expected_sum_right);
        assert!(0 <= actual_sum && actual_sum < 2);
    }

    #[test]
    fn sum_all_rows_when_modulo_some_zero_panics() {
        // Arrange
        let source_matrix = matrix_i64(vec![vec![1, 2], vec![3, 4]]);
        let modulo_value = Some(0_i64);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = source_matrix.sum_all_rows(modulo_value);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn is_matrix_2d_when_multiple_rows_same_len_returns_true() {
        // Arrange
        let input_matrix = vec![vec![1, 2], vec![3, 4]];

        // Act
        let actual_result = is_matrix_2d(&input_matrix);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_matrix_2d_when_all_rows_empty_returns_true() {
        // Arrange
        let input_matrix: Vec<Vec<i32>> = vec![vec![], vec![]];

        // Act
        let actual_result = is_matrix_2d(&input_matrix);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_matrix_2d_when_empty_returns_false() {
        // Arrange
        let input_matrix: Vec<Vec<i32>> = vec![];

        // Act
        let actual_result = is_matrix_2d(&input_matrix);

        // Assert
        assert!(!actual_result);
    }

    #[test]
    fn is_matrix_2d_when_single_row_returns_true() {
        // Arrange
        let input_matrix = vec![vec![1, 2, 3]];

        // Act
        let actual_result = is_matrix_2d(&input_matrix);

        // Assert
        assert!(actual_result);
    }

    #[test]
    fn is_matrix_2d_when_jagged_returns_false() {
        // Arrange
        let input_matrix = vec![vec![1], vec![2, 3]];

        // Act
        let actual_result = is_matrix_2d(&input_matrix);

        // Assert
        assert!(!actual_result);
    }

    #[test]
    fn is_matrix_2d_when_first_row_empty_and_second_non_empty_returns_false() {
        // Arrange
        let input_matrix = vec![vec![], vec![1]];

        // Act
        let actual_result = is_matrix_2d(&input_matrix);

        // Assert
        assert!(!actual_result);
    }
}
