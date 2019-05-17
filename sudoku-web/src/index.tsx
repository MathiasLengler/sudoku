import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "../res/styles.css";
import {TypedWasmSudoku} from "./typedWasmSudoku";

import("../../sudoku-wasm/pkg").then(module => {
  module.run();

  const typedWasmSudoku = new TypedWasmSudoku(module.get_wasm_sudoku());

  typedWasmSudoku.sayHello();

  ReactDOM.render(<App wasmSudoku={typedWasmSudoku}/>, document.getElementById('root'));
});
