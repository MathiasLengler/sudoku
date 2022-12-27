use wasm_bindgen::JsError;

pub type Error = JsError;
pub type Result<T, E = Error> = std::result::Result<T, E>;
