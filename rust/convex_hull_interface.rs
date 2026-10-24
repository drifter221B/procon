//! Shared types for convex hull based line containers.

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HullMode {
    Min,
    Max,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    m: i64,
    c: i64,
}

impl Line {
    pub fn new(m: i64, c: i64) -> Self {
        Self { m, c }
    }

    pub fn slope(&self) -> i64 {
        self.m
    }

    pub fn intercept(&self) -> i64 {
        self.c
    }

    pub fn eval(&self, x: i64) -> i64 {
        clamp_i128_to_i64(self.eval128(x))
    }

    pub fn eval128(&self, x: i64) -> i128 {
        self.eval128_with_i128(x as i128)
    }
    pub fn eval128_with_i128(&self, x: i128) -> i128 {
        (self.m as i128) * x + self.c as i128
    }
}

pub trait ConvexHull {
    fn add_line(&mut self, m: i64, c: i64);
    fn query(&mut self, x: i64) -> Option<i64>;
}

pub fn clamp_i128_to_i64(value: i128) -> i64 {
    if value < i64::MIN as i128 {
        i64::MIN
    } else if (i64::MAX as i128) < value {
        i64::MAX
    } else {
        value as i64
    }
}
#[cfg(test)]
mod todo_tests {
    use super::{clamp_i128_to_i64, Line};

    #[test]
    fn clamp_rounds_values_below_i64_min() {
        // Arrange
        let value = i64::MIN as i128 - 1;
        let expected_value = i64::MIN;

        // Act
        let actual_value = clamp_i128_to_i64(value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn clamp_rounds_values_above_i64_max() {
        // Arrange
        let value = i64::MAX as i128 + 1;
        let expected_value = i64::MAX;

        // Act
        let actual_value = clamp_i128_to_i64(value);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn clamp_preserves_values_at_and_inside_i64_bounds() {
        // Arrange
        let values = [
            i64::MIN as i128,
            i64::MIN as i128 + 1,
            i64::MAX as i128 - 1,
            i64::MAX as i128,
        ];
        let expected_values = [i64::MIN, i64::MIN + 1, i64::MAX - 1, i64::MAX];

        // Act
        let actual_values = values.map(clamp_i128_to_i64);

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn line_getters_return_constructor_values() {
        // Arrange
        let line = Line::new(-3, 5);
        let expected_values = (-3, 5);

        // Act
        let actual_values = (line.slope(), line.intercept());

        // Assert
        assert_eq!(actual_values, expected_values);
    }

    #[test]
    fn eval128_with_i128_supports_values_beyond_i64_range() {
        // Arrange
        let line = Line::new(2, 3);
        let x = i64::MAX as i128 + 2;
        let expected_value = 2 * x + 3;

        // Act
        let actual_value = line.eval128_with_i128(x);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
