import * as React from "react";
import {SudokuController} from "../../crate/pkg";
import {Grid} from "./grid";

// TODO: figure out grid updates
// TODO: history?
// TODO: cell selection
// TODO: highlight conflicting cells
// TODO: number panel below grid
// TODO: refactor flat grid into blocks with cells
//  (clean css margins with nested grid and no block border hack)

interface AppProps {
  ctrl: SudokuController
}

export const Sudoku: React.FunctionComponent<AppProps> = (props) => {
  return (
    <div className='sudoku'>
      <Grid sudoku={props.ctrl.get_sudoku()}/>
    {/*  NumberPanel*/}
    </div>
  )
};