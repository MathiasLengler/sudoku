use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::grid::Grid;
use crate::position::Position;
use crate::rng::CrateRng;
use crate::solver::strategic::strategies::BruteForce;
use crate::{base::SudokuBase, solver::strategic::strategies::selection::StrategySet};

pub use dynamic_settings::*;

/// How much to prune the solution.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PruningTarget {
    /// Prune until the grid is minimal, meaning that no clue can be removed without breaking uniqueness of the solution.
    #[default]
    Minimal,
    /// Prune until the grid is minimal, but with N additional clues.
    MinimalPlusClueCunt(u16),
    /// Prune until the grid has at most N empty cells or is minimal, whichever comes first.
    MaxEmptyCellCount(u16),
    /// Prune until the grid has N clues left or is minimal, whichever comes first.
    MinClueCount(u16),
}

// TODO: rename behaviour (UK) to behavior (US)
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PruningGroupBehaviour {
    /// Never prune cells in this group
    Retain,
    /// Only prune cells in this group
    Exclusive,
    /// Prune cells in this group first, then cells outside this group.
    First,
    /// Prune cells in this group last, then cells inside this group.
    Last,
}

impl PruningGroupBehaviour {
    pub(super) fn process_pruning_positions<Base: SudokuBase>(
        self,
        rng: &mut CrateRng,
        get_group_pruning_positions: impl FnOnce(&mut CrateRng) -> Vec<Position<Base>>,
        get_other_pruning_positions: impl FnOnce(&mut CrateRng) -> Vec<Position<Base>>,
    ) -> Vec<Position<Base>> {
        match self {
            PruningGroupBehaviour::Retain => get_other_pruning_positions(rng),
            PruningGroupBehaviour::Exclusive => get_group_pruning_positions(rng),
            PruningGroupBehaviour::First => {
                let mut pruning_positions = get_group_pruning_positions(rng);
                pruning_positions.extend(get_other_pruning_positions(rng));
                pruning_positions
            }
            PruningGroupBehaviour::Last => {
                let mut pruning_positions = get_other_pruning_positions(rng);
                pruning_positions.extend(get_group_pruning_positions(rng));
                pruning_positions
            }
        }
    }

    pub(super) fn process_non_pruning_positions<Base: SudokuBase>(
        self,
        get_group_pruning_positions: impl FnOnce() -> Vec<Position<Base>>,
        get_other_pruning_positions: impl FnOnce() -> Vec<Position<Base>>,
    ) -> Vec<Position<Base>> {
        match self {
            PruningGroupBehaviour::Retain => get_group_pruning_positions(),
            PruningGroupBehaviour::Exclusive => get_other_pruning_positions(),
            PruningGroupBehaviour::First | PruningGroupBehaviour::Last => {
                vec![]
            }
        }
    }
}

// TODO: group_breath_first vs group_depth_first
//  prioritize most empty groups vs even number of values across all groups
// TODO: test
/// Define the order in which cells should be pruned.
#[derive(Debug, Default, Clone)]
pub enum PruningOrder<Base: SudokuBase> {
    /// Prune all cells in a random order
    #[default]
    Random,
    /// Handle cells defined by a list of positions differently.
    Positions {
        /// The positions of the cells
        positions: Vec<Position<Base>>,
        /// How to handle those cells
        /// If pruning is allowed, the visit order will be defined by the list.
        behaviour: PruningGroupBehaviour,
    },
    /// Handle the set of values defined in `settings.solution.values_grid` differently.
    SolutionUnfixedValues {
        /// How to handle those cells
        /// If pruning is allowed, the visit order will be random.
        behaviour: PruningGroupBehaviour,
    },
}

/// How to prune/delete clues from a solved sudoku, while preserving the uniqueness of the solution.
#[derive(Debug, Clone)]
pub struct PruningSettings<Base: SudokuBase> {
    /// Whether to set all direct candidates after pruning is done.
    pub set_all_direct_candidates: bool,
    /// With which strategies the sudoku should remain solvable for.
    pub strategies: StrategySet,
    /// How much to prune the solution.
    pub target: PruningTarget,
    /// Adjust order in which cells are pruned.
    pub order: PruningOrder<Base>,
    /// Optimization: instead of pruning from a solved grid,
    /// first generate a near minimal grid by adding values from the solution to a empty grid,
    /// then prune from there.
    pub start_from_near_minimal_grid: bool,
}

impl<Base: SudokuBase> Default for PruningSettings<Base> {
    fn default() -> Self {
        Self {
            strategies: StrategySet::with_single(BruteForce.into()),
            set_all_direct_candidates: false,
            target: PruningTarget::default(),
            order: PruningOrder::default(),
            start_from_near_minimal_grid: false,
        }
    }
}

