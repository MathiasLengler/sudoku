import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "./styles.css";
import {RustSudoku} from "../crate/pkg";

export class TypedRustSudoku {
  private rustSudoku: RustSudoku;

  constructor(rustSudoku: RustSudoku) {
    this.rustSudoku = rustSudoku;
  }

  say_hello(): void {
    return this.rustSudoku.say_hello()
  }

  get_sudoku(): TransportSudoku {
    return this.rustSudoku.get_sudoku()
  }

  set_value(pos: CellPosition, value: number): void {
    return this.rustSudoku.set_value(pos, value);
  }
}

import("../crate/pkg").then(module => {
  module.run();

  const typedRustSudoku = new TypedRustSudoku(module.get_rust_sudoku());

  typedRustSudoku.say_hello();

  ReactDOM.render(<App rustSudoku={typedRustSudoku}/>, document.getElementById('root'));
});
