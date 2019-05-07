import * as React from "react";
import {useState} from "react";
import {Grid} from "./grid/grid";
import * as CSS from "csstype";
import {WasmSudokuController} from "./wasmSudokuController";
import {useKeyboardInput} from "./useKeyboardInput";
import {ControlPanel} from "./controlPanel/controlPanel";
import {TypedWasmSudoku} from "../typedWasmSudoku";


interface AppProps {
  wasmSudoku: TypedWasmSudoku;
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.log("App render");

  // State
  const [sudoku, setSudoku] = useState(() => {
    const sudoku = props.wasmSudoku.getSudoku();
    console.log(sudoku);
    return sudoku;
  });

  const [selectedPos, setSelectedPos] = useState<CellPosition>(() => {
    return {column: 0, row: 0}
  });

  const [candidateMode, setCandidateMode] = useState(false);

  // Dependent on state
  const sudokuController = new WasmSudokuController(
    props.wasmSudoku,
    (sudoku) => setSudoku(sudoku),
    candidateMode,
    selectedPos
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
        sudokuController={sudokuController}
        sideLength={sideLength}
        candidateMode={candidateMode}
        setCandidateMode={setCandidateMode}/>
    </div>
  )
};
