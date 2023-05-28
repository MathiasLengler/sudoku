use std::fmt::{Display, Formatter};

use wasm_bindgen::{JsError, JsValue};

use sudoku::error::Error as SudokuError;

#[derive(Debug)]
pub enum SudokuWasmError {
    SudokuError(SudokuError),
    JsValue(JsValue),
}

impl Display for SudokuWasmError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SudokuWasmError::SudokuError(err) => err.fmt(f),
            SudokuWasmError::JsValue(js_value) => crate::import::import_err(js_value).fmt(f),
        }
    }
}

impl std::error::Error for SudokuWasmError {}

impl From<SudokuError> for SudokuWasmError {
    fn from(err: SudokuError) -> Self {
        Self::SudokuError(err)
    }
}

impl From<serde_wasm_bindgen::Error> for SudokuWasmError {
    fn from(err: serde_wasm_bindgen::Error) -> Self {
        err.into()
    }
}

impl From<SudokuWasmError> for JsValue {
    fn from(err: SudokuWasmError) -> Self {
        match err {
            SudokuWasmError::SudokuError(err) => JsError::new(&format!("{err:?}")).into(),
            SudokuWasmError::JsValue(js_value) => js_value,
        }
    }
}

pub type Error = SudokuWasmError;
pub type Result<T, E = Error> = std::result::Result<T, E>;
