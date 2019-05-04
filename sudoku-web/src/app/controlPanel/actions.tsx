import * as React from "react";
import Switch from '@material-ui/core/Switch';
import FormControlLabel from "@material-ui/core/FormControlLabel";


interface ActionsProps {
  candidateMode: boolean;
  setCandidateMode: React.Dispatch<React.SetStateAction<boolean>>;
}

export const Actions: React.FunctionComponent<ActionsProps> = (props) => {
  const {candidateMode, setCandidateMode} = props;

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
    </div>
  )
};
