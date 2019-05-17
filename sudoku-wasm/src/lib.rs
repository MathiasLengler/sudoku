use std::cell::RefCell;

use log::debug;
use wasm_bindgen::prelude::*;

use sudoku::cell::Cell;
use sudoku::generator::backtracking::BacktrackingGenerator;
use sudoku::position::Position;
use sudoku::transport::TransportSudoku;
use sudoku::Sudoku;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    init();

    Ok(())
}

#[wasm_bindgen]
pub fn get_wasm_sudoku() -> WasmSudoku {
    use std::convert::TryFrom;

    let [_base_1, _base_2, _base_3] = [
        vec![vec![0]],
        vec![
            vec![0, 3, 4, 0],
            vec![4, 0, 0, 2],
            vec![1, 0, 0, 3],
            vec![0, 2, 1, 0],
        ],
        vec![
            vec![8, 0, 0, 0, 0, 0, 0, 0, 0],
            vec![0, 0, 3, 6, 0, 0, 0, 0, 0],
            vec![0, 7, 0, 0, 9, 0, 2, 0, 0],
            vec![0, 5, 0, 0, 0, 7, 0, 0, 0],
            vec![0, 0, 0, 0, 4, 5, 7, 0, 0],
            vec![0, 0, 0, 1, 0, 0, 0, 3, 0],
            vec![0, 0, 1, 0, 0, 0, 0, 6, 8],
            vec![0, 0, 8, 5, 0, 0, 0, 1, 0],
            vec![0, 9, 0, 0, 0, 0, 4, 0, 0],
        ],
    ];

    //    let mut sudoku = Sudoku::<Cell>::try_from(_base_3).unwrap();
    //    let mut sudoku = Sudoku::<Cell>::new(4);

    let mut sudoku = BacktrackingGenerator::new(3).generate();

    sudoku.fix_all_values();

    WasmSudoku {
        sudoku: RefCell::new(sudoku),
    }
}

#[wasm_bindgen]
pub struct WasmSudoku {
    sudoku: RefCell<Sudoku<Cell>>,
}

#[wasm_bindgen]
impl WasmSudoku {
    pub fn say_hello(&self) {
        debug!("Hello!");
    }

    pub fn get_sudoku(&self) -> JsValue {
        let transport_sudoku = {
            let sudoku = self.sudoku.borrow();

            TransportSudoku::from(&*sudoku)
        };

        JsValue::from_serde(&transport_sudoku).unwrap()
    }

    pub fn set_value(&self, pos: JsValue, value: usize) {
        self.sudoku
            .borrow_mut()
            .set_value(Self::import_pos(pos), value);
    }

    pub fn set_or_toggle_value(&self, pos: JsValue, value: usize) {
        self.sudoku
            .borrow_mut()
            .set_or_toggle_value(Self::import_pos(pos), value);
    }

    pub fn set_candidates(&mut self, pos: JsValue, candidates: JsValue) {
        self.sudoku
            .borrow_mut()
            .set_candidates(Self::import_pos(pos), Self::import_candidates(candidates));
    }

    pub fn toggle_candidate(&mut self, pos: JsValue, candidate: usize) {
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
}

/// Conversion Helpers
impl WasmSudoku {
    fn import_pos(pos: JsValue) -> Position {
        pos.into_serde().unwrap()
    }

    fn import_candidates(candidates: JsValue) -> Vec<usize> {
        candidates.into_serde().unwrap()
    }
}

fn init() {
    use log::Level;
    use std::panic;
    use std::sync::{Once, ONCE_INIT};
    static SET_HOOK: Once = ONCE_INIT;

    #[cfg(feature = "console")]
    SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(Level::Debug).unwrap();
    });
}
