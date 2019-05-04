import * as React from "react";
import * as CSS from "csstype";
import {isEqual} from "lodash";
import * as classnames from 'classnames'
import {indexToPosition} from "./utils";

interface CellProps {
  cell: TransportCell;
  base: TransportSudoku['base'];
  selected: boolean;
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>;
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  console.log("Cell render", props);

  const {
    cell: {position, candidates, value},
    base,
    selected,
    setSelectedPos
  } = props;

  if (selected) {
    console.log("Selected:", position);
  }

  const style: CSS.Properties = {
    '--cell-column': position.column,
    '--cell-row': position.row,
  };

  let cellClassNames = classnames("cell", {"cell--selected": selected});


  return (
    <div className={cellClassNames} style={style} onClick={() => setSelectedPos(position)}>
      {
        value ?
          <CellValue  value={value}/>:
          <Candidates candidates={candidates} base={base}/>
      }
    </div>
  )
};
export const MemoCell = React.memo(Cell, isEqual);

interface CellValueProps {
  value: TransportCell['value'];
}

const CellValue: React.FunctionComponent<CellValueProps> = (props) => {
  const {value} = props;
  return <div className='cellValue'><span className='cellValueText'>{value}</span></div>;
};


interface CandidatesProps {
  candidates: TransportCell['candidates'];
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
