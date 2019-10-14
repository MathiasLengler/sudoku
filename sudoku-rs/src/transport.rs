use serde::{Deserialize, Serialize};

use crate::cell::{view::CellView, SudokuCell};
use crate::position::Position;
use crate::Sudoku;

// TODO:
//  conflicting cells (groups?)
// TODO: can_undo
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
        let grid = sudoku.grid();
        let solved_grid = sudoku.solved_grid();

        Self {
            blocks: grid
                .all_block_positions()
                .map(|block| {
                    block
                        .map(|pos| {
                            let cell_view = grid.get(pos).view();
                            let incorrect_value = if cell_view.is_value() {
                                solved_grid
                                    .as_ref()
                                    .map(|solved_grid| solved_grid.get(pos).view() != cell_view)
                                    .unwrap_or(false)
                            } else {
                                false
                            };
                            TransportCell::new(cell_view, pos, grid.is_fixed(pos), incorrect_value)
                        })
                        .collect()
                })
                .collect(),
            base: grid.base(),
            side_length: grid.side_length(),
            cell_count: grid.cell_count(),
        }
    }
}

// TODO:
//  conflicts_with (via all_conflict_pairs)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportCell {
    #[serde(flatten)]
    cell_view: CellView,
    position: Position,
    fixed: bool,
    incorrect_value: bool,
}

impl TransportCell {
    fn new(cell_view: CellView, position: Position, fixed: bool, incorrect_value: bool) -> Self {
        Self {
            cell_view,
            position,
            fixed,
            incorrect_value,
        }
    }
}
