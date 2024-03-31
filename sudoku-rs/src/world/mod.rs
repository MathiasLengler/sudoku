use anyhow::bail;
use log::{info, trace};
use ndarray::{s, Array2, ArrayViewMut2, Axis, Dim, SliceInfo, SliceInfoElem};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};
#[cfg(feature = "wasm")]
use ts_rs::TS;

pub use grid_index::*;
use overlap_segment_filter::*;

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
use crate::solver::backtracking::availability_filter::DeniedCandidatesGrid;
use crate::world::RelativeGridDir::TopRight;

use self::dynamic::DynamicCellWorldActions;

mod overlap_segment_filter;

mod grid_index;

pub mod dynamic;

/// A two dimensional grid of overlapping sudoku grids.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CellWorld<Base: SudokuBase> {
    grid_dim: WorldDim,
    cells: Array2<Cell<Base>>,
    overlap: u8,
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

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WorldGenerationResult {
    pub backtrack_count: u32,
}

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Copy, Clone, Eq, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CellWorldDimensions {
    pub grid_dim: WorldDim,
    // FIXME: better name or separate type
    pub cell_dim: WorldDim,
    pub overlap: u8,
}

/// Constructors
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn new(grid_dim: WorldDim, overlap: u8) -> Self {
        // Various indexing patterns break down for larger overlaps.
        assert!(overlap <= Base::BASE);

        Self {
            grid_dim,
            cells: Array2::default((
                Self::grid_axis_count_to_cell_axis_count(grid_dim.row_count, overlap),
                Self::grid_axis_count_to_cell_axis_count(grid_dim.column_count, overlap),
            )),
            overlap,
        }
    }
}

impl<Base: SudokuBase> DynamicCellWorldActions for CellWorld<Base> {
    // Generation
    fn generate_solved(&mut self, seed: Option<u64>) -> Result<WorldGenerationResult> {
        let grid_indexes: Vec<ValidatedGridIndex> = self.all_validated_grid_indexes().collect();

        let mut backtrack_count = 0;

        let mut solver_stack: Vec<backtracking::Solver<Base, _, _, _>> =
            Vec::with_capacity(grid_indexes.len());

        let mut rng = new_crate_rng_with_seed(seed);

        solver_stack.push(
            backtracking::Solver::builder(self.to_grid_at(GridIndex::default())?)
                .rng(new_crate_rng_from_rng(&mut rng))
                .availability_filter(None)
                .build(),
        );

        while let Some(solver) = solver_stack.last_mut() {
            if let Some(solution) = solver.next() {
                info!(
                    "CellWorld::generate_solved: solver {}/{} found solution",
                    solver_stack.len(),
                    grid_indexes.len()
                );
                trace!("Solution found: {solution}");

                let grid_index = grid_indexes[solver_stack.len() - 1];

                self.set_grid_at_validated(&solution, grid_index);

                if solver_stack.len() == grid_indexes.len() {
                    // world generated
                    return Ok(WorldGenerationResult { backtrack_count });
                } else {
                    // next grid
                    let next_grid_index = grid_indexes[solver_stack.len()];
                    let denylist = self.direct_denylist_from_top_right_grid(next_grid_index);
                    let next_grid = self.to_grid_at_validated(next_grid_index);
                    solver_stack.push(
                        backtracking::Solver::builder(next_grid)
                            .rng(new_crate_rng_from_rng(&mut rng))
                            .availability_filter(denylist)
                            .build(),
                    );
                }
            } else {
                // Backtrack
                backtrack_count += 1;

                let grid_index = grid_indexes[solver_stack.len() - 1];

                trace!(
                    "backtrack_count {backtrack_count}, grid:\n{}",
                    self.to_grid_at_validated(grid_index)
                );

                let is_grid_at_left_world_edge = grid_index.get().is_at_left_edge();
                let is_grid_at_top_world_edge = grid_index.get().is_at_top_edge();

                self.delete_grid_overlap_segments(
                    grid_index,
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
        //  as long as its neighbours are unsolved.

        let (middle_positions, _overlap_positions): (Vec<_>, Vec<_>) = Position::<Base>::all()
            .partition(|pos| {
                let (row, column) = pos.to_row_and_column();
                let (row, column) = (row.get(), column.get());
                let middle_axis_range = self.overlap..(Base::SIDE_LENGTH - self.overlap);
                middle_axis_range.contains(&row) && middle_axis_range.contains(&column)
            });

        for (progress_index, grid_index) in (0..).zip(self.all_validated_grid_indexes()) {
            let grid = self.to_grid_at_validated(grid_index);

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
            })
            .generate()?;

            info!(
                "CellWorld::prune: pruned grid #{}/{}",
                progress_index,
                self.grid_dim.all_indexes_count()
            );

            self.set_grid_at_validated(&pruned_grid, grid_index);
        }

        Ok(())
    }

    // DynamicGrid interop
    fn to_grid_at(&self, grid_index: GridIndex) -> Result<DynamicGrid<DynamicCell>> {
        Ok(self.to_grid_at(grid_index)?.into())
    }

    fn set_grid_at(&mut self, grid: DynamicGrid<DynamicCell>, grid_index: GridIndex) -> Result<()> {
        self.set_grid_at(&grid.try_into()?, grid_index)?;
        Ok(())
    }

    // Queries
    fn dimensions(&self) -> CellWorldDimensions {
        CellWorldDimensions {
            grid_dim: self.grid_dim,
            cell_dim: WorldDim {
                row_count: self.cells.nrows().try_into().unwrap(),
                column_count: self.cells.ncols().try_into().unwrap(),
            },
            overlap: self.overlap,
        }
    }

    fn is_solved(&self) -> bool {
        self.all_validated_grid_indexes()
            .all(|grid_index| self.to_grid_at_validated(grid_index).is_solved())
    }

    fn is_directly_consistent(&self) -> bool {
        self.all_validated_grid_indexes().all(|grid_index| {
            self.to_grid_at_validated(grid_index)
                .is_directly_consistent()
        })
    }

    fn all_world_cells(&self) -> Vec<DynamicCell> {
        self.cells.iter().map(|cell| cell.into()).collect()
    }
}

/// Grid interop
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn to_grid_at(&self, grid_index: GridIndex) -> Result<Grid<Base>> {
        Ok(self.to_grid_at_validated(grid_index.validate(self.grid_dim)?))
    }

