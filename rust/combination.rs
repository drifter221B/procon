/// メモ化表を用いてパスカルの三角形を計算する二項係数。
pub fn ncr(table: &mut [Vec<i64>], n: usize, r: usize) -> i64 {
    if n < r {
        return 0;
    }

    if table.len() <= n || table[n].len() <= r {
        return 0;
    }

    if table[n][r] != -1 {
        return table[n][r];
    }

    if r == 0 || n == r {
        table[n][r] = 1;
        return 1;
    }

    let left = ncr(table, n - 1, r - 1);
    let right = ncr(table, n - 1, r);
    table[n][r] = left + right;
    table[n][r]
}

/// 前計算した階乗と逆階乗を使って二項係数を法で割った余りを求める。
///
/// `new` は O(max_n + log modulus) 時間と O(max_n) 領域を使い、
/// [`Self::combination`] は O(1) 時間で計算する。`max_n < modulus` かつ
/// `max_n!` と法が互いに素である場合にだけ構築できる。したがって、素数法では
/// `max_n < modulus` を満たせば利用でき、合成数法でも `max_n!` と法が互いに素で
/// ある範囲では利用できる。一方、階乗が法と互いに素でない場合には利用できない。
///
/// ```ignore
/// let combinations = ModularCombination::new(5, 1_000_000_007).unwrap();
/// assert_eq!(combinations.combination(5, 2), Some(10));
/// ```
pub struct ModularCombination {
    modulus: u32,
    factorial: Vec<u32>,
    inverse_factorial: Vec<u32>,
}

impl ModularCombination {
    /// `0!` から `max_n!` までを前計算する。
    pub fn new(max_n: usize, modulus: u32) -> Option<Self> {
        if modulus < 2 || max_n >= modulus as usize {
            return None;
        }

        let length = max_n.checked_add(1)?;
        let mut factorial = vec![1; length];
        for i in 1..=max_n {
            factorial[i] = ((factorial[i - 1] as u64 * i as u64) % modulus as u64) as u32;
        }

        let inverse_last = modular_inverse(factorial[max_n], modulus)?;
        let mut inverse_factorial = vec![1; length];
        inverse_factorial[max_n] = inverse_last;
        for i in (1..=max_n).rev() {
            inverse_factorial[i - 1] =
                ((inverse_factorial[i] as u64 * i as u64) % modulus as u64) as u32;
        }

        Some(Self {
            modulus,
            factorial,
            inverse_factorial,
        })
    }

    /// `n` 個から `k` 個を選ぶ通り数を法で割った余りとして返す。
    ///
    /// 前計算の範囲外の `n` には `None` を返し、範囲内の `k > n` には `Some(0)` を返す。
    pub fn combination(&self, n: usize, k: usize) -> Option<u32> {
        if n >= self.factorial.len() {
            return None;
        }
        if k > n {
            return Some(0);
        }

        let modulus = self.modulus as u64;
        let result = (self.factorial[n] as u64 * self.inverse_factorial[k] as u64) % modulus;
        Some(((result * self.inverse_factorial[n - k] as u64) % modulus) as u32)
    }
}

fn modular_inverse(value: u32, modulus: u32) -> Option<u32> {
    let (gcd, coefficient, _) = extended_gcd(value as i128, modulus as i128);
    if gcd != 1 {
        return None;
    }

    Some(coefficient.rem_euclid(modulus as i128) as u32)
}

fn extended_gcd(a: i128, b: i128) -> (i128, i128, i128) {
    if b == 0 {
        return (a, 1, 0);
    }

    let (gcd, x, y) = extended_gcd(b, a % b);
    (gcd, y, x - (a / b) * y)
}

#[cfg(test)]
mod tests {
    use super::{ncr, ModularCombination};

    fn build_table(n: usize) -> Vec<Vec<i64>> {
        let mut table = vec![vec![-1; n + 1]; n + 1];
        for i in 0..=n {
            table[i][0] = 1;
            table[i][i] = 1;
        }
        table
    }

    fn pascal_remainders(max_n: usize, modulus: u32) -> Vec<Vec<u32>> {
        let mut values = vec![vec![0; max_n + 1]; max_n + 1];
        values[0][0] = 1;
        for n in 1..=max_n {
            values[n][0] = 1;
            for k in 1..=n {
                values[n][k] = ((values[n - 1][k - 1] as u64 + values[n - 1][k] as u64)
                    % modulus as u64) as u32;
            }
        }
        values
    }

