use std::{num::NonZeroUsize, ops::Add};

use crate::{
    base::SudokuBase,
    position::Position,
    world::{
        AxisOrdering, CellMarker, Quadrant, WorldGridCellPosition, WorldGridDim, WorldGridPosition,
    },
};

use super::{WorldGridCellAxisIndex, WorldPosition};

pub type WorldCellPosition = WorldPosition<CellMarker>;

impl WorldCellPosition {
    // TODO: nearest_world_grid_position using normalize_to_nearest_world_grid_axis_index
    //  this should simplify the implementation: less return enums,
    //  since tie-breakers are handled on an axis level already

    pub fn to_grid_cell_positions<Base: SudokuBase>(
        self,
        grid_dim: WorldGridDim,
        overlap: u8,
    ) -> WorldCellPositionToGridCellPositions<Base> {
        // TODO: implement

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
        overlap: u8,
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

impl<Base: SudokuBase> WorldCellPositionToGridCellPositions<Base> {
    // TODO: delete

    // TODO: nearest grid position
    //  use case: world map click on cell: which grid to select
    //  requires clicked cell quadrant for odd overlaps as a tie-breaker
    pub fn nearest_grid_position(self, cell_quadrant: Quadrant, overlap: u8) -> WorldGridPosition {
        todo!()
    }
}

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum CellAxisIndexToGridCellAxisIndexes<Base: SudokuBase> {
    Unambiguous(WorldGridCellAxisIndex<Base>),
    Overlap {
        start: WorldGridCellAxisIndex<Base>,
        end: WorldGridCellAxisIndex<Base>,
    },
}

impl<Base: SudokuBase> CellAxisIndexToGridCellAxisIndexes<Base> {
    // TODO: delete if other approch works
    fn nearest_grid_axis_index(self, prefer: AxisOrdering) -> WorldGridCellAxisIndex<Base> {
        match self {
            CellAxisIndexToGridCellAxisIndexes::Unambiguous(world_grid_cell_axis_index) => {
                world_grid_cell_axis_index
            }
            CellAxisIndexToGridCellAxisIndexes::Overlap { start, end } => {
                // FIXME: This seems correct, but isn't. only tie-break on the middle of the overlap.
                //  this is the wrong abstraction layer, can be computed by axis directly with this enum.

                match prefer {
                    AxisOrdering::Less => start,
                    AxisOrdering::Greater => end,
                }
            }
        }
    }
}

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