/// Influence the generated solution.
#[derive(Debug, Clone)]
pub struct SolutionSettings<Base: SudokuBase> {
    /// Every value cell in this grid will be included in the solution of the generated grid.
    /// Fixed values will never be pruned.
    pub values_grid: Grid<Base>,
}

impl<Base: SudokuBase> Default for SolutionSettings<Base> {
    fn default() -> Self {
        Self {
            values_grid: Grid::default(),
        }
    }
}

#[derive(Debug, Default, Clone)]
pub struct GeneratorSettings<Base: SudokuBase> {
    /// How to prune the solution.
    pub prune: Option<PruningSettings<Base>>,
    /// How to generate the solution.
    pub solution: Option<SolutionSettings<Base>>,
    /// A seed for randomness in the generation process.
    ///
    /// If `Some`, generation of sudokus will be deterministic.
    ///
    /// If `None`, a new random seed will be chosen for each generated sudoku,
    /// making the generation non-deterministic.
    ///
    /// Influence of randomness when generating a sudoku:
    /// - The generated solution of the sudoku.
    /// - The order in which cells are pruned.
    pub seed: Option<u64>,
}

mod dynamic_settings {
    use anyhow::ensure;

    use crate::base::BaseEnum;
    use crate::cell::dynamic::DynamicCell;
    use crate::error::Error;
    use crate::grid::dynamic::DynamicGrid;
    use crate::position::DynamicPosition;

    use super::*;

    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub enum DynamicPruningOrder {
        Random,
        Positions {
            positions: Vec<DynamicPosition>,
            behaviour: PruningGroupBehaviour,
        },
        SolutionUnfixedValues {
            behaviour: PruningGroupBehaviour,
        },
    }

    impl<Base: SudokuBase> TryFrom<DynamicPruningOrder> for PruningOrder<Base> {
        type Error = Error;

        fn try_from(dynamic_pruning_order: DynamicPruningOrder) -> Result<Self> {
            Ok(match dynamic_pruning_order {
                DynamicPruningOrder::Random => Self::Random,
                DynamicPruningOrder::Positions {
                    positions,
                    behaviour,
                } => Self::Positions {
                    positions: positions
                        .into_iter()
                        .map(TryInto::try_into)
                        .collect::<Result<_>>()?,
                    behaviour,
                },
                DynamicPruningOrder::SolutionUnfixedValues { behaviour } => {
                    Self::SolutionUnfixedValues { behaviour }
                }
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicPruningSettings {
        pub set_all_direct_candidates: bool,
        pub strategies: StrategySet,
        pub target: PruningTarget,
        pub order: DynamicPruningOrder,
        pub start_from_near_minimal_grid: bool,
    }

    impl<Base: SudokuBase> TryFrom<DynamicPruningSettings> for PruningSettings<Base> {
        type Error = Error;

        fn try_from(dynamic_pruning_settings: DynamicPruningSettings) -> Result<Self> {
            let DynamicPruningSettings {
                set_all_direct_candidates,
                strategies,
                target,
                order,
                start_from_near_minimal_grid,
            } = dynamic_pruning_settings;

            Ok(Self {
                set_all_direct_candidates,
                strategies,
                target,
                order: order.try_into()?,
                start_from_near_minimal_grid,
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicSolutionSettings {
        pub values_grid: DynamicGrid<DynamicCell>,
    }

    impl<Base: SudokuBase> TryFrom<DynamicSolutionSettings> for SolutionSettings<Base> {
        type Error = Error;

        fn try_from(dynamic_solution_settings: DynamicSolutionSettings) -> Result<Self> {
            let DynamicSolutionSettings { values_grid } = dynamic_solution_settings;
            Ok(Self {
                values_grid: values_grid.try_into()?,
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Clone, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicGeneratorSettings {
        pub base: BaseEnum,
        #[cfg_attr(feature = "wasm", ts(optional = nullable))]
        pub prune: Option<DynamicPruningSettings>,
        #[cfg_attr(feature = "wasm", ts(optional = nullable))]
        pub solution: Option<DynamicSolutionSettings>,
        #[cfg_attr(feature = "wasm", ts(optional = nullable))]
        pub seed: Option<u64>,
    }

    impl<Base: SudokuBase> TryFrom<DynamicGeneratorSettings> for GeneratorSettings<Base> {
        type Error = Error;

        fn try_from(dynamic_generator_settings: DynamicGeneratorSettings) -> Result<Self> {
            let DynamicGeneratorSettings {
                base,
                prune,
                solution,
                seed,
            } = dynamic_generator_settings;

            ensure!(base == Base::ENUM);

            Ok(Self {
                prune: if let Some(prune) = prune {
                    Some(prune.try_into()?)
                } else {
                    None
                },
                solution: if let Some(solution) = solution {
                    Some(solution.try_into()?)
                } else {
                    None
                },
                seed,
            })
        }
    }
}
