import * as React from 'react';
import * as CSS from 'csstype';
import {isEqual} from "lodash";
import {MemoCell} from "./cell";
import {indexToPosition} from "../utils";

interface GridProps {
  sudoku: TransportSudoku;
  selectedPos: CellPosition;
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>;
}

export const Grid: React.FunctionComponent<GridProps> = (props) => {
  const {
    sudoku: {cells, base, sideLength},
    selectedPos,
    setSelectedPos
  } = props;

  return <div className='grid'>
    <Blocks sideLength={sideLength} base={base}/>
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
    })}
  </div>
};

interface BlocksProps {
  sideLength: TransportSudoku['sideLength'];
  base: TransportSudoku['base'];
}

const Blocks: React.FunctionComponent<BlocksProps> = (props) => {
  const {sideLength, base} = props;
  return (
    <>
      {
        Array.from(Array(sideLength).keys())
          .map(blockIndex =>
            <Block key={blockIndex} base={base} blockIndex={blockIndex}/>
          )
      }
    </>
  )
};


interface BlockProps {
  base: TransportSudoku['base'];
  blockIndex: number;
}

const Block: React.FunctionComponent<BlockProps> = (props) => {
  const {base, blockIndex} = props;

  const {column, row} = indexToPosition(blockIndex, base);

  const style: CSS.Properties = {
    '--block-column': column * base,
    '--block-row': row * base,
  };

  return (
    <div className={'block'} style={style}/>
  )
};

