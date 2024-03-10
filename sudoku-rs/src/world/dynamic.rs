use enum_dispatch::enum_dispatch;

use crate::base::{consts::*, BaseEnum};
use crate::error::Result;

use super::{CellWorld, TileDim, WorldGenerationResult};

// TODO: move more applicable methods of CellWorld to the trait
#[enum_dispatch]
pub trait DynamicCellWorldActions {
    fn generate(&mut self, seed: Option<u64>) -> WorldGenerationResult;
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
    pub fn new(base: u8, tile_dim: TileDim, overlap: u8) -> Result<Self> {
        let base: BaseEnum = base.try_into()?;

        Ok(match base {
            BaseEnum::Base2 => Self::Base2(CellWorld::<Base2>::new(tile_dim, overlap)),
            BaseEnum::Base3 => Self::Base3(CellWorld::<Base3>::new(tile_dim, overlap)),
            BaseEnum::Base4 => Self::Base4(CellWorld::<Base4>::new(tile_dim, overlap)),
            BaseEnum::Base5 => Self::Base5(CellWorld::<Base5>::new(tile_dim, overlap)),
        })
    }
}
