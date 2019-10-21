use serde::{Deserialize, Serialize};

use crate::base::SudokuBase;
use crate::cell::view::CellView;
use crate::grid::Grid;
use crate::position::Position;
use crate::sudoku::DynamicSudoku;
use crate::sudoku::Sudoku;

// TODO:
//  conflicting cells (groups?)
// TODO: can_undo
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportSudoku {
    blocks: Vec<Vec<TransportCell>>,
    base: u8,
    side_length: u8,
    cell_count: usize,
}

impl<Base: SudokuBase> From<&Sudoku<Base>> for TransportSudoku {
    fn from(sudoku: &Sudoku<Base>) -> Self {
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
                            TransportCell {
                                cell_view,
                                position: pos,
                                incorrect_value,
                            }
                        })
                        .collect()
                })
                .collect(),
            base: Grid::<Base>::base(),
            side_length: Grid::<Base>::side_length(),
            cell_count: Grid::<Base>::cell_count(),
        }
    }
}

impl From<&DynamicSudoku> for TransportSudoku {
    fn from(dynamic_sudoku: &DynamicSudoku) -> Self {
        match dynamic_sudoku {
            DynamicSudoku::Base2(sudoku) => Self::from(sudoku),
            DynamicSudoku::Base3(sudoku) => Self::from(sudoku),
            DynamicSudoku::Base4(sudoku) => Self::from(sudoku),
            DynamicSudoku::Base5(sudoku) => Self::from(sudoku),
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
    incorrect_value: bool,
}
