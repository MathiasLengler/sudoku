import * as React from "react";

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
  return (
    <div>
      {props.sudoku}
    </div>
  )
};


interface CellProps {
  cell: TransportCell
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  return (
    <div>
      Cell
    </div>
  )
};