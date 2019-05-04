import * as React from "react";
import {valuesFromSideLength} from "./utils";

export type onSelectorValue = (number: number) => void;

interface SelectorProps {
  sideLength: TransportSudoku['sideLength'];
  onSelectorValue: onSelectorValue;
}

export const Selector: React.FunctionComponent<SelectorProps> = (props) => {
  const {sideLength, onSelectorValue} = props;

  return (
    <div className='selector'>
      {valuesFromSideLength(sideLength)
        .map(value =>
          <SelectorValue key={value} value={value} onSelectorValue={onSelectorValue}/>
        )}
    </div>
  )
};

interface ValueProps {
  value: number;
  onSelectorValue: onSelectorValue;
}

const SelectorValue: React.FunctionComponent<ValueProps> = (props) => {
  const {value, onSelectorValue} = props;
  return (
    <div className='selectorValue'>
      <span className='selectorValueText' onClick={() => onSelectorValue(value)}>{value}</span>
    </div>
  );
};
