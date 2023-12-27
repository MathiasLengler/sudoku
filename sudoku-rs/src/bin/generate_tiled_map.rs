#![allow(unused_imports)]

use std::fmt::{Display, Formatter};

use ndarray::{s, Array2};
use tabled::builder::Builder;

use sudoku::base::consts::Base2;
use sudoku::cell::Cell;
use sudoku::error::Result;
use sudoku::grid::Grid;
use sudoku::samples::base_2_candidates_coordinates;

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

struct CellWorld {
    cells: Array2<Cell<Base>>,
    overlap: u8,
}

// impl CellWorld {
//     fn new()
// }

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
    let mut tiles = Tiles {
        grids: Array2::default((3, 3)),
        overlap: 1,
    };

    tiles.grids.iter_mut().enumerate().for_each(|(i, grid)| {
        *grid = base_2_candidates_coordinates()
        // grid.cells_mut().fill(Cell::with_value(
        //     u8::try_from((i % usize::from(Base::SIDE_LENGTH)) + 1)
        //         .unwrap()
        //         .try_into()
        //         .unwrap(),
        //     false,
        // ));
    });

    println!("{tiles}");

    let boundary_grid = tiles.boundary_grid((1, 1));

    println!("{boundary_grid}");

    tiles.grids[(1, 1)] = boundary_grid;

    println!("{tiles}");

    Ok(())
}
