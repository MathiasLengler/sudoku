use std::any::{Any, TypeId};

use typenum::consts::*;

pub use game::DynamicSudoku;
pub use game::Game;
use DynamicSudoku::*;

use crate::base::SudokuBase;
use crate::error::Result;
use crate::generator::backtracking::RuntimeSettings as GeneratorSettings;
use crate::sudoku::Sudoku;

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
    pub fn import(&mut self, _input: &str) -> Result<()> {
        // TODO: use split up parser to infer base from Vec<CellView>
        unimplemented!()
    }
}

impl DynamicSudoku {
    fn bail_unexpected_base(unexpected_base: u8) -> ! {
        panic!("Unexpected dynamic base: {}", unexpected_base)
    }
}
