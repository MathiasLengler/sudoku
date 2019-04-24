use std::collections::vec_deque::VecDeque;
use std::num::NonZeroUsize;

use serde::{Deserialize, Serialize};

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

#[derive(Serialize, Deserialize)]
pub struct TransportSudoku {
    cells: Vec<TransportCell>,
    base: usize,
    side_length: usize,
    cell_count: usize,
}

impl<Cell: SudokuCell> From<&Sudoku<Cell>> for TransportSudoku {
    fn from(sudoku: &Sudoku<Cell>) -> Self {
        Self {
            cells: sudoku.all_positions().map(|position| TransportCell {
                value: sudoku.get(position).value(),
                candidates: sudoku.candidates(position)
                    .into_iter()
                    .map(|cell| cell.value().unwrap())
                    .collect(),
                position,
            }).collect(),
            base: sudoku.base(),
            side_length: sudoku.side_length(),
            cell_count: sudoku.cell_count(),
        }
    }
}

#[derive(Serialize, Deserialize)]
pub struct TransportCell {
    value: Option<NonZeroUsize>,
    candidates: VecDeque<NonZeroUsize>,
    position: Position,
}