use std::collections::VecDeque;

use crate::cell::SudokuCell;
use crate::grid::Grid;

#[derive(Eq, PartialEq, Hash, Clone, Debug)]
pub struct GridHistory<Cell: SudokuCell> {
    records: VecDeque<Grid<Cell>>,
}

impl<Cell: SudokuCell> GridHistory<Cell> {
    pub fn push(&mut self, grid: Grid<Cell>, history_limit: usize) {
        if history_limit == 0 {
            return;
        }
        if self.records.len() >= history_limit {
            self.records.pop_front();
        }
        self.records.push_back(grid);
    }

    pub fn pop(&mut self) -> Option<Grid<Cell>> {
        self.records.pop_back()
    }
}

impl<Cell: SudokuCell> Default for GridHistory<Cell> {
    fn default() -> Self {
        Self {
            records: Default::default(),
        }
    }
}
