use serde::{Deserialize, Serialize};

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

// TODO:
//  conflicting cells (groups?)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportSudoku {
    cells: Vec<TransportCell>,
    base: usize,
    side_length: usize,
    cell_count: usize,
}

impl<Cell: SudokuCell> From<&Sudoku<Cell>> for TransportSudoku {
    fn from(sudoku: &Sudoku<Cell>) -> Self {
        Self {
            cells: sudoku
                .all_cell_positions()
                .map(|position| {
                    let cell = sudoku.get(position);
                    TransportCell {
                        value: cell.value(),
                        candidates: cell.candidates(),
                        position,
                    }
                })
                .collect(),
            base: sudoku.base(),
            side_length: sudoku.side_length(),
            cell_count: sudoku.cell_count(),
        }
    }
}

// TODO:
//  is_editable
//  is_correct
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportCell {
    value: usize,
    candidates: Vec<usize>,
    position: Position,
}
