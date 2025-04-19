use crate::{
    base::SudokuBase,
    world::{GridMarker, GridOverlap, WorldCellPosition},
};

use super::WorldPosition;

pub type WorldGridPosition = WorldPosition<GridMarker>;

impl WorldGridPosition {
    pub fn to_top_left_cell_position<Base: SudokuBase>(
        self,
        overlap: GridOverlap<Base>,
    ) -> WorldCellPosition {
        let Self { row, column, .. } = self;
        WorldCellPosition::new(
            Self::grid_axis_index_to_first_cell_axis_index::<Base>(row, overlap),
            Self::grid_axis_index_to_first_cell_axis_index::<Base>(column, overlap),
        )
    }

    fn grid_axis_index_to_first_cell_axis_index<Base: SudokuBase>(
        grid_axis_index: usize,
        overlap: GridOverlap<Base>,
    ) -> usize {
        grid_axis_index * overlap.grid_stride_usize()
    }
}
