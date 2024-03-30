use enum_dispatch::enum_dispatch;

use crate::base::{consts::*, BaseEnum};
use crate::cell::dynamic::DynamicCell;
use crate::error::Result;
use crate::grid::dynamic::DynamicGrid;

use super::{CellWorld, CellWorldDimensions, TileDim, TileIndex, WorldGenerationResult};

#[enum_dispatch]
pub trait DynamicCellWorldActions {
    // Generation
    fn generate_solved(&mut self, seed: Option<u64>) -> Result<WorldGenerationResult>;
    fn prune(&mut self, seed: Option<u64>) -> Result<()>;

    // DynamicGrid interop
    fn to_grid_at(&self, tile_index: TileIndex) -> Result<DynamicGrid<DynamicCell>>;
    fn set_grid_at(&mut self, grid: DynamicGrid<DynamicCell>, tile_index: TileIndex) -> Result<()>;

    // Queries
    fn dimensions(&self) -> CellWorldDimensions;
    fn is_solved(&self) -> bool;
    fn is_directly_consistent(&self) -> bool;
    fn all_world_cells(&self) -> Vec<DynamicCell>;
}

#[enum_dispatch(DynamicCellWorldActions)]
#[derive(Debug)]
pub enum DynamicCellWorld {
    Base2(CellWorld<Base2>),
    Base3(CellWorld<Base3>),
    Base4(CellWorld<Base4>),
    Base5(CellWorld<Base5>),
}

impl DynamicCellWorld {
    pub fn new(base: BaseEnum, tile_dim: TileDim, overlap: u8) -> Self {
        match base {
            BaseEnum::Base2 => Self::Base2(CellWorld::<Base2>::new(tile_dim, overlap)),
            BaseEnum::Base3 => Self::Base3(CellWorld::<Base3>::new(tile_dim, overlap)),
            BaseEnum::Base4 => Self::Base4(CellWorld::<Base4>::new(tile_dim, overlap)),
            BaseEnum::Base5 => Self::Base5(CellWorld::<Base5>::new(tile_dim, overlap)),
        }
    }
}
