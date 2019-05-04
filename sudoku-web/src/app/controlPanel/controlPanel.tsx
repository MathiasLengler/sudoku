import * as React from "react";
import {onSelectorValue, Selector} from "./selector";
import {Actions} from "./actions";

interface ControlPanelProps {
  sideLength: TransportSudoku['sideLength'];
  onSelectorValue: onSelectorValue;
  candidateMode: boolean;
  setCandidateMode: React.Dispatch<React.SetStateAction<boolean>>;
}

export const ControlPanel: React.FunctionComponent<ControlPanelProps> = (props) => {
  const {onSelectorValue, sideLength, candidateMode, setCandidateMode} = props;
  return (
    <>
      <Actions candidateMode={candidateMode} setCandidateMode={setCandidateMode}/>
      <Selector sideLength={sideLength} onSelectorValue={onSelectorValue}/>
    </>
  )
};
