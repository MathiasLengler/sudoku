use ndarray::Array2;
use ndarray::Dim;

use crate::error::Result;
use crate::world::CellMarker;

use super::WorldDim;

pub type WorldCellDim = WorldDim<CellMarker>;
impl WorldCellDim {
    pub fn cell_count(self) -> usize {
        self.object_count()
    }

    pub(in crate::world) fn as_cells_shape(
        self,
    ) -> impl ndarray::IntoDimension<Dim = Dim<[usize; 2]>> {
        [self.row_count.get(), self.column_count.get()]
    }

    pub(in crate::world) fn from_cells<TCell>(cells: &Array2<TCell>) -> Result<Self> {
        Self::new(cells.nrows(), cells.ncols())
    }
}
