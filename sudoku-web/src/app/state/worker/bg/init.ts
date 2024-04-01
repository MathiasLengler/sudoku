import wbgInit, { initThreadPool, init as wasmInit } from "../../../../../../sudoku-wasm/pkg";

export async function init(threadCount?: number) {
    // wasm-bindgen with `--target web` requires manual initialization of the module
    console.debug("Initialize wasm-bindgen");
    await wbgInit();

    // Our own init function (`console_error_panic_hook` and `console_log`)
    console.debug("Initialize sudoku-wasm");
    wasmInit();

    // `wasm_bindgen_rayon`
    const hardwareConcurrency = threadCount ?? navigator.hardwareConcurrency;
    console.debug(`Initialize wasm-bindgen-rayon with ${hardwareConcurrency} threads`);
    await initThreadPool(hardwareConcurrency);

    console.debug("WASM initialized");
}
