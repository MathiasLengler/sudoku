import * as React from "react";
import * as CSS from "csstype";
import isEqual from "lodash/isEqual";
import classnames from 'classnames'
import {indexToPosition} from "../utils";

interface CellProps {
  blockCellIndex: number;
  cell: TransportCell;
  base: TransportSudoku['base'];
  selected: boolean;
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>;
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  console.log("Cell render", props);

  const {
    blockCellIndex,
    cell,
    base,
    selected,
    setSelectedPos
  } = props;

  const {position: gridPosition} = cell;

  const blockCellPosition = indexToPosition(blockCellIndex, base);

  if (selected) {
    console.log("Selected:", "gridPosition", gridPosition, "blockCellPosition", blockCellPosition);
  }

  const style: CSS.Properties = {
    '--cell-column': blockCellPosition.column,
    '--cell-row': blockCellPosition.row,
  };

  let cellClassNames = classnames("cell", {"cell--selected": selected});

  return (
    <div className={cellClassNames} style={style} onClick={() => setSelectedPos(gridPosition)}>
      {
        cell.kind === "value" ?
          <CellValue value={cell.value}/> :
          <Candidates candidates={cell.candidates} base={base}/>
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
  return <div className='cellValue'><span className='cellValueText'>{value}</span></div>;
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
            {candidate}
          </span>
        })
      }
    </div>
  )
};
