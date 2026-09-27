use crate::{
    base::SudokuBase,
    world::{GridMarker, GridOverlap, WorldCellPosition},
};

use super::WorldPosition;

// Crosses wasm signatures as `sudoku_wasm::generic_instances::WasmWorldGridPosition` (tsify#76).
#[cfg_attr(feature = "wasm", tsify::declare)]
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
        grid_axis_index: u32,
        overlap: GridOverlap<Base>,
    ) -> u32 {
        grid_axis_index * overlap.grid_stride_u32()
    }
}
