use std::any::Any;
use std::convert::{TryFrom, TryInto};

use anyhow::{anyhow, bail, format_err};
use serde::Serialize;
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use game::DynamicSudoku;
pub use game::Game;

use crate::base::consts::*;
use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::generator::{DynamicGeneratorSettings, GeneratorProgress, GeneratorSettings};
use crate::grid::format::DynamicGridFormat;
use crate::grid::Grid;
use crate::position::DynamicPosition;
use crate::solver::strategic::deduction::transport::TransportDeductions;
use crate::solver::strategic::strategies::DynamicStrategy;
use crate::sudoku::settings::Settings as SudokuSettings;
use crate::sudoku::Sudoku;

mod game {
    use enum_dispatch::enum_dispatch;

    use super::*;

    #[enum_dispatch]
    pub trait Game {
        fn set_value(&mut self, pos: DynamicPosition, value: u8) -> Result<()>;
        fn set_or_toggle_value(&mut self, pos: DynamicPosition, value: u8) -> Result<()>;
        fn set_candidates(&mut self, pos: DynamicPosition, candidates: Vec<u8>) -> Result<()>;
        fn toggle_candidate(&mut self, pos: DynamicPosition, candidate: u8) -> Result<()>;
        fn set_candidate(&mut self, pos: DynamicPosition, candidate: u8) -> Result<()>;
        fn delete_candidate(&mut self, pos: DynamicPosition, candidate: u8) -> Result<()>;
        fn delete(&mut self, pos: DynamicPosition) -> Result<()>;
        fn set_all_direct_candidates(&mut self);
        fn undo(&mut self);
        fn redo(&mut self);
        fn settings(&self) -> SudokuSettings;
        fn update_settings(&mut self, settings: SudokuSettings);
        fn export(&self, format: &DynamicGridFormat) -> String;
    }

    /// A game of Sudoku which is able to change the size of the board at runtime.
    #[enum_dispatch(Game)]
    #[derive(Eq, PartialEq, Hash, Clone, Debug)]
    pub enum DynamicSudoku {
        Base2(Sudoku<Base2>),
        Base3(Sudoku<Base3>),
        Base4(Sudoku<Base4>),
        Base5(Sudoku<Base5>),
    }
}

// Requires runtime base
impl DynamicSudoku {
    pub fn new(base: u8) -> Result<Self> {
        Ok(match base {
            2 => Self::Base2(Sudoku::<Base2>::new()),
            3 => Self::Base3(Sudoku::<Base3>::new()),
            4 => Self::Base4(Sudoku::<Base4>::new()),
            5 => Self::Base5(Sudoku::<Base5>::new()),
            unexpected_base => bail!(Self::unexpected_base_err(unexpected_base)),
        })
    }
    pub fn with_sudoku<Base: SudokuBase>(sudoku: Sudoku<Base>) -> Result<Self> {
        let any_sudoku: Box<dyn Any> = Box::new(sudoku);

        Ok(match Base::BASE {
            2 => Self::Base2(*(any_sudoku.downcast().unwrap())),
            3 => Self::Base3(*(any_sudoku.downcast().unwrap())),
            4 => Self::Base4(*(any_sudoku.downcast().unwrap())),
            5 => Self::Base5(*(any_sudoku.downcast().unwrap())),
            _ => bail!(Self::unexpected_base_err(Base::BASE)),
        })
    }
    pub fn generate(
        &mut self,
        dynamic_generator_settings: DynamicGeneratorSettings,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<()> {
        let DynamicGeneratorSettings {
            base,
            target,
            strategies,
            seed,
        } = dynamic_generator_settings;

        *self = match base {
            2 => Self::Base2(Sudoku::<Base2>::generate(
                GeneratorSettings {
                    target,
                    givens_grid: None,
                    strategies,
                    seed,
                },
                self.settings(),
                on_progress,
            )?),
            3 => Self::Base3(Sudoku::<Base3>::generate(
                GeneratorSettings {
                    target,
                    givens_grid: None,
                    strategies,
                    seed,
                },
                self.settings(),
                on_progress,
            )?),
            4 => Self::Base4(Sudoku::<Base4>::generate(
                GeneratorSettings {
                    target,
                    givens_grid: None,
                    strategies,
                    seed,
                },
                self.settings(),
                on_progress,
            )?),
            5 => Self::Base5(Sudoku::<Base5>::generate(
                GeneratorSettings {
                    target,
                    givens_grid: None,
                    strategies,
                    seed,
                },
                self.settings(),
                on_progress,
            )?),
            unexpected_base => bail!(Self::unexpected_base_err(unexpected_base)),
        };

        Ok(())
    }
    pub fn import(&mut self, input: &str) -> Result<()> {
        *self = input.try_into()?;

        Ok(())
    }

    pub fn try_strategies(
        &mut self,
        strategies: Vec<DynamicStrategy>,
    ) -> Result<DynamicTryStrategiesReturn> {
        fn inner<Base: SudokuBase>(
            sudoku: &mut Sudoku<Base>,
            strategies: Vec<DynamicStrategy>,
        ) -> Result<DynamicTryStrategiesReturn> {
            Ok(DynamicTryStrategiesReturn(
                sudoku
                    .try_strategies(strategies)?
                    .map(|(strategy, deductions)| (strategy, deductions.into())),
            ))
        }

        match self {
            DynamicSudoku::Base2(sudoku) => inner(sudoku, strategies),
            DynamicSudoku::Base3(sudoku) => inner(sudoku, strategies),
            DynamicSudoku::Base4(sudoku) => inner(sudoku, strategies),
            DynamicSudoku::Base5(sudoku) => inner(sudoku, strategies),
        }
    }

    pub fn apply_deductions(&mut self, deductions: TransportDeductions) -> Result<()> {
        match self {
            DynamicSudoku::Base2(sudoku) => sudoku.apply_deductions(&deductions.try_into()?),
            DynamicSudoku::Base3(sudoku) => sudoku.apply_deductions(&deductions.try_into()?),
            DynamicSudoku::Base4(sudoku) => sudoku.apply_deductions(&deductions.try_into()?),
            DynamicSudoku::Base5(sudoku) => sudoku.apply_deductions(&deductions.try_into()?),
        }
    }
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize)]
pub struct DynamicTryStrategiesReturn(Option<(DynamicStrategy, TransportDeductions)>);