    fn to_grid_at_validated(&self, grid_index: ValidatedGridIndex) -> Grid<Base> {
        let grid_cells_array_view = self
            .cells
            .slice(Self::grid_cells_slice_info(grid_index, self.overlap));

        grid_cells_array_view.try_into().unwrap()
    }

    pub fn set_grid_at(&mut self, grid: &Grid<Base>, grid_index: GridIndex) -> Result<()> {
        self.set_grid_at_validated(grid, grid_index.validate(self.grid_dim)?);
        Ok(())
    }

    fn set_grid_at_validated(&mut self, grid: &Grid<Base>, grid_index: ValidatedGridIndex) {
        // TODO: update only newly set values
        //  `grid.update_direct_candidates_for_new_value`
        self.set_grid_at_no_candidates_update(grid, grid_index);

        let grid_dim = self.grid_dim;
        for adj_grid_index in
            RelativeGridDir::all().filter_map(|dir| grid_index.adjacent(dir, grid_dim))
        {
            let mut adj_grid = self.to_grid_at_validated(adj_grid_index);
            adj_grid.update_all_direct_candidates();
            self.set_grid_at_no_candidates_update(&adj_grid, adj_grid_index);
        }
    }

    fn set_grid_at_no_candidates_update(
        &mut self,
        grid: &Grid<Base>,
        grid_index: ValidatedGridIndex,
    ) {
        let world_grid_cells = self
            .cells
            .slice_mut(Self::grid_cells_slice_info(grid_index, self.overlap));
        grid.cells_view().assign_to(world_grid_cells);
    }
}

/// Iterators
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn all_grids(&self) -> impl Iterator<Item = Grid<Base>> + '_ {
        self.all_validated_grid_indexes()
            .map(move |grid_index| self.to_grid_at_validated(grid_index))
    }

    pub fn all_grid_indexes(&self) -> impl Iterator<Item = GridIndex> {
        self.grid_dim.all_indexes()
    }

    fn all_validated_grid_indexes(&self) -> impl Iterator<Item = ValidatedGridIndex> {
        self.grid_dim.all_validated_indexes()
    }
}

type GridCellsSliceInfo = SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 2]>>;

