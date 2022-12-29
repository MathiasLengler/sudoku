use wasm_bindgen::JsError;

// TODO: define custom error and remove `map_err(Self::export_error)` usages
//  impl From<sudoku::Error>
//  impl Into<JsError>
pub type Error = JsError;
pub type Result<T, E = Error> = std::result::Result<T, E>;
