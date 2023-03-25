use log::trace;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

use error::Result;
use sudoku::base::consts::*;
use sudoku::cell::view::CellView;
use sudoku::error::Error as SudokuError;
use sudoku::generator::DynamicGeneratorSettings;
use sudoku::grid::serialization::GridFormat;
use sudoku::grid::Grid;
use sudoku::position::DynamicPosition;
use sudoku::solver::strategic::strategies::DynamicStrategy;
use sudoku::transport::TransportSudoku;
use sudoku::{DynamicSudoku, Game, Sudoku};
use typescript::{ICandidates, IGridFormat, ITransportSudoku};

use crate::typescript::{ICellView, IDynamicGeneratorSettings, IDynamicStrategy, IPosition};

mod typescript;

mod error;

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
            console_log::init_with_level(Level::Trace).unwrap();
        });
    }

    trace!("WASM initialized");
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

        DynamicSudoku::with_sudoku(Sudoku::with_grid(grid))
            .unwrap()
            .into()
    }

    pub fn restore(cells: Vec<ICellView>) -> Result<WasmSudoku> {
        let cells = import_cells(cells)?;

        Ok(DynamicSudoku::try_from(cells).map_err(export_error)?.into())
    }

    #[wasm_bindgen(js_name = getSudoku)]
    pub fn get_sudoku(&self) -> Result<ITransportSudoku> {
        let transport_sudoku = TransportSudoku::from(&self.sudoku);

        export_sudoku(transport_sudoku)
    }

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&mut self, pos: IPosition, value: u8) -> Result<()> {
        self.sudoku
            .set_value(import_pos(pos)?, value)
            .map_err(export_error)
    }

    #[wasm_bindgen(js_name = setOrToggleValue)]
    pub fn set_or_toggle_value(&mut self, pos: IPosition, value: u8) -> Result<()> {
        self.sudoku
            .set_or_toggle_value(import_pos(pos)?, value)
            .map_err(export_error)
    }

    #[wasm_bindgen(js_name = setCandidates)]
    pub fn set_candidates(&mut self, pos: IPosition, candidates: ICandidates) -> Result<()> {
        self.sudoku
            .set_candidates(import_pos(pos)?, import_candidates(candidates)?)
            .map_err(export_error)
    }

    #[wasm_bindgen(js_name = toggleCandidate)]
    pub fn toggle_candidate(&mut self, pos: IPosition, candidate: u8) -> Result<()> {
        self.sudoku
            .toggle_candidate(import_pos(pos)?, candidate)
            .map_err(export_error)
    }

    #[wasm_bindgen(js_name = setCandidate)]
    pub fn set_candidate(&mut self, pos: IPosition, candidate: u8) -> Result<()> {
        self.sudoku
            .set_candidate(import_pos(pos)?, candidate)
            .map_err(export_error)
    }

    #[wasm_bindgen(js_name = deleteCandidate)]
    pub fn delete_candidate(&mut self, pos: IPosition, candidate: u8) -> Result<()> {
        self.sudoku
            .delete_candidate(import_pos(pos)?, candidate)
            .map_err(export_error)
    }

    pub fn delete(&mut self, pos: IPosition) -> Result<()> {
        self.sudoku.delete(import_pos(pos)?).map_err(export_error)
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

    pub fn generate(&mut self, generator_settings: IDynamicGeneratorSettings) -> Result<()> {
        self.sudoku
            .generate(import_dynamic_generator_settings(generator_settings)?)
            .map_err(export_error)
    }

    pub fn import(&mut self, input: &str) -> Result<()> {
        self.sudoku.import(input).map_err(export_error)
    }

    pub fn export(&self, format: IGridFormat) -> Result<String> {
        Ok(self.sudoku.export(&import_grid_format(format)?))
    }

    #[wasm_bindgen(js_name = tryStrategy)]
    pub fn try_strategy(&mut self, strategy: IDynamicStrategy) -> Result<bool> {
        self.sudoku
            .try_strategy(import_strategy(strategy)?)
            .map_err(export_error)
    }
}

// Import helpers
fn import_pos(pos: IPosition) -> Result<DynamicPosition> {
    Ok(serde_wasm_bindgen::from_value(pos.into())?)
}

fn import_candidates(candidates: ICandidates) -> Result<Vec<u8>> {
    Ok(serde_wasm_bindgen::from_value(candidates.into())?)
}

fn import_dynamic_generator_settings(
    dynamic_generator_settings: IDynamicGeneratorSettings,
) -> Result<DynamicGeneratorSettings> {
    Ok(serde_wasm_bindgen::from_value(
        dynamic_generator_settings.into(),
    )?)
}

fn import_grid_format(format: IGridFormat) -> Result<GridFormat> {
    Ok(serde_wasm_bindgen::from_value(format.into())?)
}

fn import_cells(cells: Vec<ICellView>) -> Result<Vec<CellView>> {
    cells
        .into_iter()
        .map(|cell| serde_wasm_bindgen::from_value(cell.into()).map_err(Into::into))
        .collect()
}
fn import_strategy(strategy: IDynamicStrategy) -> Result<DynamicStrategy> {
    Ok(serde_wasm_bindgen::from_value(strategy.into())?)
}
// --- Import helpers

// Export helpers
fn export_error(error: SudokuError) -> JsError {
    let message = format!("{error:?}");
    JsError::new(&message)
}

fn export_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    Ok(value.serialize(&Serializer::new().serialize_maps_as_objects(true))?)
}

fn export_sudoku(transport_sudoku: TransportSudoku) -> Result<ITransportSudoku> {
    Ok(export_value(&transport_sudoku)?.into())
}
// --- Export helpers
