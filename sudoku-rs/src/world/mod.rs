use anyhow::bail;
use log::trace;
use ndarray::{s, Array2, ArrayViewMut2, Axis, Dim, SliceInfo, SliceInfoElem};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::num::NonZeroUsize;
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use overlap_segment_filter::*;
pub use tile_index::*;

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
use crate::world::RelativeTileDir::TopRight;

use self::dynamic::DynamicCellWorldActions;

mod overlap_segment_filter;

mod tile_index;

pub mod dynamic;

/// A two dimensional grid of overlapping sudoku grids.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CellWorld<Base: SudokuBase> {
    tile_dim: TileDim,
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
            "tile_dim: {:?}, overlap: {}, cells:\n{}",
            self.tile_dim,
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
    pub tile_dim: TileDim,
    // FIXME: better name or separate type
    pub cell_dim: TileDim,
    pub overlap: u8,
}

/// Constructors
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn new(tile_dim: TileDim, overlap: u8) -> Self {
        // Various indexing patterns break down for larger overlaps.
        assert!(overlap <= Base::BASE);

        Self {
            tile_dim,
            cells: Array2::default((
                Self::tile_axis_count_to_cell_axis_count(tile_dim.row_count, overlap),
                Self::tile_axis_count_to_cell_axis_count(tile_dim.column_count, overlap),
            )),
            overlap,
        }
    }
}

