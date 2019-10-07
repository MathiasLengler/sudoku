import * as React from "react";
import IconButton from "@material-ui/core/IconButton";
import DeleteIcon from '@material-ui/icons/Delete';
import InfoIcon from '@material-ui/icons/Info';
import CreateIcon from '@material-ui/icons/Create';
import GestureIcon from '@material-ui/icons/Gesture';
import UndoIcon from '@material-ui/icons/Undo';
import {Input, WasmSudokuController} from "../wasmSudokuController";
import Tooltip from "@material-ui/core/Tooltip";
import {ElementRef} from "../useResponsiveGridSize";
import {ToolbarMenu} from "./toolbarMenu";

interface ToolbarProps {
  sudokuController: WasmSudokuController;
  input: Input;
  toolbarRef: ElementRef;
}

export const Toolbar: React.FunctionComponent<ToolbarProps> = (props) => {
  const {
    sudokuController, input: {
      candidateMode,
      stickyMode
    }, toolbarRef
  } = props;

  const enterDelay = 500;
  const leaveDelay = 200;

  return (
    <div className="toolbar" ref={toolbarRef}>
      <Tooltip title="Toggle candidate mode [space bar]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton color={candidateMode ? "primary" : "default"}
                    onClick={() => sudokuController.toggleCandidateMode()}>
          <CreateIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <Tooltip title="Toggle sticky mode [+]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton color={stickyMode ? "primary" : "default"} onClick={() => sudokuController.toggleStickyMode()}>
          <GestureIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <Tooltip title="Delete selected cell [delete]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton onClick={() => sudokuController.delete()}>
          <DeleteIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <Tooltip title="Set all direct candidates [insert]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton onClick={() => sudokuController.setAllDirectCandidates()}>
          <InfoIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <Tooltip title="Undo [backspace]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
        <IconButton onClick={() => sudokuController.undo()}>
          <UndoIcon fontSize="large"/>
        </IconButton>
      </Tooltip>
      <ToolbarMenu enterDelay={enterDelay} leaveDelay={leaveDelay} sudokuController={sudokuController}/>
    </div>
  )
};
