import * as React from "react";
import {useState} from "react";
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
  const [sudoku, setSudoku] = useState(() => props.wasmSudoku.get_sudoku());

  const _ctrl = new SudokuController(props.wasmSudoku, (sudoku) => setSudoku(sudoku));

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

    setTimeout(() => this.setValue({row: 1, column: 1}, 1), 1000);
    setTimeout(() => this.setCandidates({row: 1, column: 0}, [1,3,5,8]), 2000);
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
    return this.with_update(() =>
      this.rustSudoku.setValue(pos, value));
  }

  setCandidates(pos: CellPosition, candidates: number[]) {
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