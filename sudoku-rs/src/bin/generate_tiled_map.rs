#![allow(unused_imports)]

use std::fmt::{Display, Formatter};

use itertools::Itertools;
use ndarray::{s, Array2, ArrayViewMut2, Axis, Dim, SliceInfo, SliceInfoElem};
use rand::prelude::SliceRandom;
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};

use sudoku::base::consts::*;
use sudoku::base::SudokuBase;
use sudoku::cell::{Candidates, Cell};
use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::solver::backtracking_bitset;
use sudoku::solver::backtracking_bitset::AvailabilityDenyList;

type Base = Base3;

type TileIndex = (usize, usize);

// TODO: has_conflict/is_solved
struct CellWorld {
    tile_dim: TileIndex,
    cells: Array2<Cell<Base>>,
    overlap: u8,
}

impl Display for CellWorld {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let builder: Builder = self
            .cells
            .rows()
            .into_iter()
            .map(|cell_row| cell_row.into_iter().map(|cell| cell.to_string()))
            .collect();
        write!(
            f,
            "{}",
            builder
                .build()
                .with(Style::empty())
                .with(Padding::zero())
                .to_string()
        )
    }
}

#[derive(Debug)]
struct WorldGenerationResult {
    success: bool,
    backtrack_count: u32,
}

impl CellWorld {
    pub fn new((tile_row_count, tile_col_count): TileIndex, overlap: u8) -> Self {
        // Various indexing patterns break down for larger overlaps.
        assert!(overlap <= Base::BASE);

        Self {
            tile_dim: (tile_row_count, tile_col_count),
            cells: Array2::default((
                Self::tile_axis_count_to_cell_axis_count(tile_row_count, overlap),
                Self::tile_axis_count_to_cell_axis_count(tile_col_count, overlap),
            )),
            overlap,
        }
    }

