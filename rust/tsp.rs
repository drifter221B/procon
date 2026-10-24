/// 動的計画法で巡回セールスマン問題の最短コストを求める。
/// `dist[i][j]` は i->j のコスト。到達不能は `None` とする。
/// 始点 0 に戻る閉路を想定する。
/// 最短コストが `i64` の範囲外の場合も `None` を返す。
pub fn tsp(dist: &[Vec<Option<i64>>]) -> Option<i64> {
    let n = dist.len();
    if n == 0 {
        return Some(0);
    }
    if dist.iter().any(|row| row.len() != n) {
        return None;
    }
    if n == 1 {
        return Some(0);
    }

    let full = 1usize << n;
    let mut dp = vec![vec![None; n]; full];
    dp[1][0] = Some(0_i128);

    for mask in 1..full {
        for u in 0..n {
            if mask & (1 << u) == 0 {
                continue;
            }
            let Some(current) = dp[mask][u] else {
                continue;
            };
            for v in 0..n {
                if mask & (1 << v) != 0 {
                    continue;
                }
                if let Some(w) = dist[u][v] {
                    let next = mask | (1 << v);
                    let Some(candidate) = current.checked_add(i128::from(w)) else {
                        continue;
                    };
                    if dp[next][v].is_none_or(|cost| candidate < cost) {
                        dp[next][v] = Some(candidate);
                    }
                }
            }
        }
    }

    let all = full - 1;
    let mut best = None;
    for u in 1..n {
        if let (Some(current), Some(back)) = (dp[all][u], dist[u][0]) {
            let Some(candidate) = current.checked_add(i128::from(back)) else {
                continue;
            };
            if best.is_none_or(|cost| candidate < cost) {
                best = Some(candidate);
            }
        }
    }

    best.and_then(|cost| i64::try_from(cost).ok())
}

#[cfg(test)]
mod tests {
    use super::tsp;

    #[test]
    fn empty_graph_returns_zero() {
        // Arrange
        let dist: Vec<Vec<Option<i64>>> = vec![];
        let expected_value = Some(0);

        // Act
        let actual_value = tsp(&dist);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn single_vertex_returns_zero() {
        // Arrange
        let dist = vec![vec![Some(0)]];
        let expected_value = Some(0);

        // Act
        let actual_value = tsp(&dist);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn triangle_returns_minimum_cycle_cost_thirteen() {
        // Arrange
        let dist = vec![
            vec![None, Some(1), Some(10)],
            vec![Some(1), None, Some(2)],
            vec![Some(10), Some(2), None],
        ];
        let expected_value = Some(13);

        // Act
        let actual_value = tsp(&dist);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn disconnected_returns_none() {
        // Arrange
        let dist = vec![vec![None, Some(1)], vec![None, None]];
        let expected_value = None;

        // Act
        let actual_value = tsp(&dist);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn invalid_row_length_returns_none() {
        // Arrange
        let distances = vec![vec![None, Some(1)], vec![Some(1)]];
        let expected_value = None;

        // Act
        let actual_value = tsp(&distances);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn final_cycle_cost_at_i64_bounds_is_returned_and_outside_is_none() {
        // Arrange
        let cases = [
            (
                vec![vec![None, Some(i64::MIN)], vec![Some(0), None]],
                Some(i64::MIN),
            ),
            (
                vec![vec![None, Some(i64::MAX)], vec![Some(0), None]],
                Some(i64::MAX),
            ),
            (vec![vec![None, Some(i64::MIN)], vec![Some(-1), None]], None),
            (vec![vec![None, Some(i64::MAX)], vec![Some(1), None]], None),
        ];
        let expected_values = cases.clone().map(|(_, expected_value)| expected_value);

        // Act
        let actual_values = cases.map(|(distances, _)| tsp(&distances));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn unreachable_path_with_negative_return_edge_returns_none() {
        // Arrange
        let distances = vec![vec![None, None], vec![Some(-(i64::MAX / 4)), None]];
        let expected_value = None;

        // Act
        let actual_value = tsp(&distances);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn cycle_cost_at_previous_sentinel_threshold_is_returned() {
        // Arrange
        let distances = vec![vec![None, Some(i64::MAX / 8)], vec![Some(0), None]];
        let expected_value = Some(i64::MAX / 8);

        // Act
        let actual_value = tsp(&distances);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn intermediate_cost_outside_i64_range_can_return_to_i64_range() {
        // Arrange
        let distances = vec![
            vec![None, Some(i64::MAX), None],
            vec![None, None, Some(i64::MAX)],
            vec![Some(-i64::MAX), None, None],
        ];
        let expected_value = Some(i64::MAX);

        // Act
        let actual_value = tsp(&distances);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn asymmetric_cost_matrix_returns_directed_cycle_cost() {
        // Arrange
        let distances = vec![
            vec![None, Some(1), Some(10)],
            vec![Some(5), None, Some(2)],
            vec![Some(3), Some(4), None],
        ];
        let expected_value = Some(6);

        // Act
        let actual_value = tsp(&distances);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn four_vertices_choose_minimum_cycle() {
        // Arrange
        let distances = vec![
            vec![None, Some(1), Some(5), Some(9)],
            vec![Some(9), None, Some(1), Some(5)],
            vec![Some(5), Some(9), None, Some(1)],
            vec![Some(1), Some(5), Some(9), None],
        ];
        let expected_value = Some(4);

        // Act
        let actual_value = tsp(&distances);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
