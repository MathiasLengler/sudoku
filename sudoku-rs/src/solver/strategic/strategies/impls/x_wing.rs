use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::Deductions;
use crate::solver::strategic::strategies::Strategy;
use crate::solver::strategic::strategies::StrategyScore;

// TODO: implement: https://www.sudokuwiki.org/X_Wing_Strategy

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct XWing;

impl Strategy for XWing {
    fn name(self) -> &'static str {
        "XWing"
    }

    fn score(self) -> StrategyScore {
        200
    }

    fn execute<Base: SudokuBase>(self, _grid: &Grid<Base>) -> Result<Deductions<Base>> {
        todo!("X-Wing strategy not yet implemented")
    }
}
