pub mod backtracking;
pub mod backtracking_bitset;
pub mod strategic;

// TODO: Solver trait

#[cfg(test)]
mod test_util {
    use crate::base::SudokuBase;
    use crate::grid::Grid;

    pub fn assert_solve_result<Base: SudokuBase>(solve_result: Option<Grid<Base>>) {
        assert!(solve_result.is_some());

        let grid = solve_result.unwrap();

        assert!(grid.is_solved());
    }
}
