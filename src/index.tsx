import * as React from "react";
import * as ReactDOM from "react-dom";
import {Sudoku} from './app/sudoku';
import "./styles.css";

import("../crate/pkg").then(module => {
  module.run();

  const ctrl = module.get_sudoku_controller();

  ctrl.say_hello();

  const sudoku = ctrl.get_sudoku() as TransportSudoku;

  console.log(sudoku);

  ReactDOM.render(<Sudoku ctrl={ctrl}/>, document.getElementById('root'));
});
