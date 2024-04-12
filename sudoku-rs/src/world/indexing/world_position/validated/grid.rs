use ndarray::{s, Dim, SliceInfo, SliceInfoElem};

use crate::{
    base::SudokuBase,
    world::{GridMarker, GridOverlap, WorldCellPosition},
};

use super::ValidatedWorldPosition;

pub(in crate::world) type ValidatedWorldGridPosition = ValidatedWorldPosition<GridMarker>;

pub(in crate::world) type GridCellsSliceInfo =
    SliceInfo<[SliceInfoElem; 2], Dim<[usize; 2]>, Dim<[usize; 2]>>;

impl ValidatedWorldGridPosition {
    pub(in crate::world) fn grid_cells_slice_info<Base: SudokuBase>(
        self,
        overlap: GridOverlap<Base>,
    ) -> GridCellsSliceInfo {
        let WorldCellPosition {
            row: top_left_cell_row,
            column: top_left_cell_col,
            ..
        } = self.get().to_top_left_cell_position::<Base>(overlap);

        let side_length_usize = usize::from(Base::SIDE_LENGTH);

        s![
            top_left_cell_row..(top_left_cell_row + side_length_usize),
            top_left_cell_col..(top_left_cell_col + side_length_usize),
        ]
    }
}
