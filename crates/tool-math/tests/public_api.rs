use addzero_math::*;

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
