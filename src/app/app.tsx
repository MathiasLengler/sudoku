import * as React from "react";
import {SudokuController} from "../../crate/pkg";
import {Sudoku} from "./sudoku";

interface AppProps {
  ctrl: SudokuController
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  return (
    <div>
      <Sudoku sudoku={props.ctrl.get_sudoku()}/>
    </div>
  )
};