//! Monge 配列に対するユーティリティ。
//! - `is_monge` で行列が Monge 性を満たすかを判定する。
//! - `monge_row_argmins` と `monge_row_minima` で Monge 性を持つ行列の行ごとの最小列を SMAWK アルゴリズムで取得する。

use std::collections::HashMap;

/// 行列 `a` が Monge 性 `a[i][j] + a[k][l] <= a[i][l] + a[k][j] (i <= k, j <= l)` を満たすかどうかを判定する。
/// 行列が長方形でない場合は `false` を返す。
pub fn is_monge(a: &[Vec<i64>]) -> bool {

    if a.is_empty() {
        return true;
    }

    let col_len = a[0].len();
    if a.iter().any(|row| row.len() != col_len) {
        return false;
    }

    if col_len == 0 {
        return true;
    }

    for i in 0..a.len() {
        for k in (i + 1)..a.len() {
            for j in 0..col_len {
                for l in (j + 1)..col_len {

                    let lhs = a[i][j] + a[k][l];
                    let rhs = a[i][l] + a[k][j];

                    if lhs <= rhs {
                        continue;
                    }

                    return false;
                }
            }
        }
    }

    true
}

/// Monge 性を満たす行列（またはコスト関数）について、各行の最小値となる列番号を返す。
/// 列番号は 0 始まりで、行数または列数が 0 の場合は空配列を返す。
pub fn monge_row_argmins<F>(row_count: usize, col_count: usize, cost: &mut F) -> Vec<usize>
where
    F: FnMut(usize, usize) -> i64,
{

    if row_count == 0 || col_count == 0 {
        return Vec::new();
    }

    let rows: Vec<usize> = (0..row_count).collect();
    let cols: Vec<usize> = (0..col_count).collect();

    smawk(&rows, &cols, cost)
}

/// `monge_row_argmins` の結果を用いて、各行の最小値そのものを返す。
pub fn monge_row_minima<F>(row_count: usize, col_count: usize, cost: &mut F) -> Vec<i64>
where
    F: FnMut(usize, usize) -> i64,
{

    let argmins = monge_row_argmins(row_count, col_count, cost);
    argmins
        .iter()
        .enumerate()
        .map(|(row, &col)| cost(row, col))
        .collect()
}

fn smawk<F>(rows: &[usize], cols: &[usize], cost: &mut F) -> Vec<usize>
where
    F: FnMut(usize, usize) -> i64,
{

    if rows.is_empty() {
        return Vec::new();
    }

    let reduced_cols = prune_columns(rows, cols, cost);

    let odd_rows: Vec<usize> = rows.iter().copied().skip(1).step_by(2).collect();
    let odd_argmins = smawk(&odd_rows, &reduced_cols, cost);

    let mut col_index: HashMap<usize, usize> = HashMap::new();
    for (idx, &col) in reduced_cols.iter().enumerate() {
        col_index.insert(col, idx);
    }

    let mut argmins = vec![0; rows.len()];
    for (odd_idx, &col) in odd_argmins.iter().enumerate() {
        let row_pos = odd_idx * 2 + 1;
        if row_pos < argmins.len() {
            argmins[row_pos] = col;
        }
    }

    let mut left_bound = 0usize;
    let mut even_row_pos = 0usize;
    while even_row_pos < rows.len() {

        let row = rows[even_row_pos];

        let right_bound = if even_row_pos + 1 < rows.len() {
            let next_col = argmins[even_row_pos + 1];
            *col_index
                .get(&next_col)
                .expect("奇数行の列インデックスが列集合に存在しません。")
        } else {
            reduced_cols.len() - 1
        };

        let mut best_col = reduced_cols[left_bound];
        let mut best_value = cost(row, best_col);

        for idx in left_bound..=right_bound {

            let col = reduced_cols[idx];
            let current_value = cost(row, col);

            if current_value < best_value {
                best_value = current_value;
                best_col = col;
            }
        }

        argmins[even_row_pos] = best_col;
        left_bound = right_bound;
        even_row_pos += 2;
    }

    argmins
}

