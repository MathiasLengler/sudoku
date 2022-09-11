mod typescript;

use typescript::{
    ICandidates, ICellBlocks, ICellPosition, IGeneratorSettings, IGridFormat, ITransportSudoku,
};

use std::cell::RefCell;

use log::trace;
use wasm_bindgen::prelude::*;

use sudoku::base::consts::*;
use sudoku::cell::view::CellView;
use sudoku::error::Error as SudokuError;
use sudoku::generator::backtracking::RuntimeSettings;
use sudoku::grid::serialization::GridFormat;
use sudoku::grid::Grid;
use sudoku::position::Position;
use sudoku::transport::TransportSudoku;
use sudoku::{DynamicSudoku, Game, Sudoku};

// TODO: use wasm-bindgen "typescript_type" and replace typedWasmSudoku.tsx
//  https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-rust-exports/typescript_type.html

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

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

    pub fn restore(blocks: ICellBlocks) -> Result<WasmSudoku, JsValue> {
        let cells = Self::import_blocks(blocks);

        Ok(DynamicSudoku::try_from(cells)
            .map_err(Self::export_error)?
            .into())
    }

    #[wasm_bindgen(js_name = getSudoku)]
    pub fn get_sudoku(&self) -> ITransportSudoku {
        let transport_sudoku = TransportSudoku::from(&*self.sudoku.borrow());

        JsValue::from_serde(&transport_sudoku).unwrap().into()
    }

    #[wasm_bindgen(js_name = setValue)]
    pub fn set_value(&self, pos: ICellPosition, value: u8) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .set_value(Self::import_pos(pos), value)
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = setOrToggleValue)]
    pub fn set_or_toggle_value(&self, pos: ICellPosition, value: u8) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .set_or_toggle_value(Self::import_pos(pos), value)
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = setCandidates)]
    pub fn set_candidates(
        &mut self,
        pos: ICellPosition,
        candidates: ICandidates,
    ) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .set_candidates(Self::import_pos(pos), Self::import_candidates(candidates))
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = toggleCandidate)]
    pub fn toggle_candidate(&mut self, pos: ICellPosition, candidate: u8) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .toggle_candidate(Self::import_pos(pos), candidate)
            .map_err(Self::export_error)?)
    }

    pub fn delete(&mut self, pos: ICellPosition) {
        self.sudoku.borrow_mut().delete(Self::import_pos(pos));
    }

    #[wasm_bindgen(js_name = setAllDirectCandidates)]
    pub fn set_all_direct_candidates(&mut self) {
        self.sudoku.borrow_mut().set_all_direct_candidates();
    }

    pub fn undo(&mut self) {
        self.sudoku.borrow_mut().undo();
    }

    pub fn generate(&mut self, generator_settings: IGeneratorSettings) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .generate(Self::import_generator_settings(generator_settings))
            .map_err(Self::export_error)?)
    }

    pub fn import(&mut self, input: &str) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .import(input)
            .map_err(Self::export_error)?)
    }

    pub fn export(&self, format: IGridFormat) -> String {
        self.sudoku
            .borrow_mut()
            .export(&Self::import_grid_format(format))
    }

    #[wasm_bindgen(js_name = solveSingleCandidates)]
    pub fn solve_single_candidates(&mut self) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .solve_single_candidates()
            .map_err(Self::export_error)?)
    }

    #[wasm_bindgen(js_name = groupReduction)]
    pub fn group_reduction(&mut self) -> Result<(), JsValue> {
        Ok(self
            .sudoku
            .borrow_mut()
            .group_reduction()
            .map_err(Self::export_error)?)
    }
}

// TODO: remove unwraps
/// Conversion Helpers
impl WasmSudoku {
    fn import_pos(pos: ICellPosition) -> Position {
        pos.into_serde().unwrap()
    }

    fn import_candidates(candidates: ICandidates) -> Vec<u8> {
        candidates.into_serde().unwrap()
    }

    fn import_generator_settings(generator_settings: IGeneratorSettings) -> RuntimeSettings {
        generator_settings.into_serde().unwrap()
    }

    fn import_grid_format(format: IGridFormat) -> GridFormat {
        format.into_serde().unwrap()
    }

    fn export_error(error: SudokuError) -> js_sys::Error {
        js_sys::Error::new(&error.to_string())
    }

    fn import_blocks(cells: ICellBlocks) -> Vec<Vec<CellView>> {
        cells.into_serde().unwrap()
    }
}
