//! Source: https://www.sudokuoftheday.com/about/difficulty/
//!
//! Technique | Code | Cost for first use | Cost for subsequent uses
//! --- | --- | --- | ---
//! Single Candidate | sct | 100 | 100
//! Single Position | spt | 100 | 100
//! Candidate Lines | clt | 350 | 200
//! Double Pairs | dpt | 500 | 250
//! Multiple Lines | mlt | 700 | 400
//! Naked Pair | dj2 | 750 | 500
//! Hidden Pair | us2 | 1500 | 1200
//! Naked Triple | dj3 | 2000 | 1400
//! Hidden Triple | us3 | 2400 | 1600
//! X-Wing | xwg | 2800 | 1600
//! Forcing Chains | fct | 4200 | 2100
//! Naked Quad | dj4 | 5000 | 4000
//! Hidden Quad | us4 | 7000 | 5000
//! Swordfish | sf4 | 8000 | 6000

use std::fmt::Debug;

use backtracking::Backtracking;
use group_reduction::GroupReduction;
use single_candidate::SingleCandidate;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::Sudoku;

// TODO: bench
mod backtracking;
mod group_reduction;
mod single_candidate;

// TODO: use
enum StrategyResult {
    Modified { cell_positions: Vec<Position> },
    Unsolvable,
    MultipleSolutions,
}

pub(super) trait Strategy<Cell: SudokuCell>: Debug {
    /// Execute this strategy on the given sudoku. Returns the list of modified positions.
    fn execute(&self, sudoku: &mut Sudoku<Cell>) -> Vec<Position>;
}

pub(super) fn strategies<Cell: SudokuCell>() -> Vec<Box<dyn Strategy<Cell>>> {
    vec![
        Box::new(SingleCandidate),
        Box::new(GroupReduction),
        Box::new(Backtracking),
    ]
}
