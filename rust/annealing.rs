const INVALID_SCORE: f64 = -1e18;

#[derive(Debug, Clone)]
struct XorShift64 {
    state: u64,
}

impl XorShift64 {
    fn new(seed: u64) -> Self {
        let initial_state = if seed == 0 { 0x4d595df4d0f33173 } else { seed };
        Self { state: initial_state }
    }

    fn next_u64(&mut self) -> u64 {
        let mut x = self.state;
        x ^= x << 7;
        x ^= x >> 9;
        x ^= x << 8;
        self.state = x;
        x
    }

    fn next_bool(&mut self) -> bool {
        self.next_u64() & 1 == 0
    }

    fn next_f64(&mut self) -> f64 {
        self.next_u64() as f64 / u64::MAX as f64
    }
}

#[derive(Debug, Clone)]
pub struct Input {
    pub positions: Vec<i64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct State {
    pub x: i64,
}

impl State {
    pub fn score(&self, input: &Input) -> f64 {
        if self.x < 0 {
            return INVALID_SCORE;
        }

        let first = match input.positions.first() {
            Some(v) => *v,
            None => return INVALID_SCORE,
        };

        if input
            .positions
            .iter()
            .skip(1)
            .any(|&p| first + self.x <= p)
        {
            return INVALID_SCORE;
        }

        -(self.x as f64)
    }
}

/// 焼きなまし法で `State` を最適化する。
/// - `iterations`: 温度更新を含む反復回数
/// - 温度は start_temp から end_temp まで指数補間
pub fn annealing(
    input: &Input,
    initial: State,
    seed: u64,
    iterations: usize,
    start_temp: f64,
    end_temp: f64,
) -> State {
    if iterations == 0 {
        return initial;
    }

    let mut rng = XorShift64::new(seed);
    let mut state = initial;
    let mut best_state = state;
    let mut best_score = state.score(input);

    for step in 0..iterations {
        let progress = step as f64 / iterations as f64;
        let temp = start_temp.powf(1.0 - progress) * end_temp.powf(progress);

        let delta = if rng.next_bool() { -1 } else { 1 };
        let candidate_x = state.x + delta;
        if candidate_x < 0 {
            continue;
        }
        let candidate = State { x: candidate_x };

        let current_score = state.score(input);
        let candidate_score = candidate.score(input);
        let diff = candidate_score - current_score;

        let accept = diff >= 0.0 || rng.next_f64() < (diff / temp).exp();
        if accept {
            state = candidate;
        }

        if candidate_score > best_score {
            best_score = candidate_score;
            best_state = candidate;
        }
    }

    best_state
}

#[cfg(test)]
mod tests {
    use super::{annealing, Input, State, INVALID_SCORE};

    #[test]
    fn score_treats_first_plus_x_equal_to_position_as_invalid() {
        // Arrange
        let input = Input {
            positions: vec![5, 8],
        };
        let state = State { x: 3 };

        // Act
        let actual_score = state.score(&input);

        // Assert
        assert_eq!(actual_score, INVALID_SCORE);
    }

    #[test]
    fn finds_zero_shift_when_first_is_max() {
        // Arrange
        let input = Input {
            positions: vec![10, 8, 6],
        };
        let initial = State { x: 3 };
        let seed = 42;
        let iterations = 2_000;
        let start_temp = 1.0;
        let end_temp = 0.01;
        let expected_x = 0;

        // Act
        let actual_state = annealing(&input, initial, seed, iterations, start_temp, end_temp);

        // Assert
        assert_eq!(actual_state.x, expected_x);
    }

    #[test]
    fn finds_minimum_positive_shift() {
        // Arrange
        let input = Input {
            positions: vec![5, 7, 9],
        };
        let initial = State { x: 0 };
        let seed = 7;
        let iterations = 4_000;
        let start_temp = 1.0;
        let end_temp = 0.001;
        let expected_x = 5;

        // Act
        let actual_state = annealing(&input, initial, seed, iterations, start_temp, end_temp);

        // Assert
        // 条件 p0 + x > max(p_i) を満たす最小整数 x は 5
        assert_eq!(actual_state.x, expected_x);
    }

    #[test]
    fn annealing_returns_initial_state_when_iterations_is_zero() {
        // Arrange
        let input = Input {
            positions: vec![5, 7, 9],
        };
        let initial = State { x: 4 };
        let seed = 1;
        let iterations = 0;
        let start_temp = 1.0;
        let end_temp = 0.1;

        // Act
        let actual_state = annealing(&input, initial, seed, iterations, start_temp, end_temp);

        // Assert
        assert_eq!(actual_state, initial);
    }

    #[test]
    fn empty_positions_keep_invalid_score_and_return_initial_state() {
        // Arrange
        let input = Input { positions: vec![] };
        let initial = State { x: 3 };
        let seed = 1;
        let iterations = 32;
        let start_temp = 1.0;
        let end_temp = 0.1;

        // Act
        let actual_score = initial.score(&input);
        let actual_state = annealing(&input, initial, seed, iterations, start_temp, end_temp);

        // Assert
        assert_eq!(actual_score, INVALID_SCORE);
        assert_eq!(actual_state, initial);
    }

    #[test]
    fn score_is_invalid_at_and_below_later_position_boundary() {
        // Arrange
        let input = Input {
            positions: vec![5, 8],
        };
        let shifts = [2, 3];
        let expected_scores = [INVALID_SCORE, INVALID_SCORE];

        // Act
        let actual_scores = shifts.map(|x| State { x }.score(&input));

        // Assert
        assert_eq!(actual_scores, expected_scores);
    }

    #[test]
    fn score_is_negative_shift_for_first_position_and_valid_above_later_position_boundary() {
        // Arrange
        let input = Input {
            positions: vec![5, 8],
        };
        let shifts = [-1, 0, 1, 4];
        let expected_scores = [INVALID_SCORE, INVALID_SCORE, INVALID_SCORE, -4.0];

        // Act
        let actual_scores = shifts.map(|x| State { x }.score(&input));

        // Assert
        assert_eq!(actual_scores, expected_scores);
    }

    #[test]
    fn score_uses_first_position_for_single_duplicate_and_unsorted_inputs() {
        // Arrange
        let inputs = [
            (Input { positions: vec![5] }, State { x: 0 }),
            (
                Input {
                    positions: vec![5, 5],
                },
                State { x: 0 },
            ),
            (
                Input {
                    positions: vec![5, 3],
                },
                State { x: 0 },
            ),
        ];
        let expected_scores = [-0.0, INVALID_SCORE, -0.0];

        // Act
        let actual_scores = inputs.map(|(input, state)| state.score(&input));

        // Assert
        assert_eq!(actual_scores, expected_scores);
    }
}
