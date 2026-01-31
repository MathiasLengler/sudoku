use crate::base::{BaseEnum, consts::*};
use crate::cell::dynamic::DynamicCell;
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;
use enum_dispatch::enum_dispatch;
use ndarray::Array2;
use serde::{Deserialize, Serialize};

use super::{
    CellWorld, CellWorldDimensions, DynamicWorldGridCellPosition, Quadrant, WorldCellPosition,
    WorldGenerationResult, WorldGridDim, WorldGridPosition, WorldPruningSettings,
};

#[enum_dispatch]
pub trait DynamicCellWorldActions {
    // Generation
    fn generate_solved(&mut self, seed: Option<u64>) -> Result<WorldGenerationResult>;
    fn prune(&mut self, seed: Option<u64>) -> Result<()>;
    fn prune_with_settings(
        &mut self,
        seed: Option<u64>,
        settings: WorldPruningSettings,
    ) -> Result<()>;

    // DynamicGrid interop
    fn to_grid_at(&self, grid_position: WorldGridPosition) -> Result<DynamicGrid<DynamicCell>>;
    fn set_grid_at(
        &mut self,
        grid: DynamicGrid<DynamicCell>,
        grid_position: WorldGridPosition,
    ) -> Result<()>;

    // Queries
    fn dimensions(&self) -> CellWorldDimensions;
    fn is_solved(&self) -> bool;
    fn is_directly_consistent(&self) -> bool;
    fn all_world_cells(&self) -> Vec<DynamicCell>;

    // Indexing helpers
    fn world_cell_position_to_nearest_world_grid_cell_position(
        &self,
        cell_position: WorldCellPosition,
        tie_break: Quadrant,
    ) -> Result<DynamicWorldGridCellPosition>;
}

#[enum_dispatch(DynamicCellWorldActions)]
#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum DynamicCellWorld {
    Base2(CellWorld<Base2>),
    Base3(CellWorld<Base3>),
    Base4(CellWorld<Base4>),
    Base5(CellWorld<Base5>),
}

macro_rules! new_dynamic_cell_world_from_base_enum {
    ($base_enum_value:expr, $new_cell_world:expr) => {{
        use $crate::base::consts::*;
        match $base_enum_value {
            BaseEnum::Base2 => {
                type Base = Base2;
                Self::Base2($new_cell_world)
            }
            BaseEnum::Base3 => {
                type Base = Base3;
                Self::Base3($new_cell_world)
            }
            BaseEnum::Base4 => {
                type Base = Base4;
                Self::Base4($new_cell_world)
            }
            BaseEnum::Base5 => {
                type Base = Base5;
                Self::Base5($new_cell_world)
            }
        }
    }};
}

impl DynamicCellWorld {
    pub fn new(base: BaseEnum, grid_dim: WorldGridDim, overlap: u8) -> Result<Self> {
        Ok(new_dynamic_cell_world_from_base_enum!(
            base,
            CellWorld::<Base>::new(grid_dim, overlap.try_into()?)
        ))
    }

    pub fn with(
        base: BaseEnum,
        grid_dim: WorldGridDim,
        overlap: u8,
        cells: Vec<DynamicCell>,
    ) -> Result<Self> {
        Ok(new_dynamic_cell_world_from_base_enum!(base, {
            let overlap = overlap.try_into()?;
            let cells_shape = grid_dim.to_cell_dim::<Base>(overlap).as_cells_shape();
            let cells = cells
                .into_iter()
                .map(|dynamic_cell| dynamic_cell.try_into())
                .collect::<Result<Vec<_>>>()?;
            CellWorld::<Base>::with(
                grid_dim,
                overlap,
                Array2::from_shape_vec(cells_shape, cells)?,
            )?
        }))
    }

    pub fn base(&self) -> BaseEnum {
        match self {
            Self::Base2(_) => BaseEnum::Base2,
            Self::Base3(_) => BaseEnum::Base3,
            Self::Base4(_) => BaseEnum::Base4,
            Self::Base5(_) => BaseEnum::Base5,
        }
    }
}
