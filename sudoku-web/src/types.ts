// Re-exports only the types of the wasm module.
// This ensures only the worker loads the WASM module at runtime.
export type {
    CellBlocks,
    TransportCellBlock,
    TransportSudoku,
    CellPosition,
    TransportCellContext,
    ValueCell,
    CandidatesCell,
    Cell,
    TransportCell,
    GeneratorSettings,
    GeneratorTarget,
    GridFormat,
    WasmSudoku,
} from "../../sudoku-wasm/pkg";
