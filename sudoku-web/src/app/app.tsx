import * as React from "react";
import {useState} from "react";
import {Grid} from "./grid/grid";
import * as CSS from "csstype";
import {Input, WasmSudokuController} from "./wasmSudokuController";
import {ControlPanel} from "./controlPanel/controlPanel";
import {TypedWasmSudoku} from "../typedWasmSudoku";
import {useClientHeight, useResponsiveGridSize} from "./useResponsiveGridSize";
import CssBaseline from '@material-ui/core/CssBaseline';
import {blocksToCell} from "./utils";
import {makeKeyDownListener} from "./useKeyboardInput";

interface AppProps {
  wasmSudoku: TypedWasmSudoku;
}

export const App: React.FunctionComponent<AppProps> = (props) => {
  // State
  const [sudoku, setSudoku] = useState(() => {
    const sudoku = props.wasmSudoku.getSudoku();
    console.debug(sudoku);
    return sudoku;
  });

  const {blocks, base, sideLength} = sudoku;

  const [inputWithoutSelectedCell, setInput] = useState<Omit<Input, "selectedCell">>(() => {
    const selectedPos = {column: 0, row: 0};

    return {
      selectedPos,
      selectedValue: 1,
      stickyMode: false,
      candidateMode: false,
    };
  });

  const selectedCell = blocksToCell(blocks, inputWithoutSelectedCell.selectedPos, base);

  const input = {...inputWithoutSelectedCell, selectedCell};

  const sudokuController = new WasmSudokuController(
    props.wasmSudoku,
    sudoku,
    (sudoku) => setSudoku(sudoku),
    input,
    setInput,
    sideLength,
  );

  // Responsive Grid
  const [toolbarHeight, toolbarRef] = useClientHeight();
  const gridSize = useResponsiveGridSize(toolbarHeight, sideLength);

  const style: CSS.Properties = {
    '--sideLength': sideLength,
    '--base': base,
    "--outer-grid-size": `${gridSize}px`
  };

  return (
    <>
      <CssBaseline/>
      <div
        className='sudoku'
        style={style}
        onKeyDown={makeKeyDownListener(sudokuController, input, sideLength)}
        tabIndex={0}
      >
        <Grid
          sudokuController={sudokuController}
          input={input}
          sudoku={sudoku}
        />
        <ControlPanel
          sudokuController={sudokuController}
          input={input}
          sideLength={sideLength}
          toolbarRef={toolbarRef}
        />
      </div>
    </>
  )
};
