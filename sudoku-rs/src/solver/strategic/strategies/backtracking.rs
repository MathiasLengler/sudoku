use crate::base::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking::Solver;
use crate::solver::strategic::strategies::deduction::StrategyDeduction;

use super::Strategy;

#[derive(Debug)]
pub struct Backtracking;

impl<Base: SudokuBase> Strategy<Base> for Backtracking {
    fn execute(&self, grid: &Grid<Base>) -> Vec<StrategyDeduction<Base>> {
        todo!("port Backtracking Strategy to StrategyDeduction")

        // let mut solver = Solver::new(grid);
        //
        // if solver.next().is_some() {
        //     solver.into_empty_positions()
        // } else {
        //     vec![]
        // }
    }
}

#[cfg(test)]
mod tests {
    use crate::samples;

    use super::*;

    #[test]
    fn test_backtracking() {
        let mut grid = samples::base_3().first().unwrap().clone();

        grid.fix_all_values();

        let modified_positions = Backtracking.execute(&mut grid);

        assert!(grid.is_solved());

        assert_eq!(modified_positions.len(), modified_positions.len());
    }
}
