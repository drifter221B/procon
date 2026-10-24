/// 階乗と二項係数を前計算する構造体。
pub struct Factorial {
    facts: Vec<u128>,
}

impl Factorial {
    pub fn new(n: usize) -> Self {
        let mut facts = vec![1u128; n + 1];

        for i in 1..=n {
            facts[i] = facts[i - 1] * i as u128;
        }

        Self { facts }
    }

    #[inline]
    pub fn len(&self) -> usize {
        self.facts.len().saturating_sub(1)
    }

    #[inline]
    pub fn factorial(&self, n: usize) -> Option<u128> {
        if n <= self.len() {
            Some(self.facts[n])
        } else {
            None
        }
    }

    /// nCk を返す。`n` と `k` が前計算範囲外または `n < k` の場合は None。
    pub fn combination(&self, n: usize, k: usize) -> Option<u128> {
        if n < k || self.len() < n {
            None
        } else {
            Some(self.facts[n] / (self.facts[k] * self.facts[n - k]))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Factorial;

    #[test]
    fn len_handles_zero_and_one_boundaries() {
        // Arrange
        let zero_max_n = 0;
        let one_max_n = 1;
        let expected_lengths = (0, 1);
        let zero = Factorial::new(zero_max_n);
        let one = Factorial::new(one_max_n);

        // Act
        let actual_lengths = (zero.len(), one.len());

        // Assert
        assert_eq!(actual_lengths, expected_lengths);
    }

    #[test]
    fn factorial_for_zero_and_one_returns_one_within_precomputed_range() {
        // Arrange
        let facts = Factorial::new(1);
        let queries = (0, 1, 2);
        let expected_values = (Some(1), Some(1), None);

        // Act
        let actual_values = (
            facts.factorial(queries.0),
            facts.factorial(queries.1),
            facts.factorial(queries.2),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn factorial_returns_value_at_precomputed_upper_bound() {
        // Arrange
        let factorial = Factorial::new(6);
        let expected_value = Some(720);
        let n = 6;

        // Act
        let actual_value = factorial.factorial(n);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn combination_with_n_beyond_precomputed_range_returns_none() {
        // Arrange
        let facts = Factorial::new(3);
        let query = (4, 2);
        let expected_value = None;

        // Act
        let actual_value = facts.combination(query.0, query.1);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn combination_with_k_greater_than_n_returns_none() {
        // Arrange
        let facts = Factorial::new(3);
        let query = (3, 4);
        let expected_value = None;

        // Act
        let actual_value = facts.combination(query.0, query.1);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn combination_returns_one_for_k_zero_and_k_n() {
        // Arrange
        let factorial = Factorial::new(6);
        let expected_values = (Some(1), Some(1));
        let queries = [(6, 0), (6, 6)];

        // Act
        let actual_values = (
            factorial.combination(queries[0].0, queries[0].1),
            factorial.combination(queries[1].0, queries[1].1),
        );

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn factorial_returns_value_within_precomputed_range() {
        // Arrange
        let facts = Factorial::new(5);
        let n = 5;
        let expected_value = Some(120);

        // Act
        let actual_value = facts.factorial(n);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn combination_returns_expected_values_within_precomputed_range() {
        // Arrange
        let facts = Factorial::new(5);
        let queries = [(5, 0), (5, 2), (5, 5)];
        let expected_values = [Some(1), Some(10), Some(1)];

        // Act
        let actual_values = queries.map(|(n, k)| facts.combination(n, k));

        // Assert
        assert_eq!(actual_values, expected_values);
    }
}
