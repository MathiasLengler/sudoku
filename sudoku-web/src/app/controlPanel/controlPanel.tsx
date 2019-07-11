import * as React from "react";
import {Selector} from "./selector";
import {Toolbar} from "./toolbar";
import {Input, WasmSudokuController} from "../wasmSudokuController";
import {ElementRef} from "../useResponsiveGridSize";

interface ControlPanelProps {
  sudokuController: WasmSudokuController;
  input: Input;
  sideLength: TransportSudoku['sideLength'];
  toolbarRef: ElementRef;
}

export const ControlPanel: React.FunctionComponent<ControlPanelProps> = (props) => {
  const {
    sudokuController, sideLength, input, toolbarRef
  } = props;
  return (
    <>
      <Toolbar
        sudokuController={sudokuController}
        input={input}
        toolbarRef={toolbarRef}
      />
      <Selector sudokuController={sudokuController}
                input={input}
                sideLength={sideLength}/>
    </>
  )
};
