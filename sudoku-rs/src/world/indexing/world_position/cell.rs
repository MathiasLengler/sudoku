use std::{num::NonZeroUsize, ops::Add};

use crate::{
    base::SudokuBase,
    position::Position,
    world::{
        AxisOrdering, CellMarker, GridOverlap, Quadrant, WorldGridCellPosition, WorldGridDim,
        WorldGridPosition,
    },
};

use super::{WorldGridCellAxisIndex, WorldPosition};

pub type WorldCellPosition = WorldPosition<CellMarker>;

impl WorldCellPosition {
    pub fn to_nearest_world_grid_cell_position<Base: SudokuBase>(
        self,
        grid_dim: WorldGridDim,
        overlap: GridOverlap<Base>,
        tie_break: Quadrant,
    ) -> WorldGridCellPosition<Base> {
        let row_grid_cell_axis_index = WorldGridCellAxisIndex::<Base>::from_world_cell_axis_index(
            self.row,
            grid_dim.row_count,
            overlap,
        );
        let column_grid_cell_axis_index =
            WorldGridCellAxisIndex::<Base>::from_world_cell_axis_index(
                self.column,
                grid_dim.column_count,
                overlap,
            );

        let (row_tie_break, column_tie_break) = tie_break.to_axis_orderings();

        (
            row_grid_cell_axis_index.normalize_to_nearest_world_grid_axis_index(
                grid_dim.row_count,
                overlap,
                row_tie_break,
            ),
            column_grid_cell_axis_index.normalize_to_nearest_world_grid_axis_index(
                grid_dim.column_count,
                overlap,
                column_tie_break,
            ),
        )
            .into()
    }

    pub fn to_grid_cell_positions<Base: SudokuBase>(
        self,
        grid_dim: WorldGridDim,
        overlap: GridOverlap<Base>,
    ) -> WorldCellPositionToGridCellPositions<Base> {
        let world_grid_cell_row_indexes = Self::cell_axis_index_to_world_grid_cell_axis_indexes::<
            Base,
        >(self.row, grid_dim.row_count, overlap);

        let world_grid_cell_column_indexes = Self::cell_axis_index_to_world_grid_cell_axis_indexes::<
            Base,
        >(self.column, grid_dim.column_count, overlap);

        match (world_grid_cell_row_indexes, world_grid_cell_column_indexes) {
            (
                CellAxisIndexToGridCellAxisIndexes::Unambiguous(row),
                CellAxisIndexToGridCellAxisIndexes::Unambiguous(column),
            ) => WorldCellPositionToGridCellPositions::Unambiguous((row, column).into()),
            (
                CellAxisIndexToGridCellAxisIndexes::Overlap {
                    start: row_start,
                    end: row_end,
                },
                CellAxisIndexToGridCellAxisIndexes::Unambiguous(column),
            ) => WorldCellPositionToGridCellPositions::OverlapHorizontal {
                top: (row_start, column).into(),
                bottom: (row_end, column).into(),
            },
            (
                CellAxisIndexToGridCellAxisIndexes::Unambiguous(row),
                CellAxisIndexToGridCellAxisIndexes::Overlap {
                    start: column_start,
                    end: column_end,
                },
            ) => WorldCellPositionToGridCellPositions::OverlapVertical {
                left: (row, column_start).into(),
                right: (row, column_end).into(),
            },
            (
                CellAxisIndexToGridCellAxisIndexes::Overlap {
                    start: row_start,
                    end: row_end,
                },
                CellAxisIndexToGridCellAxisIndexes::Overlap {
                    start: column_start,
                    end: column_end,
                },
            ) => WorldCellPositionToGridCellPositions::OverlapCorner {
                top_left: (row_start, column_start).into(),
                top_right: (row_start, column_end).into(),
                bottom_left: (row_end, column_start).into(),
                bottom_right: (row_end, column_end).into(),
            },
        }
    }