/// Internal helpers
impl<Base: SudokuBase> CellWorld<Base> {
    fn direct_denylist_from_top_right_grid(
        &self,
        grid_index: ValidatedGridIndex,
    ) -> Option<DeniedCandidatesGrid<Base>> {
        let top_right_grid_index = grid_index.adjacent(TopRight, self.grid_dim)?;

        let top_right_grid_cells = self.cells.slice(Self::grid_cells_slice_info(
            top_right_grid_index,
            self.overlap,
        ));

        let overlap_isize = isize::from(self.overlap);

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

    fn grid_axis_count_to_cell_axis_count(grid_axis_count: NonZeroUsize, overlap: u8) -> usize {
        let grid_axis_count = grid_axis_count.get();
        grid_axis_count * usize::from(Base::SIDE_LENGTH)
            - (grid_axis_count - 1) * usize::from(overlap)
    }

    fn grid_cells_slice_info(grid_index: ValidatedGridIndex, overlap: u8) -> GridCellsSliceInfo {
        let grid_index = grid_index.get();
        let grid_stride = usize::from(Base::SIDE_LENGTH - overlap);
        let top_left_cell_row_i = grid_index.row * grid_stride;
        let top_left_cell_col_i = grid_index.column * grid_stride;

        let side_length_usize = usize::from(Base::SIDE_LENGTH);

        s![
            top_left_cell_row_i..(top_left_cell_row_i + side_length_usize),
            top_left_cell_col_i..(top_left_cell_col_i + side_length_usize),
        ]
    }

    fn split_cells_into_overlap_segments_single_axis(
        grid_cells: ArrayViewMut2<Cell<Base>>,
        axis: Axis,
        overlap: u8,
    ) -> [ArrayViewMut2<Cell<Base>>; 3] {
        let overlap = usize::from(overlap);

        let (first, rest) = grid_cells.split_at(axis, overlap);
        let (middle, last) = rest.split_at(axis, usize::from(Base::SIDE_LENGTH) - (overlap * 2));

        [first, middle, last]
    }

    fn delete_grid_overlap_segments(
        &mut self,
        grid_index: ValidatedGridIndex,
        overlap_segment_filter: OverlapSegmentFilter,
    ) {
        let grid_cells = self
            .cells
            .slice_mut(Self::grid_cells_slice_info(grid_index, self.overlap));

        let row_bands =
            Self::split_cells_into_overlap_segments_single_axis(grid_cells, Axis(0), self.overlap);

        let [[top_left, top, top_right], [left, middle, right], [bottom_left, bottom, bottom_right]] =
            row_bands.map(|row_band| {
                Self::split_cells_into_overlap_segments_single_axis(row_band, Axis(1), self.overlap)
            });

        for (index, mut overlap_segment) in (0..).zip([
            top_left,
            top,
            top_right,
            left,
            middle,
            right,
            bottom_left,
            bottom,
            bottom_right,
        ]) {
            if overlap_segment_filter.contains_index(index) {
                overlap_segment.fill(Cell::new());
            }
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
        let grid_dim = WorldDim {
            row_count: 3.try_into().unwrap(),
            column_count: 3.try_into().unwrap(),
        };
        let seed = 1;
        let overlap = 1;

        let mut world = CellWorld::<Base2>::new(grid_dim, overlap);
        world.generate_solved(Some(seed)).unwrap();
        assert!(world.is_solved());
        world.prune(Some(seed)).unwrap();
        assert!(world.is_directly_consistent());
    }

    #[test]
    fn test_delete_grid_overlap_segments() {
        let grid_dim = WorldDim {
            row_count: 3.try_into().unwrap(),
            column_count: 3.try_into().unwrap(),
        };
        let mut cell_world = CellWorld::<Base2>::new(grid_dim, 1);
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

            let grid_index = GridIndex { row: 1, column: 1 }.validate(grid_dim).unwrap();
            cell_world.delete_grid_overlap_segments(grid_index, overlap_segment_filter);

            dbg!(&expected_deleted_positions);

            let grid = cell_world.to_grid_at_validated(grid_index);
            let deleted_positions = grid.all_candidates_positions();
            assert_eq!(
                deleted_positions, expected_deleted_positions,
                "{overlap_segment_filter:?} => {grid}"
            );
        }
    }
}
