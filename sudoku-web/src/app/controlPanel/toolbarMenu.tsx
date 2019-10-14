import * as React from "react";
import Menu from "@material-ui/core/Menu";
import MenuItem from "@material-ui/core/MenuItem";
import {NewGameDialog} from "./newGame/newGameDialog";
import IconButton from "@material-ui/core/IconButton";
import MoreVertIcon from '@material-ui/icons/MoreVert';
import Tooltip from "@material-ui/core/Tooltip";
import {WasmSudokuController} from "../wasmSudokuController";

interface ToolbarMenuProps {
  enterDelay: number;
  leaveDelay: number;
  sudokuController: WasmSudokuController;
}

export const ToolbarMenu: React.FunctionComponent<ToolbarMenuProps> = (props) => {
  const {enterDelay, leaveDelay, sudokuController} = props;

  const [menuAnchorEl, setMenuAnchorEl] = React.useState<null | HTMLElement>(null);

  const [newGameOpen, setNewGameOpen] = React.useState(false);

  const makeHandleMenuClose = (action: () => void = () => {
  }) => () => {
    setMenuAnchorEl(null);
    action();
  };

  return <>
    <Tooltip title="Menu" enterDelay={enterDelay} leaveDelay={leaveDelay}>
      <IconButton onClick={(e) => setMenuAnchorEl(e.currentTarget)}>
        <MoreVertIcon fontSize="large"/>
      </IconButton>
    </Tooltip>
    <Menu
      open={!!menuAnchorEl}
      anchorEl={menuAnchorEl}
      keepMounted
      onClose={makeHandleMenuClose()}
    >
      <MenuItem onClick={makeHandleMenuClose(() => setNewGameOpen(true))}>New Game</MenuItem>
      <MenuItem onClick={makeHandleMenuClose(() => sudokuController.solveSingleCandidates())}>
        Solver: single candidates
      </MenuItem>
      <MenuItem onClick={makeHandleMenuClose(() => sudokuController.groupReduction())}>
        Solver: group reduction
      </MenuItem>
    </Menu>
    <div id="dialogs" tabIndex={0} onKeyDown={(e) => {
      // Disable global game shortcuts in dialog boxes.
      e.stopPropagation();
    }}>
      <NewGameDialog open={newGameOpen} onClose={() => setNewGameOpen(false)} sudokuController={sudokuController}/>
    </div>
  </>
};