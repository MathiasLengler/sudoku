#![allow(unused_imports)]

use std::fmt::{Display, Formatter};

use itertools::{iproduct, Itertools};
use ndarray::{s, Array2, ArrayViewMut2, Axis, Dim, SliceInfo, SliceInfoElem};
use rand::prelude::SliceRandom;
use rand::SeedableRng;
use rayon::prelude::*;
use tabled::builder::Builder;
use tabled::settings::{Padding, Style};

use sudoku::base::consts::*;
use sudoku::base::SudokuBase;
use sudoku::cell::{Candidates, Cell};
use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::rng::{new_crate_rng, CrateRng};
use sudoku::solver::backtracking_bitset;
use sudoku::solver::backtracking_bitset::AvailabilityDenyList;

type TileIndex = (usize, usize);

#[derive(Debug, Clone, Eq, PartialEq)]
struct CellWorld<Base: SudokuBase> {
    tile_dim: TileIndex,
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

impl<Base: SudokuBase> CellWorld<Base> {
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

    fn all_tile_indexes(&self) -> impl Iterator<Item = TileIndex> {
        let (tile_row_count, tile_col_count) = self.tile_dim;

        (0..tile_row_count).flat_map(move |tile_row_i| {
            (0..tile_col_count).map(move |tile_col_i| (tile_row_i, tile_col_i))
        })
    }

    pub fn generate(&mut self, seed: Option<u64>) -> WorldGenerationResult {
        let tile_indexes = self.all_tile_indexes().collect_vec();

        let mut backtrack_count = 0;

        // TODO: update with CandidatesVisitOrder
        let mut solver_stack: Vec<backtracking_bitset::Solver<Base, _, _>> =
            Vec::with_capacity(tile_indexes.len());

        let mut rng = new_crate_rng(seed);

        solver_stack.push(
            backtracking_bitset::Solver::new_with_optional_denylist_and_rng(
                self.to_grid_at((0, 0)),
                None,
                CrateRng::from_rng(&mut rng).unwrap(),
            ),
        );

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
                    solver_stack.push(
                        backtracking_bitset::Solver::new_with_optional_denylist_and_rng(
                            self.to_grid_at(next_tile_index),
                            denylist,
                            CrateRng::from_rng(&mut rng).unwrap(),
                        ),
                    );
                }
            } else {
                // Backtrack
                backtrack_count += 1;

                let (tile_row_i, tile_col_i) = tile_indexes[solver_stack.len() - 1];

                // println!(
                //     "backtrack_count {backtrack_count}, grid:\n{}",
                //     self.to_grid_at((tile_row_i, tile_col_i))
                // );

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

    // TODO: prune

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

    // FIXME: panics for `overflow > Base::SIDE_LENGTH/2`
    pub fn delete_grid_overlap_segments(
        &mut self,
        tile_index: TileIndex,
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

    pub fn is_solved(&self) -> bool {
        self.all_tile_indexes()
            .all(|tile_index| self.to_grid_at(tile_index).is_solved())
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
        let mut cell_world = CellWorld::<Base2>::new((3, 3), 1);
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

            let tile_index = (1, 1);
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

fn main() -> Result<()> {
    fn gen_world<Base: SudokuBase>(
        overlap: u8,
        tile_dim: TileIndex,
        seed: u64,
    ) -> WorldGenerationResult {
        let mut world = CellWorld::<Base>::new(tile_dim, overlap);
        world.generate(Some(seed))
    }

    fn gen_worlds_stats<Base: SudokuBase>() {
        for (overlap, tile_dim) in iproduct!(
            1..=Base::BASE,
            vec![
                (2, 2),
                (3, 3),
                (4, 4),
                (5, 5),
                (10, 10),
                (50, 50),
                (100, 100)
            ]
        ) {
            let tile_count = u64::try_from(tile_dim.0 * tile_dim.1).unwrap();
            let target_tile_count = 1_000_000;

            let total_seeds = target_tile_count / tile_count;
            let total_seeds_f64 = total_seeds as f64;

            let world_generation_results: Vec<_> = (0..total_seeds)
                .into_par_iter()
                .map(|seed| gen_world::<Base>(overlap, tile_dim, seed))
                .collect();

            let total_success_count: u32 = world_generation_results
                .iter()
                .flat_map(|res| res.success.then_some(1))
                .sum();

            let backtrack_counts = world_generation_results
                .iter()
                .map(|res| res.backtrack_count);
            let total_backtrack_count: u32 = backtrack_counts.clone().sum();
            let min_backtrack_count: u32 = backtrack_counts.clone().min().unwrap();
            let max_backtrack_count: u32 = backtrack_counts.max().unwrap();

            println!(
                "base {}, overlap {overlap}, tile_dim {tile_dim:?}:",
                Base::BASE
            );
            println!(
                "total_success_count {total_success_count} {:.2}%",
                (f64::from(total_success_count) / total_seeds_f64) * 100.
            );
            println!(
                "total_backtrack_count {total_backtrack_count} avg {:.2} min {min_backtrack_count} max {max_backtrack_count}",
                (f64::from(total_backtrack_count) / total_seeds_f64)
            );

            println!()
        }
    }

    fn playground() {
        let (tile_row_count, tile_col_count) = (20, 20);

        let overlap = 2;

        let mut world = CellWorld::<Base2>::new((tile_row_count, tile_col_count), overlap);
        let world_generation_result = world.generate(Some(0));

        println!("{world}");
        dbg!(world_generation_result);
        dbg!(world.is_solved());
    }

    gen_worlds_stats::<Base2>();
    gen_worlds_stats::<Base3>();

    // playground();

    Ok(())
}
