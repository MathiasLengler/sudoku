use crate::error::Result;
use crate::generic_instances::{
    WasmDynamicCells, WasmWorldCellPosition, WasmWorldGridDim, WasmWorldGridPosition,
};
use crate::serialized::{self, ISerializedDynamicCellWorld};
use crate::typescript::*;
use sudoku::base::BaseEnum;
use sudoku::base::consts::*;
use sudoku::grid::dynamic::DynamicGrid;
use sudoku::world::dynamic::{DynamicCellWorld, DynamicCellWorldActions};
use sudoku::world::{
    CellWorld, CellWorldDimensions, DynamicWorldGridCellPosition, Quadrant, WorldGenerationResult,
    WorldGridDim,
};
use tsify::Ts;
use wasm_bindgen::prelude::*;

#[allow(dead_code)]
#[wasm_bindgen]
pub struct WasmCellWorld {
    world: DynamicCellWorld,
}

impl Default for WasmCellWorld {
    fn default() -> Self {
        let world =
            CellWorld::<Base3>::new(WorldGridDim::new(3, 3).unwrap(), 1.try_into().unwrap());

        DynamicCellWorld::from(world).into()
    }
}

impl From<DynamicCellWorld> for WasmCellWorld {
    fn from(world: DynamicCellWorld) -> Self {
        Self { world }
    }
}

/// Constructors
#[wasm_bindgen]
impl WasmCellWorld {
    pub fn new(base: Ts<BaseEnum>, grid_dim: Ts<WasmWorldGridDim>, overlap: u8) -> Result<Self> {
        Ok(DynamicCellWorld::new(base.to_rust()?, grid_dim.to_rust()?.0, overlap)?.into())
    }

    pub fn with(
        base: Ts<BaseEnum>,
        grid_dim: Ts<WasmWorldGridDim>,
        overlap: u8,
        cells: Ts<WasmDynamicCells>,
    ) -> Result<Self> {
        Ok(DynamicCellWorld::with(
            base.to_rust()?,
            grid_dim.to_rust()?.0,
            overlap,
            cells.to_rust()?.0,
        )?
        .into())
    }

    #[wasm_bindgen(constructor)]
    pub fn default() -> Self {
        Default::default()
    }

    pub fn generate(
        base: Ts<BaseEnum>,
        grid_dim: Ts<WasmWorldGridDim>,
        overlap: u8,
        seed: Option<u64>,
    ) -> Result<Self> {
        let mut this = Self::new(base, grid_dim, overlap)?;
        this.generate_solved(seed)?;
        this.prune(seed)?;
        Ok(this)
    }
}

#[wasm_bindgen]
impl WasmCellWorld {
    pub fn equals(&self, other: &Self) -> bool {
        self.world == other.world
    }

    #[wasm_bindgen(js_name = generateSolved)]
    pub fn generate_solved(&mut self, seed: Option<u64>) -> Result<Ts<WorldGenerationResult>> {
        export_ts(&self.world.generate_solved(seed)?)
    }
    pub fn prune(&mut self, seed: Option<u64>) -> Result<()> {
        Ok(self.world.prune(seed)?)
    }

    // DynamicGrid interop
    #[wasm_bindgen(js_name = toGridAt)]
    pub fn to_grid_at(&self, grid_position: Ts<WasmWorldGridPosition>) -> Result<Ts<DynamicGrid>> {
        export_ts(&self.world.to_grid_at(grid_position.to_rust()?.0)?)
    }
    #[wasm_bindgen(js_name = setGridAt)]
    pub fn set_grid_at(
        &mut self,
        grid: Ts<DynamicGrid>,
        grid_position: Ts<WasmWorldGridPosition>,
    ) -> Result<()> {
        self.world
            .set_grid_at(grid.to_rust()?, grid_position.to_rust()?.0)?;
        Ok(())
    }

    // Queries
    pub fn base(&self) -> Result<Ts<BaseEnum>> {
        export_ts(&self.world.base())
    }
    pub fn dimensions(&self) -> Result<Ts<CellWorldDimensions>> {
        export_ts(&self.world.dimensions())
    }
    #[wasm_bindgen(js_name = isSolved)]
    pub fn is_solved(&self) -> bool {
        self.world.is_solved()
    }
    #[wasm_bindgen(js_name = isDirectlyConsistent)]
    pub fn is_directly_consistent(&self) -> bool {
        self.world.is_directly_consistent()
    }

    #[wasm_bindgen(js_name = allWorldCells)]
    pub fn all_world_cells(&self) -> Result<Ts<WasmDynamicCells>> {
        export_ts(&WasmDynamicCells(self.world.all_world_cells()))
    }
    // Indexing helpers
    #[wasm_bindgen(js_name = worldCellPositionToNearestWorldGridCellPosition)]
    pub fn world_cell_position_to_nearest_world_grid_cell_position(
        &self,
        cell_position: Ts<WasmWorldCellPosition>,
        tie_break: Ts<Quadrant>,
    ) -> Result<Ts<DynamicWorldGridCellPosition>> {
        export_ts(
            &self
                .world
                .world_cell_position_to_nearest_world_grid_cell_position(
                    cell_position.to_rust()?.0,
                    tie_break.to_rust()?,
                )?,
        )
    }
}

/// (De)serialization
#[wasm_bindgen]
impl WasmCellWorld {
    pub fn serialize(&self) -> Result<ISerializedDynamicCellWorld> {
        serialized::serialize(&self.world)
    }

    pub fn deserialize(bytes: &ISerializedDynamicCellWorld) -> Result<Self> {
        Ok(serialized::deserialize::<DynamicCellWorld>(bytes)?.into())
    }
}
