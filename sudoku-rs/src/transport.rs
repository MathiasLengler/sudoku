use serde::{Deserialize, Serialize};

use crate::cell::{Cell, CellView, SudokuCell};
use crate::position::Position;
use crate::Sudoku;

// TODO:
//  conflicting cells (groups?)
#[derive(Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportSudoku {
    blocks: Vec<Vec<TransportCell>>,
    base: usize,
    side_length: usize,
    cell_count: usize,
}

impl<Cell: SudokuCell> From<&Sudoku<Cell>> for TransportSudoku {
    fn from(sudoku: &Sudoku<Cell>) -> Self {
        Self {
            blocks: sudoku
                .all_block_positions()
                .map(|block| {
                    block
                        .map(|pos| TransportCell::from_cell_and_pos(sudoku.get(pos).view(), pos))
                        .collect()
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
    #[serde(flatten)]
    cell_view: CellView,
    position: Position,
}

impl TransportCell {
    fn from_cell_and_pos(cell_view: CellView, position: Position) -> Self {
        Self {
            cell_view,
            position,
        }
    }
}
