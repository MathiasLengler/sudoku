// Re-exports only the types of the wasm module.
// This ensures only the worker loads the WASM module at runtime.

export type * from "../../sudoku-wasm/pkg";
import type { DynamicCell } from "../../sudoku-wasm/pkg";

export type DynamicCells = DynamicCell[];
