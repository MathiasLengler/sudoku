use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::grid::Grid;
use crate::position::{Coordinate, Position};
use crate::rng::CrateRng;
use crate::solver::strategic::strategies::BruteForce;
use crate::{base::SudokuBase, solver::strategic::strategies::selection::StrategySet};

pub use dynamic_settings::*;

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum PruningTarget {
    #[default]
    Minimal,
    MinimalPlusClueCunt(u16),
    MaxEmptyCellCount(u16),
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

/// Symmetry constraint for pruning.
/// When a symmetry is applied, cells are deleted in pairs (or groups) that preserve the specified symmetry.
#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Default, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "camelCase")]
pub enum PruningSymmetry {
    /// No symmetry constraint. Cells are pruned independently.
    #[default]
    None,
    /// 180-degree rotational symmetry (point symmetry around the center).
    /// When a cell at (r, c) is deleted, the cell at (n-1-r, n-1-c) is also deleted.
    Rotational180,
    /// Horizontal mirror symmetry.
    /// When a cell at (r, c) is deleted, the cell at (r, n-1-c) is also deleted.
    HorizontalMirror,
    /// Vertical mirror symmetry.
    /// When a cell at (r, c) is deleted, the cell at (n-1-r, c) is also deleted.
    VerticalMirror,
    /// Diagonal symmetry (main diagonal from top-left to bottom-right).
    /// When a cell at (r, c) is deleted, the cell at (c, r) is also deleted.
    DiagonalMain,
    /// Anti-diagonal symmetry (from top-right to bottom-left).
    /// When a cell at (r, c) is deleted, the cell at (n-1-c, n-1-r) is also deleted.
    DiagonalAnti,
}

impl PruningSymmetry {
    /// Get the symmetric position for a given position under this symmetry.
    /// Returns `None` if the position maps to itself (e.g., center cell for 180° rotation).
    pub fn symmetric_position<Base: SudokuBase>(
        self,
        pos: Position<Base>,
    ) -> Option<Position<Base>> {
        let (row, column) = pos.to_row_and_column();
        let max_coord = Coordinate::<Base>::max().get();

        let (sym_row, sym_col) = match self {
            PruningSymmetry::None => return None,
            PruningSymmetry::Rotational180 => (max_coord - row.get(), max_coord - column.get()),
            PruningSymmetry::HorizontalMirror => (row.get(), max_coord - column.get()),
            PruningSymmetry::VerticalMirror => (max_coord - row.get(), column.get()),
            PruningSymmetry::DiagonalMain => (column.get(), row.get()),
            PruningSymmetry::DiagonalAnti => (max_coord - column.get(), max_coord - row.get()),
        };

        let sym_pos: Position<Base> = (sym_row, sym_col)
            .try_into()
            .expect("symmetric position should be valid");

        // Return None if position maps to itself
        if sym_pos == pos { None } else { Some(sym_pos) }
    }

    /// Check if a position is the "primary" position of a symmetric pair.
    /// Used to avoid processing the same pair twice.
    /// A position is primary if its cell index is less than its symmetric counterpart,
    /// or if it has no symmetric counterpart.
    pub fn is_primary_position<Base: SudokuBase>(self, pos: Position<Base>) -> bool {
        match self.symmetric_position(pos) {
            None => true, // Position maps to itself, so it's primary
            Some(sym_pos) => pos.cell_index() < sym_pos.cell_index(),
        }
    }
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
    /// Symmetry constraint for pruning.
    /// When enabled, cells are deleted in pairs (or groups) that preserve the specified symmetry.
    pub symmetry: PruningSymmetry,
}

impl<Base: SudokuBase> Default for PruningSettings<Base> {
    fn default() -> Self {
        Self {
            strategies: StrategySet::with_single(BruteForce.into()),
            set_all_direct_candidates: false,
            target: PruningTarget::default(),
            order: PruningOrder::default(),
            start_from_near_minimal_grid: false,
            symmetry: PruningSymmetry::default(),
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
    #[derive(Debug, Serialize, Deserialize)]
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
    #[derive(Debug, Serialize, Deserialize)]
    #[serde(rename_all = "camelCase")]
    pub struct DynamicPruningSettings {
        pub set_all_direct_candidates: bool,
        pub strategies: StrategySet,
        pub target: PruningTarget,
        pub order: DynamicPruningOrder,
        pub start_from_near_minimal_grid: bool,
        #[serde(default)]
        pub symmetry: PruningSymmetry,
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
                symmetry,
            } = dynamic_pruning_settings;

            Ok(Self {
                set_all_direct_candidates,
                strategies,
                target,
                order: order.try_into()?,
                start_from_near_minimal_grid,
                symmetry,
            })
        }
    }

    #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
    #[derive(Debug, Serialize, Deserialize)]
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
    #[derive(Debug, Serialize, Deserialize)]
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::base::consts::{Base2, Base3};

    mod pruning_symmetry {
        use super::*;

        #[test]
        fn test_symmetric_position_none() {
            // With no symmetry, no symmetric position should be returned
            let pos: Position<Base3> = (4u8, 4u8).try_into().unwrap();
            assert_eq!(PruningSymmetry::None.symmetric_position(pos), None);

            let corner: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            assert_eq!(PruningSymmetry::None.symmetric_position(corner), None);
        }

        #[test]
        fn test_symmetric_position_rotational_180() {
            // For Base3 (9x9 grid), max_coord = 8
            // (0,0) -> (8,8), (0,8) -> (8,0), (4,4) -> (4,4) (center)
            let top_left: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let bottom_right: Position<Base3> = (8u8, 8u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::Rotational180.symmetric_position(top_left),
                Some(bottom_right)
            );

            let top_right: Position<Base3> = (0u8, 8u8).try_into().unwrap();
            let bottom_left: Position<Base3> = (8u8, 0u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::Rotational180.symmetric_position(top_right),
                Some(bottom_left)
            );

            // Center cell maps to itself
            let center: Position<Base3> = (4u8, 4u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::Rotational180.symmetric_position(center),
                None
            );
        }

        #[test]
        fn test_symmetric_position_horizontal_mirror() {
            // (r, c) -> (r, n-1-c)
            // For Base3: (0,0) -> (0,8), (4,2) -> (4,6)
            let pos1: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let expected1: Position<Base3> = (0u8, 8u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::HorizontalMirror.symmetric_position(pos1),
                Some(expected1)
            );

            let pos2: Position<Base3> = (4u8, 2u8).try_into().unwrap();
            let expected2: Position<Base3> = (4u8, 6u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::HorizontalMirror.symmetric_position(pos2),
                Some(expected2)
            );

            // Center column maps to itself
            let center_col: Position<Base3> = (0u8, 4u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::HorizontalMirror.symmetric_position(center_col),
                None
            );
        }

        #[test]
        fn test_symmetric_position_vertical_mirror() {
            // (r, c) -> (n-1-r, c)
            // For Base3: (0,0) -> (8,0), (2,4) -> (6,4)
            let pos1: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let expected1: Position<Base3> = (8u8, 0u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::VerticalMirror.symmetric_position(pos1),
                Some(expected1)
            );

            let pos2: Position<Base3> = (2u8, 4u8).try_into().unwrap();
            let expected2: Position<Base3> = (6u8, 4u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::VerticalMirror.symmetric_position(pos2),
                Some(expected2)
            );

            // Center row maps to itself
            let center_row: Position<Base3> = (4u8, 0u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::VerticalMirror.symmetric_position(center_row),
                None
            );
        }

        #[test]
        fn test_symmetric_position_diagonal_main() {
            // (r, c) -> (c, r)
            // For Base3: (0,1) -> (1,0), (2,5) -> (5,2)
            let pos1: Position<Base3> = (0u8, 1u8).try_into().unwrap();
            let expected1: Position<Base3> = (1u8, 0u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::DiagonalMain.symmetric_position(pos1),
                Some(expected1)
            );

            let pos2: Position<Base3> = (2u8, 5u8).try_into().unwrap();
            let expected2: Position<Base3> = (5u8, 2u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::DiagonalMain.symmetric_position(pos2),
                Some(expected2)
            );

            // Diagonal cells map to themselves
            let diag: Position<Base3> = (3u8, 3u8).try_into().unwrap();
            assert_eq!(PruningSymmetry::DiagonalMain.symmetric_position(diag), None);
        }

        #[test]
        fn test_symmetric_position_diagonal_anti() {
            // (r, c) -> (n-1-c, n-1-r)
            // For Base3: (0,0) -> (8,8), (0,8) -> (0,8) (maps to self)
            let pos1: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let expected1: Position<Base3> = (8u8, 8u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::DiagonalAnti.symmetric_position(pos1),
                Some(expected1)
            );

            let pos2: Position<Base3> = (1u8, 2u8).try_into().unwrap();
            let expected2: Position<Base3> = (6u8, 7u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::DiagonalAnti.symmetric_position(pos2),
                Some(expected2)
            );

            // Anti-diagonal cells map to themselves
            let anti_diag: Position<Base3> = (0u8, 8u8).try_into().unwrap();
            assert_eq!(
                PruningSymmetry::DiagonalAnti.symmetric_position(anti_diag),
                None
            );
        }

        #[test]
        fn test_is_primary_position() {
            // With no symmetry, all positions are primary
            for pos in Position::<Base2>::all() {
                assert!(PruningSymmetry::None.is_primary_position(pos));
            }

            // For Rotational180, only positions with smaller index than their symmetric counterpart are primary
            let top_left: Position<Base3> = (0u8, 0u8).try_into().unwrap();
            let bottom_right: Position<Base3> = (8u8, 8u8).try_into().unwrap();
            assert!(PruningSymmetry::Rotational180.is_primary_position(top_left));
            assert!(!PruningSymmetry::Rotational180.is_primary_position(bottom_right));

            // Center is primary (maps to itself)
            let center: Position<Base3> = (4u8, 4u8).try_into().unwrap();
            assert!(PruningSymmetry::Rotational180.is_primary_position(center));
        }

        #[test]
        fn test_primary_positions_cover_all_pairs() {
            // Test that primary positions + their symmetric counterparts cover all positions exactly once
            fn test_coverage<Base: SudokuBase>(symmetry: PruningSymmetry) {
                let mut covered = vec![false; Base::CELL_COUNT as usize];

                for pos in Position::<Base>::all() {
                    if symmetry.is_primary_position(pos) {
                        let idx = pos.cell_index() as usize;
                        assert!(!covered[idx], "Position {pos} already covered");
                        covered[idx] = true;

                        if let Some(sym_pos) = symmetry.symmetric_position(pos) {
                            let sym_idx = sym_pos.cell_index() as usize;
                            assert!(!covered[sym_idx], "Symmetric position {sym_pos} already covered");
                            covered[sym_idx] = true;
                        }
                    }
                }

                assert!(
                    covered.iter().all(|&c| c),
                    "Not all positions covered for {symmetry:?}"
                );
            }

            test_coverage::<Base2>(PruningSymmetry::Rotational180);
            test_coverage::<Base3>(PruningSymmetry::Rotational180);
            test_coverage::<Base2>(PruningSymmetry::HorizontalMirror);
            test_coverage::<Base3>(PruningSymmetry::HorizontalMirror);
            test_coverage::<Base2>(PruningSymmetry::VerticalMirror);
            test_coverage::<Base3>(PruningSymmetry::VerticalMirror);
            test_coverage::<Base2>(PruningSymmetry::DiagonalMain);
            test_coverage::<Base3>(PruningSymmetry::DiagonalMain);
            test_coverage::<Base2>(PruningSymmetry::DiagonalAnti);
            test_coverage::<Base3>(PruningSymmetry::DiagonalAnti);
        }

        #[test]
        fn test_default_is_none() {
            assert_eq!(PruningSymmetry::default(), PruningSymmetry::None);
        }
    }
}
