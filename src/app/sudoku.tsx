import * as React from 'react';
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
  const {cells, base, side_length} = props.sudoku;

  const style: CSS.Properties = {
    '--sideLength': side_length,
    '--base': base,
  };

  return (
    <div className='gridContainer'>
      <div className='grid' style={style}>
        <Blocks side_length={side_length} base={base}/>
        {cells.map((cell, i) =>
          <Cell key={i} cell={cell} base={base}/>
        )}
      </div>
    </div>
  )
};

interface BlocksProps {
  side_length: TransportSudoku['side_length'],
  base: TransportSudoku['base'],
}

const Blocks: React.FunctionComponent<BlocksProps> = (props) => {
  const {side_length, base} = props;
  return (
    <>
      {
        Array.from(Array(side_length).keys())
          .map(block_index =>
            <Block key={block_index} base={base} block_index={block_index}/>
          )
      }
    </>
  )
};


interface BlockProps {
  base: TransportSudoku['base'],
  block_index: number,
}

const Block: React.FunctionComponent<BlockProps> = (props) => {
  const {base, block_index} = props;

  const {column, row} = indexToPosition(block_index, base);

  const style: CSS.Properties = {
    '--block-column': column * base,
    '--block-row': row * base,
  };

  // TODO: add blocks
  return (
    <div className={'block'} style={style}/>
  )
};


interface CellProps {
  cell: TransportCell,
  base: TransportSudoku['base'],
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  const style: CSS.Properties = {
    '--cell-column': props.cell.position.column,
    '--cell-row': props.cell.position.row,
  };

  return (
    <div className='cell' style={style}>
      {
        props.cell.value ?
          <div className='value'><span className='valueText'>{props.cell.value}</span></div> :
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

const indexToPosition = (index: number, base: TransportSudoku['base']): CellPosition => {
  return {
    column: index % base,
    row: Math.floor(index / base)
  }
};
