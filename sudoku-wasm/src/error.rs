use wasm_bindgen::JsError;

pub type Error = JsError;
pub type Result<T> = std::result::Result<T, Error>;
