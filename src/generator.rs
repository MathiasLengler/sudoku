use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use rayon::prelude::ParallelIterator;

use crate::cell::SudokuCell;
use crate::Sudoku;

pub struct ParallelRandomGenerator {
    try_limit: usize
}

impl ParallelRandomGenerator {
    pub fn new(try_limit: usize) -> ParallelRandomGenerator {
        ParallelRandomGenerator {
            try_limit
        }
    }

    pub fn generate<Cell: SudokuCell>(&self) -> Option<Sudoku<Cell>> {
        (0..self.try_limit)
            .into_par_iter()
            .filter_map(|_try_count| {
                let mut sudoku = Sudoku::<Cell>::new(3);

                if Self::try_fill(&mut sudoku) {
                    Some(sudoku)
                } else {
                    None
                }
            })
            .find_any(|_| true)
    }

    fn try_fill<Cell: SudokuCell>(sudoku: &mut Sudoku<Cell>) -> bool {
        let mut positions: Vec<_> = sudoku.all_positions().collect();

        let mut rng = thread_rng();

        positions.shuffle(&mut rng);

        let mut no_deadlock = true;

        'outer: for pos in positions {
            for value in 1..=sudoku.side_length() {
                sudoku.set(pos, Cell::new_with_value(value));

                if !sudoku.has_conflict() {
                    continue 'outer;
                }
            }
            no_deadlock = false;

            break;
        }

        no_deadlock
    }
}

