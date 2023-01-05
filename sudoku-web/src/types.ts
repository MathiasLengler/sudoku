// Re-exports only the types of the wasm module.
// This ensures only the worker loads the WASM module at runtime.
import type { CellView } from "../../sudoku-rs/bindings";

export type { GridFormat, WasmSudoku } from "../../sudoku-wasm/pkg";

export * from "../../sudoku-rs/bindings";

export type CellViews = CellView[];
