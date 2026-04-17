#![forbid(unsafe_code)]

use thiserror::Error;

const DEFAULT_TOLERANCE: f64 = 1e-8;
const DEFAULT_MAX_COMBINATIONS: u128 = 500_000;

pub type Matrix<T> = Vec<Vec<T>>;
pub type MathResult<T> = Result<T, MathError>;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum MathError {
    #[error("matrix cannot be empty")]
    EmptyMatrix,
    #[error("matrix is not rectangular")]
    NonRectangularMatrix,
    #[error("matrix element ({row}, {col}) is out of bounds")]
    MatrixIndexOutOfBounds { row: usize, col: usize },
    #[error("group size must be greater than zero")]
    InvalidGroupSize,
    #[error("index {index} is out of bounds for vector length {len}")]
    VectorIndexOutOfBounds { index: usize, len: usize },
    #[error("constraint dimension mismatch: expected {expected}, got {actual}")]
    DimensionMismatch { expected: usize, actual: usize },
    #[error("problem has no variables")]
    EmptyProblem,
    #[error("too many active-set combinations to enumerate: {count}")]
    ProblemTooLarge { count: u128 },
    #[error("problem is infeasible")]
    Infeasible,
    #[error("price matrix row count must equal line configuration count")]
    PriceMatrixRowMismatch,
    #[error("price matrix column count must equal capacity row count")]
    PriceMatrixColumnMismatch,
    #[error("capacity rows must all have the same length as zbs")]
    CapacityMatrixMismatch,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MatrixElement<T> {
    pub row: usize,
    pub col: usize,
    pub value: T,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConstraintRelation {
    LessOrEqual,
    GreaterOrEqual,
    Equal,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GoalType {
    Minimize,
    Maximize,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearConstraint {
    pub coefficients: Vec<f64>,
    pub relation: ConstraintRelation,
    pub value: f64,
}

impl LinearConstraint {
    pub fn new(
        coefficients: impl Into<Vec<f64>>,
        relation: ConstraintRelation,
        value: f64,
    ) -> Self {
        Self {
            coefficients: coefficients.into(),
            relation,
            value,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearObjective {
    pub coefficients: Vec<f64>,
    pub constant: f64,
}

impl LinearObjective {
    pub fn new(coefficients: impl Into<Vec<f64>>, constant: f64) -> Self {
        Self {
            coefficients: coefficients.into(),
            constant,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearProgrammingProblem {
    pub goal: GoalType,
    pub objective: LinearObjective,
    pub constraints: Vec<LinearConstraint>,
}

impl LinearProgrammingProblem {
    pub fn new(
        goal: GoalType,
        objective: LinearObjective,
        constraints: impl Into<Vec<LinearConstraint>>,
    ) -> Self {
        Self {
            goal,
            objective,
            constraints: constraints.into(),
        }
    }

    pub fn variable_count(&self) -> usize {
        self.objective.coefficients.len()
    }

    pub fn solve(&self) -> MathResult<LinearProgrammingSolution> {
        solve_problem(self)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct LinearProgrammingSolution {
    pub point: Vec<f64>,
    pub value: f64,
    pub active_constraint_indexes: Vec<usize>,
}

pub fn transpose_matrix(matrix: &[Vec<f64>]) -> MathResult<Matrix<f64>> {
    let cols = validate_rectangular(matrix)?;
    let rows = matrix.len();

    let mut transposed = vec![vec![0.0; rows]; cols];
    for (row_index, row) in matrix.iter().enumerate() {
        for (col_index, value) in row.iter().enumerate() {
            transposed[col_index][row_index] = *value;
        }
    }
    Ok(transposed)
}

pub fn create_matrix<T: Clone>(rows: usize, cols: usize, init_value: T) -> Matrix<T> {
    vec![vec![init_value; cols]; rows]
}

pub fn convert_rows_to_f64<T, R>(rows: &[R]) -> Matrix<f64>
where
    T: Copy + Into<f64>,
    R: AsRef<[T]>,
{
    rows.iter()
        .map(|row| row.as_ref().iter().copied().map(Into::into).collect())
        .collect()
}

pub fn get_matrix_element<T>(matrix: &[Vec<T>], row: usize, col: usize) -> MathResult<&T> {
    validate_rectangular(matrix)?;
    matrix
        .get(row)
        .and_then(|r| r.get(col))
        .ok_or(MathError::MatrixIndexOutOfBounds { row, col })
}

pub fn set_matrix_elements<T: Clone>(
    matrix: &[Vec<T>],
    updates: &[MatrixElement<T>],
) -> MathResult<Matrix<T>> {
    validate_rectangular(matrix)?;
    let mut result = matrix.to_vec();
    for update in updates {
        let cell = result
            .get_mut(update.row)
            .and_then(|row| row.get_mut(update.col))
            .ok_or(MathError::MatrixIndexOutOfBounds {
                row: update.row,
                col: update.col,
            })?;
        *cell = update.value.clone();
    }
    Ok(result)
}

pub fn generate_vector(
    rows: usize,
    cols: usize,
    indices_1_based: &[usize],
) -> MathResult<Vec<f64>> {
    let len = rows * cols;
    let mut vector = vec![0.0; len];
    for index in indices_1_based {
        if *index == 0 || *index > len {
            return Err(MathError::VectorIndexOutOfBounds { index: *index, len });
        }
        vector[index - 1] = 1.0;
    }
    Ok(vector)
}

pub fn generate_index_groups(total: usize, group_size: usize) -> MathResult<Vec<Vec<usize>>> {
    if group_size == 0 {
        return Err(MathError::InvalidGroupSize);
    }
    Ok((0..(total / group_size))
        .map(|group| ((group * group_size)..((group + 1) * group_size)).collect())
        .collect())
}

pub fn set_interval_elements(base_row: &[i32], group_size: usize) -> MathResult<Vec<Vec<i32>>> {
    let groups = generate_index_groups(base_row.len(), group_size)?;
    groups
        .into_iter()
        .map(|group| {
            let updates = group
                .into_iter()
                .map(|col| MatrixElement {
                    row: 0,
                    col,
                    value: 1,
                })
                .collect::<Vec<_>>();
            set_matrix_elements(&[base_row.to_vec()], &updates).map(|rows| rows[0].clone())
        })
        .collect()
}

pub fn solve_linear_programming(
    zbs: &[f64],
    capacities: &[Vec<f64>],
    line_configurations: &[f64],
) -> MathResult<Vec<f64>> {
    let ship_types = capacities.len();
    let zero_prices = vec![vec![0.0; ship_types]; line_configurations.len()];
    solve_linear_programming_with_prices(zbs, capacities, line_configurations, &zero_prices)
        .map(|solution| solution.point)
}

pub fn solve_linear_programming_with_prices(
    zbs: &[f64],
    capacities: &[Vec<f64>],
    line_configurations: &[f64],
    price_matrix: &[Vec<f64>],
) -> MathResult<LinearProgrammingSolution> {
    validate_transport_dimensions(zbs, capacities, line_configurations, price_matrix)?;

    let ship_types = capacities.len();
    let lines = line_configurations.len();
    let dimension = ship_types * lines;

    let mut constraints = Vec::with_capacity(lines + zbs.len());

    for (line_index, line_total) in line_configurations.iter().enumerate() {
        let mut coefficients = vec![0.0; dimension];
        for ship_index in 0..ship_types {
            coefficients[line_index * ship_types + ship_index] = 1.0;
        }
        constraints.push(LinearConstraint::new(
            coefficients,
            ConstraintRelation::Equal,
            *line_total,
        ));
    }

    for demand_index in 0..zbs.len() {
        let mut coefficients = vec![0.0; dimension];
        for line_index in 0..lines {
            for ship_index in 0..ship_types {
                coefficients[line_index * ship_types + ship_index] =
                    capacities[ship_index][demand_index];
            }
        }
        constraints.push(LinearConstraint::new(
            coefficients,
            ConstraintRelation::GreaterOrEqual,
            zbs[demand_index],
        ));
    }

    let objective = LinearObjective::new(
        price_matrix
            .iter()
            .flat_map(|row| row.iter().copied())
            .collect::<Vec<_>>(),
        0.0,
    );
    LinearProgrammingProblem::new(GoalType::Minimize, objective, constraints).solve()
}

fn solve_problem(problem: &LinearProgrammingProblem) -> MathResult<LinearProgrammingSolution> {
    let variable_count = problem.variable_count();
    if variable_count == 0 {
        return Err(MathError::EmptyProblem);
    }

    for constraint in &problem.constraints {
        if constraint.coefficients.len() != variable_count {
            return Err(MathError::DimensionMismatch {
                expected: variable_count,
                actual: constraint.coefficients.len(),
            });
        }
    }

    let mut equality_constraints = Vec::new();
    let mut candidate_constraints = Vec::new();

    for (index, constraint) in problem.constraints.iter().enumerate() {
        match constraint.relation {
            ConstraintRelation::Equal => equality_constraints.push((index, constraint.clone())),
            ConstraintRelation::LessOrEqual | ConstraintRelation::GreaterOrEqual => {
                candidate_constraints.push((index, constraint.clone()))
            }
        }
    }

    for variable_index in 0..variable_count {
        let mut coefficients = vec![0.0; variable_count];
        coefficients[variable_index] = 1.0;
        candidate_constraints.push((
            problem.constraints.len() + variable_index,
            LinearConstraint::new(coefficients, ConstraintRelation::Equal, 0.0),
        ));
    }

    if equality_constraints.len() > variable_count {
        return Err(MathError::Infeasible);
    }

    let needed_candidates = variable_count - equality_constraints.len();
    if needed_candidates > candidate_constraints.len() {
        return Err(MathError::Infeasible);
    }

    let combinations = combination_count(candidate_constraints.len(), needed_candidates);
    if combinations > DEFAULT_MAX_COMBINATIONS {
        return Err(MathError::ProblemTooLarge {
            count: combinations,
        });
    }

    let mut best: Option<LinearProgrammingSolution> = None;
    let mut current = Vec::with_capacity(needed_candidates);
    enumerate_combinations(
        candidate_constraints.len(),
        needed_candidates,
        0,
        &mut current,
        &mut |choice| {
            let mut active_constraints = equality_constraints
                .iter()
                .map(|(index, constraint)| (*index, constraint.clone()))
                .collect::<Vec<_>>();
            active_constraints.extend(
                choice
                    .iter()
                    .map(|candidate_index| candidate_constraints[*candidate_index].clone()),
            );

            if let Some(point) = solve_active_set(variable_count, &active_constraints) {
                if is_feasible(problem, &point) {
                    let value = evaluate_objective(&problem.objective, &point);
                    let solution = LinearProgrammingSolution {
                        point,
                        value,
                        active_constraint_indexes: active_constraints
                            .iter()
                            .map(|(index, _)| *index)
                            .collect(),
                    };
                    if is_better(problem.goal, best.as_ref(), &solution) {
                        best = Some(solution);
                    }
                }
            }
        },
    );

    best.ok_or(MathError::Infeasible)
}

fn solve_active_set(
    variable_count: usize,
    active_constraints: &[(usize, LinearConstraint)],
) -> Option<Vec<f64>> {
    if active_constraints.len() != variable_count {
        return None;
    }

    let matrix = active_constraints
        .iter()
        .map(|(_, constraint)| constraint.coefficients.clone())
        .collect::<Vec<_>>();
    let rhs = active_constraints
        .iter()
        .map(|(_, constraint)| constraint.value)
        .collect::<Vec<_>>();

    solve_linear_system(&matrix, &rhs)
}

fn solve_linear_system(matrix: &[Vec<f64>], rhs: &[f64]) -> Option<Vec<f64>> {
    if matrix.is_empty() || matrix.len() != rhs.len() {
        return None;
    }

    let size = matrix.len();
    if matrix.iter().any(|row| row.len() != size) {
        return None;
    }

    let mut augmented = matrix
        .iter()
        .zip(rhs.iter())
        .map(|(row, value)| {
            let mut out = row.clone();
            out.push(*value);
            out
        })
        .collect::<Vec<_>>();

    for pivot in 0..size {
        let pivot_row = (pivot..size)
            .max_by(|left, right| {
                augmented[*left][pivot]
                    .abs()
                    .partial_cmp(&augmented[*right][pivot].abs())
                    .expect("finite comparison should work")
            })
            .unwrap_or(pivot);

        if augmented[pivot_row][pivot].abs() <= DEFAULT_TOLERANCE {
            return None;
        }

        if pivot_row != pivot {
            augmented.swap(pivot_row, pivot);
        }

        let divisor = augmented[pivot][pivot];
        for col in pivot..=size {
            augmented[pivot][col] /= divisor;
        }

        for row in 0..size {
            if row == pivot {
                continue;
            }
            let factor = augmented[row][pivot];
            if factor.abs() <= DEFAULT_TOLERANCE {
                continue;
            }
            for col in pivot..=size {
                augmented[row][col] -= factor * augmented[pivot][col];
            }
        }
    }

    Some(augmented.into_iter().map(|row| row[size]).collect())
}

fn is_feasible(problem: &LinearProgrammingProblem, point: &[f64]) -> bool {
    if point
        .iter()
        .any(|value| !value.is_finite() || *value < -DEFAULT_TOLERANCE)
    {
        return false;
    }

    problem.constraints.iter().all(|constraint| {
        let lhs = dot(&constraint.coefficients, point);
        match constraint.relation {
            ConstraintRelation::LessOrEqual => lhs <= constraint.value + DEFAULT_TOLERANCE,
            ConstraintRelation::GreaterOrEqual => lhs + DEFAULT_TOLERANCE >= constraint.value,
            ConstraintRelation::Equal => (lhs - constraint.value).abs() <= DEFAULT_TOLERANCE,
        }
    })
}

fn evaluate_objective(objective: &LinearObjective, point: &[f64]) -> f64 {
    objective.constant + dot(&objective.coefficients, point)
}

fn is_better(
    goal: GoalType,
    current_best: Option<&LinearProgrammingSolution>,
    candidate: &LinearProgrammingSolution,
) -> bool {
    match current_best {
        None => true,
        Some(best) => match goal {
            GoalType::Minimize => candidate.value + DEFAULT_TOLERANCE < best.value,
            GoalType::Maximize => candidate.value > best.value + DEFAULT_TOLERANCE,
        },
    }
}

fn dot(left: &[f64], right: &[f64]) -> f64 {
    left.iter().zip(right.iter()).map(|(a, b)| a * b).sum()
}

fn combination_count(total: usize, choose: usize) -> u128 {
    if choose > total {
        return 0;
    }
    let choose = choose.min(total - choose);
    let mut result = 1u128;
    for step in 0..choose {
        result = result * (total - step) as u128 / (step + 1) as u128;
    }
    result
}

fn enumerate_combinations(
    total: usize,
    choose: usize,
    start: usize,
    current: &mut Vec<usize>,
    callback: &mut impl FnMut(&[usize]),
) {
    if choose == 0 {
        callback(current);
        return;
    }

    for next in start..=(total - choose) {
        current.push(next);
        enumerate_combinations(total, choose - 1, next + 1, current, callback);
        current.pop();
    }
}

fn validate_rectangular<T>(matrix: &[Vec<T>]) -> MathResult<usize> {
    let first = matrix.first().ok_or(MathError::EmptyMatrix)?;
    let cols = first.len();
    if matrix.iter().any(|row| row.len() != cols) {
        return Err(MathError::NonRectangularMatrix);
    }
    Ok(cols)
}

fn validate_transport_dimensions(
    zbs: &[f64],
    capacities: &[Vec<f64>],
    line_configurations: &[f64],
    price_matrix: &[Vec<f64>],
) -> MathResult<()> {
    let ship_types = capacities.len();
    if capacities.iter().any(|row| row.len() != zbs.len()) {
        return Err(MathError::CapacityMatrixMismatch);
    }
    if price_matrix.len() != line_configurations.len() {
        return Err(MathError::PriceMatrixRowMismatch);
    }
    if price_matrix.iter().any(|row| row.len() != ship_types) {
        return Err(MathError::PriceMatrixColumnMismatch);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn transpose_and_matrix_updates_work() {
        let matrix = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let transposed = transpose_matrix(&matrix).expect("matrix should transpose");
        assert_eq!(
            transposed,
            vec![vec![1.0, 4.0], vec![2.0, 5.0], vec![3.0, 6.0]]
        );

        let base = create_matrix(1, 6, 0);
        let updated = set_matrix_elements(
            &base,
            &[
                MatrixElement {
                    row: 0,
                    col: 1,
                    value: 1,
                },
                MatrixElement {
                    row: 0,
                    col: 4,
                    value: 1,
                },
            ],
        )
        .expect("matrix should update");
        assert_eq!(updated, vec![vec![0, 1, 0, 0, 1, 0]]);
    }

    #[test]
    fn vector_generation_matches_jvm_shape() {
        let vector = generate_vector(3, 4, &[1, 5, 9]).expect("vector should generate");
        assert_eq!(
            vector,
            vec![1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0]
        );

        let groups = generate_index_groups(12, 4).expect("groups should generate");
        assert_eq!(
            groups,
            vec![vec![0, 1, 2, 3], vec![4, 5, 6, 7], vec![8, 9, 10, 11]]
        );

        let rows = set_interval_elements(&[0, 0, 0, 0, 0, 0], 2).expect("rows should generate");
        assert_eq!(
            rows,
            vec![
                vec![1, 1, 0, 0, 0, 0],
                vec![0, 0, 1, 1, 0, 0],
                vec![0, 0, 0, 0, 1, 1]
            ]
        );
    }

    #[test]
    fn small_linear_problem_solves_by_vertex_enumeration() {
        let problem = LinearProgrammingProblem::new(
            GoalType::Minimize,
            LinearObjective::new(vec![3.0, 1.0], 0.0),
            vec![
                LinearConstraint::new(vec![1.0, 1.0], ConstraintRelation::Equal, 5.0),
                LinearConstraint::new(vec![1.0, 0.0], ConstraintRelation::GreaterOrEqual, 2.0),
                LinearConstraint::new(vec![0.0, 1.0], ConstraintRelation::GreaterOrEqual, 1.0),
            ],
        );

        let solution = problem.solve().expect("problem should solve");
        assert!((solution.point[0] - 2.0).abs() < 1e-6);
        assert!((solution.point[1] - 3.0).abs() < 1e-6);
        assert!((solution.value - 9.0).abs() < 1e-6);
    }

    #[test]
    fn transportation_style_problem_uses_row_major_prices() {
        let zbs = vec![7.0];
        let capacities = vec![vec![5.0], vec![2.0]];
        let lines = vec![1.0, 1.0];
        let prices = vec![vec![1.0, 10.0], vec![2.0, 3.0]];

        let solution = solve_linear_programming_with_prices(&zbs, &capacities, &lines, &prices)
            .expect("transportation problem should solve");

        assert_eq!(solution.point.len(), 4);
        assert!((solution.point[0] - 1.0).abs() < 1e-6);
        assert!((solution.point[2] - 1.0).abs() < 1e-6);
        assert!((solution.value - 3.0).abs() < 1e-6);
    }
}
