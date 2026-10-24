use std::ops::{Add, Div, Mul, Neg, Sub};

pub trait MatrixTreeElement:
    Clone
    + PartialEq
    + Add<Output = Self>
    + Sub<Output = Self>
    + Mul<Output = Self>
    + Div<Output = Self>
    + Neg<Output = Self>
{
    fn zero() -> Self;
    fn one() -> Self;
}

impl MatrixTreeElement for i64 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }
}

impl MatrixTreeElement for i128 {
    fn zero() -> Self {
        0
    }

    fn one() -> Self {
        1
    }
}

impl MatrixTreeElement for f64 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }
}

impl MatrixTreeElement for f32 {
    fn zero() -> Self {
        0.0
    }

    fn one() -> Self {
        1.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MatrixTreeEdge<T> {
    from: usize,
    to: usize,
    weight: T,
}

impl<T> MatrixTreeEdge<T> {
    pub fn new(from: usize, to: usize, weight: T) -> Self {
        Self { from, to, weight }
    }

    pub fn from(&self) -> usize {
        self.from
    }

    pub fn to(&self) -> usize {
        self.to
    }

    pub fn weight(&self) -> &T {
        &self.weight
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MatrixTreeOrientation {
    Undirected,
    DirectedInArborescence,
    DirectedOutArborescence,
}

pub trait DeterminantCalculator<T>
where
    T: MatrixTreeElement,
{
    fn determinant(&self, matrix: &[Vec<T>]) -> T;
}

#[derive(Clone, Copy, Debug, Default)]
pub struct BareissDeterminant;

impl<T> DeterminantCalculator<T> for BareissDeterminant
where
    T: MatrixTreeElement,
{
    fn determinant(&self, matrix: &[Vec<T>]) -> T {
        assert_square_matrix(matrix);

        let matrix_size = matrix.len();
        if matrix_size == 0 {
            return T::one();
        }

        if matrix_size == 1 {
            return matrix[0][0].clone();
        }

        let mut working_matrix = matrix.to_vec();
        let mut determinant_sign = T::one();
        let mut previous_pivot_value = T::one();

        for pivot_index in 0..(matrix_size - 1) {
            let pivot_row_index = find_non_zero_pivot_row(&working_matrix, pivot_index);

            if let Some(found_row_index) = pivot_row_index {
                if pivot_index != found_row_index {
                    working_matrix.swap(pivot_index, found_row_index);
                    determinant_sign = -determinant_sign;
                }
            } else {
                return T::zero();
            }

            let pivot_value = working_matrix[pivot_index][pivot_index].clone();

            if pivot_value == T::zero() {
                return T::zero();
            }

            reduce_submatrix_by_bareiss(
                &mut working_matrix,
                pivot_index,
                &pivot_value,
                &previous_pivot_value,
            );
            previous_pivot_value = pivot_value;
        }

        determinant_sign * working_matrix[matrix_size - 1][matrix_size - 1].clone()
    }
}

pub struct MatrixTreeTheorem<T, D = BareissDeterminant>
where
    T: MatrixTreeElement,
    D: DeterminantCalculator<T>,
{
    vertex_count: usize,
    orientation: MatrixTreeOrientation,
    edges: Vec<MatrixTreeEdge<T>>,
    determinant_calculator: D,
}

impl<T> MatrixTreeTheorem<T, BareissDeterminant>
where
    T: MatrixTreeElement,
{
    pub fn new(vertex_count: usize, orientation: MatrixTreeOrientation) -> Self {
        Self::new_with_determinant_calculator(vertex_count, orientation, BareissDeterminant)
    }
}

impl<T, D> MatrixTreeTheorem<T, D>
where
    T: MatrixTreeElement,
    D: DeterminantCalculator<T>,
{
    pub fn new_with_determinant_calculator(
        vertex_count: usize,
        orientation: MatrixTreeOrientation,
        determinant_calculator: D,
    ) -> Self {
        Self {
            vertex_count,
            orientation,
            edges: Vec::new(),
            determinant_calculator,
        }
    }

    pub fn vertex_count(&self) -> usize {
        self.vertex_count
    }

    pub fn edge_count(&self) -> usize {
        self.edges.len()
    }

    pub fn orientation(&self) -> MatrixTreeOrientation {
        self.orientation
    }

    pub fn edges(&self) -> &[MatrixTreeEdge<T>] {
        &self.edges
    }

    pub fn add_edge(&mut self, from: usize, to: usize, weight: T) {
        self.validate_vertex(from);
        self.validate_vertex(to);
        self.edges.push(MatrixTreeEdge::new(from, to, weight));
    }

    pub fn add_unweighted_edge(&mut self, from: usize, to: usize) {
        self.add_edge(from, to, T::one());
    }

    pub fn add_edges<I>(&mut self, edge_collection: I)
    where
        I: IntoIterator<Item = MatrixTreeEdge<T>>,
    {
        for edge in edge_collection {
            let MatrixTreeEdge { from, to, weight } = edge;
            self.add_edge(from, to, weight);
        }
    }

    pub fn laplacian_matrix(&self) -> Vec<Vec<T>> {
        let mut laplacian_matrix = create_zero_square_matrix::<T>(self.vertex_count);

        for edge in self.edges.iter() {
            self.accumulate_edge_to_laplacian(&mut laplacian_matrix, edge);
        }

        laplacian_matrix
    }

    pub fn count_by_root(&self, root_vertex: usize) -> T {
        self.validate_vertex(root_vertex);

        if self.vertex_count == 1 {
            return T::one();
        }

        let laplacian_matrix = self.laplacian_matrix();
        let principal_cofactor_minor =
            build_principal_cofactor_minor(&laplacian_matrix, root_vertex);

        self.determinant_calculator
            .determinant(&principal_cofactor_minor)
    }

    pub fn count_for_all_roots(&self) -> Vec<T> {
        if self.vertex_count == 0 {
            return vec![];
        }

        (0..self.vertex_count)
            .map(|root_vertex| self.count_by_root(root_vertex))
            .collect()
    }

    fn validate_vertex(&self, vertex: usize) {
        assert!(
            vertex < self.vertex_count,
            "vertex index must be less than vertex_count"
        );
    }

    fn accumulate_edge_to_laplacian(
        &self,
        laplacian_matrix: &mut [Vec<T>],
        edge: &MatrixTreeEdge<T>,
    ) {
        match self.orientation {
            MatrixTreeOrientation::Undirected => {
                accumulate_undirected_edge(laplacian_matrix, edge);
            }
            MatrixTreeOrientation::DirectedInArborescence => {
                accumulate_directed_in_arborescence_edge(laplacian_matrix, edge);
            }
            MatrixTreeOrientation::DirectedOutArborescence => {
                accumulate_directed_out_arborescence_edge(laplacian_matrix, edge);
            }
        }
    }
}

fn assert_square_matrix<T>(matrix: &[Vec<T>]) {
    let matrix_size = matrix.len();
    let is_square = matrix
        .iter()
        .all(|row_values| row_values.len() == matrix_size);
    assert!(is_square, "matrix must be square");
}

fn create_zero_square_matrix<T>(matrix_size: usize) -> Vec<Vec<T>>
where
    T: MatrixTreeElement,
{
    (0..matrix_size)
        .map(|_| vec![T::zero(); matrix_size])
        .collect()
}

fn add_value_to_cell<T>(matrix: &mut [Vec<T>], row_index: usize, col_index: usize, value: &T)
where
    T: MatrixTreeElement,
{
    matrix[row_index][col_index] = matrix[row_index][col_index].clone() + value.clone();
}

fn subtract_value_from_cell<T>(matrix: &mut [Vec<T>], row_index: usize, col_index: usize, value: &T)
where
    T: MatrixTreeElement,
{
    matrix[row_index][col_index] = matrix[row_index][col_index].clone() - value.clone();
}

fn accumulate_undirected_edge<T>(laplacian_matrix: &mut [Vec<T>], edge: &MatrixTreeEdge<T>)
where
    T: MatrixTreeElement,
{
    if edge.from == edge.to {
        return;
    }

    add_value_to_cell(laplacian_matrix, edge.from, edge.from, &edge.weight);
    add_value_to_cell(laplacian_matrix, edge.to, edge.to, &edge.weight);
    subtract_value_from_cell(laplacian_matrix, edge.from, edge.to, &edge.weight);
    subtract_value_from_cell(laplacian_matrix, edge.to, edge.from, &edge.weight);
}

fn accumulate_directed_in_arborescence_edge<T>(
    laplacian_matrix: &mut [Vec<T>],
    edge: &MatrixTreeEdge<T>,
) where
    T: MatrixTreeElement,
{
    if edge.from == edge.to {
        return;
    }

    add_value_to_cell(laplacian_matrix, edge.from, edge.from, &edge.weight);
    subtract_value_from_cell(laplacian_matrix, edge.from, edge.to, &edge.weight);
}

fn accumulate_directed_out_arborescence_edge<T>(
    laplacian_matrix: &mut [Vec<T>],
    edge: &MatrixTreeEdge<T>,
) where
    T: MatrixTreeElement,
{
    if edge.from == edge.to {
        return;
    }

    add_value_to_cell(laplacian_matrix, edge.to, edge.to, &edge.weight);
    subtract_value_from_cell(laplacian_matrix, edge.to, edge.from, &edge.weight);
}

fn build_principal_cofactor_minor<T>(matrix: &[Vec<T>], removed_index: usize) -> Vec<Vec<T>>
where
    T: MatrixTreeElement,
{
    assert_square_matrix(matrix);

    let matrix_size = matrix.len();
    if matrix_size == 0 {
        return vec![];
    }

    assert!(
        removed_index < matrix_size,
        "removed_index must be less than matrix size"
    );

    let minor_size = matrix_size - 1;
    let mut minor_matrix = Vec::with_capacity(minor_size);

    for row_index in 0..matrix_size {
        if row_index == removed_index {
            continue;
        }

        let mut row_values = Vec::with_capacity(minor_size);

        for col_index in 0..matrix_size {
            if col_index == removed_index {
                continue;
            }
            row_values.push(matrix[row_index][col_index].clone());
        }

        minor_matrix.push(row_values);
    }

    minor_matrix
}

fn find_non_zero_pivot_row<T>(working_matrix: &[Vec<T>], pivot_index: usize) -> Option<usize>
where
    T: MatrixTreeElement,
{
    (pivot_index..working_matrix.len())
        .find(|&row_index| working_matrix[row_index][pivot_index] != T::zero())
}

fn reduce_submatrix_by_bareiss<T>(
    working_matrix: &mut [Vec<T>],
    pivot_index: usize,
    pivot_value: &T,
    previous_pivot_value: &T,
) where
    T: MatrixTreeElement,
{
    let matrix_size = working_matrix.len();

    for row_index in (pivot_index + 1)..matrix_size {
        for col_index in (pivot_index + 1)..matrix_size {
            let left_product = working_matrix[row_index][col_index].clone() * pivot_value.clone();
            let right_product = working_matrix[row_index][pivot_index].clone()
                * working_matrix[pivot_index][col_index].clone();
            let updated_value = (left_product - right_product) / previous_pivot_value.clone();
            working_matrix[row_index][col_index] = updated_value;
        }
        working_matrix[row_index][pivot_index] = T::zero();
    }
}

#[cfg(test)]
mod tests {
    use super::{
        BareissDeterminant, DeterminantCalculator, MatrixTreeEdge, MatrixTreeOrientation,
        MatrixTreeTheorem,
    };
    use std::panic::{catch_unwind, AssertUnwindSafe};

    struct ConstantDeterminant {
        fixed_value: i64,
    }

    impl DeterminantCalculator<i64> for ConstantDeterminant {
        fn determinant(&self, _: &[Vec<i64>]) -> i64 {
            self.fixed_value
        }
    }

    #[test]
    fn determinant_when_matrix_is_singular_returns_zero() {
        // Arrange
        let determinant_calculator = BareissDeterminant;
        let matrix = vec![vec![1_i64, 2_i64], vec![2_i64, 4_i64]];
        let expected_value = 0_i64;

        // Act
        let actual_value = determinant_calculator.determinant(&matrix);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn determinant_when_matrix_size_boundary_around_one_returns_expected_values() {
        // Arrange
        let determinant_calculator = BareissDeterminant;
        let matrix_size_minus_one_case: Vec<Vec<i64>> = vec![];
        let matrix_size_center_case = vec![vec![5_i64]];
        let matrix_size_plus_one_case = vec![vec![2_i64, 3_i64], vec![4_i64, 7_i64]];
        let expected_value_minus_one_case = 1_i64;
        let expected_value_center_case = 5_i64;
        let expected_value_plus_one_case = 2_i64;

        // Act
        let actual_value_minus_one_case =
            determinant_calculator.determinant(&matrix_size_minus_one_case);
        let actual_value_center_case = determinant_calculator.determinant(&matrix_size_center_case);
        let actual_value_plus_one_case =
            determinant_calculator.determinant(&matrix_size_plus_one_case);

        // Assert
        assert_eq!(actual_value_minus_one_case, expected_value_minus_one_case);
        assert_eq!(actual_value_center_case, expected_value_center_case);
        assert_eq!(actual_value_plus_one_case, expected_value_plus_one_case);
    }

    #[test]
    fn count_by_root_when_custom_determinant_calculator_is_injected_returns_calculator_value() {
        // Arrange
        let vertex_count = 2_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let root_vertex = 0_usize;
        let fixed_value = 42_i64;
        let expected_count = fixed_value;
        let determinant_calculator = ConstantDeterminant { fixed_value };
        let mut matrix_tree = MatrixTreeTheorem::new_with_determinant_calculator(
            vertex_count,
            orientation,
            determinant_calculator,
        );
        matrix_tree.add_unweighted_edge(0_usize, 1_usize);

        // Act
        let actual_count = matrix_tree.count_by_root(root_vertex);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn determinant_when_pivot_row_swap_is_needed_applies_sign_change() {
        // Arrange
        let determinant_calculator = BareissDeterminant;
        let matrix = vec![vec![0_i64, 1_i64], vec![2_i64, 3_i64]];
        let expected_value = -2_i64;

        // Act
        let actual_value = determinant_calculator.determinant(&matrix);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn determinant_when_matrix_is_not_square_panics() {
        // Arrange
        let determinant_calculator = BareissDeterminant;
        let matrix = vec![vec![1_i64, 2_i64]];

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = determinant_calculator.determinant(&matrix);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn determinant_supports_i128_matrix_values() {
        // Arrange
        let determinant_calculator = BareissDeterminant;
        let matrix = vec![vec![2_i128, 3_i128], vec![5_i128, 7_i128]];
        let expected_value = -1_i128;

        // Act
        let actual_value = determinant_calculator.determinant(&matrix);

        // Assert
        assert_eq!(actual_value, expected_value);
    }

    #[test]
    fn new_initial_accessors_return_constructor_values() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::DirectedOutArborescence;
        let expected_vertex_count = vertex_count;
        let expected_edge_count = 0_usize;
        let expected_orientation = orientation;
        let matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        // Act
        let actual_vertex_count = matrix_tree.vertex_count();
        let actual_edge_count = matrix_tree.edge_count();
        let actual_orientation = matrix_tree.orientation();
        let actual_edges = matrix_tree.edges();

        // Assert
        assert_eq!(actual_vertex_count, expected_vertex_count);
        assert_eq!(actual_edge_count, expected_edge_count);
        assert_eq!(actual_orientation, expected_orientation);
        assert!(actual_edges.is_empty());
    }

    #[test]
    fn add_edges_when_input_is_empty_set_keeps_edge_count_zero() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let input_edge_set: Vec<MatrixTreeEdge<i64>> = vec![];
        let expected_edge_count = 0_usize;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        // Act
        matrix_tree.add_edges(input_edge_set);
        let actual_edge_count = matrix_tree.edge_count();

        // Assert
        assert_eq!(actual_edge_count, expected_edge_count);
    }

    #[test]
    fn add_edges_when_input_is_singleton_set_increases_edge_count_to_one() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let input_edge_set = vec![MatrixTreeEdge::new(0_usize, 1_usize, 1_i64)];
        let expected_edge_count = 1_usize;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        // Act
        matrix_tree.add_edges(input_edge_set);
        let actual_edge_count = matrix_tree.edge_count();

        // Assert
        assert_eq!(actual_edge_count, expected_edge_count);
    }

    #[test]
    fn add_edges_when_input_is_multiple_element_set_increases_edge_count_by_set_size() {
        // Arrange
        let vertex_count = 4_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let input_edge_set = vec![
            MatrixTreeEdge::new(0_usize, 1_usize, 1_i64),
            MatrixTreeEdge::new(1_usize, 2_usize, 1_i64),
            MatrixTreeEdge::new(2_usize, 3_usize, 1_i64),
        ];
        let expected_edge_count = 3_usize;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        // Act
        matrix_tree.add_edges(input_edge_set);
        let actual_edge_count = matrix_tree.edge_count();

        // Assert
        assert_eq!(actual_edge_count, expected_edge_count);
    }

    #[test]
    fn add_edge_with_out_of_range_vertex_panics() {
        // Arrange
        let vertex_count = 2_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let mut from_invalid_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        let mut to_invalid_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        let add_edge_to_vertex = 0_usize;
        let add_edge_capacity = 1_i64;
        let add_edge_from_vertex = 0_usize;

        // Act
        let from_invalid_result = catch_unwind(AssertUnwindSafe(|| {
            from_invalid_tree.add_edge(vertex_count, add_edge_to_vertex, add_edge_capacity);
        }));
        let to_invalid_result = catch_unwind(AssertUnwindSafe(|| {
            to_invalid_tree.add_edge(add_edge_from_vertex, vertex_count, add_edge_capacity);
        }));

        // Assert
        assert!(from_invalid_result.is_err());
        assert!(to_invalid_result.is_err());
    }

    #[test]
    fn self_loop_does_not_change_laplacian_or_tree_count() {
        // Arrange
        let vertex_count = 2_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let root_vertex = 0_usize;
        let other_vertex = 1_usize;
        let expected_count = 1_i64;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree.add_unweighted_edge(root_vertex, other_vertex);
        let expected_laplacian = matrix_tree.laplacian_matrix();

        // Act
        matrix_tree.add_unweighted_edge(root_vertex, root_vertex);
        let actual_count = matrix_tree.count_by_root(root_vertex);
        let actual_laplacian = matrix_tree.laplacian_matrix();

        // Assert
        assert_eq!(actual_count, expected_count);
        assert_eq!(actual_laplacian, expected_laplacian);
    }

    #[test]
    fn count_by_root_when_vertex_count_is_one_returns_one() {
        // Arrange
        let vertex_count = 1_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let root_vertex = 0_usize;
        let expected_count = 1_i64;
        let matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        // Act
        let actual_count = matrix_tree.count_by_root(root_vertex);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn count_by_root_when_undirected_edge_multiplicity_boundary_around_one_returns_expected_values()
    {
        // Arrange
        let vertex_count = 2_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let root_vertex = 0_usize;
        let expected_count_minus_one_case = 0_i64;
        let expected_count_center_case = 1_i64;
        let expected_count_plus_one_case = 2_i64;

        let matrix_tree_minus_one_case = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        let mut matrix_tree_center_case = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree_center_case.add_unweighted_edge(0_usize, 1_usize);

        let mut matrix_tree_plus_one_case =
            MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree_plus_one_case.add_unweighted_edge(0_usize, 1_usize);
        matrix_tree_plus_one_case.add_unweighted_edge(0_usize, 1_usize);

        // Act
        let actual_count_minus_one_case = matrix_tree_minus_one_case.count_by_root(root_vertex);
        let actual_count_center_case = matrix_tree_center_case.count_by_root(root_vertex);
        let actual_count_plus_one_case = matrix_tree_plus_one_case.count_by_root(root_vertex);

        // Assert
        assert_eq!(actual_count_minus_one_case, expected_count_minus_one_case);
        assert_eq!(actual_count_center_case, expected_count_center_case);
        assert_eq!(actual_count_plus_one_case, expected_count_plus_one_case);
    }

    #[test]
    fn count_by_root_when_root_index_is_upper_boundary_plus_one_panics() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let invalid_root_vertex = 3_usize;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree.add_unweighted_edge(0_usize, 1_usize);

        // Act
        let panic_result = catch_unwind(AssertUnwindSafe(|| {
            let _ = matrix_tree.count_by_root(invalid_root_vertex);
        }));

        // Assert
        assert!(panic_result.is_err());
    }

    #[test]
    fn count_by_root_uses_weight_products_for_weighted_undirected_graph() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let root_vertex = 0_usize;
        let expected_count = 31_i64;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);

        let add_edge_from_vertex = 0_usize;
        let add_edge_to_vertex = 1_usize;
        let add_edge_capacity = 2_i64;
        let add_edge_from_vertex_2 = 1_usize;
        let add_edge_to_vertex_2 = 2_usize;
        let add_edge_capacity_2 = 3_i64;
        let add_edge_capacity_3 = 5_i64;

        // Act
        matrix_tree.add_edge(add_edge_from_vertex, add_edge_to_vertex, add_edge_capacity);
        matrix_tree.add_edge(
            add_edge_from_vertex_2,
            add_edge_to_vertex_2,
            add_edge_capacity_2,
        );
        matrix_tree.add_edge(
            add_edge_from_vertex,
            add_edge_to_vertex_2,
            add_edge_capacity_3,
        );
        let actual_count = matrix_tree.count_by_root(root_vertex);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn count_by_root_when_orientation_is_directed_in_arborescence_returns_expected_value() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::DirectedInArborescence;
        let root_vertex = 2_usize;
        let expected_count = 2_i64;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree.add_unweighted_edge(0_usize, 1_usize);
        matrix_tree.add_unweighted_edge(1_usize, 2_usize);
        matrix_tree.add_unweighted_edge(0_usize, 2_usize);

        // Act
        let actual_count = matrix_tree.count_by_root(root_vertex);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn count_by_root_when_orientation_is_directed_out_arborescence_returns_expected_value() {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::DirectedOutArborescence;
        let root_vertex = 0_usize;
        let expected_count = 2_i64;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree.add_unweighted_edge(0_usize, 1_usize);
        matrix_tree.add_unweighted_edge(0_usize, 2_usize);
        matrix_tree.add_unweighted_edge(1_usize, 2_usize);

        // Act
        let actual_count = matrix_tree.count_by_root(root_vertex);

        // Assert
        assert_eq!(actual_count, expected_count);
    }

    #[test]
    fn count_for_all_roots_when_vertex_count_is_zero_returns_empty_vector() {
        // Arrange
        let vertex_count = 0_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        let expected_counts: Vec<i64> = vec![];

        // Act
        let actual_counts = matrix_tree.count_for_all_roots();

        // Assert
        assert_eq!(actual_counts, expected_counts);
    }

    #[test]
    fn count_for_all_roots_when_undirected_triangle_graph_returns_same_value_at_lower_center_upper_roots(
    ) {
        // Arrange
        let vertex_count = 3_usize;
        let orientation = MatrixTreeOrientation::Undirected;
        let lower_root_index = 0_usize;
        let center_root_index = 1_usize;
        let upper_root_index = 2_usize;
        let expected_lower_count = 3_i64;
        let expected_center_count = 3_i64;
        let expected_upper_count = 3_i64;
        let mut matrix_tree = MatrixTreeTheorem::<i64>::new(vertex_count, orientation);
        matrix_tree.add_unweighted_edge(0_usize, 1_usize);
        matrix_tree.add_unweighted_edge(1_usize, 2_usize);
        matrix_tree.add_unweighted_edge(0_usize, 2_usize);

        // Act
        let actual_counts = matrix_tree.count_for_all_roots();
        let actual_lower_count = actual_counts[lower_root_index];
        let actual_center_count = actual_counts[center_root_index];
        let actual_upper_count = actual_counts[upper_root_index];

        // Assert
        assert_eq!(actual_lower_count, expected_lower_count);
        assert_eq!(actual_center_count, expected_center_count);
        assert_eq!(actual_upper_count, expected_upper_count);
    }
}