    #[test]
    fn ncr_returns_zero_when_r_is_greater_than_n() {
        // Arrange
        let mut table = build_table(3);
        let n = 2;
        let r = 3;
        let expected_value = 0;

        // Act
        let actual_value = ncr(&mut table, n, r);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn non_rectangular_table_returns_zero() {
        // Arrange
        let mut table = vec![vec![1], vec![1], vec![1, -1]];
        let n = 2;
        let r = 2;
        let expected_value = 0;

        // Act
        let actual_value = ncr(&mut table, n, r);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn computes_small_coefficients() {
        // Arrange
        let mut table = build_table(5);
        let queries = [(0, 0), (5, 2), (5, 5)];
        let expected_values = [1, 10, 1];

        // Act
        let actual_values = queries.map(|(n, r)| ncr(&mut table, n, r));

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn reuses_memoized_values() {
        // Arrange
        let mut table = build_table(6);
        let n = 6;
        let r = 3;
        let expected_value = 20;
        let expected_memoized_value = 10;
        let memoized_cell = (5, 2);

        // Act
        let actual_value = ncr(&mut table, n, r);
        let actual_memoized_value = table[memoized_cell.0][memoized_cell.1];

        // Assert
        assert_eq!(actual_value, expected_value);
        assert_eq!(actual_memoized_value, expected_memoized_value);
    }

    #[test]
    fn base_cells_are_initialized_when_unknown() {
        // Arrange
        let mut table = vec![vec![-1; 5]; 5];
        let queries = [(4, 0), (4, 4)];
        let expected_values = [1, 1];

        // Act
        let actual_values = queries.map(|(n, r)| ncr(&mut table, n, r));
        let actual_cells = queries.map(|(n, r)| table[n][r]);

        // Assert
        assert_eq!(actual_values, expected_values);
        assert_eq!(actual_cells, expected_values);
    }

    #[test]
    fn neighbor_queries_reuse_memoized_results_consistently() {
        // Arrange
        let mut table = vec![vec![-1; 8]; 8];
        let queries = [(6, 3), (6, 2), (6, 1)];
        let expected_values = (20, 15, 6);
        let memoized_cells = [(5, 2), (5, 1)];
        let expected_cells = [10, 5];

        // Act
        let actual_first = ncr(&mut table, queries[0].0, queries[0].1);
        let actual_second = ncr(&mut table, queries[1].0, queries[1].1);
        let actual_third = ncr(&mut table, queries[2].0, queries[2].1);
        let actual_cells = memoized_cells.map(|(n, r)| table[n][r]);

        // Assert
        assert_eq!((actual_first, actual_second, actual_third), expected_values);
        assert_eq!(actual_cells, expected_cells);
    }

    #[test]
    fn modular_combination_matches_pascal_remainders() {
        // Arrange
        let max_n = 12;
        let modulus = 13;
        let combinations = ModularCombination::new(max_n, modulus).unwrap();
        let expected_values = pascal_remainders(max_n, modulus);
        let input_pairs: Vec<(usize, usize)> = (0..=max_n)
            .flat_map(|n| (0..=max_n).map(move |k| (n, k)))
            .collect();

        // Act
        let actual_values: Vec<u32> = input_pairs
            .iter()
            .map(|&(n, k)| combinations.combination(n, k).unwrap())
            .collect();

        // Assert
        for ((n, k), actual_value) in input_pairs.into_iter().zip(actual_values) {
            assert_eq!(actual_value, expected_values[n][k]);
        }
    }

    #[test]
    fn modular_combination_handles_a_large_u32_prime_modulus() {
        // Arrange
        let max_n = 4;
        let modulus = 4_294_967_291;
        let input = (4, 2);
        let expected_value = 6;
        let combinations = ModularCombination::new(max_n, modulus).unwrap();

        // Act
        let actual_value = combinations.combination(input.0, input.1);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn modular_combination_supports_an_invertible_composite_modulus() {
        // Arrange
        let max_n = 2;
        let modulus = u32::MAX;
        let input = (2, 1);
        let expected_value = 2;
        let combinations = ModularCombination::new(max_n, modulus).unwrap();

        // Act
        let actual_value = combinations.combination(input.0, input.1);

        // Assert
        assert_eq!(actual_value, Some(expected_value));
    }

    #[test]
    fn modular_combination_with_modulus_below_two_returns_none() {
        // Arrange
        let max_n = 0;
        let moduli = [0, 1];
        let expected_rejections = [true, true];

        // Act
        let actual_rejections =
            moduli.map(|modulus| ModularCombination::new(max_n, modulus).is_none());

        // Assert
        assert_eq!(actual_rejections, expected_rejections);
    }

    #[test]
    fn modular_combination_rejects_max_n_at_or_above_modulus() {
        // Arrange
        let modulus = 5;
        let max_ns = [4, 5, 6];
        let expected_rejections = [false, true, true];

        // Act
        let actual_rejections =
            max_ns.map(|max_n| ModularCombination::new(max_n, modulus).is_none());

        // Assert
        assert_eq!(actual_rejections, expected_rejections);
    }

    #[test]
    fn modular_combination_with_noninvertible_factorial_returns_none() {
        // Arrange
        let max_n = 3;
        let modulus = 8;
        let expected_rejection = true;

        // Act
        let actual_rejection = ModularCombination::new(max_n, modulus).is_none();

        // Assert
        assert_eq!(actual_rejection, expected_rejection);
    }

    #[test]
    fn ncr_with_empty_table_returns_zero() {
        // Arrange
        let mut table: Vec<Vec<i64>> = vec![];
        let input = (0, 0);
        let expected_value = 0;

        // Act
        let actual_value = ncr(&mut table, input.0, input.1);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn ncr_with_row_index_equal_to_table_length_returns_zero() {
        // Arrange
        let mut table = vec![vec![1]];
        let input = (table.len(), 0);
        let expected_value = 0;

        // Act
        let actual_value = ncr(&mut table, input.0, input.1);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn modular_combination_rejects_usize_maximum_before_allocation() {
        // Arrange
        let max_n = usize::MAX;
        let modulus = u32::MAX;
        let expected_value = true;

        // Act
        let actual_value = ModularCombination::new(max_n, modulus).is_none();

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn modular_combination_handles_zero_and_query_boundaries() {
        // Arrange
        let max_n = 0;
        let modulus = 2;
        let inputs = [(0, 0), (0, 1), (1, 0), (0, usize::MAX)];
        let expected_values = [Some(1), Some(0), None, Some(0)];
        let combinations = ModularCombination::new(max_n, modulus).unwrap();

        // Act
        let actual_values: Vec<Option<u32>> = inputs
            .iter()
            .map(|&(n, k)| combinations.combination(n, k))
            .collect();

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
