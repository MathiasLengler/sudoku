import * as React from "react";
import {valuesFromSideLength, valueToString} from "../utils";
import {WasmSudokuController} from "../wasmSudokuController";
import ButtonBase from '@material-ui/core/ButtonBase';

interface SelectorProps {
  sudokuController: WasmSudokuController;
  sideLength: TransportSudoku['sideLength'];
}

export const Selector: React.FunctionComponent<SelectorProps> = (props) => {
  const {sideLength, sudokuController} = props;

  return (
    <div className='selector'>
      {valuesFromSideLength(sideLength)
        .map(value =>
          <SelectorValue key={value} value={value} sudokuController={sudokuController}/>
        )}
    </div>
  )
};

interface ValueProps {
  sudokuController: WasmSudokuController;
  value: number;
}

const SelectorValue: React.FunctionComponent<ValueProps> = (props) => {
  const {value, sudokuController} = props;

  const onClick = () => {
    sudokuController.handleValue(value);
  };

  return (
    <ButtonBase className='selectorValue' onClick={onClick}>
      <span className='selectorValueText'>{valueToString(value)}</span>
    </ButtonBase>
  );
};
