use log::trace;
use sudoku::world::dynamic::{DynamicCellWorld, DynamicCellWorldActions};
use wasm_bindgen::prelude::*;

use error::Result;
use sudoku::base::consts::*;
use sudoku::grid::Grid;
use sudoku::transport::TransportSudoku;
use sudoku::world::{CellWorld, TileDim, TileIndex};
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
            console_log::init_with_level(Level::Debug).unwrap();
        });
    }

    trace!("WASM initialized");
}

// TODO: continue PoC API design
#[allow(dead_code)]
#[wasm_bindgen]
pub struct WasmCellWorld {
    world: DynamicCellWorld,
    tile_index: TileIndex,
}

impl Default for WasmCellWorld {
    fn default() -> Self {
        Self::new()
    }
}

#[wasm_bindgen]
impl WasmCellWorld {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let mut world = CellWorld::<Base3>::new(
            TileDim {
                row_count: 3,
                column_count: 3,
            },
            1,
        );

        let tile_index = TileIndex::default();
        let seed = Some(1);
        world.generate(seed);
        world.prune(seed);

        Self {
            world: world.into(),
            tile_index,
        }
    }
}

#[wasm_bindgen]
pub struct WasmSudoku {
    sudoku: DynamicSudoku,
}

impl Default for WasmSudoku {
    fn default() -> Self {
        Self::new()
    }
}

impl From<DynamicSudoku> for WasmSudoku {
    fn from(sudoku: DynamicSudoku) -> Self {
        WasmSudoku { sudoku }
    }
}

#[wasm_bindgen]
impl WasmSudoku {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        let grid: Grid<Base3> = sudoku::samples::minimal();

        DynamicSudoku::from(Sudoku::with_grid(grid)).into()
    }

    pub fn restore(cells: IDynamicCells) -> Result<WasmSudoku> {
        let cells = import_dynamic_cells(cells)?;

        Ok(DynamicSudoku::try_from(cells)?.into())
    }

    #[wasm_bindgen(js_name = getSudoku)]
    pub fn get_sudoku(&self) -> Result<ITransportSudoku> {
        let transport_sudoku = TransportSudoku::from(&self.sudoku);

        export_transport_sudoku(transport_sudoku)
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

        // *self = self.sudoku.clone().into();
        Ok(())
    }

    pub fn import(&mut self, input: &str) -> Result<()> {
        self.sudoku.import(input)?;
        Ok(())
    }

    pub fn export(&self, format: IDynamicGridFormat) -> Result<String> {
        Ok(self.sudoku.export(&import_dynamic_grid_format(format)?))
    }

    // FIXME: wasm bindgen return type
    #[wasm_bindgen(js_name = tryStrategies)]
    pub fn try_strategies(
        &mut self,
        strategies: IDynamicStrategies,
    ) -> Result<IDynamicTryStrategiesReturn> {
        let dynamic_try_strategies_return = self
            .sudoku
            .try_strategies(import_dynamic_strategies(strategies)?)?;

        export_dynamic_try_strategies_return(dynamic_try_strategies_return)
    }

    #[wasm_bindgen(js_name = applyDeductions)]
    pub fn apply_deductions(&mut self, deductions: ITransportDeductions) -> Result<()> {
        self.sudoku
            .apply_deductions(import_transport_deductions(deductions)?)?;
        Ok(())
    }

    #[wasm_bindgen(js_name = changeTile)]
    pub fn change_tile(&mut self, _dir: IRelativeTileDir) -> Result<()> {
        todo!()
        //
        // let dir = import_dir(dir)?;
        //
        // let new_tile_index =
        //     self.tile_index
        //         .adjacent(dir, self.world.tile_dim())
        //         .ok_or(anyhow!(
        //             "Currently at world boundary {:?}, can't move {:?}",
        //             self.tile_index,
        //             dir
        //         ))?;
        //
        // let DynamicSudoku::Base3(sudoku_base_3) = &self.sudoku else {
        //     panic!("POC: base 3 only")
        // };
        //
        // self.world
        //     .set_grid_at(sudoku_base_3.grid(), self.tile_index);
        //
        // self.sudoku =
        //     DynamicSudoku::Base3(Sudoku::with_grid(self.world.to_grid_at(new_tile_index)));
        // self.tile_index = new_tile_index;
        //
        // Ok(())
    }
}
