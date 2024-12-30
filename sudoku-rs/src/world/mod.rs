use anyhow::bail;
use log::{info, trace};
use ndarray::{s, Array2, ArrayView2, ArrayViewMut2, Axis};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};

pub use indexing::*;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::cell::{Candidates, Cell};
use crate::error::Result;
use crate::generator::{
    Generator, GeneratorSettings, PruningGroupBehaviour, PruningOrder, PruningSettings,
    PruningTarget, SolutionSettings,
};
use crate::grid::dynamic::DynamicGrid;
use crate::grid::Grid;
use crate::position::Position;
use crate::rng::{new_crate_rng_from_rng, new_crate_rng_with_seed};
use crate::solver::backtracking;
use crate::solver::backtracking::candidates_filter::DeniedCandidatesGrid;
use crate::world::RelativeDir::TopRight;

use self::dynamic::DynamicCellWorldActions;

mod indexing;

pub mod dynamic;

/// A two dimensional grid of overlapping sudoku grids.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CellWorld<Base: SudokuBase> {
    grid_dim: WorldGridDim,
    cells: Array2<Cell<Base>>,
    overlap: GridOverlap<Base>,
}

impl<Base: SudokuBase> Display for CellWorld<Base> {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let builder: Builder = self
            .cells
            .rows()
            .into_iter()
            .map(|cell_row| cell_row.into_iter().map(|cell| cell.to_string()))
            .collect();
        write!(
            f,
            "grid_dim: {:?}, overlap: {}, cells:\n{}",
            self.grid_dim,
            self.overlap,
            builder.build().with(Style::empty()).with(Padding::zero())
        )
    }
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldGenerationResult {
    pub backtrack_count: u32,
}

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellWorldDimensions {
    #[cfg_attr(
        feature = "wasm",
        ts(type = "import('../../sudoku-web/src/app/state/world').WorldGridDim")
    )]
    pub grid_dim: WorldGridDim,
    #[cfg_attr(
        feature = "wasm",
        ts(type = "import('../../sudoku-web/src/app/state/world').WorldCellDim")
    )]
    pub cell_dim: WorldCellDim,
    pub overlap: u8,
}

/// Constructors
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn new(grid_dim: WorldGridDim, overlap: GridOverlap<Base>) -> Self {
        Self {
            cells: Array2::default(grid_dim.to_cell_dim::<Base>(overlap).as_cells_shape()),
            grid_dim,
            overlap,
        }
    }
}

impl<Base: SudokuBase> DynamicCellWorldActions for CellWorld<Base> {
    // Generation
    fn generate_solved(&mut self, seed: Option<u64>) -> Result<WorldGenerationResult> {
        let grid_positions: Vec<ValidatedWorldGridPosition> =
            self.all_validated_grid_positions().collect();

        let mut backtrack_count = 0;

        let mut solver_stack: Vec<backtracking::Solver<Base, _, _, _>> =
            Vec::with_capacity(grid_positions.len());

        let mut rng = new_crate_rng_with_seed(seed);

        solver_stack.push(
            backtracking::Solver::builder(self.to_grid_at(WorldPosition::default())?)
                .rng(new_crate_rng_from_rng(&mut rng))
                .candidates_filter(Grid::new())
                .build(),
        );

        while let Some(solver) = solver_stack.last_mut() {
            if let Some(solution) = solver.next() {
                info!(
                    "CellWorld::generate_solved: solver {}/{} found solution",
                    solver_stack.len(),
                    grid_positions.len()
                );
                trace!("Solution found: {solution}");

                let grid_position = grid_positions[solver_stack.len() - 1];

                self.set_grid_at_validated(&solution, grid_position);

                if solver_stack.len() == grid_positions.len() {
                    // world generated
                    return Ok(WorldGenerationResult { backtrack_count });
                } else {
                    // next grid
                    let next_grid_position = grid_positions[solver_stack.len()];
                    let denylist = self
                        .direct_denylist_from_top_right_grid(next_grid_position)
                        .unwrap_or_default();
                    let next_grid = self.to_grid_at_validated(next_grid_position);
                    solver_stack.push(
                        backtracking::Solver::builder(next_grid)
                            .rng(new_crate_rng_from_rng(&mut rng))
                            .candidates_filter(denylist)
                            .build(),
                    );
                }
            } else {
                // Backtrack
                backtrack_count += 1;

                let grid_position = grid_positions[solver_stack.len() - 1];

                trace!(
                    "backtrack_count {backtrack_count}, grid:\n{}",
                    self.to_grid_at_validated(grid_position)
                );

                let is_grid_at_left_world_edge = grid_position.get().is_at_left_edge();
                let is_grid_at_top_world_edge = grid_position.get().is_at_top_edge();

                self.delete_grid_overlap_segments(
                    grid_position,
                    OverlapSegmentFilter {
                        top_left: is_grid_at_left_world_edge && is_grid_at_top_world_edge,
                        top: is_grid_at_top_world_edge,
                        top_right: is_grid_at_top_world_edge,
                        left: is_grid_at_left_world_edge,
                        middle: true,
                        right: true,
                        bottom_left: is_grid_at_left_world_edge,
                        bottom: true,
                        bottom_right: true,
                    },
                );
                solver_stack.pop().unwrap();
            }
        }

        bail!("Failed to generate world: exhausted solver stack after {backtrack_count} backtracks")
    }

