pub trait Monoid {
    type S: Clone;
    fn id() -> Self::S;
    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S;
}

#[derive(Clone, Debug)]
struct T;

impl Monoid for T {
    type S = i32;

    fn id() -> Self::S {
        0
    }

    fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
        lhs + rhs
    }

}

pub struct BinaryLifting<M, B>
where
    M: Monoid,
    B: Fn(usize) -> M::S,
{
    max_power: usize,
    doubling: Vec<Vec<isize>>,
    memo: Vec<Vec<M::S>>,
    base_func: B,
}

impl<M, B> BinaryLifting<M, B>
where
    M: Monoid,
    B: Fn(usize) -> M::S,
{
    pub fn new(max_step: usize, parents: &[isize], base_func: B) -> Self {

        let n = parents.len();
        let max_power = max_power(max_step);

        let mut instance = Self {
            max_power,
            doubling: vec![vec![-1; n]; max_power + 1],
            memo: vec![vec![M::id(); n]; max_power + 1],
            base_func,
        };

        instance.build_table(parents);

        instance
    }

    pub fn jump(&self, start: isize, steps: usize, init: M::S) -> (isize, M::S) {

        let mut pos = start;
        let mut acc = init;
        let mut remaining_steps = steps;

        for power in 0..=self.max_power {

            if pos == -1 {
                break;
            }

            if (remaining_steps & 1) == 1 {
                let idx = pos as usize;
                acc = M::binary_operation(&acc, &self.memo[power][idx]);
                pos = self.doubling[power][idx];
            }

            remaining_steps >>= 1;
        }

        (pos, acc)
    }

    pub fn fold_path(&self, start: isize, steps: usize) -> M::S {
        self.jump(start, steps, M::id()).1
    }

    fn build_table(&mut self, parents: &[isize]) {
        self.build_first_level(parents);
        self.build_higher_levels(parents.len());
    }

    fn build_first_level(&mut self, parents: &[isize]) {

        for (node, &parent) in parents.iter().enumerate() {
            self.doubling[0][node] = parent;
            self.memo[0][node] = (self.base_func)(node);
        }
    }

    fn build_higher_levels(&mut self, node_count: usize) {

        for power in 0..self.max_power {

            for node in 0..node_count {

                let mid_node = self.doubling[power][node];

                if mid_node == -1 {
                    self.doubling[power + 1][node] = -1;
                    self.memo[power + 1][node] = M::id();
                    continue;
                }

                let mid_index = mid_node as usize;
                let destination_node = self.doubling[power][mid_index];
                if destination_node == -1 {
                    self.doubling[power + 1][node] = -1;
                    self.memo[power + 1][node] = M::id();
                    continue;
                }

                self.doubling[power + 1][node] = destination_node;
                self.memo[power + 1][node] = M::binary_operation(
                    &self.memo[power][node],
                    &self.memo[power][mid_index],
                );
            }
        }
    }
}

fn max_power(max_step: usize) -> usize {

    let max_step = max_step.max(1);
    (u64::BITS as usize) - 1 - max_step.leading_zeros() as usize
}

#[cfg(test)]
mod tests {

    use super::{max_power, BinaryLifting, Monoid};

    struct SumMonoid;

    impl Monoid for SumMonoid {
        type S = i64;

        fn id() -> Self::S {
            0
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            lhs + rhs
        }
    }

    struct StringConcatMonoid;

    impl Monoid for StringConcatMonoid {
        type S = String;

        fn id() -> Self::S {
            String::new()
        }

        fn binary_operation(lhs: &Self::S, rhs: &Self::S) -> Self::S {
            let mut merged = lhs.clone();
            merged.push_str(rhs);
            merged
        }
    }

    fn node_weight_plus_one(node: usize) -> i64 {
        node as i64 + 1
    }

    fn node_label(node: usize) -> String {
        node.to_string()
    }

    fn create_sum_binary_lifting(
        max_step: usize,
        parents: &[isize],
    ) -> BinaryLifting<SumMonoid, fn(usize) -> i64> {
        BinaryLifting::new(max_step, parents, node_weight_plus_one as fn(usize) -> i64)
    }

