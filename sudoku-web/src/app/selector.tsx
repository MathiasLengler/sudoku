import * as React from "react";
import {valuesFromSideLength} from "./utils";

export type onSelectorValue = (number: number) => void;

interface SelectorProps {
  side_length: TransportSudoku['side_length'],
  onSelectorValue: onSelectorValue,
}

export const Selector: React.FunctionComponent<SelectorProps> = (props) => {
  const {side_length, onSelectorValue} = props;

  return (
    <div className='selector'>
      {valuesFromSideLength(side_length)
        .map(value =>
          <SelectorValue key={value} value={value} onSelectorValue={onSelectorValue}/>
        )}
    </div>
  )
};

interface ValueProps {
  value: number,
  onSelectorValue: onSelectorValue,
}

const SelectorValue: React.FunctionComponent<ValueProps> = (props) => {
  const {value, onSelectorValue} = props;
  return (
    <div className='selectorValue'>
      <span className='selectorValueText' onClick={() => onSelectorValue(value)}>{value}</span>
    </div>
  );
};
