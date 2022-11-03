use std::cell::RefCell;

use log::trace;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::*;

use sudoku::base::consts::*;
use sudoku::cell::view::CellView;
use sudoku::error::Error as SudokuError;
use sudoku::generator::RuntimeSettings;
use sudoku::grid::serialization::GridFormat;
use sudoku::grid::Grid;
use sudoku::position::Position;
use sudoku::transport::TransportSudoku;
use sudoku::{DynamicSudoku, Game, Sudoku};

use typescript::{
    ICandidates, ICellBlocks, ICellPosition, IGeneratorSettings, IGridFormat, IStrategyName,
    ITransportSudoku,
};

use error::Result;

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
    sudoku: RefCell<DynamicSudoku>,
}

impl From<DynamicSudoku> for WasmSudoku {
    fn from(sudoku: DynamicSudoku) -> Self {
        WasmSudoku {
            sudoku: RefCell::new(sudoku),
        }
    }
}

#[wasm_bindgen]
impl WasmSudoku {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        // Intellij-rust false positive
        #[allow(unused_variables)]
        #[cfg(debug_assertions)]
        let grid: Grid<U2> = sudoku::samples::minimal();

        #[cfg(not(debug_assertions))]
        let grid: Grid<U3> = sudoku::samples::minimal();

        DynamicSudoku::with_sudoku(Sudoku::with_grid(grid))
            .unwrap()
            .into()
    }

    pub fn restore(blocks: ICellBlocks) -> Result<WasmSudoku> {
        let cells = Self::import_blocks(blocks)?;

        Ok(DynamicSudoku::try_from(cells)
            .map_err(Self::export_error)?
            .into())
    }

    #[wasm_bindgen(js_name = getSudoku)]
    pub fn get_sudoku(&self) -> Result<ITransportSudoku> {
        let transport_sudoku = TransportSudoku::from(&*self.sudoku.borrow());

        Self::export_sudoku(transport_sudoku)
    }

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&self, pos: ICellPosition, value: u8) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .set_value(Self::import_pos(pos)?, value)
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = setOrToggleValue)]
    pub fn set_or_toggle_value(&self, pos: ICellPosition, value: u8) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .set_or_toggle_value(Self::import_pos(pos)?, value)
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = setCandidates)]
    pub fn set_candidates(&mut self, pos: ICellPosition, candidates: ICandidates) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .set_candidates(Self::import_pos(pos)?, Self::import_candidates(candidates)?)
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = toggleCandidate)]
    pub fn toggle_candidate(&mut self, pos: ICellPosition, candidate: u8) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .toggle_candidate(Self::import_pos(pos)?, candidate)
            .map_err(Self::export_error)?)
    }

    pub fn delete(&mut self, pos: ICellPosition) -> Result<()> {
        Ok(self.sudoku.borrow_mut().delete(Self::import_pos(pos)?))
    }

    #[wasm_bindgen(js_name = setAllDirectCandidates)]
    pub fn set_all_direct_candidates(&mut self) {
        self.sudoku.borrow_mut().set_all_direct_candidates();
    }

    pub fn undo(&mut self) {
        self.sudoku.borrow_mut().undo();
    }

    pub fn generate(&mut self, generator_settings: IGeneratorSettings) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .generate(Self::import_generator_settings(generator_settings)?)
            .map_err(Self::export_error)?)
    }

    pub fn import(&mut self, input: &str) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .import(input)
            .map_err(Self::export_error)?)
    }

    pub fn export(&self, format: IGridFormat) -> Result<String> {
        Ok(self
            .sudoku
            .borrow_mut()
            .export(&Self::import_grid_format(format)?))
    }

    #[wasm_bindgen(js_name = tryStrategy)]
    pub fn try_strategy(&mut self, strategy_name: IStrategyName) -> Result<bool> {
        Ok(self
            .sudoku
            .borrow_mut()
            .try_strategy(&strategy_name.as_string().unwrap())
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = solveSingleCandidates)]
    pub fn solve_single_candidates(&mut self) -> Result<bool> {
        Ok(self
            .sudoku
            .borrow_mut()
            .solve_single_candidates()
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = groupReduction)]
    pub fn group_reduction(&mut self) -> Result<()> {
        Ok(self
            .sudoku
            .borrow_mut()
            .group_reduction()
            .map_err(Self::export_error)?)
    }
}

/// Import helpers
impl WasmSudoku {
    fn import_pos(pos: ICellPosition) -> Result<Position> {
        Ok(serde_wasm_bindgen::from_value(pos.into())?)
    }

    fn import_candidates(candidates: ICandidates) -> Result<Vec<u8>> {
        Ok(serde_wasm_bindgen::from_value(candidates.into())?)
    }

    fn import_generator_settings(
        generator_settings: IGeneratorSettings,
    ) -> Result<RuntimeSettings> {
        Ok(serde_wasm_bindgen::from_value(generator_settings.into())?)
    }

    fn import_grid_format(format: IGridFormat) -> Result<GridFormat> {
        Ok(serde_wasm_bindgen::from_value(format.into())?)
    }

    fn import_blocks(cells: ICellBlocks) -> Result<Vec<Vec<CellView>>> {
        Ok(serde_wasm_bindgen::from_value(cells.into())?)
    }
}

/// Export helpers
impl WasmSudoku {
    fn export_error(error: SudokuError) -> JsError {
        let message = format!("{error:?}");
        JsError::new(&message)
    }

    pub fn export_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
        Ok(value.serialize(&Serializer::new().serialize_maps_as_objects(true))?)
    }

    fn export_sudoku(transport_sudoku: TransportSudoku) -> Result<ITransportSudoku> {
        Ok(Self::export_value(&transport_sudoku)?.into())
    }
}
