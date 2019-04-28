import * as React from "react";
import {useState} from "react";
import {Sudoku} from "./sudoku";
import {TypedRustSudoku} from "../index";

// TODO: figure out grid updates
// TODO: cell selection
//  guide highlighting (row/column/block)
// TODO: mark conflicting cells
// TODO: number selection panel
// TODO: keyboard input
// TODO: refactor flat grid into blocks with cells
//  clean css margins with nested grid and no block border hack

interface AppProps {
  rustSudoku: TypedRustSudoku,
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  const [sudoku, setSudoku] = useState(() => props.rustSudoku.get_sudoku());

  const ctrl = new SudokuController(props.rustSudoku, (sudoku) => setSudoku(sudoku));

  return (
    <div className='sudoku'>
      <Sudoku sudoku={sudoku}/>
      {/*  NumberPanel*/}
    </div>
  )
};

class SudokuController {
  private rustSudoku: TypedRustSudoku;
  private on_update: (this: void, sudoku: TransportSudoku) => void;

  constructor(rustSudoku: TypedRustSudoku, on_update: (this: void, sudoku: TransportSudoku) => void) {
    this.rustSudoku = rustSudoku;
    this.on_update = on_update;

    setTimeout(() => this.setValue(), 2000);
  }

  private update() {
    this.on_update(this.rustSudoku.get_sudoku())
  }

  setValue() {
    this.rustSudoku.set_value({row: 1, column: 1}, 1);

    this.update();
  }
}