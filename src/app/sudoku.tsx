import * as React from "react";
import {SudokuController} from "../../crate/pkg";
import {Grid} from "./grid";

// TODO: figure out grid updates
// TODO: cell selection
//  guide highlighting (row/column/block)
// TODO: mark conflicting cells
// TODO: number selection panel
// TODO: keyboard input
// TODO: refactor flat grid into blocks with cells
//  clean css margins with nested grid and no block border hack

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