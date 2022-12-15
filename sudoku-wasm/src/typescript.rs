use wasm_bindgen::prelude::*;

// TODO: replace StrategyName with DynamicStrategy

#[wasm_bindgen(typescript_custom_section)]
const TRANSPORT_TYPESCRIPT: &'static str = r#"
import type * as bindings from "../../sudoku-rs/bindings";

export type Candidates = number[];

export type CellBlocks = bindings.CellView[][];

export type GridFormat = "givensLine" | "givensGrid" | "binaryCandidatesLine";
export type StrategyName = "SingleCandidate" |
    "HiddenSingles" |
    "GroupReduction" |
    "Backtracking"
"#;

// Source: wasm_bindgen typescript_custom_section
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Candidates")]
    pub type ICandidates;
    #[wasm_bindgen(typescript_type = "CellBlocks")]
    pub type ICellBlocks;
    #[wasm_bindgen(typescript_type = "GridFormat")]
    pub type IGridFormat;
    #[wasm_bindgen(typescript_type = "StrategyName")]
    pub type IStrategyName;
}

// Source: ts_rs
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "bindings.TransportSudoku")]
    pub type ITransportSudoku;
    #[wasm_bindgen(typescript_type = "bindings.Position")]
    pub type IPosition;
    #[wasm_bindgen(typescript_type = "bindings.GeneratorSettings")]
    pub type IGeneratorSettings;
    #[wasm_bindgen(typescript_type = "bindings.DynamicStrategy")]
    pub type IDynamicStrategy;
    #[wasm_bindgen(typescript_type = "bindings.DynamicGeneratorSettings")]
    pub type IDynamicGeneratorSettings;
}
