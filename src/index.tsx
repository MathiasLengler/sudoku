import * as React from "react";
import * as ReactDOM from "react-dom";
import {App} from './app/app';
import "./styles.css";

import("../crate/pkg").then(module => {
  module.run();

  const ctrl = module.get_sudoku_controller();

  ctrl.say_hello();

  const sudoku = ctrl.get_sudoku() as TransportSudoku;

  console.log(sudoku);

  ReactDOM.render(<App ctrl={ctrl}/>, document.getElementById('root'));
});
