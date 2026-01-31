use anyhow::{bail, format_err};
use itertools::Itertools;
use log::debug;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;

use crate::cell::Value;
use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::rng::{CrateRng, new_crate_rng_with_seed};
use crate::solver::sat::AmbiguousSolutionChecker;
use crate::solver::strategic::strategies::BruteForce;
use crate::solver::{backtracking, introspective};
use crate::{base::SudokuBase, solver::strategic::strategies::selection::StrategySet};

pub use settings::*;
#[cfg(feature = "parallel")]
pub mod multi_shot;
mod settings;

/*
Ideas:
- pruning with backtracking
- symmetrical/pair-wise or other pattern-based pruning
- early abort/skip config
- from minimal insertion order
 */

// TODO: optimization: constrain ambiguous sudoku by adding solution values
//  Reference: https://github.com/t-dillon/tdoku/blob/master/src/solver_dpll_triad_simd.cc#L707
//  The constrained sudoku is not guaranteed to be minimal.
//  This has an advantage of providing a "almost" minimal sudoku way faster,
//  since checking for an ambiguous solution is way faster (early abort) than proving a unique solution.
//  The near minimal sudoku can then be minimized as usual.

#[derive(Debug, Default, Clone)]
pub struct Generator<Base: SudokuBase> {
    settings: GeneratorSettings<Base>,
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeneratorProgress {
    pruning_position_index: usize,
    pruning_position_count: usize,
    deleted_count: u16,
}

struct NearMinimalGridReturn<Base: SudokuBase> {
    near_minimal_grid: Grid<Base>,
    deleted: Vec<(Position<Base>, Value<Base>)>,
    remaining_pruning_positions: Vec<Position<Base>>,
}

impl<Base: SudokuBase> Generator<Base> {
    pub fn with_target(target: PruningTarget) -> Self {
        Self::with_pruning(PruningSettings {
            target,
            ..Default::default()
        })
    }

    pub fn with_pruning(pruning_settings: PruningSettings<Base>) -> Self {
        Self::with_settings(GeneratorSettings {
            prune: Some(pruning_settings),
            ..Default::default()
        })
    }

    pub fn with_settings(settings: GeneratorSettings<Base>) -> Self {
        Self { settings }
    }

    pub fn generate(&self) -> Result<Grid<Base>> {
        self.generate_with_progress(|_| Ok(()))
    }

    pub fn generate_with_progress(
        &self,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
    ) -> Result<Grid<Base>> {
        debug!("Generating: {self:#?}");

        let mut rng = new_crate_rng_with_seed(self.settings.seed);

        let solved_grid = self.solved_grid(&mut rng)?;

        debug!("Solution:\n{solved_grid}");

        let Some(prune_settings) = &self.settings.prune else {
            debug!("No pruning, returning solution");
            return Ok(solved_grid);
        };

        self.prune(solved_grid, prune_settings, on_progress, &mut rng)
    }

    fn solved_grid(&self, rng: &mut CrateRng) -> Result<Grid<Base>> {
        debug!("Generating solution");

        let grid = if let Some(solution_settings) = &self.settings.solution {
            solution_settings.values_grid.clone()
        } else {
            Grid::<Base>::new()
        };

        let mut solver = backtracking::Solver::builder(&grid)
            .rng(rng.clone())
            .build();

        solver.next().ok_or_else(|| {
            if self.settings.solution.is_some() {
                format_err!("'solution.values_grid' has no solution")
            } else {
                panic!("Expected empty grid to have at least one solution")
            }
        })
    }

