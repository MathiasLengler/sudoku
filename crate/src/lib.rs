use std::num::NonZeroUsize;

use log::debug;
use sudoku::cell::OptionCell;
use sudoku::Sudoku;
use sudoku::transport::TransportSudoku;
use wasm_bindgen::prelude::*;
use std::mem::drop;

use js_sys;
use std::cell::RefCell;

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    init();

    Ok(())
}


#[wasm_bindgen]
pub fn get_rust_sudoku() -> RustSudoku {
    use std::convert::TryFrom;

    // 11 Star difficulty
    let sudoku = Sudoku::<OptionCell>::try_from(vec![
        vec![8, 0, 0, 0, 0, 0, 0, 0, 0],
        vec![0, 0, 3, 6, 0, 0, 0, 0, 0],
        vec![0, 7, 0, 0, 9, 0, 2, 0, 0],
        vec![0, 5, 0, 0, 0, 7, 0, 0, 0],
        vec![0, 0, 0, 0, 4, 5, 7, 0, 0],
        vec![0, 0, 0, 1, 0, 0, 0, 3, 0],
        vec![0, 0, 1, 0, 0, 0, 0, 6, 8],
        vec![0, 0, 8, 5, 0, 0, 0, 1, 0],
        vec![0, 9, 0, 0, 0, 0, 4, 0, 0],
    ]).unwrap();

    RustSudoku {
        sudoku: RefCell::new(sudoku),
    }
}

#[wasm_bindgen]
pub struct RustSudoku {
    sudoku: RefCell<Sudoku<OptionCell>>,
}

#[wasm_bindgen]
impl RustSudoku {
    pub fn say_hello(&self) {
        debug!("Hello!");
    }

    pub fn get_sudoku(&self) -> JsValue {
        let sudoku = self.sudoku.borrow();

        let transport_sudoku = TransportSudoku::from(&*sudoku);

        drop(sudoku);

        JsValue::from_serde(&transport_sudoku).unwrap()
    }

    pub fn set_value(&self, pos: JsValue, value: usize) -> Result<(), JsValue> {
        let pos = pos.into_serde().unwrap();

        self.sudoku.borrow_mut().set(pos, OptionCell(NonZeroUsize::new(value)));

        Ok(())
    }
}

fn init() {
    use log::Level;
    use std::panic;
    use std::sync::{ONCE_INIT, Once};
    static SET_HOOK: Once = ONCE_INIT;

    #[cfg(feature = "console")]
        SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(Level::Debug).unwrap();
    });
}
