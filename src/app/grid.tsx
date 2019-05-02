import * as React from 'react';
import * as CSS from 'csstype';
import {isEqual} from "lodash"

interface GridProps {
  sudoku: TransportSudoku,
  selectedPos: CellPosition,
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>
}

export const Grid: React.FunctionComponent<GridProps> = (props) => {
  const {
    sudoku: {cells, base, side_length},
    selectedPos,
    setSelectedPos
  } = props;

  return <div className='grid'>
    <Blocks side_length={side_length} base={base}/>
    {cells.map((cell, i) => {
        if (isEqual(selectedPos, cell.position)) {
          console.log("Selected:", selectedPos);
        }

        return <MemoCell
          key={i}
          cell={cell}
          base={base}
          selected={isEqual(selectedPos, cell.position)}
          setSelectedPos={setSelectedPos}
        />
      }
    )}
  </div>
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

  return (
    <div className={'block'} style={style}/>
  )
};


interface CellProps {
  cell: TransportCell,
  base: TransportSudoku['base'],
  selected: boolean,
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>,
}

const Cell: React.FunctionComponent<CellProps> = (props) => {
  console.log("Cell render", props);

  const {
    cell: {position, candidates, value},
    base,
    selected,
    setSelectedPos
  } = props;

  const style: CSS.Properties = {
    '--cell-column': position.column,
    '--cell-row': position.row,
    backgroundColor: selected ? "red" : "green",
  };

  return (
    <div className='cell' style={style} onClick={() => setSelectedPos(position)}>
      {
        value ?
          <div className='cellValue'><span className='cellValueText'>{value}</span></div> :
          <Candidates candidates={candidates} base={base}/>
      }
    </div>
  )
};


const MemoCell = React.memo(Cell, (prevProps, nextProps) => {
  return isEqual(prevProps, nextProps)
});

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