    // TODO: use in prune_from_minimal
    // TODO: optimize
    //  find minimal clue count for a unique solution for each Base
    //  add them without checking for a unique solution
    // TODO: optimize: re-add logic
    //  - get two ambiguous solutions
    //  - find differing values
    //  - add one of those values
    //  could result in generation of biased sudokus?
    fn near_minimal_grid(
        &self,
        solved_grid: &Grid<Base>,
        prune_settings: &PruningSettings<Base>,
        rng: &mut CrateRng,
    ) -> Result<NearMinimalGridReturn<Base>> {
        debug!("Generating near minimal grid");

        // FIXME: prune_settings.strategies

        debug_assert!(solved_grid.is_solved());

        let mut near_minimal_grid = Grid::<Base>::new();

        let pruning_positions = self.pruning_positions(prune_settings, rng)?;
        let non_pruning_positions = self.non_pruning_positions(prune_settings)?;

        let non_pruning_position_count = u16::try_from(non_pruning_positions.len()).unwrap();

        // Copy over non pruning positions into near_minimal_grid, since they must be contained in the final grid.
        if !non_pruning_positions.is_empty() {
            debug!("Restoring non-pruning positions: {non_pruning_positions:?}");
            for non_pruning_position in non_pruning_positions {
                let solution_value = solved_grid[non_pruning_position].clone();
                debug_assert!(solution_value.has_value());
                near_minimal_grid[non_pruning_position] = solution_value;
            }
            // Check if the non pruning positions already result in a unique solution.
            if let Some(unique_solution) = near_minimal_grid.unique_solution() {
                debug!("Non-pruning positions result in unique solution");
                debug_assert_eq!(unique_solution, *solved_grid);
                return Ok(NearMinimalGridReturn {
                    near_minimal_grid,
                    deleted: pruning_positions
                        .iter()
                        .map(|&pos| (pos, solved_grid[pos].value().unwrap()))
                        .collect(),
                    remaining_pruning_positions: vec![],
                });
            }
        }

        debug!("Restoring pruning positions");

        let pruning_position_count = pruning_positions.len();
        let mut pruning_position_i = 0;

        let solution_guided = false;
        if solution_guided {
            let mut pruning_positions = pruning_positions;
            let mut restored_positions = vec![];

            while let Some(ambiguous_solution) = {
                debug!("Checking grid for ambiguous_solution_pair:\n{near_minimal_grid}");

                let mut solver = introspective::Solver::new(near_minimal_grid.clone());
                if let Some(first_solution) = solver.next() {
                    if &first_solution == solved_grid {
                        if let Some(second_solution) = solver.next() {
                            debug_assert_ne!(&second_solution, solved_grid);
                            Some(second_solution)
                        } else {
                            None
                        }
                    } else {
                        Some(first_solution)
                    }
                } else {
                    None
                }
            } {
                let non_equal_value_positions: BTreeSet<_> = Position::<Base>::all()
                    .filter(|&pos| ambiguous_solution[pos] != solved_grid[pos])
                    .collect();

                let Some(next_pruning_i) = pruning_positions
                    .iter()
                    .position(|pos| non_equal_value_positions.contains(pos))
                else {
                    unreachable!()
                };

                let pruning_position = pruning_positions.remove(next_pruning_i);
                restored_positions.push(pruning_position);

                let solution_value = solved_grid[pruning_position].clone();
                debug!(
                    "Restoring pruning position #{}/{pruning_position_count}: {solution_value} at {pruning_position}",
                    restored_positions.len()
                );
                debug_assert!(solution_value.has_value());
                near_minimal_grid[pruning_position] = solution_value;
            }

            debug!("Restored value resulted in unique solution, stop restoring");

            let deleted: Vec<(Position<Base>, Value<Base>)> = pruning_positions
                .iter()
                .map(|&pos| (pos, solved_grid[pos].value().unwrap()))
                .collect();

            Ok(NearMinimalGridReturn {
                near_minimal_grid,
                deleted,
                remaining_pruning_positions: restored_positions,
            })
        } else {
            // Copy over pruning positions until the grid has a unique solution.
            let Some(stop_index) = pruning_positions.iter().rposition(|&pruning_position| {
                pruning_position_i += 1;

                let solution_value = solved_grid[pruning_position].clone();
                // FIXME: breaks rustfmt
                // debug!("Restoring pruning position #{pruning_position_i}/{pruning_position_count}: {solution_value} at {pruning_position}");
                debug_assert!(solution_value.has_value());

                near_minimal_grid[pruning_position] = solution_value;

                let restored_value_count = pruning_position_i + non_pruning_position_count;
                let minimum_clue_count_for_unique_solution =
                    Base::ENUM.minimum_clue_count_for_unique_solution();
                if restored_value_count < minimum_clue_count_for_unique_solution {
                    // FIXME: breaks rustfmt
                    // debug!("Skip check for unique solution, since restored value count {restored_value_count} is less than minimum clue count for unique solution {minimum_clue_count_for_unique_solution}");
                    return false;
                }

                if let Some(unique_solution) = near_minimal_grid.unique_solution() {
                    debug!("Restored value resulted in unique solution, stop restoring");
                    debug_assert_eq!(unique_solution, *solved_grid);
                    true
                } else {
                    false
                }
            }) else {
                if self.settings.solution.is_some() {
                    bail!("'solution.values_grid' has no solution")
                } else {
                    panic!(
                        "Expected adding of pruning positions to eventually result in a unique solution"
                    )
                }
            };

            let (deleted_pruning_positions, remaining_pruning_positions) =
                pruning_positions.split_at(stop_index);

            let deleted: Vec<(Position<Base>, Value<Base>)> = deleted_pruning_positions
                .iter()
                .map(|&pos| (pos, solved_grid[pos].value().unwrap()))
                .collect();
            let remaining_pruning_positions = remaining_pruning_positions.to_vec();

            Ok(NearMinimalGridReturn {
                near_minimal_grid,
                deleted,
                remaining_pruning_positions,
            })
        }
    }

