import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "./styles.css";
import {WasmSudoku} from "../../sudoku-wasm/pkg";

export class TypedWasmSudoku {
  private rustSudoku: WasmSudoku;

  constructor(rustSudoku: WasmSudoku) {
    this.rustSudoku = rustSudoku;
  }

  say_hello(): void {
    return this.rustSudoku.say_hello()
  }

  get_sudoku(): TransportSudoku {
    return this.rustSudoku.get_sudoku()
  }

  setValue(pos: CellPosition, value: number): number {
    return this.rustSudoku.set_value(pos, value);
  }

  setCandidates(pos: CellPosition, candidates: number[]) {
    return this.rustSudoku.set_candidates(pos, candidates);
  }
}

import("../../sudoku-wasm/pkg").then(module => {
  module.run();

  const typedWasmSudoku = new TypedWasmSudoku(module.get_rust_sudoku());

  typedWasmSudoku.say_hello();

  ReactDOM.render(<App wasmSudoku={typedWasmSudoku}/>, document.getElementById('root'));
});
