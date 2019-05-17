use std::fmt::{self, Display};

use rand::seq::SliceRandom;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choice {
    pos: Position,
    candidates: Vec<usize>,
    selection: usize,
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

        let selection = Self::next_selection(&mut candidates);

        Self {
            pos,
            candidates,
            selection,
        }
    }

    fn next_selection(candidates: &mut Vec<usize>) -> usize {
        match candidates.pop() {
            Some(candidate) => candidate,
            None => 0,
        }
    }

    pub fn set_next(&mut self) {
        self.selection = Self::next_selection(&mut self.candidates)
    }

    pub fn is_exhausted(&self) -> bool {
        self.selection == 0
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn selection(&self) -> usize {
        self.selection
    }
}

impl Display for Choice {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={} ({:?})", self.pos, self.selection, self.candidates)
    }
}
