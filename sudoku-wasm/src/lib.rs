#![allow(unused_imports)]

use std::cell::RefCell;

use log::info;
use wasm_bindgen::prelude::*;

use sudoku::base::consts::*;
use sudoku::cell::Cell;
use sudoku::error::Error;
use sudoku::generator::backtracking::RuntimeSettings;
use sudoku::grid::Grid;
use sudoku::position::Position;
use sudoku::transport::TransportSudoku;
use sudoku::{DynamicSudoku, Game, Sudoku};

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    init();

    info!("Hello WASM");

    Ok(())
}

#[wasm_bindgen]
pub fn get_wasm_sudoku() -> WasmSudoku {
    let mut grid: Grid<U3> = sudoku::samples::minimal();

    WasmSudoku {
        sudoku: RefCell::new(DynamicSudoku::with_sudoku(Sudoku::with_grid(grid))),
    }
}

#[wasm_bindgen]
pub struct WasmSudoku {
    sudoku: RefCell<DynamicSudoku>,
}

#[wasm_bindgen]
impl WasmSudoku {
    pub fn get_sudoku(&self) -> JsValue {
        let transport_sudoku = {
            let sudoku = self.sudoku.borrow();

            TransportSudoku::from(&*sudoku)
        };

        JsValue::from_serde(&transport_sudoku).unwrap()
    }

    pub fn set_value(&self, pos: JsValue, value: u8) {
        self.sudoku
            .borrow_mut()
            .set_value(Self::import_pos(pos), value);
    }

    pub fn set_or_toggle_value(&self, pos: JsValue, value: u8) {
        self.sudoku
            .borrow_mut()
            .set_or_toggle_value(Self::import_pos(pos), value);
    }

    pub fn set_candidates(&mut self, pos: JsValue, candidates: JsValue) {
        self.sudoku
            .borrow_mut()
            .set_candidates(Self::import_pos(pos), Self::import_candidates(candidates));
    }

    pub fn toggle_candidate(&mut self, pos: JsValue, candidate: u8) {
        self.sudoku
            .borrow_mut()
            .toggle_candidate(Self::import_pos(pos), candidate);
    }

    pub fn delete(&mut self, pos: JsValue) {
        self.sudoku.borrow_mut().delete(Self::import_pos(pos));
    }

    pub fn set_all_direct_candidates(&mut self) {
        self.sudoku.borrow_mut().set_all_direct_candidates();
    }

    pub fn undo(&mut self) {
        self.sudoku.borrow_mut().undo();
    }

    pub fn generate(&mut self, generator_settings: JsValue) -> Result<(), JsValue> {
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

    pub fn solve_single_candidates(&mut self) {
        self.sudoku.borrow_mut().solve_single_candidates();
    }

    pub fn group_reduction(&mut self) {
        self.sudoku.borrow_mut().group_reduction();
    }
}

/// Conversion Helpers
impl WasmSudoku {
    fn import_pos(pos: JsValue) -> Position {
        pos.into_serde().unwrap()
    }

    fn import_candidates(candidates: JsValue) -> Vec<u8> {
        candidates.into_serde().unwrap()
    }

    fn import_generator_settings(generator_settings: JsValue) -> RuntimeSettings {
        generator_settings.into_serde().unwrap()
    }

    fn export_error(error: Error) -> JsValue {
        format!("{0}\n{0:?}", error).into()
    }
}

fn init() {
    use log::Level;
    use std::panic;
    use std::sync::Once;
    static SET_HOOK: Once = Once::new();

    #[cfg(feature = "console")]
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(Level::Debug).unwrap();
    });
}