    fn prune(&mut self, seed: Option<u64>) -> Result<()> {
        let mut rng = new_crate_rng_with_seed(seed);

        assert!(self.is_solved());

        // TODO: abstract world pruning
        //  - overlap/middle
        //  - PruningGroupBehaviour
        //  - retain/modify already pruned values in overlap

        // FIXME: how do we prevent pruning of fixed values in the overlap while exposing pruning settings?
        //  *should* we prevent that? this could result in subgrids without a unique solution,
        //  as long as its neighbors are unsolved.

        let (middle_positions, _overlap_positions): (Vec<_>, Vec<_>) = Position::<Base>::all()
            .partition(|pos| {
                let (row, column) = pos.to_row_and_column();
                let (row, column) = (row.get(), column.get());
                let middle_axis_range =
                    self.overlap.get()..(Base::SIDE_LENGTH - self.overlap.get());
                middle_axis_range.contains(&row) && middle_axis_range.contains(&column)
            });

        for (progress_index, grid_position) in (0..).zip(self.all_validated_grid_positions()) {
            let grid = self.to_grid_at_validated(grid_position);

            let pruned_grid = Generator::with_settings(GeneratorSettings {
                // TODO: expose
                prune: Some(PruningSettings {
                    set_all_direct_candidates: true,
                    order: PruningOrder::Positions {
                        positions: middle_positions
                            .iter()
                            .filter(|pos| !grid.get(**pos).has_fixed_value())
                            .copied()
                            .collect(),
                        behaviour: PruningGroupBehaviour::Exclusive,
                    },
                    target: PruningTarget::Minimal,
                    ..Default::default()
                }),
                solution: Some(SolutionSettings { values_grid: grid }),
                seed: Some(rng.gen()),
                ..Default::default()
            })
            .generate()?;

            info!(
                "CellWorld::prune: pruned grid #{}/{}",
                progress_index,
                self.grid_dim.all_positions_count()
            );

            self.set_grid_at_validated(&pruned_grid, grid_position);
        }

        Ok(())
    }

    // DynamicGrid interop
    fn to_grid_at(&self, grid_position: WorldGridPosition) -> Result<DynamicGrid<DynamicCell>> {
        Ok(self.to_grid_at(grid_position)?.into())
    }

    fn set_grid_at(
        &mut self,
        grid: DynamicGrid<DynamicCell>,
        grid_position: WorldGridPosition,
    ) -> Result<()> {
        self.set_grid_at(&grid.try_into()?, grid_position)?;
        Ok(())
    }

    // Queries
    fn dimensions(&self) -> CellWorldDimensions {
        CellWorldDimensions {
            grid_dim: self.grid_dim,
            cell_dim: self.cell_dim(),
            overlap: self.overlap.get(),
        }
    }

    fn is_solved(&self) -> bool {
        self.all_validated_grid_positions()
            .all(|grid_position| self.to_grid_at_validated(grid_position).is_solved())
    }

    fn is_directly_consistent(&self) -> bool {
        self.all_validated_grid_positions().all(|grid_position| {
            self.to_grid_at_validated(grid_position)
                .is_directly_consistent()
        })
    }

    fn all_world_cells(&self) -> Vec<DynamicCell> {
        self.cells.iter().map(|cell| cell.into()).collect()
    }

    // Indexing helpers
    fn world_cell_position_to_nearest_world_grid_cell_position(
        &self,
        cell_position: WorldCellPosition,
        tie_break: Quadrant,
    ) -> Result<DynamicWorldGridCellPosition> {
        Ok(self
            .world_cell_position_to_nearest_world_grid_cell_position(
                cell_position.validate(self.cell_dim())?,
                tie_break,
            )
            .into())
    }
}

