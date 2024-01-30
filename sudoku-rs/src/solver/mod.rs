pub mod backtracking;
pub mod strategic;

// TODO: exact-cover based solver:
//  Could be more performant for bigger sudokus
//  References:
//  Section 4.4 Sequential DLX algorithm (0.3 seconds?)
//   https://github.com/huaminghuangtw/Parallel-Sudoku-Solver/blob/master/Project_Report.pdf
//  Rust exact cover solver
//   https://github.com/jw013/exact-cover-rs

// TODO: base adaptive solver
//  strategic solver gets better for base >= 4 than backtracking::Solver
//  even simple strategies seem to reduce the backtracking space enough to be more efficient.

#[cfg(test)]
mod test_util {
    use std::collections::HashSet;

    use crate::base::consts::Base2;
    use crate::base::SudokuBase;
    use crate::grid::Grid;

    pub(crate) fn assert_solver_single_solution<Base: SudokuBase>(
        mut solver: impl Iterator<Item = Grid<Base>>,
    ) {
        let solution = solver
            .next()
            .expect("Solver should produce at least one solution");

        assert!(
            solution.is_solved(),
            "The solution should be solved, instead got: {solution}"
        );

        assert!(
            solver.next().is_none(),
            "Solver should produce not more than one solution"
        );
    }

    pub(crate) fn assert_solver_all_solutions_base_2(solver: impl Iterator<Item = Grid<Base2>>) {
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
