import assertNever from "assert-never";
import wbgInit, { initThreadPool, init as wasmInit } from "sudoku-wasm";

// Based on benchmarking: more threads don't improve performance of parallel grid generation.
const MAX_THREADS = 4;

let initState: { status: "pending" } | { status: "initializing"; promise: Promise<void> } | { status: "initialized" } =
    { status: "pending" };

const isWorkerThread = typeof WorkerGlobalScope !== "undefined" && self instanceof WorkerGlobalScope;

// Actual initialization logic
// This function should only be called once.
async function initWasmOnce(threadCount?: number) {
    // wasm-bindgen with `--target web` requires manual initialization of the module
    console.debug("Initialize wasm-bindgen");
    await wbgInit();

    // Our own init function (`console_error_panic_hook` and `console_log`)
    console.debug("Initialize sudoku-wasm");
    wasmInit();

    // `wasm_bindgen_rayon`
    if (isWorkerThread) {
        const hardwareConcurrency = threadCount ?? Math.min(navigator.hardwareConcurrency, MAX_THREADS);
        console.debug(`Initialize wasm-bindgen-rayon with ${hardwareConcurrency} threads`);
        await initThreadPool(hardwareConcurrency);
    } else if (threadCount !== undefined) {
        console.warn("Ignoring threadCount in main thread initialization");
    }
    console.debug("WASM initialized");
}

export async function initWasm(threadCount?: number) {
    switch (initState.status) {
        case "pending": {
            const initPromise = initWasmOnce(threadCount);
            initState = { status: "initializing", promise: initPromise };
            await initPromise;
            initState = { status: "initialized" };
            break;
        }
        case "initializing": {
            await initState.promise;
            break;
        }
        case "initialized": {
            // Already initialized
            break;
        }
        default: {
            assertNever(initState);
        }
    }
}
