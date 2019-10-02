use std::ops::Range;

use rand::seq::SliceRandom;
use rand::thread_rng;

use crate::cell::SudokuCell;
use crate::grid::Grid;

/// Naive completed sudoku generator by inserting candidates at random positions.
/// Clears the sudoku if a deadlock is encountered and tries again until the limit is reached.
#[derive(Debug)]
pub struct Generator {
    try_limit: usize,
    base: usize,
}

impl Generator {
    pub fn new(base: usize, try_limit: usize) -> Generator {
        Generator { try_limit, base }
    }

    pub fn generate<Cell: SudokuCell>(&self) -> Option<Grid<Cell>> {
        let tries: Range<usize> = 0..self.try_limit;

        let filter_function = |_try_count| {
            let mut grid = Grid::<Cell>::new(self.base);

            if Self::try_fill(&mut grid) {
                Some(grid)
            } else {
                None
            }
        };

        self.run(tries, filter_function)
    }

    #[cfg(not(feature = "parallel"))]
    fn run<Cell: SudokuCell>(
        &self,
        tries: Range<usize>,
        filter_function: impl Fn(usize) -> Option<Grid<Cell>>,
    ) -> Option<Grid<Cell>> {
        tries.filter_map(filter_function).next()
    }

    #[cfg(feature = "parallel")]
    fn run<Cell, F>(&self, tries: Range<usize>, filter_function: F) -> Option<Grid<Cell>>
    where
        Cell: SudokuCell,
        F: Fn(usize) -> Option<Grid<Cell>>,
        F: Send + Sync,
    {
        use rayon::prelude::ParallelIterator;
        use rayon::prelude::*;

        tries
            .into_par_iter()
            .filter_map(filter_function)
            .find_any(|_| true)
    }

    fn try_fill<Cell: SudokuCell>(grid: &mut Grid<Cell>) -> bool {
        let mut positions = grid.all_candidates_positions();

        let mut rng = thread_rng();

        positions.shuffle(&mut rng);

        let mut no_deadlock = true;

        'outer: for pos in positions {
            for value in grid.direct_candidates(pos) {
                grid.set_value(pos, value);

                if !grid.has_conflict_at(pos) {
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
    use crate::cell::Cell;

    use super::*;

    #[test]
    fn test_generate() {
        let generator = Generator::new(2, 1_000);
        let grid = generator.generate::<Cell>().unwrap();

        assert!(grid.is_solved());
    }
}
