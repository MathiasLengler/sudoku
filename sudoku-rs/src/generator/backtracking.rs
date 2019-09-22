use rand::seq::SliceRandom;

use crate::cell::SudokuCell;
use crate::position::Position;
use crate::settings::Settings as SudokuSettings;
use crate::solver::backtracking;
use crate::Sudoku;

// TODO: replace with separate generate methods (return type)
pub enum Target {
    Filled,
    FromFilled { distance: usize },
    Minimal,
    FromMinimal { distance: usize },
}

impl Default for Target {
    fn default() -> Self {
        Target::Filled
    }
}

pub struct Settings {
    pub base: usize,
    pub target: Target,
}

pub struct Generator {
    settings: Settings,
}

impl Generator {
    // TODO: change parameter back to base
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    pub fn generate<Cell: SudokuCell>(&self) -> Option<Sudoku<Cell>> {
        use self::Target::*;

        let filled_sudoku = self.filled_sudoku();

        match self.settings.target {
            Filled => Some(filled_sudoku),
            FromFilled {
                distance: _distance,
            } => unimplemented!(),
            Minimal => Self::minimal(filled_sudoku, 0),
            FromMinimal { distance } => Self::minimal(filled_sudoku, distance),
        }
    }

    fn filled_sudoku<Cell: SudokuCell>(&self) -> Sudoku<Cell> {
        let mut sudoku = Sudoku::<Cell>::new_with_settings(
            self.settings.base,
            SudokuSettings {
                update_candidates_on_set_value: false,
                history_limit: 0,
            },
        );

        let mut solver = backtracking::Solver::new_with_settings(
            &mut sudoku,
            backtracking::Settings {
                shuffle_candidates: true,
                ..Default::default()
            },
        );

        solver.next().unwrap()
    }

    // TODO: optimize performance for base >= 3
    fn minimal<Cell: SudokuCell>(
        mut sudoku: Sudoku<Cell>,
        distance: usize,
    ) -> Option<Sudoku<Cell>> {
        assert!(sudoku.grid().all_candidates_positions().is_empty());

        let mut all_positions: Vec<_> = sudoku.grid().all_positions().collect();

        all_positions.shuffle(&mut rand::thread_rng());

        let mut deleted: Vec<(Position, usize)> = vec![];

        for pos in all_positions {
            let cell: &Cell = sudoku.get(pos);

            if let Some(value) = cell.value() {
                sudoku.delete(pos);

                deleted.push((pos, value));

                // TODO: use strategic solver
                if !backtracking::Solver::has_unique_solution(&sudoku) {
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

        sudoku.update_settings(SudokuSettings::default());

        Some(sudoku)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    fn is_minimal<Cell: SudokuCell>(sudoku: &Sudoku<Cell>) -> bool {
        let mut sudoku = sudoku.clone();

        backtracking::Solver::has_unique_solution(&sudoku)
            && sudoku.grid().all_value_positions().into_iter().all(|pos| {
                let prev_cell = sudoku.delete(pos);
                let has_multiple_solutions = !backtracking::Solver::has_unique_solution(&sudoku);
                sudoku.set_value(pos, prev_cell.value().unwrap());
                has_multiple_solutions
            })
    }

    #[test]
    fn test_minimal() {
        let generator = Generator::new(Settings {
            base: 2,
            target: Target::Minimal,
        });

        let sudoku = generator.generate::<Cell>().unwrap();

        println!("{}", sudoku);

        assert!(is_minimal(&sudoku));
    }
}
