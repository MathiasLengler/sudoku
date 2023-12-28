#![allow(unused_imports)]

use std::fmt::{Display, Formatter};

use ndarray::{s, Array2, Dim, SliceInfo, SliceInfoElem};
use tabled::builder::Builder;

use sudoku::base::consts::*;
use sudoku::base::SudokuBase;
use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::generator::{Generator, GeneratorSettings, SolutionSettings};
use sudoku::grid::Grid;

type Base = Base2;

// Idea for different data structure:
// "world of cells" => grid index => generate Grid instance on demand
// - cells overlapping with other grids are not duplicated (2x at edges, 4x at the corners)
// - one active play Grid, needs to be back-propagated to the world of cells
//   - alternative generic Grid cells: https://docs.rs/ndarray/latest/ndarray/type.ArrayViewMut2.html
// - would solve grid cross-synchronization at least partially
//   - tbd. `update_candidates` on edges for adjacent grids

// Strategies for generating sudokus with matching boundaries
// - Boundaries are assumed as fixed values
// - Boundaries are not assumed as fixed values
//   - Starting from a solved sudoku, deletion of givens while ensuring a unique solution will *not* change the single solution.
//     If the solved sudoku was tileable, all minimized sudokus derived from it will remain tileable.
// Tileable sudokus are easier to solve, since two/four sudoku grids constrain the edges/corners.

type TileIndex = (usize, usize);

struct CellWorld {
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
        write!(f, "{}", builder.build().to_string())
    }
}

impl CellWorld {
    pub fn new((tile_row_count, tile_col_count): TileIndex, overlap: u8) -> Self {
        assert!(overlap < Base::SIDE_LENGTH);

        Self {
            cells: Array2::default((
                Self::tile_axis_count_to_cell_axis_count(tile_row_count, overlap),
                Self::tile_axis_count_to_cell_axis_count(tile_col_count, overlap),
            )),
            overlap,
        }
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
}

#[derive(Debug)]
struct Tiles {
    grids: Array2<Grid<Base>>,
    overlap: u8,
}

impl Tiles {
    fn boundary_grid(&self, (tile_row_i, tile_col_i): (usize, usize)) -> Grid<Base> {
        let center_grid = &self.grids[(tile_col_i, tile_col_i)];

        let overlap = isize::from(self.overlap);

        let mut boundary_grid = Grid::new();

        let top_grid = self.grids.get((tile_row_i - 1, tile_col_i));
        if let Some(top_grid) = top_grid {
            top_grid
                .cells()
                .slice(s![-overlap..=-1, ..])
                .assign_to(boundary_grid.cells_mut().slice_mut(s![0..overlap, ..]));
        }
        let left_grid = self.grids.get((tile_row_i, tile_col_i - 1));
        if let Some(left_grid) = left_grid {
            left_grid
                .cells()
                .slice(s![.., -overlap..=-1])
                .assign_to(boundary_grid.cells_mut().slice_mut(s![.., 0..overlap]));
        }
        let right_grid = self.grids.get((tile_row_i, tile_col_i + 1));
        if let Some(right_grid) = right_grid {
            right_grid
                .cells()
                .slice(s![.., 0..overlap])
                .assign_to(boundary_grid.cells_mut().slice_mut(s![.., -overlap..=-1]));
        }
        let bottom_grid = self.grids.get((tile_row_i + 1, tile_col_i));
        if let Some(bottom_grid) = bottom_grid {
            bottom_grid
                .cells()
                .slice(s![0..overlap, ..])
                .assign_to(boundary_grid.cells_mut().slice_mut(s![-overlap..=-1, ..]));
        }

        boundary_grid
    }
}

impl Display for Tiles {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let builder: Builder = self
            .grids
            .rows()
            .into_iter()
            .map(|row_of_grids| row_of_grids.into_iter().map(|grid| grid.to_string()))
            .collect();
        write!(f, "{}", builder.build().to_string())
    }
}

fn main() -> Result<()> {
    let (tile_row_count, tile_col_count) = (3, 3);

    // FIXME: breaks for overlap % Base::BASE != 0
    //  root cause are the corners, where 4 different grids meet.
    //  nothing prevents the selection of the same values across the diagonal:
    //   2 1
    //   1 0
    //  this breaks the top left block uniqueness constraint of the bottom right grid, resulting in an unsolvable border configuration.

    // Potential solutions:
    // - when generating the bottom left grid,
    //   disallow assignment of values contained in the top right "overlap-slice"
    //   to the bottom right "overlap-slice" of the top left block.
    // - Prototype with Constraint programming based solver:
    //   Are all scenarios solvable? Especially for overlap > base, which results in multiple intersected blocks.
    let overlap = 1;
    let mut world = CellWorld::new((tile_row_count, tile_col_count), overlap);
    println!("{world}");

    for tile_row_i in 0..tile_row_count {
        for tile_col_i in 0..tile_col_count {
            let tile_index = (tile_row_i, tile_col_i);
            let grid = world.to_grid_at(tile_index);

            println!("{grid}");
            let generated_grid = Generator::with_settings(GeneratorSettings {
                prune: None,
                solution: Some(SolutionSettings { values_grid: grid }),
                seed: Some(0),
            })
            .generate()
            .unwrap();

            println!("{generated_grid}");

            world.set_grid_at(&generated_grid, tile_index);

            println!("{world}");
        }
    }

    println!("{world}");

    let grid = world.to_grid_at((0, 0));
    println!("{grid}");

    Ok(())
}
