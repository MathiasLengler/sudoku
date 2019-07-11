import * as React from "react";
import {PointerEventHandler} from "react";
import * as CSS from "csstype";
import isEqual from "lodash/isEqual";
import classnames from "classnames";
import {indexToPosition, valueToString} from "../utils";
import {WasmSudokuController} from "../wasmSudokuController";

function cellBackgroundClass(selected: boolean, guideValue: boolean, guideGroup: boolean, guideValueGroup: boolean) {
  if (selected) {
    return "cell--selected";
  }
  if (guideValue) {
    return "cell--guide-value";
  }
  if (guideGroup) {
    return "cell--guide-group";
  }
  if (guideValueGroup) {
    return "cell--guide-value-group";
  }
}

function cellColorClass(fixed: boolean) {
  if (fixed) {
    return "cell--fixed";
  } else {
    return "cell--user";
  }
}

interface CellProps {
  blockCellIndex: number;
  cell: TransportCell;
  base: TransportSudoku['base'];
  selected: boolean;
  sudokuController: WasmSudokuController;
  guideGroup: boolean;
  guideValue: boolean;
  guideValueGroup: boolean;
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  const {
    blockCellIndex,
    cell,
    base,
    selected,
    sudokuController,
    guideGroup,
    guideValue,
    guideValueGroup
  } = props;

  const {position: gridPosition} = cell;

  const blockCellPosition = indexToPosition(blockCellIndex, base);

  const style: CSS.Properties = {
    '--cell-column': blockCellPosition.column,
    '--cell-row': blockCellPosition.row,
  };

  let cellClassNames = classnames(
    "cell",
    cellBackgroundClass(selected, guideValue, guideGroup, guideValueGroup),
    cellColorClass(cell.fixed),
  );

  const onPointerMove: PointerEventHandler = (e) => {
    // Left Mouse, Touch Contact, Pen contact
    if (e.buttons !== 1) {
      return;
    }

    sudokuController.handlePosition(gridPosition, true);

    // Workaround for touch drag cell selection
    if (e.pointerType !== "mouse") {
      let el = document.elementFromPoint(e.clientX, e.clientY);
      if (el) {
        while (el.parentElement !== null) {
          if (el.classList.contains("cell")) {
            el.setPointerCapture(e.pointerId);
            break;
          }
          el = el.parentElement;
        }
      }
    }
  };

  return (
    <div className={cellClassNames}
         style={style}
         onPointerDown={() => sudokuController.handlePosition(gridPosition)}
         onPointerMove={onPointerMove}
    >
      {
        cell.kind === "value" ?
          <CellValue value={cell.value}/> :
          <MemoCandidates candidates={cell.candidates} base={base}/>
      }
    </div>
  )
};
export const MemoCell = React.memo(Cell, isEqual);

interface CellValueProps {
  value: ValueCell['value'];
}

const CellValue: React.FunctionComponent<CellValueProps> = (props) => {
  const {value} = props;
  return <div className='cellValue'><span className='cellValueText'>{valueToString(value)}</span></div>;
};


interface CandidatesProps {
  candidates: CandidatesCell['candidates'];
  base: TransportSudoku['base'];
}

const Candidates: React.FunctionComponent<CandidatesProps> = (props) => {
  const {base} = props;

  return (
    <div className='candidates'>
      {
        props.candidates.map((candidate, i) => {
          // Candidates are 1 based, grid calculations are 0 based.
          const {column, row} = indexToPosition(candidate - 1, base);

          const style: CSS.Properties = {
            '--candidate-column': column,
            '--candidate-row': row,
          };

          return <span key={i} className='candidate' style={style}>
            {valueToString(candidate)}
          </span>
        })
      }
    </div>
  )
};

export const MemoCandidates = React.memo(Candidates, isEqual);
