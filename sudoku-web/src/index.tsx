import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "../res/styles.css";
import * as Comlink from "comlink";
import {WorkerApi} from "./worker";
import {TypedWasmSudoku} from "./typedWasmSudoku";

(async () => {
  // const worker = new Worker('./worker.tsx', {name: 'worker', type: 'module'});
  const worker = new Worker(new URL("./worker.tsx", import.meta.url));
  const workerApi = Comlink.wrap<WorkerApi>(worker);

  console.debug(await workerApi.init());

  if (workerApi.typedWasmSudoku) {
    ReactDOM.render(<App
      wasmSudokuProxy={workerApi.typedWasmSudoku as unknown as Comlink.Remote<TypedWasmSudoku>}/>, document.getElementById('root'));
  } else {
    throw new Error("Race condition while initializing wasm sudoku worker");
  }
})();

if ('serviceWorker' in navigator) {
  window.addEventListener('load', () => {
    navigator.serviceWorker.register('service-worker.js').then(registration => {
      console.log('SW registered: ', registration);
    }).catch(registrationError => {
      console.log('SW registration failed: ', registrationError);
    });
  });
}