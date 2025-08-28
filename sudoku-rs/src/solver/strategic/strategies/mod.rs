use std::fmt::Debug;

use enum_dispatch::enum_dispatch;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::grid::Grid;
use crate::solver::strategic::deduction::Deductions;

pub use impls::*;

pub use strategy_enum::*;

mod strategy_enum;

mod impls;

pub type StrategyScore = u64;

#[enum_dispatch(StrategyEnum)]
pub trait Strategy: Debug + Copy + Clone + Eq + Sized {
    /// The name of the strategy.
    fn name(self) -> &'static str;

    /// The score/difficulty of the strategy.
    /// Higher scores are more difficult.
    fn score(self) -> StrategyScore;

    // TODO: optimize with param: enable reasons
    //  reasons are only needed for debugging and hinting, not for strategic generation.

    /// Execute this strategy on the given grid. Returns a list of deductions.
    fn execute<Base: SudokuBase>(self, grid: &Grid<Base>) -> Result<Deductions<Base>>;

    /// Execute this strategy on the given grid and applies the deductions to it.
    /// Returns a list of applied deductions.
    fn execute_and_apply<Base: SudokuBase>(
        self,
        grid: &mut Grid<Base>,
    ) -> Result<Deductions<Base>> {
        let deductions = self.execute(grid)?;
        deductions.apply(grid)?;
        Ok(deductions)
    }
}

#[cfg(test)]
mod test_util {
    use crate::base::SudokuBase;
    use crate::grid::Grid;
    use crate::solver::strategic::deduction::Deductions;

    pub(crate) fn assert_deductions<Base: SudokuBase>(
        deductions: &Deductions<Base>,
        expected_deductions: &Deductions<Base>,
    ) {
        assert_eq!(
            deductions, expected_deductions,
            "\n{deductions}\n!=\n{expected_deductions}"
        );
    }

    pub(crate) fn assert_deductions_with_grid<Base: SudokuBase>(
        deductions: &Deductions<Base>,
        expected_deductions: &Deductions<Base>,
        grid: &mut Grid<Base>,
    ) {
        assert_deductions(deductions, expected_deductions);

        deductions.apply(grid).unwrap();
    }
}
