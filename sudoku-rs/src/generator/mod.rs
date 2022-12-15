use log::debug;
use rand::prelude::SliceRandom;
use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::compact::value::Value;
use crate::grid::Grid;
use crate::position::Position;
use crate::solver::backtracking;
use crate::solver::strategic::strategies;

// TODO: strategic
//  needed strategies to solve
//  target difficulty: sum of weighted strategy applications

#[cfg_attr(feature = "wasm", derive(TS))]
#[cfg_attr(feature = "wasm", ts(export))]
#[derive(Debug, Serialize, Deserialize, Copy, Clone)]
#[serde(rename_all = "camelCase")]
pub enum GeneratorTarget {
    Filled,
    FromFilled {
        distance: usize,
        set_all_direct_candidates: bool,
    },
    Minimal {
        set_all_direct_candidates: bool,
    },
    FromMinimal {
        distance: usize,
        set_all_direct_candidates: bool,
    },
}

impl Default for GeneratorTarget {
    fn default() -> Self {
        GeneratorTarget::Filled
    }
}

#[cfg_attr(feature = "wasm", derive(TS))]
#[cfg_attr(feature = "wasm", ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorSettings {
    pub base: u8,
    pub target: GeneratorTarget,
}

#[derive(Debug)]
pub struct Generator {
    target: GeneratorTarget,
}

// TODO: expose random seed for deterministic benchmarking
impl Generator {
    pub fn with_target(target: GeneratorTarget) -> Self {
        Self { target }
    }

    pub fn generate<Base: SudokuBase>(&self) -> Grid<Base> {
        let filled_sudoku = self.filled_grid();

        let (mut grid, set_all_direct_candidates) = match self.target {
            GeneratorTarget::Filled => (filled_sudoku, false),
            GeneratorTarget::FromFilled {
                distance,
                set_all_direct_candidates,
            } => (
                Self::filled(filled_sudoku, distance),
                set_all_direct_candidates,
            ),
            GeneratorTarget::Minimal {
                set_all_direct_candidates,
            } => (Self::minimal(filled_sudoku, 0), set_all_direct_candidates),
            GeneratorTarget::FromMinimal {
                distance,
                set_all_direct_candidates,
            } => (
                Self::minimal(filled_sudoku, distance),
                set_all_direct_candidates,
            ),
        };

        grid.fix_all_values();

        if set_all_direct_candidates {
            grid.set_all_direct_candidates();
        }

        grid
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

    /// Try to delete a cell at specific position in a grid while preserving uniqueness of the grid solution.
    ///
    /// Returns the value of the deleted cell, if any.
    fn try_delete_cell_at_pos<Base: SudokuBase>(
        grid: &mut Grid<Base>,
        pos: Position,
    ) -> Option<Value<Base>> {
        let cell = grid.get(pos);

        if let Some(value) = cell.value() {
            grid.get_mut(pos).delete();

            match grid.is_solvable_with_strategies(vec![
                // TODO: introduce parameter
                strategies::SingleCandidate.into(),
                // Box::new(strategies::HiddenSingles),
                // Box::new(strategies::GroupReduction),
                // Box::new(strategies::Backtracking),
            ]) {
                Some(Ok(_)) if grid.has_unique_solution() => {
                    // current position can be removed without losing uniqueness of the grid solution.
                    Some(value)
                }
                _ => {
                    // current position is necessary for unique solution
                    grid.get_mut(pos).set_value(value);
                    None
                }
            }
        } else {
            panic!("Expected value at {} but got: {:?}", pos, cell)
        }
    }

    fn filled<Base: SudokuBase>(mut grid: Grid<Base>, distance: usize) -> Grid<Base> {
        if distance == 0 {
            return grid;
        }

        assert!(grid.is_solved());

        let mut all_positions: Vec<_> = Grid::<Base>::all_positions().collect();
        all_positions.shuffle(&mut rand::thread_rng());
        let all_positions_count = Grid::<Base>::cell_count_usize();

        let mut deleted_count = 0;
        for (i, pos) in all_positions.into_iter().enumerate() {
            let i = i + 1;

            if deleted_count >= distance {
                break;
            }

            if Self::try_delete_cell_at_pos::<Base>(&mut grid, pos).is_some() {
                deleted_count += 1;
                debug!("Position {i}/{all_positions_count} deleted, totaling {deleted_count}/{distance} deleted positions");
            } else {
                debug!("Position {i}/{all_positions_count} is required for unique solution");
            }
        }

        grid
    }

    // TODO: optimize performance for base >= 3
    fn minimal<Base: SudokuBase>(mut grid: Grid<Base>, distance: usize) -> Grid<Base> {
        // If the distance results in a filled sudoku, return it directly.
        if Grid::<Base>::cell_count_usize() <= distance {
            return grid;
        }

        assert!(grid.is_solved());

        let mut all_positions: Vec<_> = Grid::<Base>::all_positions().collect();
        all_positions.shuffle(&mut rand::thread_rng());
        let all_positions_count = Grid::<Base>::cell_count_usize();

        let mut deleted: Vec<(Position, Value<Base>)> = vec![];

        // Reduce grid to a minimal solution.
        for (i, pos) in all_positions.into_iter().enumerate() {
            let i = i + 1;

            if let Some(deleted_value) = Self::try_delete_cell_at_pos::<Base>(&mut grid, pos) {
                let deleted_count = deleted.len();
                debug!("Position {i}/{all_positions_count} deleted, totaling {deleted_count}/{distance} deleted positions");

                deleted.push((pos, deleted_value))
            } else {
                debug!("Position {i}/{all_positions_count} is required for unique solution");
            }
        }

        // Restore the required amount of values, specified by distance.
        for (deleted_pos, deleted_value) in deleted.into_iter().take(distance) {
            grid.get_mut(deleted_pos).set_value(deleted_value);
        }

        grid
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    fn is_minimal<Base: SudokuBase>(grid: &Grid<Base>) -> bool {
        let mut grid = grid.clone();

        grid.unfix_all_values();

        grid.has_unique_solution()
            && grid.all_value_positions().into_iter().all(|pos| {
                let cell = grid.get_mut(pos);
                let prev_value = cell.value().unwrap();
                cell.delete();
                let has_multiple_solutions = !grid.has_unique_solution();
                grid.get_mut(pos).set_value(prev_value);
                has_multiple_solutions
            })
    }

    #[test]
    fn test_minimal() {
        let grid = Generator::with_target(GeneratorTarget::Minimal {
            set_all_direct_candidates: false,
        })
        .generate::<U2>();

        assert!(is_minimal(&grid));
    }

    #[test]
    fn test_filled() {
        let grid = Generator::with_target(GeneratorTarget::Filled).generate::<U2>();

        assert!(grid.is_solved());
    }

    #[test]
    fn test_from_filled() {
        let grid = Generator::with_target(GeneratorTarget::FromFilled {
            distance: 2,
            set_all_direct_candidates: false,
        })
        .generate::<U2>();

        assert_eq!(grid.all_candidates_positions().len(), 2);

        assert!(grid.has_unique_solution());
    }
}
