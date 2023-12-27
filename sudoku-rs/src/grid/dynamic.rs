use serde::{Deserialize, Serialize};
#[cfg(feature = "wasm")]
use ts_rs::TS;

use crate::base::SudokuBase;
use crate::cell::dynamic::DynamicCell;
use crate::error::{Error, Result};
use crate::grid::Grid;

#[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DynamicGrid {
    cells: Vec<DynamicCell>,
}

impl<Base: SudokuBase> TryFrom<DynamicGrid> for Grid<Base> {
    type Error = Error;

    fn try_from(dynamic_grid: DynamicGrid) -> Result<Self> {
        dynamic_grid.cells.try_into()
    }
}
