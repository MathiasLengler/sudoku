import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "../res/styles.css";
import {WasmSudoku} from "../../sudoku-wasm/pkg";

export class TypedWasmSudoku {
  public constructor(private rustSudoku: WasmSudoku) {
  }

  public sayHello(): void {
    return this.rustSudoku.say_hello()
  }

  public getSudoku(): TransportSudoku {
    return this.rustSudoku.get_sudoku()
  }

  public setValue(pos: CellPosition, value: number): number {
    return this.rustSudoku.set_value(pos, value);
  }

  public setCandidates(pos: CellPosition, candidates: number[]) {
    return this.rustSudoku.set_candidates(pos, candidates);
  }
}

import("../../sudoku-wasm/pkg").then(module => {
  module.run();

  const typedWasmSudoku = new TypedWasmSudoku(module.get_rust_sudoku());

  typedWasmSudoku.sayHello();

  ReactDOM.render(<App wasmSudoku={typedWasmSudoku}/>, document.getElementById('root'));
});
