// Re-exports only the types of the wasm module.
// This ensures only the worker loads the WASM module at runtime.
export type { CellBlocks, GridFormat, WasmSudoku } from "../../sudoku-wasm/pkg";

export * from "../../sudoku-rs/bindings";
