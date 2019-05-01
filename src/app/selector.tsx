import * as React from "react";

interface SelectorProps {
  side_length: TransportSudoku['side_length'],
}

export const Selector: React.FunctionComponent<SelectorProps> = (props) => {
  const {side_length} = props;

  // TODO:
  const on_click = (number: number) => {
    console.log(number)
  };

  return (
    <div className='selector'>
      {Array.from(Array(side_length).keys())
        .map(value =>
          <SelectorValue key={value} value={value + 1} on_click={on_click}/>
        )}
    </div>
  )
};

interface ValueProps {
  value: number,
  on_click: (number: number) => void,
}

const SelectorValue: React.FunctionComponent<ValueProps> = (props) => {
  const {value, on_click} = props;
  return (
    <div className='selectorValue'>
      <span className='selectorValueText' onClick={() => on_click(value)}>{value}</span>
    </div>
  );
};