    fn prune(
        &self,
        solved_grid: Grid<Base>,
        prune_settings: &PruningSettings<Base>,
        on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
        rng: &mut CrateRng,
    ) -> Result<Grid<Base>> {
        let mut pruned_grid = match prune_settings.target {
            PruningTarget::Minimal => {
                self.prune_from_minimal(solved_grid, 0, prune_settings, on_progress, rng)?
            }
            PruningTarget::MinimalPlusClueCunt(clue_count) => {
                self.prune_from_minimal(solved_grid, clue_count, prune_settings, on_progress, rng)?
            }
            PruningTarget::MaxEmptyCellCount(empty_cell_count) => self.prune_from_filled(
                solved_grid,
                empty_cell_count,
                prune_settings,
                on_progress,
                rng,
            )?,
            PruningTarget::MinClueCount(clue_count) => self.prune_from_filled(
                solved_grid,
                Base::CELL_COUNT - clue_count,
                prune_settings,
                on_progress,
                rng,
            )?,
        };

        pruned_grid.fix_all_values();

        if prune_settings.set_all_direct_candidates {
            pruned_grid.set_all_direct_candidates();
        }

        Ok(pruned_grid)
    }

    /// Try to delete a cell using the optimized [`AmbiguousSolutionChecker`].
    ///
    /// This method reuses the SAT solver and leverages incremental solving for better performance.
    ///
    /// Returns the value of the deleted cell, if any.
    fn try_delete_cell_at_pos_with_checker(
        grid: &mut Grid<Base>,
        pos: Position<Base>,
        prune_settings: &PruningSettings<Base>,
        checker: &mut AmbiguousSolutionChecker<Base>,
    ) -> Result<Option<Value<Base>>> {
        let cell = grid.get(pos);

        let Some(deleted_value) = cell.value() else {
            panic!("Expected value at {pos}, instead got: {cell:?}")
        };

        grid.get_mut(pos).delete();

        let can_be_deleted: bool = (
            // Either default strategies
            prune_settings.strategies == StrategySet::with_single(BruteForce.into())
            ||
                // Or ensure the grid remains solvable with the non-default strategies
                grid
                .is_solvable_with_strategies(prune_settings.strategies)
                .is_ok_and(|solution| solution.is_some())
        ) && {
            // Use the optimized checker instead of creating a new solver
            let has_ambiguous_solution = checker.has_ambiguous_solution(pos, deleted_value)?;
            !has_ambiguous_solution
        };

        if can_be_deleted {
            // current position can be removed without losing uniqueness of the grid solution.
            // Update the checker state to reflect the removal
            checker.confirm_removal(pos);
            Ok(Some(deleted_value))
        } else {
            // current position is necessary for unique solution
            // No checker update needed - its state already reflects the position has a value
            grid.get_mut(pos).set_value(deleted_value);
            Ok(None)
        }
    }

    fn shuffle_vec<T>(rng: &mut impl Rng, mut vec: Vec<T>) -> Vec<T> {
        vec.shuffle(rng);
        vec
    }

