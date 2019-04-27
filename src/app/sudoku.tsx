import * as React from "react";
import {SudokuController} from "../../crate/pkg";
import {Grid} from "./grid";

// TODO: figure out grid updates
// TODO: cell selection
// TODO: number panel below grid

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