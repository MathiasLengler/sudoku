import * as Comlink from "comlink";
import {TypedWasmSudoku} from "./typedWasmSudoku";

export interface WorkerApi {
  init: typeof init;
  // We need to lie about the nullability of typedWasmSudoku
  // or else Comlink.Remote<TypedWasmSudoku> doesn't narrow
  typedWasmSudoku: TypedWasmSudoku;
}

const workerApi: WorkerApi = {
  init,
  typedWasmSudoku: undefined as unknown as TypedWasmSudoku
};

async function init() {
  workerApi.typedWasmSudoku = await getWasmSudoku();

  return "Worker initialized";
}

async function getWasmSudoku() {
  const module = await import("../../sudoku-wasm/pkg");
  module.run();
  return new TypedWasmSudoku(module.get_wasm_sudoku())
}

Comlink.expose(workerApi);
