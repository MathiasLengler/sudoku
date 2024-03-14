import * as Comlink from "comlink";
import wbgInit, { WasmSudoku, WasmCellWorld, init as wasmInit, initThreadPool } from "../../sudoku-wasm/pkg";

import { WORKER_BOOT_UP_MESSAGE } from "./constants";
import type { DynamicCells } from "./types";

if (process.env.NODE_ENV !== "production") {
    self.addEventListener("message", ev => {
        console.debug("Worker message RX:", ev.data);
    });
}

export interface WorkerApi {
    init: typeof init;
    // We need to lie about the nullability of typedWasmSudoku
    // or else Comlink.Remote<WasmSudoku> doesn't narrow
    wasmSudoku: WasmSudoku;
    wasmCellWorld: WasmCellWorld;
}

const workerApi: WorkerApi = {
    init,
    wasmSudoku: undefined as unknown as WasmSudoku,
    wasmCellWorld: undefined as unknown as WasmCellWorld,
};

// Send boot up message
// Background: worker.tsx is an async module. (TODO: is this still the case?)
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
            workerApi.wasmSudoku = WasmSudoku.restore(cells);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    if (!workerApi.wasmSudoku) {
        console.debug("Generating initial sudoku");
        workerApi.wasmSudoku = new WasmSudoku();
    }

    // TODO: restore
    workerApi.wasmCellWorld = new WasmCellWorld();

    console.debug("Worker init done");
}
