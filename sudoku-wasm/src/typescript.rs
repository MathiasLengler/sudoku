#![allow(clippy::empty_docs)]

use crate::error::Result;
use anyhow::anyhow;
use paste::paste;
use serde_wasm_bindgen::Serializer;
use sudoku::{
    base::BaseEnum,
    cell::dynamic::{DynamicCandidates, DynamicCell, DynamicValue},
    error::Error as SudokuError,
    generator::{
        multi_shot::{DynamicMultiShotGeneratorSettings, MultiShotGeneratorProgress},
        DynamicGeneratorSettings, DynamicPruningOrder, DynamicPruningSettings,
        DynamicSolutionSettings, GeneratorProgress, PruningGroupBehaviour, PruningTarget,
    },
    grid::{dynamic::DynamicGrid, format::GridFormatEnum},
    position::DynamicPosition,
    solver::strategic::{
        deduction::transport::{
            PositionedTransportAction, PositionedTransportReason, TransportAction,
            TransportDeduction, TransportDeductions, TransportReason,
        },
        strategies::StrategyEnum,
        DynamicSolveStep,
    },
    transport::{TransportCell, TransportSudoku},
    world::{
        CellWorldDimensions, DynamicWorldGridCellPosition, Quadrant, RelativeDir, WorldCellDim,
        WorldCellPosition, WorldGenerationResult, WorldGridDim, WorldGridPosition,
    },
};
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
    Ok(value.serialize(&Serializer::json_compatible())?)
}

// Bridge ts_rs and wasm_bindgen using serde_wasm_bindgen
// Macro should be called with a list of (de)serializable types
macro_rules! serde_wasm_bindgen_interop {
    (IMPORT_TS_RS_BINDINGS; $($ty_name:ty),* $(,)?) => {
        #[wasm_bindgen(typescript_custom_section)]
        const IMPORT_TS_RS_BINDINGS: &'static str = concat!(
            "import type {\n",
            $("    ", stringify!($ty_name) , ",\n"),*
            , r#"} from "../../sudoku-rs/bindings";"#);

        serde_wasm_bindgen_interop!($($ty_name),*);
    };
    ($($ty_name:ty),* $(,)?) => {
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

// All ts-rs annotated types:
//  #[cfg_attr(feature = "wasm", derive(ts_rs::TS), ts(export))]
serde_wasm_bindgen_interop! {
    IMPORT_TS_RS_BINDINGS;
    BaseEnum,
    CellWorldDimensions,
    DynamicCandidates,
    DynamicCell,
    DynamicGeneratorSettings,
    DynamicGrid,
    DynamicMultiShotGeneratorSettings,
    DynamicPosition,
    DynamicPruningOrder,
    DynamicPruningSettings,
    DynamicSolutionSettings,
    DynamicSolveStep,
    DynamicValue,
    DynamicWorldGridCellPosition,
    GeneratorProgress,
    GridFormatEnum,
    MultiShotGeneratorProgress,
    PositionedTransportAction,
    PositionedTransportReason,
    PruningGroupBehaviour,
    PruningTarget,
    Quadrant,
    RelativeDir,
    StrategyEnum,
    TransportAction,
    TransportCell,
    TransportDeduction,
    TransportDeductions,
    TransportReason,
    TransportSudoku,
    WorldGenerationResult,
}

// Serde-compatbile aliases
pub type StrategyEnums = Vec<StrategyEnum>;
pub type DynamicCells = Vec<DynamicCell>;

// Must be keept in sync with aliases above
#[wasm_bindgen(typescript_custom_section)]
const TS_SERDE_ALIASES: &'static str = r#"
export type StrategyEnums = StrategyEnum[];
export type DynamicCells = DynamicCell[];
"#;
serde_wasm_bindgen_interop! {
    DynamicCells,
    StrategyEnums,
}

// external types (zod branded types)
#[wasm_bindgen(typescript_custom_section)]
const TS_EXTERNAL_SERDE_TYPES: &'static str = r#"
import type {
    WorldCellDim,
    WorldCellPosition,
    WorldGridDim,
    WorldGridPosition,
} from "../../sudoku-web/src/app/state/world/schema";
"#;
serde_wasm_bindgen_interop! {
    WorldCellDim,
    WorldCellPosition,
    WorldGridDim,
    WorldGridPosition,
}

// non-serde types - custom conversion functions
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

// serialized data (postcard)
#[wasm_bindgen(typescript_custom_section)]
const TS_SERIALIZED: &'static str = r#"
import type {
    SerializedDynamicCellWorld,
    SerializedDynamicSudoku,
} from "../../sudoku-web/src/app/utils/serializedData";
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "SerializedDynamicCellWorld")]
    pub type ISerializedDynamicCellWorld;

    #[wasm_bindgen(typescript_type = "SerializedDynamicSudoku")]
    pub type ISerializedDynamicSudoku;
}
