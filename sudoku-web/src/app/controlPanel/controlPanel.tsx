import * as React from "react";
import {Selector} from "./selector";
import {Toolbar} from "./toolbar";
import {WasmSudokuController} from "../wasmSudokuController";

interface ControlPanelProps {
  sudokuController: WasmSudokuController;
  sideLength: TransportSudoku['sideLength'];
  candidateMode: boolean;
  setCandidateMode: React.Dispatch<React.SetStateAction<boolean>>;
}

export const ControlPanel: React.FunctionComponent<ControlPanelProps> = (props) => {
  const {sudokuController, sideLength, candidateMode, setCandidateMode} = props;
  return (
    <>
      <Toolbar candidateMode={candidateMode} setCandidateMode={setCandidateMode} sudokuController={sudokuController}/>
      <Selector sudokuController={sudokuController} sideLength={sideLength}/>
    </>
  )
};
