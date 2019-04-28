use std::ops::Range;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::cell::SudokuCell;
use crate::Sudoku;

pub struct RandomGenerator {
    try_limit: usize,
    base: usize,
}

impl RandomGenerator {
    pub fn new(base: usize, try_limit: usize) -> RandomGenerator {
        RandomGenerator {
            try_limit,
            base,
        }
    }

    pub fn generate<Cell: SudokuCell>(&self) -> Option<Sudoku<Cell>> {
        let tries: Range<usize> = 0..self.try_limit;

        let filter_function = |_try_count| {
            let mut sudoku = Sudoku::<Cell>::new(self.base);

            if Self::try_fill(&mut sudoku) {
                Some(sudoku)
            } else {
                None
            }
        };

        self.run(tries, filter_function)
    }

    #[cfg(not(feature = "parallel"))]
    fn run<Cell: SudokuCell>(&self, tries: Range<usize>, filter_function: impl Fn(usize) -> Option<Sudoku<Cell>>) -> Option<Sudoku<Cell>> {
        tries
            .filter_map(filter_function)
            .next()
    }

    #[cfg(feature = "parallel")]
    fn run<Cell: SudokuCell>(&self, tries: Range<usize>, filter_function: impl Fn(usize) -> Option<Sudoku<Cell>>) -> Option<Sudoku<Cell>> {
        use rayon::prelude::*;
        use rayon::prelude::ParallelIterator;

        tries
            .into_par_iter()
            .filter_map(filter_function)
            .find_any(|_| true)
    }

    fn try_fill<Cell: SudokuCell>(sudoku: &mut Sudoku<Cell>) -> bool {
        let mut positions: Vec<_> = sudoku.all_cell_positions().collect();

        let mut rng = thread_rng();

        positions.shuffle(&mut rng);

        let mut no_deadlock = true;

        'outer: for pos in positions {
            for value in 1..=sudoku.side_length() {
                sudoku.set_value(pos, value);

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
    use crate::cell::OptionCell;

    use super::*;

    #[test]
    fn test_generate() {
        let generator = RandomGenerator::new(2, 1_000);
        let sudoku = generator.generate::<OptionCell>().unwrap();

        assert!(sudoku.empty_positions().is_empty());
        assert!(!sudoku.has_conflict());
    }
}
