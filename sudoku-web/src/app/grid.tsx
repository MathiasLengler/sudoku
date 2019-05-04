import * as React from 'react';
import * as CSS from 'csstype';
import {isEqual} from "lodash";
import {MemoCell} from "./cell";

// TODO: modularize

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

export const indexToPosition = (index: number, base: TransportSudoku['base']): CellPosition => {
  return {
    column: index % base,
    row: Math.floor(index / base)
  }
};