fn prune_columns<F>(rows: &[usize], cols: &[usize], cost: &mut F) -> Vec<usize>
where
    F: FnMut(usize, usize) -> i64,
{

    let mut reduced = Vec::new();

    for &col in cols {

        while let Some(&last_col) = reduced.last() {

            let row_idx = rows[reduced.len() - 1];
            let last_cost = cost(row_idx, last_col);
            let current_cost = cost(row_idx, col);

            if last_cost <= current_cost {
                break;
            }

            reduced.pop();
        }

        if reduced.len() < rows.len() {
            reduced.push(col);
        }
    }

    reduced
}

#[cfg(test)]
mod tests {
    use super::{is_monge, monge_row_argmins, monge_row_minima};
    use std::panic::{catch_unwind, AssertUnwindSafe};

    fn build_matrix_from_cost<F>(row_count: usize, col_count: usize, cost: F) -> Vec<Vec<i64>>
    where
        F: Fn(usize, usize) -> i64,
    {
        (0..row_count)
            .map(|row_index| {
                (0..col_count)
                    .map(|col_index| cost(row_index, col_index))
                    .collect()
            })
            .collect()
    }

    fn squared_distance_cost(row_index: usize, col_index: usize) -> i64 {
        let row_value = row_index as i64;
        let col_value = col_index as i64;
        let difference = row_value - col_value;
        difference * difference
    }

    fn shifted_squared_distance_cost(row_index: usize, col_index: usize) -> i64 {
        let row_value = row_index as i64;
        let shifted_col_value = col_index as i64 - 1;
        let difference = row_value - shifted_col_value;
        difference * difference + row_value
    }

    fn monge_cost_with_negative_values(row_index: usize, col_index: usize) -> i64 {
        let base_cost = squared_distance_cost(row_index, col_index);
        base_cost - 20
    }

    fn flat_row_cost(row_index: usize, _col_index: usize) -> i64 {
        row_index as i64
    }

    fn non_monge_cost(row_index: usize, col_index: usize) -> i64 {
        let non_monge_matrix = [[0_i64, 2, 0], [2, 0, 2], [0, 2, 0]];
        non_monge_matrix[row_index][col_index]
    }

    fn compute_bruteforce_argmins<F>(row_count: usize, col_count: usize, cost: F) -> Vec<usize>
    where
        F: Fn(usize, usize) -> i64,
    {
        if row_count == 0 || col_count == 0 {
            return Vec::new();
        }

        (0..row_count)
            .map(|row_index| {
                let mut best_col_index = 0usize;
                let mut best_cost_value = cost(row_index, best_col_index);
                for col_index in 1..col_count {
                    let current_cost_value = cost(row_index, col_index);
                    if current_cost_value < best_cost_value {
                        best_cost_value = current_cost_value;
                        best_col_index = col_index;
                    }
                }
                best_col_index
            })
            .collect()
    }

    fn compute_bruteforce_row_minima<F>(row_count: usize, col_count: usize, cost: F) -> Vec<i64>
    where
        F: Fn(usize, usize) -> i64,
    {
        if row_count == 0 || col_count == 0 {
            return Vec::new();
        }

        (0..row_count)
            .map(|row_index| {
                let mut min_cost_value = cost(row_index, 0);
                for col_index in 1..col_count {
                    let current_cost_value = cost(row_index, col_index);
                    if current_cost_value < min_cost_value {
                        min_cost_value = current_cost_value;
                    }
                }
                min_cost_value
            })
            .collect()
    }

