import * as React from "react";
import * as CSS from 'csstype';

interface SudokuProps {
  sudoku: TransportSudoku
}

export const Sudoku: React.FunctionComponent<SudokuProps> = (props) => {
  return (
    <div>
      <Grid sudoku={props.sudoku}/>
    </div>
  )
};


interface GridProps {
  sudoku: TransportSudoku
}

const Grid: React.FunctionComponent<GridProps> = (props) => {
  const style: CSS.Properties = {
    "--sideLength": props.sudoku.side_length,
    '--base': props.sudoku.base,
  };

  // TODO: add blocks
  return (
    <div className={"grid"} style={style}>
      {props.sudoku.cells.map((cell, i) =>
        <Cell key={i} cell={cell} base={props.sudoku.base}/>
      )}
    </div>
  )
};


interface CellProps {
  cell: TransportCell,
  base: TransportSudoku['base']
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  const style: CSS.Properties = {
    "--cell-column": props.cell.position.column,
    '--cell-row': props.cell.position.row,
  };

  return (
    <div className="cell" style={style}>
      {
        props.cell.value ?
          <div className="value"><span className="valueText">{props.cell.value}</span></div> :
          <Candidates candidates={props.cell.candidates} base={props.base}/>
      }
    </div>
  )
};

interface CandidatesProps {
  candidates: TransportCell['candidates'],
  base: TransportSudoku['base']
}

const Candidates: React.FunctionComponent<CandidatesProps> = (props) => {
  return (
    <div className="candidates">
      {
        props.candidates.map((candidate, i) => {
          const style: CSS.Properties = {
            "--candidate-column": (candidate - 1) % props.base,
            '--candidate-row': Math.floor((candidate - 1) / props.base),
          };

          return <div key={i} className="candidate" style={style}>
            {candidate}
          </div>
        })
      }
    </div>
  )
};