    fn cell_axis_index_to_world_grid_cell_axis_indexes<Base: SudokuBase>(
        world_cell_axis_index: usize,
        world_grid_axis_count: NonZeroUsize,
        overlap: GridOverlap<Base>,
    ) -> CellAxisIndexToGridCellAxisIndexes<Base> {
        let world_grid_cell_axis_index = WorldGridCellAxisIndex::<Base>::from_world_cell_axis_index(
            world_cell_axis_index,
            world_grid_axis_count,
            overlap,
        );

        if let Some((overlap_neighbor, ordering)) =
            world_grid_cell_axis_index.overlap_neighbor(world_grid_axis_count, overlap)
        {
            match ordering {
                AxisOrdering::Less => CellAxisIndexToGridCellAxisIndexes::Overlap {
                    start: overlap_neighbor,
                    end: world_grid_cell_axis_index,
                },
                AxisOrdering::Greater => CellAxisIndexToGridCellAxisIndexes::Overlap {
                    start: world_grid_cell_axis_index,
                    end: overlap_neighbor,
                },
            }
        } else {
            CellAxisIndexToGridCellAxisIndexes::Unambiguous(world_grid_cell_axis_index)
        }
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum WorldCellPositionToGridCellPositions<Base: SudokuBase> {
    Unambiguous(WorldGridCellPosition<Base>),
    OverlapHorizontal {
        top: WorldGridCellPosition<Base>,
        bottom: WorldGridCellPosition<Base>,
    },
    OverlapVertical {
        left: WorldGridCellPosition<Base>,
        right: WorldGridCellPosition<Base>,
    },
    OverlapCorner {
        top_left: WorldGridCellPosition<Base>,
        top_right: WorldGridCellPosition<Base>,
        bottom_left: WorldGridCellPosition<Base>,
        bottom_right: WorldGridCellPosition<Base>,
    },
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CellAxisIndexToGridCellAxisIndexes<Base: SudokuBase> {
    Unambiguous(WorldGridCellAxisIndex<Base>),
    Overlap {
        start: WorldGridCellAxisIndex<Base>,
        end: WorldGridCellAxisIndex<Base>,
    },
}

impl<Base: SudokuBase> CellAxisIndexToGridCellAxisIndexes<Base> {}

impl<Base: SudokuBase> Add<Position<Base>> for WorldCellPosition {
    type Output = Self;

    fn add(self, cell_pos: Position<Base>) -> Self::Output {
        let Self { row, column, .. } = self;

        Self::new(
            row + cell_pos.to_row().get_usize(),
            column + cell_pos.to_column().get_usize(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::base::consts::*;

    use super::*;

    #[test]
    fn test_to_nearest_world_grid_cell_position() {
        type Base = Base3;

        let overlap = GridOverlap::<Base>::new(3).unwrap();
        let grid_dim = WorldGridDim::new(2, 2).unwrap();
        let cell_dim = grid_dim.to_cell_dim(overlap);

        let tie_break_independent_test_cases = vec![
            // World top left: no neighboring grids
            ((0, 0), ((0, 0), (0, 0))),
            // Top left grid, inside overlap, but not on the edge
            ((1, 1), ((0, 0), (1, 1))),
            // Top left grid, outside overlap
            ((2, 2), ((0, 0), (2, 2))),
            // Grid cross intersection, top left overlap
            ((6, 6), ((0, 0), (6, 6))),
            // Grid cross intersection, top right overlap
            ((6, 8), ((0, 1), (6, 2))),
            // Grid cross intersection, bottom left overlap
            ((8, 6), ((1, 0), (2, 6))),
            // Grid cross intersection, bottom left overlap
            ((8, 8), ((1, 1), (2, 2))),
            // World bottom right: no neighboring grids
            ((14, 14), ((1, 1), (8, 8))),
        ]
        .into_iter()
        .flat_map(
            |(world_cell_pos, (expected_world_grid_position, expected_position))| {
                Quadrant::all().map(move |tie_break| {
                    (
                        (world_cell_pos, tie_break),
                        (expected_world_grid_position, expected_position),
                    )
                })
            },
        );

        let test_cases = vec![
            // World top middle: horziontal neighboring grids
            (((0, 7), Quadrant::TopLeft), ((0, 0), (0, 7))),
            (((0, 7), Quadrant::TopRight), ((0, 1), (0, 1))),
            (((0, 7), Quadrant::BottomLeft), ((0, 0), (0, 7))),
            (((0, 7), Quadrant::BottomRight), ((0, 1), (0, 1))),
            // World left middle: horziontal neighboring grids
            (((7, 0), Quadrant::TopLeft), ((0, 0), (7, 0))),
            (((7, 0), Quadrant::TopRight), ((0, 0), (7, 0))),
            (((7, 0), Quadrant::BottomLeft), ((1, 0), (1, 0))),
            (((7, 0), Quadrant::BottomRight), ((1, 0), (1, 0))),
            // Grid cross intersection
            (((7, 7), Quadrant::TopLeft), ((0, 0), (7, 7))),
            (((7, 7), Quadrant::TopRight), ((0, 1), (7, 1))),
            (((7, 7), Quadrant::BottomLeft), ((1, 0), (1, 7))),
            (((7, 7), Quadrant::BottomRight), ((1, 1), (1, 1))),
        ]
        .into_iter()
        .chain(tie_break_independent_test_cases)
        .map(|(input, expected)| {
            let (world_cell_pos, tie_break) = input;
            let (expected_world_grid_position, expected_position) = expected;

            let world_cell_pos = WorldCellPosition::from(world_cell_pos);
            assert!(world_cell_pos.contained_in(cell_dim));

            let expected_world_grid_cell_position = WorldGridCellPosition::from((
                WorldGridPosition::from(expected_world_grid_position),
                Position::try_from(expected_position).unwrap(),
            ));
            assert!(expected_world_grid_cell_position
                .to_world_cell_pos(overlap)
                .contained_in(cell_dim));

            (
                (world_cell_pos, tie_break),
                expected_world_grid_cell_position,
            )
        });

        for ((world_cell_pos, tie_break), expected_world_grid_cell_position) in test_cases {
            let nearest_world_grid_cell_position =
                world_cell_pos.to_nearest_world_grid_cell_position(grid_dim, overlap, tie_break);
            assert_eq!(
                nearest_world_grid_cell_position, expected_world_grid_cell_position,
                "{nearest_world_grid_cell_position} != {expected_world_grid_cell_position}",
            );
        }
    }
}