    fn get_solution_values_grid(&self) -> Result<&Grid<Base>> {
        let Some(SolutionSettings { values_grid }) = &self.settings.solution else {
            bail!(
                "'PruningOrder::SolutionUnfixedValues' requires 'settings.solution.values_grid' to be defined"
            )
        };
        Ok(values_grid)
    }

    fn non_pruning_positions(
        &self,
        prune_settings: &PruningSettings<Base>,
    ) -> Result<Vec<Position<Base>>> {
        Ok(match &prune_settings.order {
            PruningOrder::Random => {
                vec![]
            }
            PruningOrder::Positions {
                positions,
                behaviour,
            } => behaviour.process_non_pruning_positions(
                || positions.clone(),
                || Position::complement(positions.clone()).collect(),
            ),
            PruningOrder::SolutionUnfixedValues { behaviour } => {
                let values_grid = self.get_solution_values_grid()?;

                behaviour.process_non_pruning_positions(
                    || values_grid.all_value_positions(),
                    || values_grid.all_candidates_positions(),
                )
            }
        })
    }

    fn pruning_positions(
        &self,
        prune_settings: &PruningSettings<Base>,
        rng: &mut CrateRng,
    ) -> Result<Vec<Position<Base>>> {
        Ok(match &prune_settings.order {
            PruningOrder::Random => {
                let prunable_positions = if let Some(SolutionSettings { values_grid }) =
                    &self.settings.solution
                {
                    let mut all_unfixed_value_positions = values_grid.all_unfixed_value_positions();
                    all_unfixed_value_positions.extend(values_grid.all_candidates_positions());
                    all_unfixed_value_positions
                } else {
                    Grid::<Base>::all_positions().collect_vec()
                };

                Self::shuffle_vec(rng, prunable_positions)
            }
            PruningOrder::Positions {
                positions,
                behaviour,
            } => behaviour.process_pruning_positions(
                rng,
                |_| positions.clone(),
                |rng| Self::shuffle_vec(rng, Position::complement(positions.clone()).collect()),
            ),
            PruningOrder::SolutionUnfixedValues { behaviour } => {
                let values_grid = self.get_solution_values_grid()?;

                behaviour.process_pruning_positions(
                    rng,
                    |rng| Self::shuffle_vec(rng, values_grid.all_value_positions()),
                    |rng| Self::shuffle_vec(rng, values_grid.all_candidates_positions()),
                )
            }
        })
    }

    fn prune_from_filled(
        &self,
        mut grid: Grid<Base>,
        distance_from_filled: u16,
        prune_settings: &PruningSettings<Base>,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
        rng: &mut CrateRng,
    ) -> Result<Grid<Base>> {
        debug_assert!(grid.is_solved());

        if distance_from_filled == 0 {
            return Ok(grid);
        }

        let pruning_positions: Vec<_> = self.pruning_positions(prune_settings, rng)?;
        let pruning_position_count = pruning_positions.len();

        // Create optimized checker for incremental solving
        let mut checker = AmbiguousSolutionChecker::new(&grid);

        let mut deleted_count = 0;
        for (i, pos) in pruning_positions.into_iter().enumerate() {
            let pruning_position_index = i + 1;

            if deleted_count >= distance_from_filled {
                break;
            }

            if Self::try_delete_cell_at_pos_with_checker(&mut grid, pos, prune_settings, &mut checker)?.is_some() {
                deleted_count += 1;
                debug!(
                    "Position {pruning_position_index}/{pruning_position_count} deleted, totaling {deleted_count}/{distance_from_filled} deleted positions"
                );
            } else {
                debug!(
                    "Position {pruning_position_index}/{pruning_position_count} is required for unique solution"
                );
            }

            on_progress(GeneratorProgress {
                pruning_position_index,
                pruning_position_count,
                deleted_count,
            })?;
        }

        Ok(grid)
    }

