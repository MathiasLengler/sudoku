import * as Comlink from "comlink";
import wbgInit, { WasmSudoku as WasmSudokuValue, init as wasmInit, initThreadPool } from "../../sudoku-wasm/pkg";

import { WORKER_BOOT_UP_MESSAGE } from "./constants";
import type { DynamicCells, WasmSudoku } from "./types";

if (process.env.NODE_ENV !== "production") {
    self.addEventListener("message", ev => {
        console.debug("Worker message RX:", ev.data);
    });
}

export interface WorkerApi {
    init: typeof init;
    // We need to lie about the nullability of typedWasmSudoku
    // or else Comlink.Remote<WasmSudoku> doesn't narrow
    typedWasmSudoku: WasmSudoku;
}

const workerApi: WorkerApi = {
    init,
    typedWasmSudoku: undefined as unknown as WasmSudoku,
};

// Send boot up message
// Background: worker.tsx is an async module.
// This requires manual synchronization between Comlink.wrap and Comlink.expose,
// otherwise initialization messages from comlink would get lost, resulting in a deadlock.
postMessage(WORKER_BOOT_UP_MESSAGE);

Comlink.expose(workerApi);

async function init(cells?: DynamicCells) {
    console.debug("Worker init");

    console.debug("Initializing WASM module");
    // wasm-bindgen with `--target web` requires manual initialization of the module
    await wbgInit();

    // Our own init function (`console_error_panic_hook` and `console_log`)
    wasmInit();

    // `wasm_bindgen_rayon`
    await initThreadPool(navigator.hardwareConcurrency);

    if (cells) {
        console.debug("Restoring sudoku from cells");
        try {
            workerApi.typedWasmSudoku = WasmSudokuValue.restore(cells);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    if (!workerApi.typedWasmSudoku) {
        console.debug("Generating initial sudoku");
        workerApi.typedWasmSudoku = new WasmSudokuValue();
    }

    console.debug("Worker init done");
}
