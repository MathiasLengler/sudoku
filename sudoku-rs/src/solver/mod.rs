pub mod backtracking;
pub mod backtracking_bitset;
#[cfg(feature = "sudoku_emerentius")]
pub mod emerentius;
pub mod strategic;

// TODO: Solver trait

#[cfg(test)]
mod test_util {
    use std::collections::HashSet;

    use crate::base::consts::Base2;
    use crate::base::SudokuBase;
    use crate::grid::Grid;

    pub(crate) fn assert_solve_result<Base: SudokuBase>(solve_result: Option<Grid<Base>>) {
        assert!(solve_result.is_some());

        let grid = solve_result.unwrap();

        assert!(grid.is_solved());
    }

    pub(crate) fn assert_solver_solutions_base_2(solver: impl Iterator<Item = Grid<Base2>>) {
        const NUMBER_OF_BASE_2_SOLUTIONS: usize = 288;

        let solutions = solver.take(300).collect::<Vec<_>>();

        assert_eq!(solutions.len(), NUMBER_OF_BASE_2_SOLUTIONS);

        solutions
            .iter()
            .for_each(|solution| assert!(solution.is_solved()));

        let unique_solutions = solutions.into_iter().collect::<HashSet<_>>();

        assert_eq!(unique_solutions.len(), NUMBER_OF_BASE_2_SOLUTIONS);
    }
}
