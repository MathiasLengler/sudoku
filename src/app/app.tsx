import * as React from "react";
import {useState} from "react";
import {Grid} from "./grid";
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

  const _ctrl = new SudokuController(props.rustSudoku, (sudoku) => setSudoku(sudoku));

  return (
    <div className='sudoku'>
      <Grid sudoku={sudoku}/>
      <ValueSelector side_length={sudoku.side_length}/>
    </div>
  )
};

class SudokuController {
  private rustSudoku: TypedRustSudoku;
  private readonly on_update: (this: void, sudoku: TransportSudoku) => void;

  constructor(rustSudoku: TypedRustSudoku, on_update: (this: void, sudoku: TransportSudoku) => void) {
    this.rustSudoku = rustSudoku;
    this.on_update = on_update;

    setTimeout(() => this.setValue({row: 1, column: 1}, 1), 2000);
  }

  private update() {
    this.on_update(this.rustSudoku.get_sudoku())
  }

  setValue(pos: CellPosition, value: number) {
    this.rustSudoku.set_value(pos, value);

    this.update();
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