    #[test]
    fn is_monge_with_empty_matrix_returns_true() {
        // Arrange
        let matrix = Vec::<Vec<i64>>::new();
        let expected_is_monge = true;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_single_empty_row_returns_true() {
        // Arrange
        let matrix = vec![Vec::<i64>::new()];
        let expected_is_monge = true;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_multiple_empty_rows_returns_true() {
        // Arrange
        let matrix = vec![Vec::<i64>::new(), Vec::<i64>::new(), Vec::<i64>::new()];
        let expected_is_monge = true;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_non_rectangular_matrix_returns_false() {
        // Arrange
        let matrix = vec![vec![0_i64, 1_i64], vec![2_i64]];
        let expected_is_monge = false;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_row_count_boundary_0_1_2_does_not_panic_and_returns_true() {
        // Arrange
        let lower_matrix = Vec::<Vec<i64>>::new();
        let center_matrix = vec![vec![0_i64, 1_i64]];
        let upper_matrix = vec![vec![0_i64, 1_i64], vec![1_i64, 2_i64]];
        let expected_results = vec![true, true, true];

        // Act
        let lower_result = is_monge(&lower_matrix);
        let center_result = is_monge(&center_matrix);
        let upper_result = is_monge(&upper_matrix);
        let actual_results = vec![lower_result, center_result, upper_result];

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn is_monge_with_col_count_boundary_0_1_2_does_not_panic_and_returns_true() {
        // Arrange
        let lower_matrix = vec![Vec::<i64>::new(), Vec::<i64>::new()];
        let center_matrix = vec![vec![0_i64], vec![1_i64]];
        let upper_matrix = vec![vec![0_i64, 1_i64], vec![1_i64, 2_i64]];
        let expected_results = vec![true, true, true];

        // Act
        let lower_result = is_monge(&lower_matrix);
        let center_result = is_monge(&center_matrix);
        let upper_result = is_monge(&upper_matrix);
        let actual_results = vec![lower_result, center_result, upper_result];

        // Assert
        assert_eq!(actual_results, expected_results);
    }

    #[test]
    fn is_monge_with_2x2_monge_matrix_returns_true() {
        // Arrange
        let matrix = vec![vec![0_i64, 1_i64], vec![1_i64, 2_i64]];
        let expected_is_monge = true;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_2x2_non_monge_matrix_returns_false() {
        // Arrange
        let matrix = vec![vec![10_i64, 0_i64], vec![0_i64, 10_i64]];
        let expected_is_monge = false;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_3x4_monge_matrix_returns_true() {
        // Arrange
        let row_count = 3usize;
        let col_count = 4usize;
        let matrix = build_matrix_from_cost(row_count, col_count, squared_distance_cost);
        let expected_is_monge = true;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[test]
    fn is_monge_with_negative_values_matrix_returns_true() {
        // Arrange
        let row_count = 3usize;
        let col_count = 4usize;
        let matrix = build_matrix_from_cost(row_count, col_count, monge_cost_with_negative_values);
        let expected_is_monge = true;

        // Act
        let actual_is_monge = is_monge(&matrix);

        // Assert
        assert_eq!(actual_is_monge, expected_is_monge);
    }

    #[cfg(debug_assertions)]
    #[test]
    fn is_monge_with_i64_boundary_values_panics_on_addition_overflow() {
        // Arrange
        let matrix = vec![vec![i64::MAX, 0_i64], vec![0_i64, i64::MAX]];

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| is_monge(&matrix)));
        let did_panic = panic_result.is_err();

        // Assert
        assert!(did_panic);
    }

    #[test]
    fn monge_row_argmins_with_row_count_zero_returns_empty_vector() {
        // Arrange
        let row_count = 0usize;
        let col_count = 3usize;
        let expected_argmins = Vec::<usize>::new();
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_col_count_zero_returns_empty_vector() {
        // Arrange
        let row_count = 3usize;
        let col_count = 0usize;
        let expected_argmins = Vec::<usize>::new();
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_row_count_boundary_0_1_2_returns_expected_lengths() {
        // Arrange
        let lower_row_count = 0usize;
        let center_row_count = 1usize;
        let upper_row_count = 2usize;
        let col_count = 3usize;
        let expected_lengths = vec![0usize, 1usize, 2usize];
        let mut lower_cost_function = squared_distance_cost;
        let mut center_cost_function = squared_distance_cost;
        let mut upper_cost_function = squared_distance_cost;

        // Act
        let lower_argmins = monge_row_argmins(lower_row_count, col_count, &mut lower_cost_function);
        let center_argmins = monge_row_argmins(center_row_count, col_count, &mut center_cost_function);
        let upper_argmins = monge_row_argmins(upper_row_count, col_count, &mut upper_cost_function);
        let actual_lengths = vec![lower_argmins.len(), center_argmins.len(), upper_argmins.len()];

        // Assert
        assert_eq!(actual_lengths, expected_lengths);
    }

    #[test]
    fn monge_row_argmins_with_col_count_boundary_0_1_2_returns_indices_within_range() {
        // Arrange
        let row_count = 2usize;
        let lower_col_count = 0usize;
        let center_col_count = 1usize;
        let upper_col_count = 2usize;
        let mut lower_cost_function = squared_distance_cost;
        let mut center_cost_function = squared_distance_cost;
        let mut upper_cost_function = squared_distance_cost;

        // Act
        let lower_argmins = monge_row_argmins(row_count, lower_col_count, &mut lower_cost_function);
        let center_argmins = monge_row_argmins(row_count, center_col_count, &mut center_cost_function);
        let upper_argmins = monge_row_argmins(row_count, upper_col_count, &mut upper_cost_function);
        let center_in_range = center_argmins.iter().all(|&col_index| col_index < center_col_count);
        let upper_in_range = upper_argmins.iter().all(|&col_index| col_index < upper_col_count);

        // Assert
        assert!(lower_argmins.is_empty());
        assert!(center_in_range);
        assert!(upper_in_range);
    }

    #[test]
    fn monge_row_argmins_with_singleton_row_and_col_returns_zero() {
        // Arrange
        let row_count = 1usize;
        let col_count = 1usize;
        let expected_argmins = vec![0usize];
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_single_column_and_multiple_rows_returns_all_zero() {
        // Arrange
        let row_count = 5usize;
        let col_count = 1usize;
        let expected_argmins = vec![0usize, 0usize, 0usize, 0usize, 0usize];
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_more_rows_than_cols_matches_bruteforce_result() {
        // Arrange
        let row_count = 5usize;
        let col_count = 2usize;
        let expected_argmins = compute_bruteforce_argmins(row_count, col_count, shifted_squared_distance_cost);
        let mut cost_function = shifted_squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_known_monge_cost_matches_bruteforce_result() {
        // Arrange
        let row_count = 6usize;
        let col_count = 7usize;
        let expected_argmins = compute_bruteforce_argmins(row_count, col_count, shifted_squared_distance_cost);
        let mut cost_function = shifted_squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_odd_row_count_matches_bruteforce_result() {
        // Arrange
        let row_count = 5usize;
        let col_count = 6usize;
        let expected_argmins = compute_bruteforce_argmins(row_count, col_count, squared_distance_cost);
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_even_row_count_matches_bruteforce_result() {
        // Arrange
        let row_count = 4usize;
        let col_count = 6usize;
        let expected_argmins = compute_bruteforce_argmins(row_count, col_count, squared_distance_cost);
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_tied_minimum_columns_returns_leftmost_column() {
        // Arrange
        let row_count = 4usize;
        let col_count = 5usize;
        let expected_argmins = vec![0usize, 0usize, 0usize, 0usize];
        let mut cost_function = flat_row_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_argmins, expected_argmins);
    }

    #[test]
    fn monge_row_argmins_with_monge_cost_returns_monotone_non_decreasing_columns() {
        // Arrange
        let row_count = 8usize;
        let col_count = 8usize;
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut cost_function);
        let is_non_decreasing = actual_argmins
            .windows(2)
            .all(|window_values| window_values[0] <= window_values[1]);

        // Assert
        assert!(is_non_decreasing);
    }

    #[test]
    fn monge_row_argmins_with_non_monge_cost_does_not_panic_and_returns_in_range_columns() {
        // Arrange
        let row_count = 3usize;
        let col_count = 3usize;

        let mut cost_function = non_monge_cost;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            monge_row_argmins(row_count, col_count, &mut cost_function)
        }));
        let (actual_argmins, call_succeeded) = match panic_result {
            Ok(argmins) => (argmins, true),
            Err(_) => (Vec::new(), false),
        };
        let actual_length = actual_argmins.len();
        let indices_are_in_range = actual_argmins
            .iter()
            .all(|&col_index| col_index < col_count);

        // Assert
        assert!(call_succeeded);
        assert_eq!(actual_length, row_count);
        assert!(indices_are_in_range);
    }

    #[test]
    fn monge_row_minima_with_row_count_zero_returns_empty_vector() {
        // Arrange
        let row_count = 0usize;
        let col_count = 3usize;
        let expected_minima = Vec::<i64>::new();
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_with_col_count_zero_returns_empty_vector() {
        // Arrange
        let row_count = 3usize;
        let col_count = 0usize;
        let expected_minima = Vec::<i64>::new();
        let mut cost_function = squared_distance_cost;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_with_singleton_row_and_col_returns_cost_at_0_0() {
        // Arrange
        let row_count = 1usize;
        let col_count = 1usize;
        let expected_minima = vec![shifted_squared_distance_cost(0, 0)];
        let mut cost_function = shifted_squared_distance_cost;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_with_known_monge_cost_matches_bruteforce_result() {
        // Arrange
        let row_count = 6usize;
        let col_count = 7usize;
        let expected_minima = compute_bruteforce_row_minima(row_count, col_count, shifted_squared_distance_cost);
        let mut cost_function = shifted_squared_distance_cost;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_with_negative_values_cost_matches_bruteforce_result() {
        // Arrange
        let row_count = 6usize;
        let col_count = 7usize;
        let expected_minima = compute_bruteforce_row_minima(row_count, col_count, monge_cost_with_negative_values);
        let mut cost_function = monge_cost_with_negative_values;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_with_tied_minimum_columns_returns_correct_minimum_values() {
        // Arrange
        let row_count = 4usize;
        let col_count = 5usize;
        let expected_minima = vec![0_i64, 1_i64, 2_i64, 3_i64];
        let mut cost_function = flat_row_cost;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_with_more_rows_than_cols_matches_bruteforce_result() {
        // Arrange
        let row_count = 5usize;
        let col_count = 2usize;
        let expected_minima = compute_bruteforce_row_minima(row_count, col_count, shifted_squared_distance_cost);
        let mut cost_function = shifted_squared_distance_cost;

        // Act
        let actual_minima = monge_row_minima(row_count, col_count, &mut cost_function);

        // Assert
        assert_eq!(actual_minima, expected_minima);
    }

    #[test]
    fn monge_row_minima_values_match_costs_selected_by_monge_row_argmins() {
        // Arrange
        let row_count = 6usize;
        let col_count = 7usize;
        let mut argmin_cost_function = shifted_squared_distance_cost;
        let mut minima_cost_function = shifted_squared_distance_cost;

        // Act
        let actual_argmins = monge_row_argmins(row_count, col_count, &mut argmin_cost_function);
        let actual_minima = monge_row_minima(row_count, col_count, &mut minima_cost_function);
        let minima_from_argmins: Vec<i64> = actual_argmins
            .iter()
            .enumerate()
            .map(|(row_index, &col_index)| shifted_squared_distance_cost(row_index, col_index))
            .collect();

        // Assert
        assert_eq!(actual_minima, minima_from_argmins);
    }

    #[test]
    fn monge_row_minima_with_non_monge_cost_does_not_panic_and_returns_row_count_values() {
        // Arrange
        let row_count = 3usize;
        let col_count = 3usize;

        let mut cost_function = non_monge_cost;

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            monge_row_minima(row_count, col_count, &mut cost_function)
        }));
        let (actual_minima, call_succeeded) = match panic_result {
            Ok(minima) => (minima, true),
            Err(_) => (Vec::new(), false),
        };
        let actual_length = actual_minima.len();

        // Assert
        assert!(call_succeeded);
        assert_eq!(actual_length, row_count);
    }
}
