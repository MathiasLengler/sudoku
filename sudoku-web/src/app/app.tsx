import * as React from "react";
import {useCallback, useState} from "react";
import {Grid} from "./grid/grid";
import {TypedWasmSudoku} from "../index";
import * as CSS from "csstype";
import {onSelectorValue} from "./controlPanel/selector";
import {WasmSudokuController} from "./controllers";
import {useDebugSetters, useKeyboardInput} from "./hooks";
import {ControlPanel} from "./controlPanel/controlPanel";


interface AppProps {
  wasmSudoku: TypedWasmSudoku;
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.log("App render");

  const [sudoku, setSudoku] = useState(() => props.wasmSudoku.getSudoku());

  const sudokuController = new WasmSudokuController(
    props.wasmSudoku,
    (sudoku) => setSudoku(sudoku),
  );

  const [selectedPos, setSelectedPos] = useState<CellPosition>(() => {
    return {column: 0, row: 0}
  });

  const onSelectorValue: onSelectorValue = useCallback(
    (selectorValue) => {
      sudokuController.setValue(selectedPos, selectorValue);
    },
    [sudokuController, selectedPos],
  );

  const {base, sideLength} = sudoku;

  useDebugSetters(sudokuController);
  useKeyboardInput(sudokuController, selectedPos, sideLength, setSelectedPos);

  const style: CSS.Properties = {
    '--sideLength': sideLength,
    '--base': base,
  };

  return (
    <div className='sudoku' style={style}>
      <Grid sudoku={sudoku} selectedPos={selectedPos} setSelectedPos={setSelectedPos}/>
      <ControlPanel sideLength={sideLength} onSelectorValue={onSelectorValue}/>
    </div>
  )
};