impl<Base: SudokuBase> DynamicCellWorldActions for CellWorld<Base> {
    // Generation
    fn generate_solved(&mut self, seed: Option<u64>) -> Result<WorldGenerationResult> {
        let tile_indexes: Vec<ValidatedTileIndex> = self.all_validated_tile_indexes().collect();

        let mut backtrack_count = 0;

        let mut solver_stack: Vec<backtracking::Solver<Base, _, _, _>> =
            Vec::with_capacity(tile_indexes.len());

        let mut rng = new_crate_rng_with_seed(seed);

        solver_stack.push(
            backtracking::Solver::builder(self.to_grid_at(TileIndex::default())?)
                .rng(new_crate_rng_from_rng(&mut rng))
                .availability_filter(None)
                .build(),
        );

        while let Some(solver) = solver_stack.last_mut() {
            if let Some(solution) = solver.next() {
                let tile_index = tile_indexes[solver_stack.len() - 1];

                self.set_grid_at_validated(&solution, tile_index);

                if solver_stack.len() == tile_indexes.len() {
                    // world generated
                    return Ok(WorldGenerationResult { backtrack_count });
                } else {
                    // next grid
                    let next_tile_index = tile_indexes[solver_stack.len()];
                    let denylist = self.direct_denylist_from_top_right_grid(next_tile_index);
                    let next_grid = self.to_grid_at_validated(next_tile_index);
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

                let tile_index = tile_indexes[solver_stack.len() - 1];

                trace!(
                    "backtrack_count {backtrack_count}, grid:\n{}",
                    self.to_grid_at_validated(tile_index)
                );

                let is_tile_at_left_world_edge = tile_index.get().is_at_left_edge();
                let is_tile_at_top_world_edge = tile_index.get().is_at_top_edge();

                self.delete_grid_overlap_segments(
                    tile_index,
                    OverlapSegmentFilter {
                        top_left: is_tile_at_left_world_edge && is_tile_at_top_world_edge,
                        top: is_tile_at_top_world_edge,
                        top_right: is_tile_at_top_world_edge,
                        left: is_tile_at_left_world_edge,
                        middle: true,
                        right: true,
                        bottom_left: is_tile_at_left_world_edge,
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

        for tile_index in self.all_validated_tile_indexes() {
            let grid = self.to_grid_at_validated(tile_index);

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

            self.set_grid_at_validated(&pruned_grid, tile_index);
        }

        Ok(())
    }

    // DynamicGrid interop
    fn to_grid_at(&self, tile_index: TileIndex) -> Result<DynamicGrid<DynamicCell>> {
        Ok(self.to_grid_at(tile_index)?.into())
    }

    fn set_grid_at(&mut self, grid: DynamicGrid<DynamicCell>, tile_index: TileIndex) -> Result<()> {
        self.set_grid_at(&grid.try_into()?, tile_index)?;
        Ok(())
    }

    // Queries
    fn dimensions(&self) -> CellWorldDimensions {
        CellWorldDimensions {
            tile_dim: self.tile_dim,
            cell_dim: TileDim {
                row_count: self.cells.nrows().try_into().unwrap(),
                column_count: self.cells.ncols().try_into().unwrap(),
            },
            overlap: self.overlap,
        }
    }

    fn is_solved(&self) -> bool {
        self.all_validated_tile_indexes()
            .all(|tile_index| self.to_grid_at_validated(tile_index).is_solved())
    }

    fn is_directly_consistent(&self) -> bool {
        self.all_validated_tile_indexes().all(|tile_index| {
            self.to_grid_at_validated(tile_index)
                .is_directly_consistent()
        })
    }

    fn all_world_cells(&self) -> Vec<DynamicCell> {
        self.cells.iter().map(|cell| cell.into()).collect()
    }
}

/// Grid interop
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn to_grid_at(&self, tile_index: TileIndex) -> Result<Grid<Base>> {
        Ok(self.to_grid_at_validated(tile_index.validate(self.tile_dim)?))
    }

    fn to_grid_at_validated(&self, tile_index: ValidatedTileIndex) -> Grid<Base> {
        let grid_cells_array_view = self
            .cells
            .slice(Self::grid_cells_slice_info(tile_index, self.overlap));

        grid_cells_array_view.try_into().unwrap()
    }

    pub fn set_grid_at(&mut self, grid: &Grid<Base>, tile_index: TileIndex) -> Result<()> {
        self.set_grid_at_validated(grid, tile_index.validate(self.tile_dim)?);
        Ok(())
    }

    fn set_grid_at_validated(&mut self, grid: &Grid<Base>, tile_index: ValidatedTileIndex) {
        // TODO: update only newly set values
        //  `grid.update_direct_candidates_for_new_value`
        self.set_grid_at_no_candidates_update(grid, tile_index);

        let tile_dim = self.tile_dim;
        for adj_tile_index in
            RelativeTileDir::all().filter_map(|dir| tile_index.adjacent(dir, tile_dim))
        {
            let mut adj_grid = self.to_grid_at_validated(adj_tile_index);
            adj_grid.update_all_direct_candidates();
            self.set_grid_at_no_candidates_update(&adj_grid, adj_tile_index);
        }
    }

    fn set_grid_at_no_candidates_update(
        &mut self,
        grid: &Grid<Base>,
        tile_index: ValidatedTileIndex,
    ) {
        let world_grid_cells = self
            .cells
            .slice_mut(Self::grid_cells_slice_info(tile_index, self.overlap));
        grid.cells_view().assign_to(world_grid_cells);
    }
}

/// Iterators
impl<Base: SudokuBase> CellWorld<Base> {
    pub fn all_grids(&self) -> impl Iterator<Item = Grid<Base>> + '_ {
        self.all_validated_tile_indexes()
            .map(move |tile_index| self.to_grid_at_validated(tile_index))
    }

    pub fn all_tile_indexes(&self) -> impl Iterator<Item = TileIndex> {
        self.tile_dim.all_indexes()
    }

    fn all_validated_tile_indexes(&self) -> impl Iterator<Item = ValidatedTileIndex> {
        self.tile_dim.all_validated_indexes()
    }
}

type GridCellsSliceInfo = SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 2]>>;

/// Internal helpers
impl<Base: SudokuBase> CellWorld<Base> {
    fn direct_denylist_from_top_right_grid(
        &self,
        tile_index: ValidatedTileIndex,
    ) -> Option<DeniedCandidatesGrid<Base>> {
        let top_right_tile_index = tile_index.adjacent(TopRight, self.tile_dim)?;

        let top_right_grid_cells = self.cells.slice(Self::grid_cells_slice_info(
            top_right_tile_index,
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

    fn tile_axis_count_to_cell_axis_count(tile_axis_count: NonZeroUsize, overlap: u8) -> usize {
        let tile_axis_count = tile_axis_count.get();
        tile_axis_count * usize::from(Base::SIDE_LENGTH)
            - (tile_axis_count - 1) * usize::from(overlap)
    }

    fn grid_cells_slice_info(tile_index: ValidatedTileIndex, overlap: u8) -> GridCellsSliceInfo {
        let tile_index = tile_index.get();
        let tile_stride = usize::from(Base::SIDE_LENGTH - overlap);
        let top_left_cell_row_i = tile_index.row * tile_stride;
        let top_left_cell_col_i = tile_index.column * tile_stride;

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
        tile_index: ValidatedTileIndex,
        overlap_segment_filter: OverlapSegmentFilter,
    ) {
        let grid_cells = self
            .cells
            .slice_mut(Self::grid_cells_slice_info(tile_index, self.overlap));

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
    use super::*;
    use crate::base::consts::*;

    #[test]
    fn test_prune_is_directly_consistent() {
        let tile_dim = TileDim {
            row_count: 3,
            column_count: 3,
        };
        let seed = 1;
        let overlap = 1;

        let mut world = CellWorld::<Base2>::new(tile_dim, overlap);
        world.generate_solved(Some(seed));
        assert!(world.is_solved());
        world.prune(Some(seed));
        assert!(world.is_directly_consistent());
    }

    #[test]
    fn test_delete_grid_overlap_segments() {
        let mut cell_world = CellWorld::<Base2>::new(
            TileDim {
                row_count: 3,
                column_count: 3,
            },
            1,
        );
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

            let tile_index = TileIndex { row: 1, column: 1 };
            cell_world.delete_grid_overlap_segments(tile_index, overlap_segment_filter);

            dbg!(&expected_deleted_positions);

            let grid = cell_world.to_grid_at(tile_index);
            let deleted_positions = grid.all_candidates_positions();
            assert_eq!(
                deleted_positions, expected_deleted_positions,
                "{overlap_segment_filter:?} => {grid}"
            );
        }
    }
}
