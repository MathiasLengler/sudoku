import * as React from "react";
import {useEffect, useState} from "react";
import {Grid} from "./grid";
import {TypedWasmSudoku} from "../index";
import * as CSS from "csstype";
import {Selector} from "./selector";

// TODO: figure out grid updates
// TODO: guide highlighting (row/column/block)
// TODO: mark conflicting cells
// TODO: keyboard input
// TODO: refactor flat grid into blocks with cells
//  clean css margins with nested grid and no block border hack
// TODO: fix selection performance

interface AppProps {
  wasmSudoku: TypedWasmSudoku,
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.log("App render");

  const [sudoku, setSudoku] = useState(() => props.wasmSudoku.get_sudoku());

  const [selectedPos, setSelectedPos] = useState<CellPosition>(() => {
    return {column: 0, row: 0}
  });

  const sudokuController = new SudokuController(
    props.wasmSudoku,
    (sudoku) => setSudoku(sudoku)
  );

  useEffect(
    () => {
      let timer1 = setTimeout(() =>
        sudokuController.setValue({row: 1, column: 1}, 1), 1000);
      let timer2 = setTimeout(() =>
        sudokuController.setCandidates({row: 1, column: 0}, [1, 3, 5, 8]), 2000);

      return () => {
        clearTimeout(timer1);
        clearTimeout(timer2);
      }
    },
    []
  );

  const style: CSS.Properties = {
    '--sideLength': sudoku.side_length,
    '--base': sudoku.base,
  };

  return (
    <div className='sudoku' style={style}>
      <Grid sudoku={sudoku} selectedPos={selectedPos} setSelectedPos={setSelectedPos}/>
      <Selector side_length={sudoku.side_length}/>
    </div>
  )
};

class SudokuController {
  private rustSudoku: TypedWasmSudoku;
  private readonly onSudokuUpdate: (this: void, sudoku: TransportSudoku) => void;

  constructor(rustSudoku: TypedWasmSudoku, onSudokuUpdate: (this: void, sudoku: TransportSudoku) => void) {
    this.rustSudoku = rustSudoku;
    this.onSudokuUpdate = onSudokuUpdate;
  }

  private updateSudoku() {
    this.onSudokuUpdate(this.rustSudoku.get_sudoku())
  }

  private withSudokuUpdate<T>(f: () => T): T {
    let ret = f();

    this.updateSudoku();

    return ret;
  }

  setValue(pos: CellPosition, value: number): number {
    console.log("SudokuController", "setValue", pos, value);
    return this.withSudokuUpdate(() =>
      this.rustSudoku.setValue(pos, value));
  }

  setCandidates(pos: CellPosition, candidates: number[]) {
    console.log("SudokuController", "setCandidates", pos, candidates);
    return this.withSudokuUpdate(() =>
      this.rustSudoku.setCandidates(pos, candidates));
  }
}
