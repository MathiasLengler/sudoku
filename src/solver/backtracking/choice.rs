use std::collections::vec_deque::VecDeque;
use std::fmt::{self, Display};

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Choice<Cell: SudokuCell> {
    pos: Position,
    candidates: VecDeque<Cell>,
    selection: Cell,
}

impl<Cell: SudokuCell> Choice<Cell> {
    pub fn new(pos: Position, sudoku: &Sudoku<Cell>) -> Choice<Cell> {
        let mut candidates = sudoku.direct_candidates(pos);
        let selection = Self::next_selection(&mut candidates);

        Self {
            pos,
            candidates,
            selection,
        }
    }

    fn next_selection(candidates: &mut VecDeque<Cell>) -> Cell {
        match candidates.pop_front() {
            Some(candidate) => candidate,
            None => Cell::new_with_value(0),
        }
    }

    pub fn set_next(&mut self) {
        self.selection = Self::next_selection(&mut self.candidates)
    }

    pub fn is_exhausted(&self) -> bool {
        !self.selection.has_value()
    }

    pub fn position(&self) -> Position {
        self.pos
    }

    pub fn selection(&self) -> Cell {
        self.selection.clone()
    }
}

impl<Cell: SudokuCell> Display for Choice<Cell> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}={:?} ({:?})", self.pos, self.selection, self.candidates)
    }
}