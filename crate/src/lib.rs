use log::debug;
use serde::{Deserialize, Serialize};
use sudoku::cell::OptionCell;
use sudoku::Sudoku;
use sudoku::transport::TransportSudoku;
use wasm_bindgen::prelude::*;

// TODO: add typescript to webpack

#[cfg(feature = "wee_alloc")]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn run() -> Result<(), JsValue> {
    init();

    append_hello_dom()?;

    Ok(())
}


#[wasm_bindgen]
pub fn get_sudoku_controller() -> SudokuController {
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

    SudokuController {
        sudoku
    }
}

#[wasm_bindgen]
pub struct SudokuController {
    sudoku: Sudoku<OptionCell>
}

#[wasm_bindgen]
impl SudokuController {
    pub fn say_hello(&self) {
        debug!("Hello!");
    }

    pub fn get_sudoku(&self) -> JsValue {
        JsValue::from_serde(&TransportSudoku::from(&self.sudoku)).unwrap()
    }

    pub fn test_typescript(&self) -> JsValue {
        JsValue::from_serde(&TypescriptTest::V1 { foo: false }).unwrap()
    }
}

use wasm_typescript_definition::TypescriptDefinition;

#[derive(Serialize, TypescriptDefinition)]
#[serde(tag = "tag", content = "fields")]
enum TypescriptTest {
    #[allow(unused)]
    V1 {
        #[serde(rename = "Foo")]
        foo: bool,
    },
    #[allow(unused)]
    V2 {
        #[serde(rename = "Bar")]
        bar: i64,
        #[serde(rename = "Baz")]
        baz: u64,
    },
    #[allow(unused)]
    V3 {
        #[serde(rename = "Quux")]
        quux: String,
    },
}

fn append_hello_dom() -> Result<(), JsValue> {
    let window = web_sys::window().expect("should have a Window");
    let document = window.document().expect("should have a Document");

    let p: web_sys::Node = document.create_element("p")?.into();
    p.set_text_content(Some("Hello from Rust, WebAssembly, and Webpack!"));

    let body = document.body().expect("should have a body");
    let body: &web_sys::Node = body.as_ref();
    body.append_child(&p)?;

    Ok(())
}

fn init() {
    use log::Level;
    use std::panic;
    use std::sync::{ONCE_INIT, Once};
    static SET_HOOK: Once = ONCE_INIT;

    #[cfg(feature = "console")]
        SET_HOOK.call_once(|| {
        panic::set_hook(Box::new(console_error_panic_hook::hook));
        console_log::init_with_level(Level::Debug);
    });
}
