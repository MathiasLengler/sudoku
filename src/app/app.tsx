import * as React from "react";
import {useCallback, useEffect, useState} from "react";
import {Grid} from "./grid";
import {TypedWasmSudoku} from "../index";
import * as CSS from "csstype";
import {onSelectorValue, Selector} from "./selector";
import {WasmSudokuController} from "./controllers";


interface AppProps {
  wasmSudoku: TypedWasmSudoku,
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.log("App render");

  const [sudoku, setSudoku] = useState(() => props.wasmSudoku.get_sudoku());

  const sudokuController = new WasmSudokuController(
    props.wasmSudoku,
    (sudoku) => setSudoku(sudoku),
  );

  const [selectedPos, setSelectedPos] = useState<CellPosition>(() => {
    return {column: 0, row: 0}
  });

  // TODO: abstraction?
  const onSelectorValue: onSelectorValue = useCallback(
    (selectorValue) => {
      sudokuController.setValue(selectedPos, selectorValue);
    },
    [sudokuController, selectedPos],
  );

  useEffect(() => {
    const keyDownListener = (ev: KeyboardEvent) => {
      const value = keyToValue(ev.key);

      if (value !== undefined) {
        console.log("keyDownListener", value);

        sudokuController.setValue(selectedPos, value)
      }
    };

    document.addEventListener('keydown', keyDownListener);

    return () => {
      document.removeEventListener('keydown', keyDownListener);
    };
  }, [sudokuController, selectedPos]);

  // Debug setters
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

  const {base, side_length} = sudoku;

  const style: CSS.Properties = {
    '--sideLength': side_length,
    '--base': base,
  };

  return (
    <div className='sudoku' style={style}>
      <Grid sudoku={sudoku} selectedPos={selectedPos} setSelectedPos={setSelectedPos}/>
      <Selector side_length={side_length} onSelectorValue={onSelectorValue}/>
    </div>
  )
};

const keyToValue = (key: string): number | undefined => {
  const value = parseInt(key);

  if (Number.isInteger(value)) {
    return value
  } else {
    return undefined;
  }
};

