import * as React from "react";
import {SudokuController} from "../../crate/pkg";
import {Sudoku} from "./sudoku";

// TODO: figure out grid updates
// TODO: cell selection
// TODO: number panel below grid

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