    fn prune_from_minimal(
        &self,
        solved_grid: Grid<Base>,
        distance_from_minimal: u16,
        prune_settings: &PruningSettings<Base>,
        mut on_progress: impl FnMut(GeneratorProgress) -> Result<()>,
        rng: &mut CrateRng,
    ) -> Result<Grid<Base>> {
        debug!("Pruning solution to be minimal");

        debug_assert!(solved_grid.is_solved());

        // If the distance results in a filled sudoku, return it directly.
        if distance_from_minimal >= Base::CELL_COUNT {
            debug!(
                "Distance {distance_from_minimal} will result in a solved sudoku, exiting early"
            );

            return Ok(solved_grid);
        }

        // Create optimized checker for incremental solving from the solved grid
        let mut checker = AmbiguousSolutionChecker::new(&solved_grid);

        // TODO: evaluate if near_minimal_grid is always a pessimization
        //  root cause could be the basic backtracking solver implementation
        //  DPLL-based solver could be faster at counting ambiguous solutions
        let NearMinimalGridReturn {
            near_minimal_grid: mut grid,
            mut deleted,
            remaining_pruning_positions,
        } = if prune_settings.start_from_near_minimal_grid {
            let result = self.near_minimal_grid(&solved_grid, prune_settings, rng)?;
            // Synchronize the checker state with the deleted positions from near_minimal_grid
            for (pos, _) in &result.deleted {
                checker.confirm_removal(*pos);
            }
            result
        } else {
            NearMinimalGridReturn {
                near_minimal_grid: solved_grid,
                deleted: vec![],
                remaining_pruning_positions: self.pruning_positions(prune_settings, rng)?,
            }
        };

        debug!(
            "Pruning grid by trying to delete values at positions {remaining_pruning_positions:?} in grid:\n{grid}"
        );

        let remaining_pruning_position_count = remaining_pruning_positions.len();

        // Reduce grid to a minimal solution.
        for (i, pos) in remaining_pruning_positions.into_iter().enumerate() {
            let pruning_position_index = i + 1;

            let deleted_count = u16::try_from(deleted.len()).unwrap();

            if let Some(deleted_value) =
                Self::try_delete_cell_at_pos_with_checker(&mut grid, pos, prune_settings, &mut checker)?
            {
                debug!(
                    "Position {pruning_position_index}/{remaining_pruning_position_count} deleted, totaling {deleted_count} deleted positions"
                );

                deleted.push((pos, deleted_value));
            } else {
                debug!(
                    "Position {pruning_position_index}/{remaining_pruning_position_count} is required for unique solution"
                );
            }

            on_progress(GeneratorProgress {
                pruning_position_index,
                pruning_position_count: remaining_pruning_position_count,
                deleted_count,
            })?;
        }

        // Restore the required amount of values, specified by distance.
        for (restore_i, (deleted_pos, deleted_value)) in
            (1..).zip(deleted.into_iter().rev().take(distance_from_minimal.into()))
        {
            debug!(
                "Restoring deleted value #{restore_i}/{distance_from_minimal}: {deleted_value} at {deleted_pos}"
            );

            grid.get_mut(deleted_pos).set_value(deleted_value);
        }

        Ok(grid)
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;
    use crate::position::Coordinate;
    use crate::samples;

    use super::*;

    #[test]
    fn test_solved() {
        let grid = Generator::<Base2>::default().generate().unwrap();

        assert!(grid.is_solved());
    }

    mod unit {
        use super::*;

        #[test]
        fn test_pruning_positions() {
            type Base = Base2;
            struct Input {
                order: PruningOrder<Base>,
                solution_values_grid: Option<Grid<Base>>,
            }

            struct ExpectedOutput {
                pruning_position_sets: Vec<Vec<Position<Base>>>,
                non_pruning_positions: Vec<Position<Base>>,
            }

            let solution_values_grid = {
                let mut solution_values_grid = samples::base_2_solved();
                solution_values_grid.unfix_all_values();

                // Delete lower half of solution grid
                Position::all_rows()
                    .skip(2)
                    .flatten()
                    .for_each(|pos| solution_values_grid[pos].delete());

                solution_values_grid
            };

            let all_positions = Position::<Base>::all().collect_vec();
            let top_positions = solution_values_grid.all_value_positions();
            let bottom_positions = solution_values_grid.all_candidates_positions();

            let test_cases = vec![
                (
                    Input {
                        order: PruningOrder::Random,
                        solution_values_grid: None,
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![all_positions.clone()],
                        non_pruning_positions: vec![],
                    },
                ),
                (
                    Input {
                        order: PruningOrder::Positions {
                            positions: top_positions.clone(),
                            behaviour: PruningGroupBehaviour::Retain,
                        },
                        solution_values_grid: None,
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![bottom_positions.clone()],
                        non_pruning_positions: top_positions.clone(),
                    },
                ),
                (
                    Input {
                        order: PruningOrder::Positions {
                            positions: top_positions.clone(),
                            behaviour: PruningGroupBehaviour::Exclusive,
                        },
                        solution_values_grid: None,
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![top_positions.clone()],
                        non_pruning_positions: bottom_positions.clone(),
                    },
                ),
                (
                    Input {
                        order: PruningOrder::Positions {
                            positions: top_positions.clone(),
                            behaviour: PruningGroupBehaviour::First,
                        },
                        solution_values_grid: None,
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![
                            top_positions.clone(),
                            bottom_positions.clone(),
                        ],
                        non_pruning_positions: vec![],
                    },
                ),
                (
                    Input {
                        order: PruningOrder::Positions {
                            positions: top_positions.clone(),
                            behaviour: PruningGroupBehaviour::Last,
                        },
                        solution_values_grid: None,
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![
                            bottom_positions.clone(),
                            top_positions.clone(),
                        ],
                        non_pruning_positions: vec![],
                    },
                ),
                (
                    Input {
                        order: PruningOrder::SolutionUnfixedValues {
                            behaviour: PruningGroupBehaviour::Retain,
                        },
                        solution_values_grid: Some(solution_values_grid.clone()),
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![bottom_positions.clone()],
                        non_pruning_positions: top_positions.clone(),
                    },
                ),
                (
                    Input {
                        order: PruningOrder::SolutionUnfixedValues {
                            behaviour: PruningGroupBehaviour::Exclusive,
                        },
                        solution_values_grid: Some(solution_values_grid.clone()),
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![top_positions.clone()],
                        non_pruning_positions: bottom_positions.clone(),
                    },
                ),
                (
                    Input {
                        order: PruningOrder::SolutionUnfixedValues {
                            behaviour: PruningGroupBehaviour::First,
                        },
                        solution_values_grid: Some(solution_values_grid.clone()),
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![
                            top_positions.clone(),
                            bottom_positions.clone(),
                        ],
                        non_pruning_positions: vec![],
                    },
                ),
                (
                    Input {
                        order: PruningOrder::SolutionUnfixedValues {
                            behaviour: PruningGroupBehaviour::Last,
                        },
                        solution_values_grid: Some(solution_values_grid.clone()),
                    },
                    ExpectedOutput {
                        pruning_position_sets: vec![
                            bottom_positions.clone(),
                            top_positions.clone(),
                        ],
                        non_pruning_positions: vec![],
                    },
                ),
            ];

            for (input, expected_output) in test_cases {
                for seed in 0..10 {
                    let generator = Generator::with_settings(GeneratorSettings {
                        prune: Some(PruningSettings {
                            order: input.order.clone(),
                            ..Default::default()
                        }),
                        solution: input
                            .solution_values_grid
                            .clone()
                            .map(|values_grid| SolutionSettings { values_grid }),
                        // Unused
                        seed: Some(seed),
                    });

                    let mut rng = new_crate_rng_with_seed(Some(seed));

                    let prune_settings = generator.settings.prune.as_ref().unwrap().clone();
                    let pruning_positions = generator
                        .pruning_positions(&prune_settings, &mut rng)
                        .unwrap();
                    let non_pruning_positions =
                        generator.non_pruning_positions(&prune_settings).unwrap();

                    assert!(pruning_positions.iter().all_unique());
                    assert!(non_pruning_positions.iter().all_unique());
                    assert!(
                        pruning_positions
                            .iter()
                            .chain(&non_pruning_positions)
                            .all_unique()
                    );

                    assert_eq!(
                        pruning_positions.len() + non_pruning_positions.len(),
                        Base::CELL_COUNT as usize
                    );

                    assert_eq!(
                        pruning_positions.len(),
                        expected_output
                            .pruning_position_sets
                            .iter()
                            .map(|position_set| position_set.len())
                            .sum::<usize>()
                    );
                    let mut pruning_positions_iter = pruning_positions.into_iter();
                    for expected_position_set in expected_output.pruning_position_sets.clone() {
                        let (mut position_set, mut expected_position_set): (Vec<_>, Vec<_>) =
                            (&mut pruning_positions_iter)
                                .zip(expected_position_set)
                                .unzip();
                        position_set.sort();
                        expected_position_set.sort();
                        assert_eq!(position_set, expected_position_set);
                    }

                    assert_eq!(non_pruning_positions, expected_output.non_pruning_positions);
                }
            }
        }

        // #[test]
        // fn test_near_minimal_grid() {
        //     let solution = samples::base_2_solved();
        //     let mut rng = new_crate_rng_with_seed(None);
        //
        //     let near_minimal_grid = Generator::near_minimal_grid(&solution, &mut rng);
        //     assert!(!near_minimal_grid.is_solved());
        //     assert_eq!(near_minimal_grid.unique_solution().unwrap(), solution);
        // }
    }

    mod prune {
        use super::*;
        use crate::solver::strategic::strategies::selection::StrategySelection;

        mod target {
            use super::*;

            #[test]
            fn test_minimal() {
                let grid = Generator::<Base2>::with_target(PruningTarget::Minimal)
                    .generate()
                    .unwrap();

                assert!(grid.is_minimal());
            }

            #[test]
            fn test_minimal_plus_clue_cunt() {
                let grid = Generator::<Base2>::with_target(PruningTarget::MinimalPlusClueCunt(1))
                    .generate()
                    .unwrap();

                assert!(grid.has_unique_solution());

                assert!(grid.all_value_positions().into_iter().any(|value_pos| {
                    let mut grid = grid.clone();
                    grid.unfix_all_values();
                    grid.get_mut(value_pos).delete();
                    grid.is_minimal()
                }));
            }

            #[test]
            fn test_max_empty_cell_count() {
                let grid = Generator::<Base2>::with_target(PruningTarget::MaxEmptyCellCount(2))
                    .generate()
                    .unwrap();

                assert_eq!(grid.all_candidates_positions().len(), 2);

                assert!(grid.has_unique_solution());
            }
            #[test]
            fn test_min_clue_count() {
                let grid = Generator::<Base2>::with_target(PruningTarget::MinClueCount(14))
                    .generate()
                    .unwrap();

                assert_eq!(grid.all_candidates_positions().len(), 2);

                assert!(grid.has_unique_solution());
            }
        }

        // FIXME: currently the slowest test
        //  PASS [   2.334s] sudoku generator::tests::prune::test_strategies
        // Either:
        //  optimize
        //  split into smaller tests (parallelize, rstest)
        //  reduce search space
        #[test]
        fn test_strategies() {
            use crate::solver::strategic::strategies::*;

            fn generate(target: PruningTarget, strategies: StrategySet) -> Grid<Base3> {
                Generator::<Base3>::with_pruning(PruningSettings {
                    strategies,
                    target,
                    ..Default::default()
                })
                .generate()
                .unwrap()
            }

            let targets = vec![PruningTarget::Minimal, PruningTarget::MaxEmptyCellCount(20)];

            for target in targets {
                let grid = generate(target, StrategySet::with_all(false));
                assert!(grid.is_solved());

                let default_strategies = StrategyEnum::default_solver_strategies();
                for i in 1..default_strategies.count() {
                    let strategies = default_strategies.iter_strategies().take(i).collect();
                    let grid = generate(target, strategies);
                    assert!(
                        grid.is_solvable_with_strategies(strategies)
                            .unwrap()
                            .is_some()
                    );
                }
            }
        }
    }

    mod seed {
        use super::*;

        #[test]
        fn test_seed() {
            let pruning_settings_list = vec![
                None,
                Some(PruningSettings {
                    target: PruningTarget::Minimal,
                    ..Default::default()
                }),
                Some(PruningSettings {
                    target: PruningTarget::MinClueCount(0),
                    ..Default::default()
                }),
            ];

            for pruning_settings in pruning_settings_list {
                let gen_1 = Generator::<Base3>::with_settings(GeneratorSettings {
                    seed: Some(1),
                    prune: pruning_settings.clone(),
                    ..Default::default()
                });
                assert_eq!(gen_1.generate().unwrap(), gen_1.generate().unwrap());
                let gen_2 = Generator::<Base3>::with_settings(GeneratorSettings {
                    seed: Some(2),
                    prune: pruning_settings,
                    ..Default::default()
                });
                assert_eq!(gen_2.generate().unwrap(), gen_2.generate().unwrap());
                assert_ne!(gen_1.generate().unwrap(), gen_2.generate().unwrap());
            }
        }

        #[test]
        fn test_no_seed() {
            let generator = Generator::<Base3>::default();

            let grid1 = generator.generate().unwrap();
            let grid2 = generator.generate().unwrap();

            assert!(grid1.is_solved());
            assert!(grid2.is_solved());
            assert_ne!(grid1, grid2);
        }
    }

    mod solution {
        use super::*;

        #[test]
        fn test_partial_values_grid_no_prune() {
            let values_grid: Grid<Base2> = "
  4  3  │  0  0  
  2  1  │  0  0  
────────┼────────
  0  0  │  0  0  
  0  0  │  0  0  "
                .parse()
                .unwrap();

            let grid = Generator::<Base2>::with_settings(GeneratorSettings {
                solution: Some(SolutionSettings {
                    values_grid: values_grid.clone(),
                }),
                ..Default::default()
            })
            .generate()
            .unwrap();

            // Top left block is unchanged
            itertools::assert_equal(
                values_grid.block_cells(Coordinate::default()),
                grid.block_cells(Coordinate::default()),
            );

            assert!(grid.is_solved());
        }

        #[test]
        fn test_partial_values_grid_prune_minimal_retain_solution_values() {
            let values_grid: Grid<Base2> = "
  4  3  │  0  0  
  2  1  │  0  0  
────────┼────────
  0  0  │  0  0  
  0  0  │  0  0  "
                .parse()
                .unwrap();

            let grid = Generator::<Base2>::with_settings(GeneratorSettings {
                solution: Some(SolutionSettings {
                    values_grid: values_grid.clone(),
                }),
                prune: Some(PruningSettings {
                    target: PruningTarget::Minimal,
                    order: PruningOrder::SolutionUnfixedValues {
                        behaviour: PruningGroupBehaviour::Retain,
                    },
                    ..Default::default()
                }),
                ..Default::default()
            })
            .generate()
            .unwrap();

            // Top left block is unchanged
            itertools::assert_equal(
                values_grid.block_cells(Coordinate::default()),
                grid.block_cells(Coordinate::default()),
            );

            // Grid has unique solution
            assert!(grid.has_unique_solution());

            // Grid does not have a unique solution, if any value outside the top left is deleted.
            for non_top_left_block_pos in grid
                .all_value_positions()
                .into_iter()
                .filter(|pos| pos.to_block() != Coordinate::default())
            {
                let mut grid = grid.clone();
                grid.unfix_all_values();
                grid.get_mut(non_top_left_block_pos).delete();
                assert!(!grid.has_unique_solution());
            }
        }

        #[test]
        fn test_solved_values_grid_no_prune() {
            let values_grid = samples::base_2_solved();

            let grid = Generator::<Base2>::with_settings(GeneratorSettings {
                solution: Some(SolutionSettings {
                    values_grid: values_grid.clone(),
                }),
                ..Default::default()
            })
            .generate()
            .unwrap();

            // No-op
            assert_eq!(values_grid, grid);
        }

        #[test]
        fn test_solved_values_grid_prune_minimal() {
            let values_grid = {
                let mut solved_grid = samples::base_2_solved();
                solved_grid.unfix_all_values();
                solved_grid
            };

            let grid = Generator::<Base2>::with_settings(GeneratorSettings {
                solution: Some(SolutionSettings {
                    values_grid: values_grid.clone(),
                }),
                prune: Some(PruningSettings {
                    target: PruningTarget::Minimal,
                    order: PruningOrder::Random,
                    ..Default::default()
                }),
                ..Default::default()
            })
            .generate()
            .unwrap();

            assert!(grid.is_minimal());

            let mut solution = grid.unique_solution().unwrap();
            solution.unfix_all_values();
            assert_eq!(
                solution, values_grid,
                "Solution differs from target solution:\n{solution}\n!=\n{values_grid}"
            );
        }
    }
}
