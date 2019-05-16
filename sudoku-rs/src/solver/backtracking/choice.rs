use std::collections::vec_deque::VecDeque;
use std::fmt::{self, Display};

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

// TODO: shuffle the candidates when using the solver as a generator on an empty sudoku

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choice {
    pos: Position,
    candidates: VecDeque<usize>,
    selection: usize,
}

impl Choice {
    pub fn new<Cell: SudokuCell>(pos: Position, sudoku: &Sudoku<Cell>) -> Choice {
        let mut candidates = sudoku.direct_candidates(pos).into();
        let selection = Self::next_selection(&mut candidates);

        Self {
            pos,
            candidates,
            selection,
        }
    }

    fn next_selection(candidates: &mut VecDeque<usize>) -> usize {
        match candidates.pop_front() {
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
