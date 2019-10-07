import * as React from "react";
import Dialog from '@material-ui/core/Dialog';
import {NewGameTabs} from "./tabs";
import useMediaQuery from '@material-ui/core/useMediaQuery';
import useTheme from "@material-ui/core/styles/useTheme";
import {WasmSudokuController} from "../../wasmSudokuController";

interface NewGameDialogProps {
  open: boolean;
  sudokuController: WasmSudokuController;
  onClose: () => void;
}

export const NewGameDialog: React.FunctionComponent<NewGameDialogProps> = (props) => {
  const {open, onClose, sudokuController} = props;

  const theme = useTheme();
  const fullScreen = useMediaQuery(theme.breakpoints.down('sm'));

  return (
    <Dialog
      open={open}
      onClose={onClose}
      fullWidth
      fullScreen={fullScreen}
    >
      <NewGameTabs sudokuController={sudokuController} onClose={onClose}/>
    </Dialog>
  );
};
