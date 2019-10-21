use rand::seq::SliceRandom;
use serde::{Deserialize, Serialize};

use crate::base::SudokuBase;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking;

// TODO: replace with separate generate methods (return type)
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
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
pub struct RuntimeSettings {
    pub base: usize,
    pub target: Target,
}

#[derive(Debug)]
pub struct Generator {
    target: Target,
}

impl Generator {
    // TODO: change parameter back to base
    pub fn with_target(target: Target) -> Self {
        Self { target }
    }

    // TODO: also return solution for checking
    pub fn generate<Base: SudokuBase>(&self) -> Option<Grid<Base>> {
        use self::Target::*;

        let filled_sudoku = self.filled_grid();

        match self.target {
            Filled => Some(filled_sudoku),
            FromFilled {
                distance: _distance,
            } => unimplemented!(),
            Minimal => Self::minimal(filled_sudoku, 0),
            FromMinimal { distance } => Self::minimal(filled_sudoku, distance),
        }
    }

    fn filled_grid<Base: SudokuBase>(&self) -> Grid<Base> {
        let mut grid = Grid::<Base>::new();

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
    fn minimal<Base: SudokuBase>(mut grid: Grid<Base>, distance: usize) -> Option<Grid<Base>> {
        // If the distance results in a filled sudoku, return it directly.
        if Grid::<Base>::cell_count() <= distance {
            return Some(grid);
        }

        assert!(grid.all_candidates_positions().is_empty());

        let mut all_positions: Vec<_> = grid.all_positions().collect();

        all_positions.shuffle(&mut rand::thread_rng());

        let mut deleted: Vec<(Position, u8)> = vec![];

        for pos in all_positions {
            let cell = grid.get(pos);

            if let Some(value) = cell.value() {
                grid.get_mut(pos).delete();

                deleted.push((pos, value));

                // TODO: use strategic solver
                if !backtracking::Solver::has_unique_solution(&grid) {
                    // current position is necessary for unique solution
                    grid.get_mut(pos).set_value(value);

                    deleted.pop();
                }
            } else {
                panic!("Expected value at {} but got: {:?}", pos, cell)
            }
        }

        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance) {
            grid.get_mut(deleted_pos).set_value(deleted_value);
        }

        Some(grid)
    }
}

#[cfg(test)]
mod tests {
    use typenum::consts::*;

    use crate::cell::Cell;

    use super::*;

    fn is_minimal<Base: SudokuBase>(grid: &Grid<Base>) -> bool {
        let mut grid = grid.clone();

        backtracking::Solver::has_unique_solution(&grid)
            && grid.all_value_positions().into_iter().all(|pos| {
                let cell = grid.get_mut(pos);
                let prev_value = cell.value().unwrap();
                cell.delete();
                let has_multiple_solutions = !backtracking::Solver::has_unique_solution(&grid);
                grid.get_mut(pos).set_value(prev_value);
                has_multiple_solutions
            })
    }

    #[test]
    fn test_minimal() {
        let generator = Generator::with_target(Target::Minimal);

        let sudoku = generator.generate::<U2>().unwrap();

        println!("{}", sudoku);

        assert!(is_minimal(&sudoku));
    }
}
