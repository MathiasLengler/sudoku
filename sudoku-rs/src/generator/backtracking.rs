use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::cell::SudokuCell;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking;

// TODO: replace with separate generate methods (return type)
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
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

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Settings {
    pub base: usize,
    pub target: Target,
}

#[derive(Debug)]
pub struct Generator {
    settings: Settings,
}

impl Generator {
    // TODO: change parameter back to base
    pub fn new(settings: Settings) -> Self {
        Self { settings }
    }

    // TODO: also return solution for checking
    pub fn generate<Cell: SudokuCell>(&self) -> Option<Grid<Cell>> {
        use self::Target::*;

        let filled_sudoku = self.filled_grid();

        match self.settings.target {
            Filled => Some(filled_sudoku),
            FromFilled {
                distance: _distance,
            } => unimplemented!(),
            Minimal => Self::minimal(filled_sudoku, 0),
            FromMinimal { distance } => Self::minimal(filled_sudoku, distance),
        }
    }

    fn filled_grid<Cell: SudokuCell>(&self) -> Grid<Cell> {
        let mut grid = Grid::<Cell>::new(self.settings.base);

        let mut solver = backtracking::Solver::new_with_settings(
            &mut grid,
            backtracking::Settings {
                shuffle_candidates: true,
                ..Default::default()
            },
        );

        solver.next().unwrap()
    }

    // TODO: optimize performance for base >= 3
    fn minimal<Cell: SudokuCell>(mut grid: Grid<Cell>, distance: usize) -> Option<Grid<Cell>> {
        assert!(grid.all_candidates_positions().is_empty());

        let mut all_positions: Vec<_> = grid.all_positions().collect();

        all_positions.shuffle(&mut rand::thread_rng());

        let mut deleted: Vec<(Position, usize)> = vec![];

        for pos in all_positions {
            let cell: &Cell = grid.get(pos);

            if let Some(value) = cell.value() {
                grid.delete(pos);

                deleted.push((pos, value));

                // TODO: use strategic solver
                if !backtracking::Solver::has_unique_solution(&grid) {
                    // current position is necessary for unique solution
                    grid.set_value(pos, value);

                    deleted.pop();
                }
            } else {
                panic!("Expected value at {} but got: {:?}", pos, cell)
            }
        }

        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance) {
            grid.set_value(deleted_pos, deleted_value);
        }

        Some(grid)
    }
}

#[cfg(test)]
mod tests {
    use crate::cell::Cell;

    use super::*;

    fn is_minimal<Cell: SudokuCell>(grid: &Grid<Cell>) -> bool {
        let mut grid = grid.clone();

        backtracking::Solver::has_unique_solution(&grid)
            && grid.all_value_positions().into_iter().all(|pos| {
                let prev_cell = grid.delete(pos);
                let has_multiple_solutions = !backtracking::Solver::has_unique_solution(&grid);
                grid.set_value(pos, prev_cell.value().unwrap());
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
