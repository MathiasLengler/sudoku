use rand::seq::SliceRandom;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::solver::backtracking::{BacktrackingSolver, BacktrackingSolverSettings};
use crate::Sudoku;

// TODO: replace with separate generate methods (return type)
pub enum BacktrackingGeneratorTarget {
    Filled,
    FromFilled { distance: usize },
    Critical,
    FromCritical { distance: usize },
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

    pub fn generate<Cell: SudokuCell>(&self) -> Option<Sudoku<Cell>> {
        use self::BacktrackingGeneratorTarget::*;

        let filled_sudoku = self.filled_sudoku();

        match self.settings.target {
            Filled => Some(filled_sudoku),
            FromFilled {
                distance: _distance,
            } => unimplemented!(),
            Critical => Self::critical(filled_sudoku, 0),
            FromCritical { distance } => Self::critical(filled_sudoku, distance),
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

    // TODO: optimize performance for base >= 3
    fn critical<Cell: SudokuCell>(
        mut sudoku: Sudoku<Cell>,
        distance: usize,
    ) -> Option<Sudoku<Cell>> {
        assert!(sudoku.grid().all_empty_positions().is_empty());

        let mut all_positions: Vec<_> = sudoku.grid().all_positions().collect();

        all_positions.shuffle(&mut rand::thread_rng());

        let mut deleted: Vec<(Position, usize)> = vec![];

        for pos in all_positions {
            let cell: &Cell = sudoku.get(pos);

            if let Some(value) = cell.value() {
                sudoku.delete(pos);

                deleted.push((pos, value));

                if !BacktrackingSolver::has_unique_solution(&sudoku) {
                    // current position is necessary for unique solution
                    sudoku.set_value(pos, value);

                    deleted.pop();
                }
            } else {
                panic!("Expected value at {} but got: {:?}", pos, cell)
            }
        }

        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance) {
            sudoku.set_value(deleted_pos, deleted_value);
        }

        Some(sudoku)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    fn is_critical<Cell: SudokuCell>(sudoku: &Sudoku<Cell>) -> bool {
        let mut sudoku = sudoku.clone();

        BacktrackingSolver::has_unique_solution(&sudoku)
            && sudoku.grid().all_value_positions().into_iter().all(|pos| {
                let prev_cell = sudoku.delete(pos);
                let has_multiple_solutions = !BacktrackingSolver::has_unique_solution(&sudoku);
                sudoku.set_value(pos, prev_cell.value().unwrap());
                has_multiple_solutions
            })
    }

    #[test]
    fn test_critical() {
        let generator = BacktrackingGenerator::new(BacktrackingGeneratorSettings {
            base: 2,
            target: BacktrackingGeneratorTarget::Critical,
        });

        let sudoku = generator.generate::<Cell>().unwrap();

        println!("{}", sudoku);

        assert!(is_critical(&sudoku));
    }
}
