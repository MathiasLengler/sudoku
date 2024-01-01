use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TRANSPORT_TYPESCRIPT: &'static str = r#"
import type * as bindings from "../../sudoku-rs/bindings";

export type Candidates = number[];

export type DynamicStrategies = bindings.DynamicStrategy[];

export type GenerateOnProgress = (progress: bindings.GeneratorProgress) => void;
"#;

// Source: wasm_bindgen typescript_custom_section
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Candidates")]
    pub type ICandidates;
    #[wasm_bindgen(typescript_type = "DynamicStrategies")]
    pub type IDynamicStrategies;
    #[wasm_bindgen(typescript_type = "GenerateOnProgress")]
    pub type IGenerateOnProgress;
}

// FIXME: improve type safety (ts-rs <=> wasm_bindgen)
//  each extern type is associated with a specific Rust type
//  this is defined by:
//  - the `I*` name
//  - typescript_type = "bindings.*"
//  - import_* and export_* functions in lib.rs
//  if there is a mismatch at any one of these points,
//  the E2E type chain is broken.
//  Possible improvements:
//  - macro_rules for extern "C" content
//  - macro_rules for both extern "C" an import/export functions (conversion always via serde)

// Source: ts_rs
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "bindings.TransportSudoku")]
    pub type ITransportSudoku;
    #[wasm_bindgen(typescript_type = "bindings.DynamicPosition")]
    pub type IDynamicPosition;
    #[wasm_bindgen(typescript_type = "bindings.GeneratorSettings")]
    pub type IGeneratorSettings;
    #[wasm_bindgen(typescript_type = "bindings.DynamicStrategy")]
    pub type IDynamicStrategy;
    #[wasm_bindgen(typescript_type = "bindings.DynamicGeneratorSettings")]
    pub type IDynamicGeneratorSettings;
    #[wasm_bindgen(typescript_type = "bindings.DynamicCell")]
    pub type IDynamicCell;
    #[wasm_bindgen(typescript_type = "bindings.DynamicGridFormat")]
    pub type IDynamicGridFormat;
    #[wasm_bindgen(typescript_type = "bindings.TransportDeductions")]
    pub type ITransportDeductions;
    #[wasm_bindgen(typescript_type = "bindings.DynamicTryStrategiesReturn")]
    pub type IDynamicTryStrategiesReturn;
    #[wasm_bindgen(typescript_type = "bindings.RelativeTileDir")]
    pub type IRelativeTileDir;
}
