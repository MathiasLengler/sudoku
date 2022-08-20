use serde::{Deserialize, Serialize};

use crate::base::SudokuBase;
use crate::cell::view::CellView;
use crate::grid::Grid;
use crate::position::Position;
use crate::sudoku::DynamicSudoku;
use crate::sudoku::Sudoku;

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
            blocks: Grid::<Base>::all_block_positions()
                .map(|block| {
                    block
                        .map(|pos| {
                            let cell = grid.get(pos);
                            let incorrect_value = if cell.has_value() {
                                solved_grid
                                    .as_ref()
                                    .map(|solved_grid| solved_grid.get(pos) != cell)
                                    .unwrap_or(false)
                            } else {
                                false
                            };
                            TransportCell {
                                cell_view: cell.view(),
                                position: pos,
                                incorrect_value,
                            }
                        })
                        .collect()
                })
                .collect(),
            base: Grid::<Base>::base(),
            side_length: Grid::<Base>::side_length(),
            cell_count: Grid::<Base>::cell_count_usize(),
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportCell {
    #[serde(flatten)]
    cell_view: CellView,
    position: Position,
    incorrect_value: bool,
}
