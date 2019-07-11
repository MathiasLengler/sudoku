import * as React from "react";
import {valuesFromSideLength, valueToString} from "../utils";
import {Input, WasmSudokuController} from "../wasmSudokuController";
import ButtonBase from '@material-ui/core/ButtonBase';
import classnames from "classnames";

interface SelectorProps {
  sudokuController: WasmSudokuController;
  sideLength: TransportSudoku['sideLength'];
  input: Input;
}

export const Selector: React.FunctionComponent<SelectorProps> = (props) => {
  const {sideLength, sudokuController, input: {stickyMode, selectedValue}} = props;

  return (
    <div className='selector'>
      {valuesFromSideLength(sideLength)
        .map(value =>
          <SelectorValue key={value} value={value} sudokuController={sudokuController}
                         selected={stickyMode && selectedValue === value}/>
        )}
    </div>
  )
};

interface ValueProps {
  sudokuController: WasmSudokuController;
  selected: boolean;
  value: number;
}

const SelectorValue: React.FunctionComponent<ValueProps> = (props) => {
  const {
    sudokuController,
    selected,
    value
  } = props;

  const buttonClassNames = classnames('selectorValue', {"selectorValue--selected": selected});

  const onClick = () => {
    sudokuController.handleValue(value);
  };

  return (
    <ButtonBase className={buttonClassNames} onClick={onClick}>
      <span className='selectorValueText'>{valueToString(value)}</span>
    </ButtonBase>
  );
};
