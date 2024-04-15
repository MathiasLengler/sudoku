use serde::{Deserialize, Serialize};

use crate::{base::SudokuBase, position::DynamicPosition, world::WorldGridPosition};

use super::WorldGridCellPosition;

#[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export), ts(concrete(T = crate::world::CellMarker)))]
#[derive(Debug, Default, Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Serialize, Deserialize)]
pub struct DynamicWorldGridCellPosition {
    #[ts(type = "import('../../sudoku-web/src/app/state/world').WorldGridPosition")]
    world_grid_pos: WorldGridPosition,
    cell_pos: DynamicPosition,
}

impl<Base: SudokuBase> From<WorldGridCellPosition<Base>> for DynamicWorldGridCellPosition {
    fn from(world_grid_cell_position: WorldGridCellPosition<Base>) -> Self {
        Self {
            world_grid_pos: world_grid_cell_position.world_grid_pos(),
            cell_pos: world_grid_cell_position.cell_pos().into(),
        }
    }
}
