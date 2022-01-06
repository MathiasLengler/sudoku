// This re-exports the wasm module directly. Should only be imported by the worker.
export { WasmSudoku, init } from "../../sudoku-wasm/pkg";
