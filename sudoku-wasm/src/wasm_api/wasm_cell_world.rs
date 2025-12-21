use crate::error::Result;
use crate::typescript::*;
use sudoku::base::consts::*;
use sudoku::error::Error as SudokuError;
use sudoku::world::dynamic::{DynamicCellWorld, DynamicCellWorldActions};
use sudoku::world::{CellWorld, WorldGridDim};
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

#[wasm_bindgen]
impl WasmCellWorld {
    pub fn new(base: IBaseEnum, grid_dim: IWorldGridDim, overlap: u8) -> Result<Self> {
        Ok(DynamicCellWorld::new(
            import_base_enum(base)?,
            import_world_grid_dim(grid_dim)?,
            overlap,
        )?
        .into())
    }

    pub fn with(
        base: IBaseEnum,
        grid_dim: IWorldGridDim,
        overlap: u8,
        cells: IDynamicCells,
    ) -> Result<Self> {
        Ok(DynamicCellWorld::with(
            import_base_enum(base)?,
            import_world_grid_dim(grid_dim)?,
            overlap,
            import_dynamic_cells(cells)?,
        )?
        .into())
    }

    #[allow(clippy::should_implement_trait)]
    pub fn default() -> Self {
        Default::default()
    }

    pub fn generate(
        base: IBaseEnum,
        grid_dim: IWorldGridDim,
        overlap: u8,
        seed: Option<u64>,
    ) -> Result<Self> {
        let mut this = Self::new(base, grid_dim, overlap)?;
        this.generate_solved(seed)?;
        this.prune(seed)?;
        Ok(this)
    }

    pub fn equals(&self, other: &WasmCellWorld) -> bool {
        self.world == other.world
    }

    #[wasm_bindgen(js_name = generateSolved)]
    pub fn generate_solved(&mut self, seed: Option<u64>) -> Result<IWorldGenerationResult> {
        export_world_generation_result(self.world.generate_solved(seed)?)
    }
    pub fn prune(&mut self, seed: Option<u64>) -> Result<()> {
        Ok(self.world.prune(seed)?)
    }

    // DynamicGrid interop
    #[wasm_bindgen(js_name = toGridAt)]
    pub fn to_grid_at(&self, grid_position: IWorldGridPosition) -> Result<IDynamicGrid> {
        export_dynamic_grid(
            self.world
                .to_grid_at(import_world_grid_position(grid_position)?)?,
        )
    }
    #[wasm_bindgen(js_name = setGridAt)]
    pub fn set_grid_at(
        &mut self,
        grid: IDynamicGrid,
        grid_position: IWorldGridPosition,
    ) -> Result<()> {
        self.world.set_grid_at(
            import_dynamic_grid(grid)?,
            import_world_grid_position(grid_position)?,
        )?;
        Ok(())
    }

    // Queries
    pub fn base(&self) -> Result<IBaseEnum> {
        export_base_enum(self.world.base())
    }
    pub fn dimensions(&self) -> Result<ICellWorldDimensions> {
        export_cell_world_dimensions(self.world.dimensions())
    }
    #[wasm_bindgen(js_name = isSolved)]
    pub fn is_solved(&self) -> bool {
        self.world.is_solved()
    }
    #[wasm_bindgen(js_name = isDirectlyConsistent)]
    pub fn is_directly_consistent(&self) -> bool {
        self.world.is_directly_consistent()
    }
    // TODO: evaluate non-DynamicCell solution
    //  Serialize `DynamicCellWorld` directly

    #[wasm_bindgen(js_name = allWorldCells)]
    pub fn all_world_cells(&self) -> Result<IDynamicCells> {
        export_dynamic_cells(self.world.all_world_cells())
    }
    #[wasm_bindgen(js_name = allWorldCellsPostcardDynamicCellVec)]
    pub fn all_world_cells_postcard_vec(&self) -> Result<Vec<u8>> {
        let all_world_cells = self.world.all_world_cells();
        Ok(postcard::to_stdvec(&all_world_cells).map_err(SudokuError::from)?)
    }
    #[wasm_bindgen(js_name = allWorldCellsPostcardDynamicCellBoxedSlice)]
    pub fn all_world_cells_postcard_boxed_slice(&self) -> Result<Box<[u8]>> {
        let all_world_cells = self.world.all_world_cells();

        Ok(postcard::to_stdvec(&all_world_cells)
            .map_err(SudokuError::from)?
            .into_boxed_slice())
    }

    // TODO: evaluate branded types for serialized data
    //  once we have multiple postcard serializations in use,
    //  we could confuse the `Uint8Array`s in TS.
    //  This is not *unsafe*, but would lead to unclear deserialization errors.
    // TODO: decide on Box<[u8]> vs Vec<u8>
    //  No perf difference in benchmark.
    pub fn serialize(&self) -> Result<Vec<u8>> {
        Ok(postcard::to_stdvec(&self.world).map_err(SudokuError::from)?)
    }

    pub fn deserialize(&self, bytes: &[u8]) -> Result<Self> {
        let world: DynamicCellWorld = postcard::from_bytes(bytes).map_err(SudokuError::from)?;
        Ok(Self { world })
    }

    // Indexing helpers
    #[wasm_bindgen(js_name = worldCellPositionToNearestWorldGridCellPosition)]
    pub fn world_cell_position_to_nearest_world_grid_cell_position(
        &self,
        cell_position: IWorldCellPosition,
        tie_break: IQuadrant,
    ) -> Result<IDynamicWorldGridCellPosition> {
        export_dynamic_world_grid_cell_position(
            self.world
                .world_cell_position_to_nearest_world_grid_cell_position(
                    import_world_cell_position(cell_position)?,
                    import_quadrant(tie_break)?,
                )?,
        )
    }
}
