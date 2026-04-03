use serde::{Deserialize, Serialize};

use crate::base::BaseEnum;
use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::grid::Grid;
use crate::grid::solution_state::SolutionState;
use crate::position::DynamicPosition;
use crate::sudoku::DynamicSudoku;
use crate::sudoku::Sudoku;

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportSudoku {
    cells: Vec<TransportCell>,
    base: BaseEnum,

    // TODO: move base-only dependent state out of TransportSudoku
    blocks_indexes: Vec<Vec<u16>>,
    side_length: u8,
    cell_count: u16,

    history: TransportHistory,

    is_solved: bool,
    solution: TransportSolutionState,
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportCell {
    #[serde(flatten)]
    dynamic_cell: DynamicCell,
    position: DynamicPosition,
    incorrect_value: bool,
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TransportHistory {
    can_undo: bool,
    can_redo: bool,
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum TransportSolutionState {
    NoSolution,
    MultipleSolutions,
    SingleSolution,
}

impl<Base: SudokuBase> From<&Sudoku<Base>> for TransportSudoku {
    fn from(sudoku: &Sudoku<Base>) -> Self {
        let grid = sudoku.grid();
        let unique_solution = sudoku.solution.as_unique_solution();

        Self {
            cells: Grid::<Base>::all_positions()
                .map(|pos| {
                    let cell = grid.get(pos);
                    let incorrect_value = if cell.has_value() {
                        unique_solution
                            .as_ref()
                            .is_some_and(|solved_grid| solved_grid.get(pos) != cell)
                    } else {
                        false
                    };
                    TransportCell {
                        dynamic_cell: cell.into(),
                        position: pos.into(),
                        incorrect_value,
                    }
                })
                .collect(),
            blocks_indexes: Grid::<Base>::all_block_positions()
                .map(|block| block.map(|pos| pos.cell_index()).collect())
                .collect(),
            base: Base::ENUM,
            side_length: Base::SIDE_LENGTH,
            cell_count: Base::CELL_COUNT,
            history: TransportHistory {
                can_undo: sudoku.history.can_go_back(),
                can_redo: sudoku.history.can_go_forward(),
            },
            is_solved: grid.is_solved(),
            solution: TransportSolutionState::from(&sudoku.solution),
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

impl<Base: SudokuBase> From<&SolutionState<Base>> for TransportSolutionState {
    fn from(solution_state: &SolutionState<Base>) -> Self {
        match solution_state {
            SolutionState::NoSolution => Self::NoSolution,
            SolutionState::MultipleSolutions => Self::MultipleSolutions,
            SolutionState::UniqueSolution(_) => Self::SingleSolution,
        }
    }
}
