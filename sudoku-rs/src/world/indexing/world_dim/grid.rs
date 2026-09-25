use std::num::NonZeroU32;

use crate::{
    base::SudokuBase,
    world::{GridMarker, GridOverlap},
};

use super::{WorldCellDim, WorldDim};

pub type WorldGridDim = WorldDim<GridMarker>;

impl WorldGridDim {
    pub fn grid_count(self) -> u32 {
        self.object_count()
    }

    pub fn to_cell_dim<Base: SudokuBase>(self, overlap: GridOverlap<Base>) -> WorldCellDim {
        let Self {
            row_count,
            column_count,
            ..
        } = self;
        WorldCellDim::new(
            Self::grid_axis_count_to_cell_axis_count::<Base>(row_count, overlap),
            Self::grid_axis_count_to_cell_axis_count::<Base>(column_count, overlap),
        )
        .unwrap()
    }

    fn grid_axis_count_to_cell_axis_count<Base: SudokuBase>(
        grid_axis_count: NonZeroU32,
        overlap: GridOverlap<Base>,
    ) -> u32 {
        let grid_axis_count = grid_axis_count.get();
        grid_axis_count * u32::from(Base::SIDE_LENGTH)
            - (grid_axis_count - 1) * u32::from(overlap.get())
    }
}
