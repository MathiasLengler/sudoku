use crate::{
    base::SudokuBase,
    world::{GridMarker, WorldCellPosition},
};

use super::WorldPosition;

pub type WorldGridPosition = WorldPosition<GridMarker>;

impl WorldGridPosition {
    pub fn to_top_left_cell_position<Base: SudokuBase>(self, overlap: u8) -> WorldCellPosition {
        let Self { row, column, .. } = self;
        WorldCellPosition::new(
            Self::grid_axis_index_to_first_cell_axis_index::<Base>(row, overlap),
            Self::grid_axis_index_to_first_cell_axis_index::<Base>(column, overlap),
        )
    }

    fn grid_axis_index_to_first_cell_axis_index<Base: SudokuBase>(
        grid_axis_index: usize,
        overlap: u8,
    ) -> usize {
        grid_axis_index * Self::stride::<Base>(overlap)
    }

    /// The cell distance between the start of grids in the world.
    pub fn stride<Base: SudokuBase>(overlap: u8) -> usize {
        debug_assert!(overlap <= Base::BASE);

        usize::from(Base::SIDE_LENGTH) - usize::from(overlap)
    }
}
