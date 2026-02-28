import { atom } from "jotai";
import { WasmCellWorld, type WasmSudoku, WasmSudoku as WasmSudokuMaybeUninit } from "sudoku-wasm";
import { initWasm } from "./init";

// Classes from "sudoku-wasm" need WASM to be initialized before they can be used.

export const wasmSudokuClassState = atom<Promise<typeof WasmSudoku>>(async () => {
    await initWasm();
    return WasmSudokuMaybeUninit;
});

export const wasmCellWorldClassState = atom<Promise<typeof WasmCellWorld>>(async () => {
    await initWasm();
    return WasmCellWorld;
});
