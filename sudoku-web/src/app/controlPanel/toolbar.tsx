import * as React from "react";
import IconButton from "@material-ui/core/IconButton";
import DeleteIcon from '@material-ui/icons/Delete';
import InfoIcon from '@material-ui/icons/Info';
import NoteIcon from '@material-ui/icons/Note';
import {WasmSudokuController} from "../wasmSudokuController";
import Tooltip from "@material-ui/core/Tooltip";
import {ElementRef} from "../useResponsiveGridSize";


interface ToolbarProps {
  sudokuController: WasmSudokuController;
  candidateMode: boolean;
  setCandidateMode: React.Dispatch<React.SetStateAction<boolean>>;
  toolbarRef: ElementRef;
}

export const Toolbar: React.FunctionComponent<ToolbarProps> = (props) => {
  const {sudokuController, candidateMode, setCandidateMode, toolbarRef} = props;

  const enterDelay = 500;
  const leaveDelay = 200;

  return (
    <div className="toolbar" ref={toolbarRef}>
      <Tooltip title="Toggle candidate mode" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton color={candidateMode ? "primary": "default"} onClick={() => setCandidateMode(!candidateMode)}>
          <NoteIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <Tooltip title="Delete selected cell" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton onClick={() => sudokuController.delete()}>
          <DeleteIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <Tooltip title="Set all direct candidates" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton onClick={() => sudokuController.setAllDirectCandidates()}>
          <InfoIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
    </div>
  )
};
