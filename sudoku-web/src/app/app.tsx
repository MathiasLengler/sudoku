import * as React from "react";
import {useCallback, useState} from "react";
import {Grid} from "./grid/grid";
import * as CSS from "csstype";
import {onSelectorValue} from "./controlPanel/selector";
import {WasmSudokuController} from "./controllers";
import {useKeyboardInput} from "./hooks";
import {ControlPanel} from "./controlPanel/controlPanel";
import {TypedWasmSudoku} from "../typedWasmSudoku";


interface AppProps {
  wasmSudoku: TypedWasmSudoku;
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.log("App render");

  // State
  const [sudoku, setSudoku] = useState(() => props.wasmSudoku.getSudoku());

  const [selectedPos, setSelectedPos] = useState<CellPosition>(() => {
    return {column: 0, row: 0}
  });

  const [candidateMode, setCandidateMode] = useState(false);

  // Dependent on state
  const sudokuController = new WasmSudokuController(props.wasmSudoku, (sudoku) => setSudoku(sudoku), candidateMode, selectedPos);

  const onSelectorValue: onSelectorValue = useCallback(
    (selectorValue) => {
      sudokuController.handleValue(selectorValue);
    },
    [sudokuController],
  );

  const {base, sideLength} = sudoku;

  useKeyboardInput(sudokuController, selectedPos, setSelectedPos, sideLength);

  const style: CSS.Properties = {
    '--sideLength': sideLength,
    '--base': base,
  };

  return (
    <div className='sudoku' style={style}>
      <Grid
        sudoku={sudoku}
        selectedPos={selectedPos}
        setSelectedPos={setSelectedPos}/>
      <ControlPanel
        sideLength={sideLength}
        onSelectorValue={onSelectorValue}
        candidateMode={candidateMode}
        setCandidateMode={setCandidateMode}/>
    </div>
  )
};
