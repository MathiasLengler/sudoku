import * as Comlink from "comlink";
import {TypedWasmSudoku} from "./typedWasmSudoku";

let onReady = () => {
  console.error("onReady was not set");
};

const setOnReady = (cb: () => {}) => {
  onReady = cb;
};

const workerApi: WorkerApi = {
  setOnReady,
  typedWasmSudoku: undefined as any
};

export interface WorkerApi {
  setOnReady: typeof setOnReady;
  typedWasmSudoku: TypedWasmSudoku;
}

Comlink.expose(workerApi);

(async () => {
  const module = await import("../../sudoku-wasm/pkg");
  module.run();

  workerApi.typedWasmSudoku = new TypedWasmSudoku(module.get_wasm_sudoku());

  onReady();
})();
