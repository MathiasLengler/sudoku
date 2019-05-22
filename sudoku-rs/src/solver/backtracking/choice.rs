use std::fmt::{self, Display};

use rand::seq::SliceRandom;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choice {
    pos: Position,
    candidates: Vec<usize>,
}

impl Choice {
    pub fn new<Cell: SudokuCell>(
        sudoku: &Sudoku<Cell>,
        pos: Position,
        shuffle_candidates: bool,
    ) -> Choice {
        let mut candidates = sudoku.direct_candidates(pos);

        if shuffle_candidates {
            candidates.shuffle(&mut rand::thread_rng())
        } else {
            // Ascending value selection order when selecting values from the end of the vec
            candidates.reverse();
        }

        Self { pos, candidates }
    }

    pub fn set_next(&mut self) {
        let prev_selection = self.candidates.pop();

        debug_assert!(prev_selection.is_some());
    }

    pub fn is_exhausted(&self) -> bool {
        self.candidates.is_empty()
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn selection(&self) -> usize {
        self.candidates.last().copied().unwrap_or(0)
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}=({:?})", self.pos, self.candidates)
    }
}
