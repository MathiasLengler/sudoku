import * as React from "react";
import {onSelectorValue, Selector} from "./selector";

interface ControlPanelProps {
  sideLength: TransportSudoku['sideLength'];
  onSelectorValue: onSelectorValue;
}

export const ControlPanel: React.FunctionComponent<ControlPanelProps> = (props) => {
  const {onSelectorValue, sideLength} = props;
  return (
    <>
      <Selector sideLength={sideLength} onSelectorValue={onSelectorValue}/>
    </>
  )
};