    pub fn generate(&mut self) -> WorldGenerationResult {
        let (tile_row_count, tile_col_count) = self.tile_dim;

        let tile_indexes = (0..tile_row_count)
            .flat_map(|tile_row_i| {
                (0..tile_col_count).map(move |tile_col_i| (tile_row_i, tile_col_i))
            })
            .collect_vec();

        let mut backtrack_count = 0;

        // TODO: update with CandidatesVisitOrder
        let mut solver_stack: Vec<backtracking_bitset::Solver<Base, _>> =
            Vec::with_capacity(tile_indexes.len());

        solver_stack.push(backtracking_bitset::Solver::new(self.to_grid_at((0, 0))));

        while let Some(solver) = solver_stack.last_mut() {
            if let Some(solution) = solver.next() {
                let tile_index = tile_indexes[solver_stack.len() - 1];

                self.set_grid_at(&solution, tile_index);

                if solver_stack.len() == tile_indexes.len() {
                    // world generated
                    return WorldGenerationResult {
                        success: true,
                        backtrack_count,
                    };
                } else {
                    // next grid
                    let next_tile_index = tile_indexes[solver_stack.len()];
                    let denylist = self.direct_denylist_from_top_right_grid(next_tile_index);
                    solver_stack.push(backtracking_bitset::Solver::new_with_optional_denylist(
                        self.to_grid_at(next_tile_index),
                        denylist,
                    ));
                }
            } else {
                // Backtrack
                backtrack_count += 1;

                let (tile_row_i, tile_col_i) = tile_indexes[solver_stack.len() - 1];

                println!(
                    "backtrack_count {backtrack_count}, grid:\n{}",
                    self.to_grid_at((tile_row_i, tile_col_i))
                );

                let is_tile_at_left_world_edge = tile_col_i == 0;
                let is_tile_at_top_world_edge = tile_row_i == 0;

                self.delete_grid_overlap_segments(
                    (tile_row_i, tile_col_i),
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

            // println!("{self}\n");
        }

        WorldGenerationResult {
            success: false,
            backtrack_count,
        }
    }

    fn direct_denylist_from_top_right_grid(
        &self,
        (tile_row_i, tile_col_i): TileIndex,
    ) -> Option<AvailabilityDenyList<Base>> {
        let top_right_tile_index = (
            tile_row_i.checked_sub(1)?,
            if tile_col_i + 1 != self.tile_dim.1 {
                Some(tile_col_i + 1)
            } else {
                None
            }?,
        );

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

        let mut denylist = Array2::<Candidates<Base>>::default((
            Base::SIDE_LENGTH.into(),
            Base::SIDE_LENGTH.into(),
        ));

        denylist
            .slice_mut(s![
                // top block row band without overlap
                overlap_isize..isize::from(Base::BASE),
                // right overlap column band
                -overlap_isize..=-1,
            ])
            .fill(denied_corner_candidates);

        assert!(denylist.is_standard_layout());
        Some(denylist.into())
    }

    fn tile_axis_count_to_cell_axis_count(tile_axis_count: usize, overlap: u8) -> usize {
        tile_axis_count * usize::from(Base::SIDE_LENGTH)
            - (tile_axis_count - 1) * usize::from(overlap)
    }

    fn grid_cells_slice_info(
        (tile_row_i, tile_col_i): TileIndex,
        overlap: u8,
    ) -> SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 2]>> {
        let tile_stride = usize::from(Base::SIDE_LENGTH - overlap);
        let top_left_cell_row_i = tile_row_i * tile_stride;
        let top_left_cell_col_i = tile_col_i * tile_stride;

        let side_length_usize = usize::from(Base::SIDE_LENGTH);

        s![
            top_left_cell_row_i..(top_left_cell_row_i + side_length_usize),
            top_left_cell_col_i..(top_left_cell_col_i + side_length_usize),
        ]
    }

    pub fn to_grid_at(&self, tile_index: TileIndex) -> Grid<Base> {
        let grid_cells_array_view = self
            .cells
            .slice(Self::grid_cells_slice_info(tile_index, self.overlap));

        grid_cells_array_view.try_into().unwrap()
    }

    pub fn set_grid_at(&mut self, grid: &Grid<Base>, tile_index: TileIndex) {
        let world_grid_cells = self
            .cells
            .slice_mut(Self::grid_cells_slice_info(tile_index, self.overlap));
        grid.cells().assign_to(world_grid_cells);
    }

    // FIXME: panics for `overflow > Base::SIDE_LENGTH/2`
    pub fn delete_grid_overlap_segments(
        &mut self,
        tile_index: TileIndex,
        overlap_segment_filter: OverlapSegmentFilter,
    ) {
        fn split_cells_into_overlap_segments_single_axis(
            grid_cells: ArrayViewMut2<Cell<Base>>,
            axis: Axis,
            overlap: u8,
        ) -> [ArrayViewMut2<Cell<Base>>; 3] {
            let overlap = usize::from(overlap);

            let (first, rest) = grid_cells.split_at(axis, overlap);
            let (middle, last) =
                rest.split_at(axis, usize::from(Base::SIDE_LENGTH) - (overlap * 2));

            [first, middle, last]
        }

        let grid_cells = self
            .cells
            .slice_mut(Self::grid_cells_slice_info(tile_index, self.overlap));

        let row_bands =
            split_cells_into_overlap_segments_single_axis(grid_cells, Axis(0), self.overlap);

        let [[top_left, top, top_right], [left, middle, right], [bottom_left, bottom, bottom_right]] =
            row_bands.map(|row_band| {
                split_cells_into_overlap_segments_single_axis(row_band, Axis(1), self.overlap)
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

#[derive(Copy, Clone, Debug, Default)]
struct OverlapSegmentFilter {
    top_left: bool,
    top: bool,
    top_right: bool,
    left: bool,
    middle: bool,
    right: bool,
    bottom_left: bool,
    bottom: bool,
    bottom_right: bool,
}

impl OverlapSegmentFilter {
    fn contains_index(&self, index: u8) -> bool {
        match index {
            0 => self.top_left,
            1 => self.top,
            2 => self.top_right,
            3 => self.left,
            4 => self.middle,
            5 => self.right,
            6 => self.bottom_left,
            7 => self.bottom,
            8 => self.bottom_right,
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delete_grid_overlap_segments() {
        let mut cell_world = CellWorld::new((1, 1), 2);
        cell_world
            .cells
            .fill(Cell::with_value(1.try_into().unwrap(), false));

        cell_world.delete_grid_overlap_segments(
            (0, 0),
            OverlapSegmentFilter {
                top_left: true,
                top_right: true,
                bottom_left: true,
                bottom_right: true,
                ..Default::default()
            },
        );

        println!("{cell_world}");

        todo!("assert")
    }
}

fn main() -> Result<()> {
    let (tile_row_count, tile_col_count) = (100, 100);

    // TODO: statistics
    //  - base
    //  - overlap
    //  - tile_count
    //  =>
    //  - how many single shot generations succeed?
    //  - how often is backtracking required?
    let overlap = 2;
    let mut world = CellWorld::new((tile_row_count, tile_col_count), overlap);
    let world_generation_result = world.generate();

    println!("{world}");

    dbg!(world_generation_result);

    Ok(())
}
