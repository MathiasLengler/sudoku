use rand::seq::SliceRandom;
use rand::thread_rng;
use rayon::prelude::*;
use rayon::prelude::ParallelIterator;

use crate::cell::SudokuCell;
use crate::Sudoku;

pub struct RandomGenerator {
    try_limit: usize,
    base: usize,
    parallel: bool,
}

impl RandomGenerator {
    pub fn new(base: usize, try_limit: usize, parallel: bool) -> RandomGenerator {
        RandomGenerator {
            try_limit,
            base,
            parallel,
        }
    }

    pub fn generate<Cell: SudokuCell>(&self) -> Option<Sudoku<Cell>> {
        let tries = 0..self.try_limit;

        let filter_function = |_try_count| {
            let mut sudoku = Sudoku::<Cell>::new(self.base);

            if Self::try_fill(&mut sudoku) {
                Some(sudoku)
            } else {
                None
            }
        };

        if self.parallel {
            tries
                .into_par_iter()
                .filter_map(filter_function)
                .find_any(|_| true)
        } else {
            tries
                .filter_map(filter_function)
                .next()
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cell::OptionCell;

    #[test]
    fn test_generate() {
        let generator = RandomGenerator::new(2, 1_000, false);
        let sudoku = generator.generate::<OptionCell>().unwrap();

        assert!(sudoku.all_empty_positions().is_empty());
        assert!(!sudoku.has_conflict());
    }
}
