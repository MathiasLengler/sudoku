use serde::{Deserialize, Serialize};

use crate::cell::{view::CellView, SudokuCell};
use crate::position::Position;
use crate::Sudoku;

// TODO:
//  conflicting cells (groups?)
#[derive(Debug, Serialize, Deserialize)]
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
                .grid()
                .all_block_positions()
                .map(|block| {
                    block
                        .map(|pos| {
                            TransportCell::new(sudoku.get(pos).view(), pos, sudoku.is_fixed(pos))
                        })
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
//  is_correct
//  conflicts_with
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportCell {
    #[serde(flatten)]
    cell_view: CellView,
    position: Position,
    fixed: bool,
}

impl TransportCell {
    fn new(cell_view: CellView, position: Position, fixed: bool) -> Self {
        Self {
            cell_view,
            position,
            fixed,
        }
    }
}
