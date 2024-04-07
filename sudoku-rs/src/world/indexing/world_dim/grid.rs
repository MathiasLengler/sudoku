use std::num::NonZeroUsize;

use crate::{base::SudokuBase, world::GridMarker};

use super::{WorldCellDim, WorldDim};

pub type WorldGridDim = WorldDim<GridMarker>;

impl WorldGridDim {
    pub fn grid_count(self) -> usize {
        self.object_count()
    }

    pub fn to_cell_dim<Base: SudokuBase>(self, overlap: u8) -> WorldCellDim {
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
        grid_axis_count: NonZeroUsize,
        overlap: u8,
    ) -> usize {
        let grid_axis_count = grid_axis_count.get();
        grid_axis_count * usize::from(Base::SIDE_LENGTH)
            - (grid_axis_count - 1) * usize::from(overlap)
    }
}