/// Grid interop
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn to_grid_at(&self, grid_position: WorldGridPosition) -> Result<Grid<Base>> {
        Ok(self.to_grid_at_validated(grid_position.validate(self.grid_dim)?))
    }

    fn to_grid_at_validated(&self, grid_position: ValidatedWorldGridPosition) -> Grid<Base> {
        let grid_cells_array_view = self.grid_cells(grid_position);

        grid_cells_array_view.try_into().unwrap()
    }

    pub fn set_grid_at(
        &mut self,
        grid: &Grid<Base>,
        grid_position: WorldGridPosition,
    ) -> Result<()> {
        self.set_grid_at_validated(grid, grid_position.validate(self.grid_dim)?);
        Ok(())
    }

    fn set_grid_at_validated(
        &mut self,
        grid: &Grid<Base>,
        grid_position: ValidatedWorldGridPosition,
    ) {
        // TODO: update only newly set values
        //  `grid.update_direct_candidates_for_new_value`
        self.set_grid_at_no_candidates_update(grid, grid_position);

        let grid_dim = self.grid_dim;
        for adj_grid_position in
            RelativeDir::all().filter_map(|dir| grid_position.adjacent(dir, grid_dim))
        {
            let mut adj_grid = self.to_grid_at_validated(adj_grid_position);
            adj_grid.update_all_direct_candidates();
            self.set_grid_at_no_candidates_update(&adj_grid, adj_grid_position);
        }
    }

    fn set_grid_at_no_candidates_update(
        &mut self,
        grid: &Grid<Base>,
        grid_position: ValidatedWorldGridPosition,
    ) {
        let world_grid_cells = self.grid_cells_mut(grid_position);
        grid.cells_view().assign_to(world_grid_cells);
    }
}

/// Indexing helpers
impl<Base: SudokuBase> CellWorld<Base> {
    fn world_cell_position_to_nearest_world_grid_cell_position(
        &self,
        cell_position: ValidatedWorldCellPosition,
        tie_break: Quadrant,
    ) -> WorldGridCellPosition<Base> {
        cell_position.get().to_nearest_world_grid_cell_position(
            self.grid_dim,
            self.overlap,
            tie_break,
        )
    }
}

/// Iterators
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn all_grids(&self) -> impl Iterator<Item = Grid<Base>> + '_ {
        self.all_validated_grid_positions()
            .map(move |grid_position| self.to_grid_at_validated(grid_position))
    }

    pub fn all_grid_positions(&self) -> impl Iterator<Item = WorldGridPosition> {
        self.grid_dim.all_positions()
    }

    fn all_validated_grid_positions(&self) -> impl Iterator<Item = ValidatedWorldGridPosition> {
        self.grid_dim.all_validated_positions()
    }
}

/// Internal helpers
impl<Base: SudokuBase> CellWorld<Base> {
    fn cell_dim(&self) -> WorldCellDim {
        WorldCellDim::new(self.cells.nrows(), self.cells.ncols()).unwrap()
    }

    fn grid_cells(&self, grid_position: ValidatedWorldGridPosition) -> ArrayView2<Cell<Base>> {
        self.cells
            .slice(Self::grid_cells_slice_info(grid_position, self.overlap))
    }
    fn grid_cells_mut(
        &mut self,
        grid_position: ValidatedWorldGridPosition,
    ) -> ArrayViewMut2<Cell<Base>> {
        self.cells
            .slice_mut(Self::grid_cells_slice_info(grid_position, self.overlap))
    }

    fn grid_cells_slice_info(
        grid_position: ValidatedWorldGridPosition,
        overlap: GridOverlap<Base>,
    ) -> GridCellsSliceInfo {
        grid_position.grid_cells_slice_info::<Base>(overlap)
    }

    fn direct_denylist_from_top_right_grid(
        &self,
        grid_position: ValidatedWorldGridPosition,
    ) -> Option<DeniedCandidatesGrid<Base>> {
        let top_right_grid_position = grid_position.adjacent(TopRight, self.grid_dim)?;

        let top_right_grid_cells = self.grid_cells(top_right_grid_position);

        let overlap_isize = self.overlap.get_isize();

        let top_right_constraining_corner_cells = top_right_grid_cells.slice(s![
            // bottom overlap row band
            -overlap_isize..=-1,
            // left block column band without overlap
            overlap_isize..isize::from(Base::BASE)
        ]);

        let denied_corner_candidates: Candidates<Base> = top_right_constraining_corner_cells
            .into_iter()
            .map(|cell| cell.value().expect("top right grid to contain only values"))
            .collect();

        let mut denylist = Grid::new();
        denylist
            .cells_view_mut()
            .slice_mut(s![
                // top block row band without overlap
                overlap_isize..isize::from(Base::BASE),
                // right overlap column band
                -overlap_isize..=-1,
            ])
            .fill(denied_corner_candidates);

        Some(denylist)
    }

