import * as Comlink from "comlink";
import wasm from "./wasmSudoku";
import { WORKER_BOOT_UP_MESSAGE, WORKER_GENERATION_ABORTED_MESSAGE } from "./constants";
import type { DynamicCells, WasmSudoku, DynamicGeneratorSettings, GeneratorProgress } from "./types";

const { WasmSudoku: WasmSudokuValue, init: wasmInit } = wasm;

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
    generateWithChannel: typeof generateWithChannel;
}

const workerApi: WorkerApi = {
    init,
    typedWasmSudoku: undefined as unknown as WasmSudoku,
    generateWithChannel,
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
    wasmInit();

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

function generateWithChannel(generatorSettings: DynamicGeneratorSettings, onProgressPort: MessagePort) {
    console.log("generateWithChannel", generatorSettings, onProgressPort);
    function postOnProgressMessage(progress: GeneratorProgress) {
        onProgressPort.postMessage(progress);
    }

    const controller = new AbortController();

    // FIXME: still broken
    //  workerApi.typedWasmSudoku.generate blocks the event loop *inside* the worker.
    //  This delays the execution of onmessage until *after* the generation is complete,
    //  which invalidates this architecture.
    // Alternatives:
    //  - on_progress *must* return a Promise, converted to a Rust Future, which yields back to JS, which unblocks the event loop
    //    - this makes sudoku-rs async
    //    - unclear if `wasm-bindgen-futures` supports async callbacks
    //    - Rust consumers need to bring a runtime
    //  - Generator driven by JS
    //    - fundamental rewrite of Generator
    //    - manual state machine rewrite, even though Rust can do that via async fns natively.
    //    - additional complexity in the generation routines
    //    - bad API for Rust consumers
    //    - split of generation logic between JS and Rust (ping/pong)
    //  - terminate Worker and restart using pre-generation state
    //    - inefficient, but possible?
    //    - refactor of spawnWorker
    //    - init with `generatorSettings`?
    //    - Spawn multiple workers for multi-threaded generation?
    //      - Racing: fastest wins
    //      - Wait for all:
    //        - User selection
    //        - Auto select "best"
    //      - Keep generating until some criterion is met

    onProgressPort.onmessage = (ev: MessageEvent<string>) => {
        console.log("onProgressPort.onmessage", ev.data);

        if (ev.data === WORKER_GENERATION_ABORTED_MESSAGE) {
            controller.abort();
        } else {
            throw new Error("Unexpected non-abort message");
        }
    };

    workerApi.typedWasmSudoku.generate(generatorSettings, progress => {
        controller.signal.throwIfAborted();
        postOnProgressMessage(progress);
    });
}
