import * as React from "react";

export type onSelectorValue = (number: number) => void;

interface SelectorProps {
  side_length: TransportSudoku['side_length'],
  onSelectorValue: onSelectorValue,
}

export const Selector: React.FunctionComponent<SelectorProps> = (props) => {
  const {side_length, onSelectorValue} = props;

  return (
    <div className='selector'>
      {Array.from(Array(side_length).keys())
        .map(value =>
          <SelectorValue key={value} value={value + 1} onSelectorValue={onSelectorValue}/>
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