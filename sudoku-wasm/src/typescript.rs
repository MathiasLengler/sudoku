use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TRANSPORT_TYPESCRIPT: &'static str = r#"export type CellBlocks = Cell[][];

export type TransportCellBlock = TransportCell[];

export type Candidates = number[];

export interface TransportSudoku {
    blocks: TransportCellBlock[];
    base: number;
    sideLength: number;
    cellCount: number;
    isSolved: boolean;
}

export interface CellPosition {
    column: number;
    row: number;
}

export interface TransportCellContext {
    position: CellPosition;
    incorrectValue: boolean;
}

export interface ValueCell {
    kind: "value";
    fixed: boolean;
    value: number;
}

export interface CandidatesCell {
    kind: "candidates";
    candidates: Candidates;
}

export type Cell = ValueCell | CandidatesCell;

export type TransportCell = TransportCellContext & Cell;

export interface GeneratorSettings {
    base: number;
    target: GeneratorTarget;
}

export type GeneratorTarget =
    | "minimal"
    | "filled"
    | {
    fromMinimal: {
        distance: number;
    };
}
    | {
    fromFilled: {
        distance: number;
    };
};

export type GridFormat = "givensLine" | "givensGrid" | "binaryCandidatesLine";
export type StrategyName = "SingleCandidate" |
    "HiddenSingles" |
    "GroupReduction" |
    "Backtracking"
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "CellBlocks")]
    pub type ICellBlocks;
    #[wasm_bindgen(typescript_type = "TransportCellBlock")]
    pub type ITransportCellBlock;
    #[wasm_bindgen(typescript_type = "Candidates")]
    pub type ICandidates;
    #[wasm_bindgen(typescript_type = "TransportSudoku")]
    pub type ITransportSudoku;
    #[wasm_bindgen(typescript_type = "CellPosition")]
    pub type ICellPosition;
    #[wasm_bindgen(typescript_type = "TransportCellContext")]
    pub type ITransportCellContext;
    #[wasm_bindgen(typescript_type = "ValueCell")]
    pub type IValueCell;
    #[wasm_bindgen(typescript_type = "CandidatesCell")]
    pub type ICandidatesCell;
    #[wasm_bindgen(typescript_type = "Cell")]
    pub type ICell;
    #[wasm_bindgen(typescript_type = "TransportCell")]
    pub type ITransportCell;
    #[wasm_bindgen(typescript_type = "GeneratorSettings")]
    pub type IGeneratorSettings;
    #[wasm_bindgen(typescript_type = "GeneratorTarget")]
    pub type IGeneratorTarget;
    #[wasm_bindgen(typescript_type = "GridFormat")]
    pub type IGridFormat;
    #[wasm_bindgen(typescript_type = "StrategyName")]
    pub type IStrategyName;
}
