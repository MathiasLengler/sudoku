import * as React from "react";
import {Selector} from "./selector";
import {Toolbar} from "./toolbar";
import {WasmSudokuController} from "../wasmSudokuController";
import {ElementRef} from "../useResponsiveGridSize";

interface ControlPanelProps {
  sudokuController: WasmSudokuController;
  sideLength: TransportSudoku['sideLength'];
  candidateMode: boolean;
  setCandidateMode: React.Dispatch<React.SetStateAction<boolean>>;
  toolbarRef: ElementRef;
}

export const ControlPanel: React.FunctionComponent<ControlPanelProps> = (props) => {
  const {
    sudokuController, sideLength, candidateMode, setCandidateMode, toolbarRef
  } = props;
  return (
    <>
      <Toolbar
        candidateMode={candidateMode}
        setCandidateMode={setCandidateMode}
        sudokuController={sudokuController}
        toolbarRef={toolbarRef}
      />
      <Selector sudokuController={sudokuController} sideLength={sideLength}/>
    </>
  )
};
