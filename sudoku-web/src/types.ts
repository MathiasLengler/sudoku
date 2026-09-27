// Re-exports only the types of the wasm module.
// This ensures only the worker loads the WASM module at runtime.

import type { DynamicCell, GridMetric } from "sudoku-wasm";

export type * from "sudoku-wasm";

export type DynamicCellValue = Extract<DynamicCell, { kind: "value" }>;
export type DynamicCellCandidates = Extract<DynamicCell, { kind: "candidates" }>;

export type GridMetricName = GridMetric["kind"];
