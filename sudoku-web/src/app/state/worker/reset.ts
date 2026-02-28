import { atom } from "jotai";
import { RESET } from "jotai/utils";
import { wasmSudokuState } from "../mainThread/wasmSudoku";
import { workerState } from "./index";

/**
 * Write-only atom to reset the worker after a panic.
 * This terminates the current worker, spawns a new one, and restores the Sudoku state.
 * Use by calling `set(resetWorkerAfterPanicAction)` from a component or action.
 */
export const resetWorkerAfterPanicAction = atom(null, (_get, set) => {
    console.warn("Resetting worker after WASM panic");

    // Refresh the worker state - calling set() with no arguments on an atomWithRefresh
    // triggers a refresh, which will run the factory function again.
    // This terminates the old worker (via currentWorker.terminate()) and spawns a new one.
    set(workerState);

    // Reset the sudoku state to force re-initialization with persisted data.
    // wasmSudokuState is an atomWithDefault, so RESET causes it to re-derive
    // from the new worker via the mainThreadWasmSudokuClassState chain.
    set(wasmSudokuState, RESET);
});
