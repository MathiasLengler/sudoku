import * as Comlink from "comlink";
import { TypedWasmSudoku } from "./typedWasmSudoku";
import * as wasmModule from "../../sudoku-wasm/pkg";
import { WORKER_BOOT_UP_MESSAGE } from "./constants";

if (process.env.NODE_ENV !== "production") {
    self.addEventListener("message", ev => {
        console.debug("Worker RX", ev.data);
    });
}

export interface WorkerApi {
    init: typeof init;
    // We need to lie about the nullability of typedWasmSudoku
    // or else Comlink.Remote<TypedWasmSudoku> doesn't narrow
    typedWasmSudoku: TypedWasmSudoku;
}

const workerApi: WorkerApi = {
    init,
    typedWasmSudoku: undefined as unknown as TypedWasmSudoku,
};

// Send boot up message
// Background: worker.tsx is an async module.
// This requires manual synchronization between Comlink.wrap and Comlink.expose,
// otherwise initialization messages from comlink would get lost, resulting in a deadlock.
postMessage(WORKER_BOOT_UP_MESSAGE);

Comlink.expose(workerApi);

async function init(cells?: Cell[]) {
    console.debug("Worker init");

    console.debug("Initializing WASM module");
    wasmModule.run();

    if (cells) {
        console.debug("Restoring sudoku from cells");
        workerApi.typedWasmSudoku = TypedWasmSudoku.restore(cells);
    } else {
        console.debug("Generating initial sudoku");
        workerApi.typedWasmSudoku = new TypedWasmSudoku(new wasmModule.WasmSudoku());
    }

    console.debug("Worker init done");
}
