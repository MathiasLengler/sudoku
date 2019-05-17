use rand::seq::SliceRandom;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::solver::backtracking::{BacktrackingSolver, BacktrackingSolverSettings};
use crate::Sudoku;

// TODO: replace with separate generate methods (return type)
pub enum BacktrackingGeneratorTarget {
    Filled,
    EmptyCells { count: usize },
    Critical,
    CriticalDistance { count: usize },
}

impl Default for BacktrackingGeneratorTarget {
    fn default() -> Self {
        BacktrackingGeneratorTarget::Filled
    }
}

pub struct BacktrackingGeneratorSettings {
    pub base: usize,
    pub target: BacktrackingGeneratorTarget,
}

pub struct BacktrackingGenerator {
    settings: BacktrackingGeneratorSettings,
}

impl BacktrackingGenerator {
    // TODO: change parameter back to base
    pub fn new(settings: BacktrackingGeneratorSettings) -> Self {
        Self { settings }
    }

    pub fn generate<Cell: SudokuCell>(mut self) -> Option<Sudoku<Cell>> {
        use self::BacktrackingGeneratorTarget::*;

        let filled_sudoku = self.filled_sudoku();

        match self.settings.target {
            Filled => Some(filled_sudoku),
            EmptyCells { count } => unimplemented!(),
            Critical => Self::critical(filled_sudoku, 0),
            CriticalDistance { count } => unimplemented!(),
        }
    }

    fn filled_sudoku<Cell: SudokuCell>(&self) -> Sudoku<Cell> {
        let mut solver = BacktrackingSolver::new_with_settings(
            Sudoku::<Cell>::new(self.settings.base),
            BacktrackingSolverSettings {
                shuffle_candidates: true,
                ..Default::default()
            },
        );

        solver.next().unwrap()
    }

    fn critical<Cell: SudokuCell>(
        mut sudoku: Sudoku<Cell>,
        distance_count: usize,
    ) -> Option<Sudoku<Cell>> {
        assert!(sudoku.all_empty_positions().is_empty());

        let mut all_positions: Vec<_> = sudoku.all_positions().collect();

        all_positions.shuffle(&mut rand::thread_rng());

        let mut deleted_values: Vec<(Position, usize)> = vec![];

        loop {
            if let Some(pos) = all_positions.pop() {
                let cell: &Cell = sudoku.get(pos);

                if let Some(value) = cell.value() {
                    sudoku.delete(pos);

                    deleted_values.push((pos, value));

                    // TODO: try other positions before returning
                    if !Self::has_unique_solution(&sudoku) {
                        // sudoku is critical
                        // TODO: set value distance_count times
                        sudoku.set_value(pos, value);

                        return Some(sudoku);
                    } else {
                    }
                } else {
                    panic!("Expected value at {} but got: {:?}", pos, cell)
                }
            } else {
                return None;
            }
        }

        unimplemented!()
    }

    // TODO: move to sudoku
    fn has_unique_solution<Cell: SudokuCell>(sudoku: &Sudoku<Cell>) -> bool {
        let mut solver = BacktrackingSolver::new(sudoku.clone());

        solver.nth(1).is_none()
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    #[test]
    fn test_critical() {
        let generator = BacktrackingGenerator::new(BacktrackingGeneratorSettings {
            base: 2,
            target: BacktrackingGeneratorTarget::Critical,
        });

        let sudoku = generator.generate::<Cell>().unwrap();

        // TODO: asserts
        println!("{}", sudoku);
    }
}
