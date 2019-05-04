import * as React from "react";
import Switch from '@material-ui/core/Switch';
import FormControlLabel from "@material-ui/core/FormControlLabel";
import IconButton from "@material-ui/core/IconButton";
import DeleteIcon from '@material-ui/icons/Delete';
import {WasmSudokuController} from "../wasmSudokuController";


interface ToolbarProps {
  sudokuController: WasmSudokuController;
  candidateMode: boolean;
  setCandidateMode: React.Dispatch<React.SetStateAction<boolean>>;
}

export const Toolbar: React.FunctionComponent<ToolbarProps> = (props) => {
  const {sudokuController, candidateMode, setCandidateMode} = props;

  return (
    <div className='actions'>
      <FormControlLabel
        control={
          <Switch
            checked={candidateMode}
            onChange={(event, checked) => setCandidateMode(checked)}
          />
        }
        label="candidate"
      />
      <IconButton onClick={() => sudokuController.delete()} aria-label="Delete Cell">
        <DeleteIcon />
      </IconButton>
    </div>
  )
};
