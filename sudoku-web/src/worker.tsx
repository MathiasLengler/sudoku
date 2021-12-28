import * as Comlink from "comlink";
import { TypedWasmSudoku } from "./typedWasmSudoku";

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

Comlink.expose(workerApi);

async function init() {
    console.debug("Worker init");

    console.debug("Loading WASM module");
    const module = await import("../../sudoku-wasm/pkg");

    console.debug("Initializing WASM module");
    module.run();

    console.debug("Exposing typed WASM sudoku");
    workerApi.typedWasmSudoku = new TypedWasmSudoku(module.get_wasm_sudoku());

    console.debug("Worker init done");
}
