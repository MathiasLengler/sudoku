#![allow(clippy::empty_docs)]

use crate::error::Result;
use anyhow::anyhow;
use paste::paste;
use serde_wasm_bindgen::Serializer;
use sudoku::{
    cell::dynamic::{DynamicCandidates, DynamicCell, DynamicValue},
    error::Error as SudokuError,
    generator::{
        DynamicGeneratorSettings, DynamicPruningOrder, DynamicPruningSettings,
        DynamicSolutionSettings, GeneratorProgress, PruningGroupBehaviour, PruningTarget,
    },
    grid::{dynamic::DynamicGrid, format::DynamicGridFormat},
    position::DynamicPosition,
    solver::strategic::{
        deduction::transport::{
            PositionedTransportAction, PositionedTransportReason, TransportAction,
            TransportDeduction, TransportDeductions, TransportReason,
        },
        strategies::DynamicStrategy,
    },
    transport::{TransportCell, TransportSudoku},
    world::RelativeTileDir,
    DynamicTryStrategiesReturn,
};
use wasm_bindgen::prelude::*;

// TODO: is sudoku-rs/bindings/index.ts needed?
// All ts-rs annotated types:
//  #[cfg_attr(feature = "wasm", derive(TS), ts(export))]
#[wasm_bindgen(typescript_custom_section)]
const IMPORT_TS_RS_BINDINGS: &'static str = r#"
import type {
    DynamicCandidates,
    DynamicCell,
    DynamicGeneratorSettings,
    DynamicGrid,
    DynamicGridFormat,
    DynamicPosition,
    DynamicPruningOrder,
    DynamicPruningSettings,
    DynamicSolutionSettings,
    DynamicStrategy,
    DynamicTryStrategiesReturn,
    DynamicValue,
    GeneratorProgress,
    PositionedTransportAction,
    PositionedTransportReason,
    PruningGroupBehaviour,
    PruningTarget,
    RelativeTileDir,
    TransportAction,
    TransportCell,
    TransportDeduction,
    TransportDeductions,
    TransportReason,
    TransportSudoku,
} from "../../sudoku-rs/bindings";
"#;

pub(crate) fn export_value<T: serde::ser::Serialize + ?Sized>(value: &T) -> Result<JsValue> {
    Ok(value.serialize(&Serializer::json_compatible())?)
}

// Bridge ts_rs and wasm_bindgen using serde_wasm_bindgen
// Macro should be called with a list of (de)serializable types
macro_rules! serde_wasm_bindgen_interop {
    ($($ty_name:ty),*) => {
        paste! {
            // wasm_bindgen interfaces "ITypeName" refercing bindings from ts_rs "bindings.TypeName"
            #[wasm_bindgen]
            extern "C" {
                $(
                    #[wasm_bindgen(typescript_type = $ty_name)]
                    pub type [<I $ty_name>];
                )*
            }

            // conversion functions using serde_wasm_bindgen
            //  import_type_name(value: ITypeName) -> Result<TypeName>
            //  export_type_name(value: TypeName) -> Result<ITypeName>
            $(
                #[allow(dead_code)]
                pub(crate) fn [<import_ $ty_name:snake>](value: [<I $ty_name>]) -> Result<$ty_name> {
                    Ok(serde_wasm_bindgen::from_value(value.into())?)
                }
                #[allow(dead_code)]
                pub(crate) fn [<export_ $ty_name:snake>](value: $ty_name) -> Result<[<I $ty_name>]> {
                    Ok(export_value(&value)?.into())
                }
            )*
        }
    };
}

// Serde-compatbile aliases
pub type DynamicStrategies = Vec<DynamicStrategy>;
pub type DynamicCells = Vec<DynamicCell>;

// Must be keept in sync with aliases above
#[wasm_bindgen(typescript_custom_section)]
const SERDE_ALIASES: &'static str = r#"
export type DynamicStrategies = bindings.DynamicStrategy[];
export type DynamicCells = bindings.DynamicCell[];
"#;

// ts-rs
serde_wasm_bindgen_interop! {
    DynamicCandidates,
    DynamicCell,
    DynamicGeneratorSettings,
    DynamicGrid,
    DynamicGridFormat,
    DynamicPosition,
    DynamicPruningOrder,
    DynamicPruningSettings,
    DynamicSolutionSettings,
    DynamicStrategy,
    DynamicTryStrategiesReturn,
    DynamicValue,
    GeneratorProgress,
    PositionedTransportAction,
    PositionedTransportReason,
    PruningGroupBehaviour,
    PruningTarget,
    RelativeTileDir,
    TransportAction,
    TransportCell,
    TransportDeduction,
    TransportDeductions,
    TransportReason,
    TransportSudoku
}

// aliases
serde_wasm_bindgen_interop! {
    DynamicStrategies,
    DynamicCells
}
// non-serde types - custom conversion functions
#[wasm_bindgen(typescript_custom_section)]
const EXTRA: &'static str = r#"
export type GenerateOnProgress = (progress: bindings.GeneratorProgress) => void;
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "GenerateOnProgress")]
    pub type IGenerateOnProgress;
}

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

pub(crate) fn import_generate_on_progress(
    on_progress: IGenerateOnProgress,
) -> Result<impl FnMut(GeneratorProgress) -> Result<(), SudokuError>> {
    let function = on_progress
        .dyn_into::<js_sys::Function>()
        .map_err(|value| anyhow!("Expected function, instead got: {:?}", JsValue::from(value)))?;

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
