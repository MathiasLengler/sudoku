use enum_dispatch::enum_dispatch;

use crate::base::consts::*;
use crate::base::match_base_enum;
use crate::base::BaseEnum;
use crate::cell::dynamic::DynamicCandidates;
use crate::cell::dynamic::DynamicCell;
use crate::cell::dynamic::DynamicValue;
use crate::error::{Error, Result};
use crate::generator::multi_shot::DynamicMultiShotGeneratorSettings;
use crate::generator::multi_shot::MultiShotGeneratorProgress;
use crate::generator::{DynamicGeneratorSettings, GeneratorProgress};
use crate::grid::dynamic::DynamicGrid;
use crate::grid::format::GridFormatEnum;
use crate::grid::Grid;
use crate::position::DynamicPosition;
use crate::solver::strategic::deduction::transport::TransportDeductions;
use crate::solver::strategic::strategies::StrategyEnum;
use crate::solver::strategic::DynamicSolveStep;
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
    fn try_strategies(&mut self, strategies: Vec<StrategyEnum>)
        -> Result<Option<DynamicSolveStep>>;
    fn apply_deductions(&mut self, deductions: TransportDeductions) -> Result<()>;

    // actions that don't depend on base
    fn set_all_direct_candidates(&mut self);
    fn undo(&mut self);
    fn redo(&mut self);
    fn settings(&self) -> SudokuSettings;
    fn update_settings(&mut self, settings: SudokuSettings);
    fn export(&self, format: GridFormatEnum) -> String;

    fn to_dynamic_grid(&self) -> DynamicGrid;
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
        let base: BaseEnum = base.try_into()?;

        Ok(match base {
            BaseEnum::Base2 => Self::Base2(Sudoku::<Base2>::new()),
            BaseEnum::Base3 => Self::Base3(Sudoku::<Base3>::new()),
            BaseEnum::Base4 => Self::Base4(Sudoku::<Base4>::new()),
            BaseEnum::Base5 => Self::Base5(Sudoku::<Base5>::new()),
        })
    }
    pub fn generate(
        &mut self,
        dynamic_generator_settings: DynamicGeneratorSettings,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<()> {
        let base: BaseEnum = dynamic_generator_settings.base.try_into()?;

        *self = match_base_enum!(
            base,
            Self::from(Sudoku::<Base>::generate(
                dynamic_generator_settings.try_into()?,
                self.settings(),
                on_progress,
            )?)
        );

        Ok(())
    }

    pub fn generate_multi_shot(
        &mut self,
        multi_shot_generator_settings: DynamicMultiShotGeneratorSettings,
        on_progress: impl FnMut(MultiShotGeneratorProgress) -> Result<()>,
    ) -> Result<()> {
        let base: BaseEnum = multi_shot_generator_settings
            .generator_settings
            .base
            .try_into()?;

        *self = match_base_enum!(
            base,
            Self::from(Sudoku::<Base>::generate_multi_shot(
                multi_shot_generator_settings.try_into()?,
                self.settings(),
                on_progress,
            )?)
        );

        Ok(())
    }
    pub fn import(&mut self, input: &str) -> Result<()> {
        *self = input.try_into()?;

        Ok(())
    }
}

impl TryFrom<Vec<DynamicCell>> for DynamicSudoku {
    type Error = Error;

    fn try_from(views: Vec<DynamicCell>) -> Result<Self> {
        Ok(match_base_enum!(
            BaseEnum::try_from_cell_count_usize(views.len())?,
            Sudoku::<Base>::with_grid(views.try_into()?).into()
        ))
    }
}

impl TryFrom<Vec<Vec<DynamicCell>>> for DynamicSudoku {
    type Error = Error;

    fn try_from(blocks: Vec<Vec<DynamicCell>>) -> Result<Self> {
        Ok(match_base_enum!(
            BaseEnum::try_from_cell_count_usize(blocks.iter().map(|block| block.len()).sum(),)?,
            Sudoku::with_grid(Grid::<Base>::try_from_blocks(blocks)?).into()
        ))
    }
}

impl TryFrom<DynamicGrid> for DynamicSudoku {
    type Error = Error;

    fn try_from(dynamic_grid: DynamicGrid) -> Result<Self> {
        Ok(match_base_enum!(
            dynamic_grid.base(),
            Sudoku::<Base>::with_grid(dynamic_grid.try_into()?).into()
        ))
    }
}

impl TryFrom<&str> for DynamicSudoku {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self> {
        GridFormatEnum::detect_and_parse(input)?.try_into()
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