    fn create_string_concat_binary_lifting(
        max_step: usize,
        parents: &[isize],
    ) -> BinaryLifting<StringConcatMonoid, fn(usize) -> String> {
        BinaryLifting::new(max_step, parents, node_label as fn(usize) -> String)
    }

    #[test]
    fn new_with_empty_parents_allows_sentinel_jump_with_identity_accumulator() {
        // Arrange
        let max_step = 8;
        let parents: [isize; 0] = [];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = -1;
        let steps = 0;
        let init = 42;
        let expected_position = -1;
        let expected_acc = init;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn new_with_single_vertex_returns_expected_position_and_accumulator_for_one_step_jump() {
        // Arrange
        let max_step = 1;
        let parents = [-1];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 0;
        let steps = 1;
        let init = 5;
        let expected_position = -1;
        let expected_acc = 6;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn new_with_chain_graph_returns_expected_position_and_accumulator_for_two_step_jump() {
        // Arrange
        let max_step = 8;
        let parents = [-1, 0, 1, 2];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 3;
        let steps = 2;
        let init = 7;
        let expected_position = 1;
        let expected_acc = 14;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn new_panics_when_parent_index_is_equal_to_length() {
        // Arrange
        let max_step = 2;
        let parents = [2, -1];

        // Act
        let actual_result =
            std::panic::catch_unwind(|| create_sum_binary_lifting(max_step, &parents));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn new_currently_panics_when_parent_index_is_less_than_minus_one() {
        // Arrange
        let max_step = 2;
        let parents = [-2isize];

        // Act
        let actual_result =
            std::panic::catch_unwind(|| create_sum_binary_lifting(max_step, &parents));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn jump_with_zero_steps_keeps_position_and_accumulator_unchanged() {
        // Arrange
        let max_step = 4;
        let parents = [-1, 0];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 1;
        let steps = 0;
        let init = 9;
        let expected_position = 1;
        let expected_acc = init;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn jump_with_sentinel_start_returns_sentinel_and_initial_accumulator() {
        // Arrange
        let max_step = 8;
        let parents = [-1, 0, 1];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = -1;
        let steps = 5;
        let init = 11;
        let expected_position = -1;
        let expected_acc = init;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn jump_returns_expected_values_for_boundary_around_depth_two() {
        // Arrange
        let max_step = 3;
        let parents = [-1, 0, 1];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 2;
        let init = 0;
        let steps_minus_one = 1;
        let steps_center = 2;
        let steps_plus_one = 3;
        let expected_result_minus_one = (1, 3);
        let expected_result_center = (0, 5);
        let expected_result_plus_one = (-1, 3);

        // Act
        let actual_result_minus_one = binary_lifting.jump(start, steps_minus_one, init);
        let actual_result_center = binary_lifting.jump(start, steps_center, init);
        let actual_result_plus_one = binary_lifting.jump(start, steps_plus_one, init);

        // Assert
        assert_eq!(actual_result_minus_one, expected_result_minus_one);
        assert_eq!(actual_result_center, expected_result_center);
        assert_eq!(actual_result_plus_one, expected_result_plus_one);
    }

    #[test]
    fn jump_with_two_steps_and_missing_two_step_destination_returns_identity_accumulator() {
        // Arrange
        let max_step = 2;
        let parents = [-1, 0];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 1;
        let steps = 2;
        let init = 0;
        let expected_position = -1;
        let expected_acc = 0;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn jump_with_steps_greater_than_max_step_currently_ignores_higher_bits() {
        // Arrange
        let max_step = 1;
        let parents = [-1, 0, 1];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 2;
        let steps = 3;
        let smaller_steps = 1;
        let init = 0;
        let expected_result = (1, 3);

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);
        let actual_result_with_smaller_steps = binary_lifting.jump(start, smaller_steps, init);

        // Assert
        assert_eq!(actual_result, expected_result);
        assert_eq!(actual_result_with_smaller_steps, expected_result);
    }

    #[test]
    fn jump_panics_when_start_index_is_equal_to_length() {
        // Arrange
        let max_step = 2;
        let parents = [-1, 0];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = parents.len() as isize;
        let steps = 1;
        let init = 0;

        // Act
        let actual_result = std::panic::catch_unwind(|| binary_lifting.jump(start, steps, init));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn jump_panics_when_start_index_is_less_than_minus_one() {
        // Arrange
        let max_step = 2;
        let parents = [-1];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = -2;
        let steps = 1;
        let init = 0;

        // Act
        let actual_result = std::panic::catch_unwind(|| binary_lifting.jump(start, steps, init));

        // Assert
        assert!(actual_result.is_err());
    }

    #[test]
    fn jump_handles_large_steps_in_cycle_graph() {
        // Arrange
        let max_step = 16;
        let parents = [1, 2, 0];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 0;
        let steps = 10;
        let init = 0;
        let expected_position = 1;
        let expected_acc = 19;

        // Act
        let actual_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_result.0, expected_position);
        assert_eq!(actual_result.1, expected_acc);
    }

    #[test]
    fn fold_path_returns_same_value_as_jump_with_identity_accumulator() {
        // Arrange
        let max_step = 8;
        let parents = [-1, 0, 1, 2];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 3;
        let steps = 2;
        let init = SumMonoid::id();

        // Act
        let actual_fold_value = binary_lifting.fold_path(start, steps);
        let actual_jump_result = binary_lifting.jump(start, steps, init);

        // Assert
        assert_eq!(actual_fold_value, actual_jump_result.1);
    }

    #[test]
    fn fold_path_returns_identity_when_steps_is_zero() {
        // Arrange
        let max_step = 4;
        let parents = [-1, 0];
        let binary_lifting = create_sum_binary_lifting(max_step, &parents);
        let start = 1;
        let steps = 0;
        let expected_value = SumMonoid::id();

        // Act
        let actual_value = binary_lifting.fold_path(start, steps);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn fold_path_preserves_start_to_parent_order_for_non_commutative_monoid() {
        // Arrange
        let max_step = 4;
        let parents = [-1, 0, 1];
        let binary_lifting = create_string_concat_binary_lifting(max_step, &parents);
        let start = 2;
        let steps = 2;
        let expected_value = String::from("21");

        // Act
        let actual_value = binary_lifting.fold_path(start, steps);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_power_returns_zero_when_max_step_is_zero() {
        // Arrange
        let max_step = 0;
        let expected_value = 0;

        // Act
        let actual_value = max_power(max_step);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn max_power_returns_expected_values_for_boundary_around_two() {
        // Arrange
        let max_step_minus_one = 1;
        let max_step_center = 2;
        let max_step_plus_one = 3;
        let expected_value_minus_one = 0;
        let expected_value_center = 1;
        let expected_value_plus_one = 1;

        // Act
        let actual_value_minus_one = max_power(max_step_minus_one);
        let actual_value_center = max_power(max_step_center);
        let actual_value_plus_one = max_power(max_step_plus_one);

        // Assert
        assert_eq!(actual_value_minus_one, expected_value_minus_one);
        assert_eq!(actual_value_center, expected_value_center);
        assert_eq!(actual_value_plus_one, expected_value_plus_one);
    }

    #[test]
    fn max_power_returns_expected_values_for_power_of_two_boundary_around_four() {
        // Arrange
        let max_step_minus_one = 3;
        let max_step_center = 4;
        let max_step_plus_one = 5;
        let expected_value_minus_one = 1;
        let expected_value_center = 2;
        let expected_value_plus_one = 2;

        // Act
        let actual_value_minus_one = max_power(max_step_minus_one);
        let actual_value_center = max_power(max_step_center);
        let actual_value_plus_one = max_power(max_step_plus_one);

        // Assert
        assert_eq!(actual_value_minus_one, expected_value_minus_one);
        assert_eq!(actual_value_center, expected_value_center);
        assert_eq!(actual_value_plus_one, expected_value_plus_one);
    }

    #[test]
    fn max_power_returns_usize_bits_minus_one_for_usize_max() {
        // Arrange
        let max_step = usize::MAX;
        let expected_value = usize::BITS as usize - 1;

        // Act
        let actual_value = max_power(max_step);

        // Assert
        assert_eq!(actual_value, expected_value);
    }
}
