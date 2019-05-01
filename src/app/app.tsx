import * as React from "react";
import {useEffect, useState} from "react";
import {Grid} from "./grid";
import {TypedWasmSudoku} from "../index";

// TODO: figure out grid updates
// TODO: cell selection
//  guide highlighting (row/column/block)
// TODO: mark conflicting cells
// TODO: number selection panel
// TODO: keyboard input
// TODO: refactor flat grid into blocks with cells
//  clean css margins with nested grid and no block border hack

interface AppProps {
  wasmSudoku: TypedWasmSudoku,
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.log("App render");

  const [sudoku, setSudoku] = useState(() => props.wasmSudoku.get_sudoku());

  const ctrl = new SudokuController(props.wasmSudoku, (sudoku) => setSudoku(sudoku));

  useEffect(
    () => {
      let timer1 = setTimeout(() =>
        ctrl.setValue({row: 1, column: 1}, 1), 5000);
      let timer2 = setTimeout(() =>
        ctrl.setCandidates({row: 1, column: 0}, [1,3,5,8]), 10000);

      return () => {
        clearTimeout(timer1);
        clearTimeout(timer2);
      }
    },
    [] //useEffect will run only one time
  );

  return (
    <div className='sudoku'>
      <Grid sudoku={sudoku}/>
      <ValueSelector side_length={sudoku.side_length}/>
    </div>
  )
};

class SudokuController {
  private rustSudoku: TypedWasmSudoku;
  private readonly onUpdate: (this: void, sudoku: TransportSudoku) => void;

  constructor(rustSudoku: TypedWasmSudoku, onUpdate: (this: void, sudoku: TransportSudoku) => void) {
    this.rustSudoku = rustSudoku;
    this.onUpdate = onUpdate;
  }

  private update() {
    this.onUpdate(this.rustSudoku.get_sudoku())
  }

  private with_update<T>(f: () => T): T {
    let ret = f();

    this.update();

    return ret;
  }

  setValue(pos: CellPosition, value: number): number {
    console.log("SudokuController", "setValue", pos, value);
    return this.with_update(() =>
      this.rustSudoku.setValue(pos, value));
  }

  setCandidates(pos: CellPosition, candidates: number[]) {
    console.log("SudokuController", "setCandidates", pos, candidates);
    return this.with_update(() =>
      this.rustSudoku.setCandidates(pos, candidates));
  }
}

// TODO: move to module
interface ValueSelectorProps {
  side_length: TransportSudoku['side_length'],
}


const ValueSelector: React.FunctionComponent<ValueSelectorProps> = (props) => {
  const {side_length} = props;

  // TODO:
  const on_click = (number: number) => {
    console.log(number)
  };

  return (
    <div className='numberPanel'>
      {Array.from(Array(side_length).keys())
        .map(value =>
          <Value key={value}  number={value + 1} on_click={on_click}/>
        )}
    </div>
  )
};

interface ValueProps {
  number: number,
  on_click: (number: number) => void,
}

const Value: React.FunctionComponent<ValueProps> = (props) => {
  const {number, on_click} = props;
  return (
    <span className='number' onClick={() => on_click(number)}>{number}</span>
  );
};