use log::trace;
use sudoku::world::dynamic::{DynamicCellWorld, DynamicCellWorldActions};
use wasm_bindgen::prelude::*;

use error::Result;
use sudoku::base::consts::*;
use sudoku::grid::Grid;
use sudoku::transport::TransportSudoku;
use sudoku::world::{CellWorld, WorldGridDim};
use sudoku::{DynamicSudoku, DynamicSudokuActions, Sudoku};

use crate::typescript::*;

#[cfg(target_family = "wasm")]
pub use wasm_bindgen_rayon::init_thread_pool;

mod typescript;

mod error;

/*
TODO: design API for sudoku world

Requirements:
- Change active grid
- Generate world with settings
    - Settings regions?
- View state of the world
- Play single vs world
    - Different entry points?
    - Game states?
*/

#[wasm_bindgen]
pub fn init() {
    #[cfg(feature = "console")]
    {
        use log::Level;
        use std::panic;
        use std::sync::Once;

        static SET_HOOK: Once = Once::new();
        SET_HOOK.call_once(|| {
            panic::set_hook(Box::new(console_error_panic_hook::hook));
            console_log::init_with_level(Level::Info).unwrap();
        });
    }

    trace!("sudoku-wasm initialized");
}

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
    pub fn new(base: IBaseEnum, grid_dim: IWorldGridDim, overlap: u8) -> Result<WasmCellWorld> {
        Ok(DynamicCellWorld::new(
            import_base_enum(base)?,
            import_world_grid_dim(grid_dim)?,
            overlap,
        )
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
    ) -> Result<WasmCellWorld> {
        let mut this = Self::new(base, grid_dim, overlap)?;
        this.generate_solved(seed).unwrap();
        this.prune(seed).unwrap();
        Ok(this)
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
    #[wasm_bindgen(js_name = allWorldCells)]
    pub fn all_world_cells(&self) -> Result<IDynamicCells> {
        export_dynamic_cells(self.world.all_world_cells())
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

#[wasm_bindgen]
pub struct WasmSudoku {
    sudoku: DynamicSudoku,
}

impl Default for WasmSudoku {
    fn default() -> Self {
        let grid: Grid<Base3> = sudoku::samples::minimal();

        DynamicSudoku::from(Sudoku::with_grid(grid)).into()
    }
}

impl From<DynamicSudoku> for WasmSudoku {
    fn from(sudoku: DynamicSudoku) -> Self {
        Self { sudoku }
    }
}

#[wasm_bindgen]
impl WasmSudoku {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_dynamic_cells(cells: IDynamicCells) -> Result<WasmSudoku> {
        let cells = import_dynamic_cells(cells)?;

        Ok(DynamicSudoku::try_from(cells)?.into())
    }

    pub fn from_dynamic_grid(dynamic_grid: IDynamicGrid) -> Result<WasmSudoku> {
        let dynamic_grid = import_dynamic_grid(dynamic_grid)?;

        Ok(DynamicSudoku::try_from(dynamic_grid)?.into())
    }

    #[wasm_bindgen(js_name = getTransportSudoku)]
    pub fn get_transport_sudoku(&self) -> Result<ITransportSudoku> {
        let transport_sudoku = TransportSudoku::from(&self.sudoku);

        export_transport_sudoku(transport_sudoku)
    }

    #[wasm_bindgen(js_name = toDynamicGrid)]
    pub fn to_dynamic_grid(&self) -> Result<IDynamicGrid> {
        export_dynamic_grid(self.sudoku.to_dynamic_grid())
    }

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&mut self, pos: IDynamicPosition, value: IDynamicValue) -> Result<()> {
        self.sudoku
            .set_value(import_dynamic_position(pos)?, import_dynamic_value(value)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setOrToggleValue)]
    pub fn set_or_toggle_value(
        &mut self,
        pos: IDynamicPosition,
        value: IDynamicValue,
    ) -> Result<()> {
        self.sudoku
            .set_or_toggle_value(import_dynamic_position(pos)?, import_dynamic_value(value)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setCandidates)]
    pub fn set_candidates(
        &mut self,
        pos: IDynamicPosition,
        candidates: IDynamicCandidates,
    ) -> Result<()> {
        self.sudoku.set_candidates(
            import_dynamic_position(pos)?,
            import_dynamic_candidates(candidates)?,
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = toggleCandidate)]
    pub fn toggle_candidate(
        &mut self,
        pos: IDynamicPosition,
        candidate: IDynamicValue,
    ) -> Result<()> {
        self.sudoku.toggle_candidate(
            import_dynamic_position(pos)?,
            import_dynamic_value(candidate)?,
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setCandidate)]
    pub fn set_candidate(&mut self, pos: IDynamicPosition, candidate: IDynamicValue) -> Result<()> {
        self.sudoku.set_candidate(
            import_dynamic_position(pos)?,
            import_dynamic_value(candidate)?,
        )?;
        Ok(())
    }

    #[wasm_bindgen(js_name = deleteCandidate)]
    pub fn delete_candidate(
        &mut self,
        pos: IDynamicPosition,
        candidate: IDynamicValue,
    ) -> Result<()> {
        self.sudoku.delete_candidate(
            import_dynamic_position(pos)?,
            import_dynamic_value(candidate)?,
        )?;
        Ok(())
    }

    pub fn delete(&mut self, pos: IDynamicPosition) -> Result<()> {
        self.sudoku.delete(import_dynamic_position(pos)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = setAllDirectCandidates)]
    pub fn set_all_direct_candidates(&mut self) {
        self.sudoku.set_all_direct_candidates();
    }

    pub fn undo(&mut self) {
        self.sudoku.undo();
    }

    pub fn redo(&mut self) {
        self.sudoku.redo();
    }

    pub fn generate(
        &mut self,
        generator_settings: IDynamicGeneratorSettings,
        on_progress: IGenerateOnProgress,
    ) -> Result<()> {
        self.sudoku.generate(
            import_dynamic_generator_settings(generator_settings)?,
            import_generate_on_progress(on_progress)?,
        )?;

        Ok(())
    }

    pub fn generate_multi_shot(
        &mut self,
        multi_shot_generator_settings: IDynamicMultiShotGeneratorSettings,
        on_progress: IGenerateMultiShotOnProgress,
    ) -> Result<()> {
        self.sudoku.generate_multi_shot(
            import_dynamic_multi_shot_generator_settings(multi_shot_generator_settings)?,
            import_generate_multi_shot_on_progress(on_progress)?,
        )?;

        Ok(())
    }

    pub fn import(&mut self, input: &str) -> Result<()> {
        self.sudoku.import(input)?;
        Ok(())
    }

    pub fn export(&self, format: IGridFormatEnum) -> Result<String> {
        Ok(self.sudoku.export(import_grid_format_enum(format)?))
    }

    #[wasm_bindgen(js_name = tryStrategies)]
    pub fn try_strategies(
        &mut self,
        strategies: IStrategyEnums,
    ) -> Result<IDynamicTryStrategiesReturnAlias> {
        let dynamic_try_strategies_return = self
            .sudoku
            .try_strategies(import_strategy_enums(strategies)?)?;

        export_dynamic_try_strategies_return_alias(dynamic_try_strategies_return)
    }

    #[wasm_bindgen(js_name = applyDeductions)]
    pub fn apply_deductions(&mut self, deductions: ITransportDeductions) -> Result<()> {
        self.sudoku
            .apply_deductions(import_transport_deductions(deductions)?)?;
        Ok(())
    }
}
