import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "../res/styles.css";
import * as Comlink from "comlink";
import {WorkerApi} from "./worker";

(async () => {
  const worker = new Worker('./worker.js');

  const workerApi = Comlink.wrap<WorkerApi>(worker);

  console.debug(await workerApi.init());

  if (workerApi.typedWasmSudoku) {
    ReactDOM.render(<App wasmSudokuProxy={workerApi.typedWasmSudoku}/>, document.getElementById('root'));
  } else {
    throw new Error("Race condition while initializing wasm sudoku worker");
  }
})();
