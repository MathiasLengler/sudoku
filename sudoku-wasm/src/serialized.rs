//! Postcard bytes crossing the boundary, branded in TS by their wasm class.
//!
//! - The brand key isn't exported: only `serialize` can produce a value.
//! - `SerializedJs` pairs each Rust type with its JS type, so a mismatched `deserialize` doesn't compile.

use js_sys::Uint8Array;
use serde::{Serialize, de::DeserializeOwned};
use sudoku::DynamicSudoku;
use sudoku::error::Error as SudokuError;
use sudoku::world::dynamic::DynamicCellWorld;
use wasm_bindgen::prelude::*;

use crate::error::Result;

#[wasm_bindgen(typescript_custom_section)]
const TS_SERIALIZED: &'static str = r#"
declare const serialized: unique symbol;
type Serialized<T> = Uint8Array<ArrayBuffer> & { readonly [serialized]: T };
export type SerializedDynamicSudoku = Serialized<WasmSudoku>;
export type SerializedDynamicCellWorld = Serialized<WasmCellWorld>;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SerializedDynamicSudoku")]
    pub type ISerializedDynamicSudoku;

    #[wasm_bindgen(typescript_type = "SerializedDynamicCellWorld")]
    pub type ISerializedDynamicCellWorld;
}

pub(crate) trait SerializedJs: Serialize + DeserializeOwned {
    type Js: JsCast;
}

impl SerializedJs for DynamicSudoku {
    type Js = ISerializedDynamicSudoku;
}

impl SerializedJs for DynamicCellWorld {
    type Js = ISerializedDynamicCellWorld;
}

pub(crate) fn serialize<T: SerializedJs>(value: &T) -> Result<T::Js> {
    let bytes = postcard::to_stdvec(value).map_err(SudokuError::from)?;
    Ok(Uint8Array::from(bytes.as_slice()).unchecked_into())
}

pub(crate) fn deserialize<T: SerializedJs>(js: &T::Js) -> Result<T> {
    let bytes = js.unchecked_ref::<Uint8Array>().to_vec();
    Ok(postcard::from_bytes(&bytes).map_err(SudokuError::from)?)
}
