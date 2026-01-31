import wbgInit, { init as wasmInit, WasmSudoku } from "sudoku-wasm";
import { atom } from "jotai";
import { atomWithDefault } from "jotai/utils";
import type { DynamicGrid } from "../../../types";
import type { SerializedDynamicSudoku } from "../../utils/serializedData";
import { loadCells } from "../cellsPersistence";
import { GENERATE_FORM_DEFAULT_VALUES } from "../forms/generate";

let isInitialized = false;

async function initMainThreadWasm(): Promise<void> {
    if (isInitialized) {
        return;
    }

    // wasm-bindgen with `--target web` requires manual initialization of the module
    console.debug("Initialize wasm-bindgen (main thread)");
    await wbgInit();

    // Our own init function (`console_error_panic_hook` and `console_log`)
    console.debug("Initialize sudoku-wasm (main thread)");
    wasmInit();

    console.debug("WASM initialized (main thread)");

    isInitialized = true;
}

async function createMainThreadWasmSudoku(dynamicGrid?: DynamicGrid): Promise<WasmSudoku> {
    await initMainThreadWasm();

    if (dynamicGrid) {
        console.debug("Restoring sudoku (main thread)");
        try {
            return WasmSudoku.fromDynamicGrid(dynamicGrid);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }

    console.debug("Creating empty sudoku (main thread)");
    return WasmSudoku.new(GENERATE_FORM_DEFAULT_VALUES.base);
}

export const isMainThreadWasmReadyState = atom<Promise<boolean>>(async () => {
    await initMainThreadWasm();
    return true;
});

/**
 * Main thread WasmSudoku instance for cheap operations.
 * Expensive operations (generate, tryStrategies) should use the worker.
 */
export const mainThreadWasmSudokuState = atomWithDefault<WasmSudoku | Promise<WasmSudoku>>(async () => {
    const cells = loadCells();
    return await createMainThreadWasmSudoku(cells);
});

/**
 * Serialize the main thread WasmSudoku for transfer to worker.
 * Uses postcard binary serialization for efficient transfer.
 */
export function serializeForTransfer(wasmSudoku: WasmSudoku): SerializedDynamicSudoku {
    return wasmSudoku.serialize();
}

/**
 * Deserialize a WasmSudoku from worker transfer.
 */
export async function deserializeFromTransfer(bytes: SerializedDynamicSudoku): Promise<WasmSudoku> {
    await initMainThreadWasm();
    return WasmSudoku.deserialize(bytes);
}

// Re-export for convenience
export { WasmSudoku };
export { initMainThreadWasm };
