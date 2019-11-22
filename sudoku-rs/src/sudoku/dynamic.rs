use failure::ensure;
use std::any::{Any, TypeId};

use typenum::consts::*;

pub use game::DynamicSudoku;
pub use game::Game;
use DynamicSudoku::*;

use crate::base::SudokuBase;
use crate::cell::view::parser::parse_cells;
use crate::cell::view::CellView;
use crate::error::{Error, Result};
use crate::generator::backtracking::RuntimeSettings as GeneratorSettings;
use crate::sudoku::Sudoku;
use std::convert::{TryFrom, TryInto};

mod game {
    use typenum::consts::*;

    use enum_dispatch::enum_dispatch;

    use crate::position::Position;
    use crate::sudoku::settings::Settings as SudokuSettings;
    use crate::Sudoku;

    #[enum_dispatch]
    pub trait Game {
        fn set_value(&mut self, pos: Position, value: u8);
        fn set_or_toggle_value(&mut self, pos: Position, value: u8);
        fn set_candidates(&mut self, pos: Position, candidates: Vec<u8>);
        fn toggle_candidate(&mut self, pos: Position, candidate: u8);
        fn delete(&mut self, pos: Position);
        fn set_all_direct_candidates(&mut self);
        fn solve_single_candidates(&mut self);
        fn group_reduction(&mut self);
        fn undo(&mut self);
        fn settings(&self) -> SudokuSettings;
        fn update_settings(&mut self, settings: SudokuSettings);
        fn export(&self) -> String;
    }

    /// A game of Sudoku which is able to change the size of the board at runtime.
    #[enum_dispatch(Game)]
    #[derive(Eq, PartialEq, Hash, Clone, Debug)]
    pub enum DynamicSudoku {
        Base2(Sudoku<U2>),
        Base3(Sudoku<U3>),
        Base4(Sudoku<U4>),
        Base5(Sudoku<U5>),
    }
}

// Requires runtime base
impl DynamicSudoku {
    pub fn new(base: u8) -> Self {
        match base {
            2 => Base2(Sudoku::<U2>::new()),
            3 => Base3(Sudoku::<U3>::new()),
            4 => Base4(Sudoku::<U4>::new()),
            5 => Base5(Sudoku::<U5>::new()),
            unexpected_base => Self::bail_unexpected_base(unexpected_base),
        }
    }
    pub fn with_sudoku<Base: SudokuBase + 'static>(sudoku: Sudoku<Base>) -> Self {
        let any_sudoku: Box<dyn Any> = Box::new(sudoku);

        match TypeId::of::<Base>() {
            id if id == TypeId::of::<U2>() => Base2(*(any_sudoku.downcast().unwrap())),
            id if id == TypeId::of::<U3>() => Base3(*(any_sudoku.downcast().unwrap())),
            id if id == TypeId::of::<U4>() => Base4(*(any_sudoku.downcast().unwrap())),
            id if id == TypeId::of::<U5>() => Base5(*(any_sudoku.downcast().unwrap())),
            _ => Self::bail_unexpected_base(Base::to_u8()),
        }
    }
    pub fn generate(&mut self, generator_settings: GeneratorSettings) -> Result<()> {
        let GeneratorSettings { base, target } = generator_settings;

        *self = match base {
            2 => Base2(Sudoku::<U2>::with_target_and_settings(
                target,
                self.settings(),
            )?),
            3 => Base3(Sudoku::<U3>::with_target_and_settings(
                target,
                self.settings(),
            )?),
            4 => Base4(Sudoku::<U4>::with_target_and_settings(
                target,
                self.settings(),
            )?),
            5 => Base5(Sudoku::<U5>::with_target_and_settings(
                target,
                self.settings(),
            )?),
            unexpected_base => Self::bail_unexpected_base(unexpected_base),
        };

        Ok(())
    }
    pub fn import(&mut self, input: &str) -> Result<()> {
        *self = input.try_into()?;

        Ok(())
    }
}

impl TryFrom<Vec<CellView>> for DynamicSudoku {
    type Error = Error;

    fn try_from(views: Vec<CellView>) -> Result<Self> {
        Ok(match Self::cell_count_to_base(views.len())? {
            2 => Base2(Sudoku::<U2>::with_grid(views.try_into()?)),
            3 => Base3(Sudoku::<U3>::with_grid(views.try_into()?)),
            4 => Base4(Sudoku::<U4>::with_grid(views.try_into()?)),
            5 => Base5(Sudoku::<U5>::with_grid(views.try_into()?)),
            unexpected_base => Self::bail_unexpected_base(unexpected_base),
        })
    }
}

impl TryFrom<&str> for DynamicSudoku {
    type Error = Error;

    fn try_from(input: &str) -> Result<Self> {
        Ok(parse_cells(input)?.try_into()?)
    }
}

impl DynamicSudoku {
    fn bail_unexpected_base(unexpected_base: u8) -> ! {
        panic!("Unexpected dynamic base: {}", unexpected_base)
    }

    // TODO: use pow probing
    fn cell_count_to_base(cell_count: usize) -> Result<u8> {
        let approx_base = (cell_count as f64).sqrt().sqrt().round() as u8;

        ensure!(
            Self::base_to_cell_count(approx_base) == cell_count,
            "Cell count {} has no valid sudoku base",
            cell_count
        );

        Ok(approx_base)
    }

    fn base_to_cell_count(base: u8) -> usize {
        (base as usize).pow(4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cell_count_to_base() -> Result<()> {
        let test_cases = vec![(0, 0), (1, 1), (16, 2), (81, 3), (256, 4), (625, 5)];

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
                "Expected err, got {:?} for cell_count: {}",
                res_base,
                cell_count
            );
        }
        Ok(())
    }

    #[test]
    fn test_try_from_str() -> Result<()> {
        let inputs = [
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/res/candidates.txt"
            )),
            include_str!(concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/tests/res/givens_line.txt"
            )),
        ];

        inputs
            .into_iter()
            .map(|input| DynamicSudoku::try_from(*input))
            .collect::<Result<Vec<_>>>()?;

        Ok(())
    }
}
