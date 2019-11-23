import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "../res/styles.css";
import {TypedWasmSudoku} from "./typedWasmSudoku";
import * as Comlink from "comlink";
import {WorkerApi} from "./worker";

(async () => {
  const worker = new Worker('./worker.js');

  const workerApi = Comlink.wrap(worker) as Comlink.Remote<WorkerApi>;

  const onReady = async () => {
    ReactDOM.render(<App wasmSudokuProxy={workerApi.typedWasmSudoku}/>, document.getElementById('root'));
  };
  await workerApi.setOnReady(Comlink.proxy(onReady));
})();