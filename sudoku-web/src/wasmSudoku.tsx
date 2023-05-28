// This re-exports the wasm module directly. Should only be imported by the worker.
import * as wasm from "../../sudoku-wasm/pkg";

// Workaround: https://github.com/rustwasm/wasm-bindgen/issues/3306#issuecomment-1492676376
await wasm;

export default wasm;
