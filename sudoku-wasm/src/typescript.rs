#![allow(clippy::empty_docs)]

use crate::error::Result;
use anyhow::anyhow;
use serde_wasm_bindgen::Serializer;
use sudoku::{
    error::Error as SudokuError,
    generator::{GeneratorProgress, multi_shot::MultiShotGeneratorProgress},
};
use tsify::{Ts, Tsify};
use wasm_bindgen::prelude::*;

pub(crate) fn import_err(err: &JsValue) -> SudokuError {
    if let Some(err) = err.dyn_ref::<js_sys::Error>() {
        if let Some(message) = err.message().as_string() {
            anyhow!(message)
        } else {
            anyhow!("JsValue err message not convertible to string")
        }
    } else {
        anyhow!("JsValue err not convertible to Error")
    }
}

pub(crate) fn export_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    Ok(value.serialize(
        &Serializer::json_compatible()
            // tsify has no global equivalent: every (i64, u64) needs a `bigint` TS type.
            // Constraint: usize/isize never cross the WASM boundary.
            .serialize_large_number_types_as_bigints(true),
    )?)
}

/// Like `Ts::from_rust`, but with the crate-wide serializer config of `export_value`.
///
/// `Ts::from_rust` uses the top-level type's `SERIALIZATION_CONFIG` for all nested types.
pub(crate) fn export_ts<T: Tsify + serde::ser::Serialize>(value: &T) -> Result<Ts<T>> {
    Ok(Ts::new_unchecked(export_value(value)?))
}

// Callbacks
#[wasm_bindgen(typescript_custom_section)]
const TS_CUSTOM: &'static str = r#"
export type GenerateOnProgress = (progress: GeneratorProgress) => void;
export type GenerateMultiShotOnProgress = (progress: MultiShotGeneratorProgress) => void;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "GenerateOnProgress")]
    pub type IGenerateOnProgress;
    #[wasm_bindgen(typescript_type = "GenerateMultiShotOnProgress")]
    pub type IGenerateMultiShotOnProgress;
}

fn import_function(function: impl Into<JsValue>) -> Result<js_sys::Function> {
    Ok(function
        .into()
        .dyn_into::<js_sys::Function>()
        .map_err(|value| anyhow!("Expected function, instead got: {:?}", value))?)
}

pub(crate) fn import_generate_on_progress(
    on_progress: IGenerateOnProgress,
) -> Result<impl Fn(GeneratorProgress) -> Result<(), SudokuError>> {
    let function = import_function(on_progress)?;

    Ok(
        move |progress: GeneratorProgress| -> Result<(), SudokuError> {
            function
                .call1(
                    &JsValue::undefined(),
                    &export_value(&progress).map_err(|err| import_err(&err.into()))?,
                )
                .map_err(|err| import_err(&err))?;
            Ok(())
        },
    )
}

pub(crate) fn import_generate_multi_shot_on_progress(
    on_progress: IGenerateMultiShotOnProgress,
) -> Result<impl Fn(MultiShotGeneratorProgress) -> Result<(), SudokuError>> {
    let function = import_function(on_progress)?;

    Ok(
        move |progress: MultiShotGeneratorProgress| -> Result<(), SudokuError> {
            function
                .call1(
                    &JsValue::undefined(),
                    &export_value(&progress).map_err(|err| import_err(&err.into()))?,
                )
                .map_err(|err| import_err(&err))?;
            Ok(())
        },
    )
}
