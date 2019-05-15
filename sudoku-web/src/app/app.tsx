import * as React from "react";
import {useState} from "react";
import {Grid} from "./grid/grid";
import * as CSS from "csstype";
import {WasmSudokuController} from "./wasmSudokuController";
import {useKeyboardInput} from "./useKeyboardInput";
import {ControlPanel} from "./controlPanel/controlPanel";
import {TypedWasmSudoku} from "../typedWasmSudoku";
import {useClientHeight, useResponsiveGridSize} from "./useResponsiveGridSize";
import CssBaseline from '@material-ui/core/CssBaseline';

interface AppProps {
  wasmSudoku: TypedWasmSudoku;
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  console.debug("App render");

  // State
  const [sudoku, setSudoku] = useState(() => {
    const sudoku = props.wasmSudoku.getSudoku();
    console.debug(sudoku);
    return sudoku;
  });

  const [selectedPos, setSelectedPos] = useState<CellPosition>(() => {
    return {column: 0, row: 0}
  });

  const [candidateMode, setCandidateMode] = useState(false);

  // Responsive Grid
  const [toolbarHeight, toolbarRef] = useClientHeight();

  const {base, sideLength} = sudoku;

  const gridSize = useResponsiveGridSize(toolbarHeight, sideLength);

  // Dependent on state
  const sudokuController = new WasmSudokuController(
    props.wasmSudoku,
    (sudoku) => setSudoku(sudoku),
    candidateMode,
    selectedPos,
    sideLength
  );

  useKeyboardInput(sudokuController, selectedPos, setSelectedPos, sideLength, candidateMode, setCandidateMode);

  const style: CSS.Properties = {
    '--sideLength': sideLength,
    '--base': base,
    "--outer-grid-size": `${gridSize}px`
  };

  return (
    <>
      <CssBaseline/>
      <div className='sudoku' style={style}>
        <Grid
          sudoku={sudoku}
          selectedPos={selectedPos}
          setSelectedPos={setSelectedPos}/>
        <ControlPanel
          sudokuController={sudokuController}
          sideLength={sideLength}
          candidateMode={candidateMode}
          setCandidateMode={setCandidateMode}
          toolbarRef={toolbarRef}
        />
      </div>
    </>
  )
};