impl TryFrom<Vec<DynamicCell>> for DynamicSudoku {
    type Error = Error;

    fn try_from(views: Vec<DynamicCell>) -> Result<Self> {
        Ok(match Self::cell_count_to_base(views.len())? {
            2 => Self::Base2(Sudoku::<Base2>::with_grid(views.try_into()?)),
            3 => Self::Base3(Sudoku::<Base3>::with_grid(views.try_into()?)),
            4 => Self::Base4(Sudoku::<Base4>::with_grid(views.try_into()?)),
            5 => Self::Base5(Sudoku::<Base5>::with_grid(views.try_into()?)),
            unexpected_base => bail!(Self::unexpected_base_err(unexpected_base)),
        })
    }
}

impl TryFrom<Vec<Vec<DynamicCell>>> for DynamicSudoku {
    type Error = Error;

    fn try_from(blocks: Vec<Vec<DynamicCell>>) -> Result<Self> {
        let sudoku = match Self::cell_count_to_base(blocks.iter().map(|block| block.len()).sum())? {
            2 => Self::Base2(Sudoku::with_grid(Grid::<Base2>::try_from_blocks(blocks)?)),
            3 => Self::Base3(Sudoku::with_grid(Grid::<Base3>::try_from_blocks(blocks)?)),
            4 => Self::Base4(Sudoku::with_grid(Grid::<Base4>::try_from_blocks(blocks)?)),
            5 => Self::Base5(Sudoku::with_grid(Grid::<Base5>::try_from_blocks(blocks)?)),
            unexpected_base => bail!(Self::unexpected_base_err(unexpected_base)),
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

impl DynamicSudoku {
    fn unexpected_base_err(base: u8) -> Error {
        format_err!("Unexpected dynamic base: {}", base)
    }

    fn cell_count_to_base(cell_count: usize) -> Result<u8> {
        Ok(
            match u16::try_from(cell_count)
                .map_err(|_| anyhow!("Cell count {cell_count} too large"))?
            {
                Base2::CELL_COUNT => 2,
                Base3::CELL_COUNT => 3,
                Base4::CELL_COUNT => 4,
                Base5::CELL_COUNT => 5,
                _ => bail!("Cell count {cell_count} has no valid sudoku base"),
            },
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_count_to_base() -> Result<()> {
        let test_cases = vec![(16, 2), (81, 3), (256, 4), (625, 5)];

        for &(cell_count, expected_base) in &test_cases {
            let base = DynamicSudoku::cell_count_to_base(cell_count)?;

            assert_eq!(base, expected_base);
        }

        let legal_cell_counts: Vec<_> = test_cases
            .into_iter()
            .map(|(cell_count, _)| cell_count)
            .collect();

        for cell_count in (0..=1000).filter(|x| !legal_cell_counts.contains(x)) {
            let res_base = DynamicSudoku::cell_count_to_base(cell_count);
            assert!(
                res_base.is_err(),
                "Expected err, got {res_base:?} for cell_count: {cell_count}"
            );
        }
        Ok(())
    }

    // #[test]
    // fn test_try_from_str() {
    //     let inputs = [INPUT_CANDIDATES, INPUT_GIVENS_LINE, INPUT_GIVENS_GRID];
    //
    //     for input in inputs {
    //         DynamicSudoku::try_from(input).unwrap();
    //     }
    // }
}
