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
        grid_axis_index * Self::stride_usize::<Base>(overlap)
    }

    /// The cell distance between the start of grids in the world.
    pub(crate) fn stride<Base: SudokuBase>(overlap: u8) -> u8 {
        debug_assert!(
            overlap <= Base::BASE,
            "overlap {overlap} must be less than or equal to the base {}",
            Base::BASE
        );

        let grid_stride = Base::SIDE_LENGTH - overlap;
        debug_assert!(grid_stride > 0, "grid_stride must be positive");

        grid_stride
    }

    pub(crate) fn stride_usize<Base: SudokuBase>(overlap: u8) -> usize {
        usize::from(Self::stride::<Base>(overlap))
    }
}
