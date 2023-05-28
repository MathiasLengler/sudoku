// Re-exports only the types of the wasm module.
// This ensures only the worker loads the WASM module at runtime.

export type * from "../../sudoku-rs/bindings";
export type { WasmSudoku, DynamicStrategies } from "../../sudoku-wasm/pkg";

import type { DynamicCell } from "../../sudoku-rs/bindings";

export type DynamicCells = DynamicCell[];
