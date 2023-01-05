import * as Comlink from "comlink";
import { init as wasmInit, WasmSudoku } from "./wasmSudoku";
import { WORKER_BOOT_UP_MESSAGE } from "./constants";
import type { CellViews } from "./types";

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

async function init(cellViews?: CellViews) {
    console.debug("Worker init");

    console.debug("Initializing WASM module");
    wasmInit();

    if (cellViews) {
        console.debug("Restoring sudoku from cells");
        try {
            workerApi.typedWasmSudoku = WasmSudoku.restore(cellViews);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    if (!workerApi.typedWasmSudoku) {
        console.debug("Generating initial sudoku");
        workerApi.typedWasmSudoku = new WasmSudoku();
    }

    console.debug("Worker init done");
}