    fn split_cells_into_overlap_segments_single_axis(
        grid_cells: ArrayViewMut2<Cell<Base>>,
        axis: Axis,
        overlap: GridOverlap<Base>,
    ) -> [ArrayViewMut2<Cell<Base>>; 3] {
        let overlap = overlap.get_usize();

        let (first, rest) = grid_cells.split_at(axis, overlap);
        let (middle, last) = rest.split_at(axis, usize::from(Base::SIDE_LENGTH) - (overlap * 2));

        [first, middle, last]
    }

    // TODO: convert into OverlapSegments impl
    fn split_cells_into_overlap_segments(
        grid_cells: ArrayViewMut2<Cell<Base>>,
        overlap: GridOverlap<Base>,
    ) -> OverlapSegments<ArrayViewMut2<Cell<Base>>> {
        let row_bands =
            Self::split_cells_into_overlap_segments_single_axis(grid_cells, Axis(0), overlap);

        let [[top_left, top, top_right], [left, middle, right], [bottom_left, bottom, bottom_right]] =
            row_bands.map(|row_band| {
                Self::split_cells_into_overlap_segments_single_axis(row_band, Axis(1), overlap)
            });

        OverlapSegments {
            top_left,
            top,
            top_right,
            left,
            middle,
            right,
            bottom_left,
            bottom,
            bottom_right,
        }
    }

    fn delete_grid_overlap_segments(
        &mut self,
        grid_position: ValidatedWorldGridPosition,
        overlap_segment_filter: OverlapSegmentFilter,
    ) {
        let overlap = self.overlap;
        let grid_cells = self.grid_cells_mut(grid_position);

        let grid_cells_overlap_segments =
            Self::split_cells_into_overlap_segments(grid_cells, overlap);

        for mut overlap_segment in
            grid_cells_overlap_segments.into_iter_filtered(overlap_segment_filter)
        {
            overlap_segment.fill(Cell::new());
        }
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;
    use crate::base::consts::*;

    #[test]
    fn test_prune_is_directly_consistent() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let seed = 1;
        let overlap = 1.try_into().unwrap();

        let mut world = CellWorld::<Base2>::new(grid_dim, overlap);
        world.generate_solved(Some(seed)).unwrap();
        assert!(world.is_solved());
        world.prune(Some(seed)).unwrap();
        assert!(world.is_directly_consistent());
    }

    #[test]
    fn test_delete_grid_overlap_segments() {
        let grid_dim = WorldGridDim::new(3, 3).unwrap();
        let mut cell_world = CellWorld::<Base2>::new(grid_dim, 1.try_into().unwrap());
        cell_world
            .cells
            .fill(Cell::with_value(1.try_into().unwrap(), false));

        let test_cases = vec![
            (
                OverlapSegmentFilter {
                    top_left: true,
                    ..Default::default()
                },
                vec![(0, 0)],
            ),
            (
                OverlapSegmentFilter {
                    top: true,
                    ..Default::default()
                },
                vec![(0, 1), (0, 2)],
            ),
            (
                OverlapSegmentFilter {
                    top_right: true,
                    ..Default::default()
                },
                vec![(0, 3)],
            ),
            (
                OverlapSegmentFilter {
                    left: true,
                    ..Default::default()
                },
                vec![(1, 0), (2, 0)],
            ),
            (
                OverlapSegmentFilter {
                    middle: true,
                    ..Default::default()
                },
                vec![(1, 1), (1, 2), (2, 1), (2, 2)],
            ),
            (
                OverlapSegmentFilter {
                    right: true,
                    ..Default::default()
                },
                vec![(1, 3), (2, 3)],
            ),
            (
                OverlapSegmentFilter {
                    bottom_left: true,
                    ..Default::default()
                },
                vec![(3, 0)],
            ),
            (
                OverlapSegmentFilter {
                    bottom: true,
                    ..Default::default()
                },
                vec![(3, 1), (3, 2)],
            ),
            (
                OverlapSegmentFilter {
                    bottom_right: true,
                    ..Default::default()
                },
                vec![(3, 3)],
            ),
        ];

        for (overlap_segment_filter, expected_deleted_positions) in test_cases {
            let expected_deleted_positions = expected_deleted_positions
                .into_iter()
                .map(|pos| pos.try_into().unwrap())
                .collect_vec();

            let mut cell_world = cell_world.clone();

            let grid_position = WorldGridPosition::new(1, 1).validate(grid_dim).unwrap();
            cell_world.delete_grid_overlap_segments(grid_position, overlap_segment_filter);

            dbg!(&expected_deleted_positions);

            let grid = cell_world.to_grid_at_validated(grid_position);
            let deleted_positions = grid.all_candidates_positions();
            assert_eq!(
                deleted_positions, expected_deleted_positions,
                "{overlap_segment_filter:?} => {grid}"
            );
        }
    }
}
