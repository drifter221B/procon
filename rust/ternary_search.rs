#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchType {
    Downward,
    Upward,
}

const DEFAULT_LEFT: f64 = -1.0;
const DEFAULT_RIGHT: f64 = 1e18;
const DEFAULT_LIMIT: usize = 500;

pub fn ternary_search_left<F>(
    convex_func: F,
    search_type: SearchType,
    mut left: f64,
    mut right: f64,
    limit_count: usize,
) -> f64
where
    F: Fn(f64) -> f64,
{
    let mut count = 0;

    while count < limit_count {

        let left_middle = (2.0 * left + right) / 3.0;
        let right_middle = (left + 2.0 * right) / 3.0;

        match search_type {
            SearchType::Downward => {
                if convex_func(right_middle) < convex_func(left_middle) {
                    left = left_middle;
                } else {
                    right = right_middle;
                }
            }
            SearchType::Upward => {
                if convex_func(right_middle) < convex_func(left_middle) {
                    right = right_middle;
                } else {
                    left = left_middle;
                }
            }
        }

        count += 1;
    }

    left
}

pub fn ternary_search_right<F>(
    convex_func: F,
    search_type: SearchType,
    mut left: f64,
    mut right: f64,
    limit_count: usize,
) -> f64
where
    F: Fn(f64) -> f64,
{
    let mut count = 0;

    while count < limit_count {

        let left_middle = (2.0 * left + right) / 3.0;
        let right_middle = (left + 2.0 * right) / 3.0;

        match search_type {
            SearchType::Downward => {
                if convex_func(right_middle) < convex_func(left_middle) {
                    left = left_middle;
                } else {
                    right = right_middle;
                }
            }
            SearchType::Upward => {
                if convex_func(right_middle) < convex_func(left_middle) {
                    right = right_middle;
                } else {
                    left = left_middle;
                }
            }
        }

        count += 1;
    }

    right
}

pub fn ternary_search_left_default<F>(convex_func: F, search_type: SearchType) -> f64
where
    F: Fn(f64) -> f64,
{
    ternary_search_left(convex_func, search_type, DEFAULT_LEFT, DEFAULT_RIGHT, DEFAULT_LIMIT)
}

pub fn ternary_search_right_default<F>(convex_func: F, search_type: SearchType) -> f64
where
    F: Fn(f64) -> f64,
{
    ternary_search_right(convex_func, search_type, DEFAULT_LEFT, DEFAULT_RIGHT, DEFAULT_LIMIT)
}


#[cfg(test)]
mod tests {
    use super::{
        SearchType, ternary_search_left, ternary_search_left_default, ternary_search_right,
        ternary_search_right_default,
    };

    #[test]
    fn finds_minimum_downward() {
        // Arrange
        let function = |x: f64| (x - 2.0).powi(2) + 1.0;
        let search_type = SearchType::Downward;
        let left_bound = 0.0;
        let right_bound = 5.0;
        let limit_count = 200;
        let expected_optimum = 2.0;
        let tolerance = 1e-6;

        // Act
        let actual_left =
            ternary_search_left(function, search_type, left_bound, right_bound, limit_count);
        let actual_right =
            ternary_search_right(function, search_type, left_bound, right_bound, limit_count);

        // Assert
        assert!((actual_left - expected_optimum).abs() < tolerance);
        assert!((actual_right - expected_optimum).abs() < tolerance);
    }

    #[test]
    fn finds_maximum_upward() {
        // Arrange
        let function = |x: f64| -(x - 5.0).powi(2) + 10.0;
        let search_type = SearchType::Upward;
        let left_bound = 0.0;
        let right_bound = 10.0;
        let limit_count = 200;
        let expected_optimum = 5.0;
        let tolerance = 1e-6;

        // Act
        let actual_left =
            ternary_search_left(function, search_type, left_bound, right_bound, limit_count);
        let actual_right =
            ternary_search_right(function, search_type, left_bound, right_bound, limit_count);

        // Assert
        assert!((actual_left - expected_optimum).abs() < tolerance);
        assert!((actual_right - expected_optimum).abs() < tolerance);
    }

    #[test]
    fn zero_iteration_returns_initial_bounds() {
        // Arrange
        let function = |x: f64| x * x;
        let search_type = SearchType::Downward;
        let left_bound = 0.0;
        let right_bound = 5.0;
        let iterations = 0;
        let expected = (left_bound, right_bound);
        // Act
        let actual = (
            ternary_search_left(function, search_type, left_bound, right_bound, iterations),
            ternary_search_right(function, search_type, left_bound, right_bound, iterations),
        );
        // Assert
        assert_eq!(actual, expected);
    }

    #[test]
    fn optimum_at_interval_edge_is_approximated() {
        // Arrange
        let function = |x: f64| x * x;
        let search_type = SearchType::Downward;
        let left_bound = 0.0;
        let right_bound = 5.0;
        let iterations = 200;
        let expected_optimum = 0.0;
        let tolerance = 1e-6;
        // Act
        let actual_left =
            ternary_search_left(function, search_type, left_bound, right_bound, iterations);
        let actual_right =
            ternary_search_right(function, search_type, left_bound, right_bound, iterations);
        // Assert
        assert!((actual_left - expected_optimum).abs() < tolerance);
        assert!((actual_right - expected_optimum).abs() < tolerance);
    }

    #[test]
    fn default_bounds_converge_to_downward_optimum() {
        // Arrange
        let function = |x: f64| (x - 3.0).powi(2);
        let search_type = SearchType::Downward;
        let expected_optimum = 3.0;
        let tolerance = 1e-3;

        // Act
        let actual_left = ternary_search_left_default(function, search_type);
        let actual_right = ternary_search_right_default(function, search_type);

        // Assert
        assert!((actual_left - expected_optimum).abs() < tolerance);
        assert!((actual_right - expected_optimum).abs() < tolerance);
    }

    #[test]
    fn flat_function_converges_with_both_search_types() {
        // Arrange
        let function = |_| 1.0;
        let search_types = [SearchType::Downward, SearchType::Upward];
        let left_bound = 0.0;
        let right_bound = 9.0;
        let limit_count = 30;

        // Act
        let actual_bounds = search_types.map(|search_type| {
            (
                ternary_search_left(function, search_type, left_bound, right_bound, limit_count),
                ternary_search_right(function, search_type, left_bound, right_bound, limit_count),
            )
        });

        // Assert
        for (actual_left, actual_right) in actual_bounds {
            assert!((left_bound..=right_bound).contains(&actual_left));
            assert!((left_bound..=right_bound).contains(&actual_right));
            assert!(actual_left <= actual_right);
        }
    }
}
