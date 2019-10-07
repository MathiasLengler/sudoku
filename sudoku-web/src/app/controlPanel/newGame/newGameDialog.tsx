import * as React from "react";
import Dialog from '@material-ui/core/Dialog';
import {NewGameTabs} from "./tabs";
import useMediaQuery from '@material-ui/core/useMediaQuery';
import useTheme from "@material-ui/core/styles/useTheme";
import makeStyles from "@material-ui/core/styles/makeStyles";
import {WasmSudokuController} from "../../wasmSudokuController";

interface NewGameDialogProps {
  open: boolean;
  sudokuController: WasmSudokuController;
  onClose: () => void;
}

const useStyles = makeStyles({
  dialogPaper: {
    // minHeight: '80vh',
    // maxHeight: '80vh',
  },
});

export const NewGameDialog: React.FunctionComponent<NewGameDialogProps> = (props) => {
  const {open, onClose, sudokuController} = props;

  const theme = useTheme();
  const fullScreen = useMediaQuery(theme.breakpoints.down('sm'));
  const classes = useStyles();

  return (
    <Dialog
      open={open}
      onClose={onClose}
      fullWidth
      fullScreen={fullScreen}
      classes={fullScreen ? {} : {paper: classes.dialogPaper}}
    >
      <NewGameTabs sudokuController={sudokuController} onClose={onClose}/>
    </Dialog>
  );
};
