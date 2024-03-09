use serde::Serialize;
#[cfg(feature = "wasm")]
use ts_rs::TS;

use enum_dispatch::enum_dispatch;

use crate::base::consts::*;
use crate::base::{DynamicBase, SudokuBase};
use crate::cell::dynamic::DynamicCandidates;
use crate::cell::dynamic::DynamicCell;
use crate::cell::dynamic::DynamicValue;
use crate::error::{Error, Result};
use crate::generator::{DynamicGeneratorSettings, GeneratorProgress};
use crate::grid::format::DynamicGridFormat;
use crate::grid::Grid;
use crate::position::DynamicPosition;
use crate::solver::strategic::deduction::transport::TransportDeductions;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::sudoku::settings::Settings as SudokuSettings;
use crate::sudoku::Sudoku;

/// All methods provided by `Sudoku`, which:
/// - do not depend on the generic `SudokuBase`
/// - can't change the size of the board
/// - take self by reference / are static
#[enum_dispatch]
pub trait DynamicSudokuActions {
    // actions that handle base-dependend types
    fn set_value(&mut self, pos: DynamicPosition, value: DynamicValue) -> Result<()>;
    fn set_or_toggle_value(&mut self, pos: DynamicPosition, value: DynamicValue) -> Result<()>;
    fn set_candidates(&mut self, pos: DynamicPosition, candidates: DynamicCandidates)
        -> Result<()>;
    fn toggle_candidate(&mut self, pos: DynamicPosition, candidate: DynamicValue) -> Result<()>;
    fn set_candidate(&mut self, pos: DynamicPosition, candidate: DynamicValue) -> Result<()>;
    fn delete_candidate(&mut self, pos: DynamicPosition, candidate: DynamicValue) -> Result<()>;
    fn delete(&mut self, pos: DynamicPosition) -> Result<()>;
    fn try_strategies(
        &mut self,
        strategies: Vec<DynamicStrategy>,
    ) -> Result<DynamicTryStrategiesReturn>;
    fn apply_deductions(&mut self, deductions: TransportDeductions) -> Result<()>;

    // actions that don't depend on base
    fn set_all_direct_candidates(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
    fn settings(&self) -> SudokuSettings;
    fn update_settings(&mut self, settings: SudokuSettings);
    fn export(&self, format: &DynamicGridFormat) -> String;
}

/// A game of Sudoku which is able to change the size of the board at runtime.
#[enum_dispatch(DynamicSudokuActions)]
#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub enum DynamicSudoku {
    Base2(Sudoku<Base2>),
    Base3(Sudoku<Base3>),
    Base4(Sudoku<Base4>),
    Base5(Sudoku<Base5>),
}

// TODO: re-evaluate constructors vs replacing &mut self methods (*self =)
/// Actions which can change the currently used base
impl DynamicSudoku {
    pub fn new(base: u8) -> Result<Self> {
        let base: DynamicBase = base.try_into()?;

        Ok(match base {
            DynamicBase::Base2 => Self::Base2(Sudoku::<Base2>::new()),
            DynamicBase::Base3 => Self::Base3(Sudoku::<Base3>::new()),
            DynamicBase::Base4 => Self::Base4(Sudoku::<Base4>::new()),
            DynamicBase::Base5 => Self::Base5(Sudoku::<Base5>::new()),
        })
    }
    pub fn generate(
        &mut self,
        dynamic_generator_settings: DynamicGeneratorSettings,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<()> {
        let base: DynamicBase = dynamic_generator_settings.base.try_into()?;

        *self = match base {
            DynamicBase::Base2 => Self::Base2(Sudoku::<Base2>::generate(
                dynamic_generator_settings.try_into()?,
                self.settings(),
                on_progress,
            )?),
            DynamicBase::Base3 => Self::Base3(Sudoku::<Base3>::generate(
                dynamic_generator_settings.try_into()?,
                self.settings(),
                on_progress,
            )?),
            DynamicBase::Base4 => Self::Base4(Sudoku::<Base4>::generate(
                dynamic_generator_settings.try_into()?,
                self.settings(),
                on_progress,
            )?),
            DynamicBase::Base5 => Self::Base5(Sudoku::<Base5>::generate(
                dynamic_generator_settings.try_into()?,
                self.settings(),
                on_progress,
            )?),
        };

        Ok(())
    }
    pub fn import(&mut self, input: &str) -> Result<()> {
        *self = input.try_into()?;

        Ok(())
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize)]
pub struct DynamicTryStrategiesReturn(pub Option<(DynamicStrategy, TransportDeductions)>);

impl TryFrom<Vec<DynamicCell>> for DynamicSudoku {
    type Error = Error;

    fn try_from(views: Vec<DynamicCell>) -> Result<Self> {
        Ok(match DynamicBase::try_from_cell_count_usize(views.len())? {
            DynamicBase::Base2 => Self::Base2(Sudoku::<Base2>::with_grid(views.try_into()?)),
            DynamicBase::Base3 => Self::Base3(Sudoku::<Base3>::with_grid(views.try_into()?)),
            DynamicBase::Base4 => Self::Base4(Sudoku::<Base4>::with_grid(views.try_into()?)),
            DynamicBase::Base5 => Self::Base5(Sudoku::<Base5>::with_grid(views.try_into()?)),
        })
    }
}

impl TryFrom<Vec<Vec<DynamicCell>>> for DynamicSudoku {
    type Error = Error;

    fn try_from(blocks: Vec<Vec<DynamicCell>>) -> Result<Self> {
        let sudoku = match DynamicBase::try_from_cell_count_usize(
            blocks.iter().map(|block| block.len()).sum(),
        )? {
            DynamicBase::Base2 => {
                Self::Base2(Sudoku::with_grid(Grid::<Base2>::try_from_blocks(blocks)?))
            }
            DynamicBase::Base3 => {
                Self::Base3(Sudoku::with_grid(Grid::<Base3>::try_from_blocks(blocks)?))
            }
            DynamicBase::Base4 => {
                Self::Base4(Sudoku::with_grid(Grid::<Base4>::try_from_blocks(blocks)?))
            }
            DynamicBase::Base5 => {
                Self::Base5(Sudoku::with_grid(Grid::<Base5>::try_from_blocks(blocks)?))
            }
        };

        Ok(sudoku)
    }
}

impl TryFrom<&str> for DynamicSudoku {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self> {
        DynamicGridFormat::detect_and_parse(input)?.try_into()
    }
}

#[cfg(test)]
mod tests {
    // use super::*;

    // TODO: re-enable

    // #[test]
    // fn test_try_from_str() {
    //     let inputs = [INPUT_CANDIDATES, INPUT_GIVENS_LINE, INPUT_GIVENS_GRID];
    //
    //     for input in inputs {
    //         DynamicSudoku::try_from(input).unwrap();
    //     }
    // }
}
