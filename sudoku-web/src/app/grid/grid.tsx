import * as React from 'react';
import * as CSS from 'csstype';
import isEqual from "lodash/isEqual";
import {MemoCell} from "./cell";
import {indexToPosition} from "../utils";

interface GridProps {
  sudoku: TransportSudoku;
  selectedPos: CellPosition;
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>;
}

export const Grid: React.FunctionComponent<GridProps> = (props) => {
  const {
    sudoku: {base, blocks},
    selectedPos,
    setSelectedPos
  } = props;

  return <div className='grid'>
    {
      blocks
        .map((block, blockIndex) =>
          <Block
            key={blockIndex}
            block={block}
            blockIndex={blockIndex}
            base={base}
            selectedPos={selectedPos}
            setSelectedPos={setSelectedPos}
          />
        )
    }

  </div>
};

interface BlockProps {
  block: Block;
  blockIndex: number;
  base: TransportSudoku['base'];
  selectedPos: CellPosition;
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>;
}

const Block: React.FunctionComponent<BlockProps> = (props) => {
  const {
    block,
    blockIndex,
    base,
    selectedPos,
    setSelectedPos
  } = props;

  const {column: blockColumn, row: blockRow} = indexToPosition(blockIndex, base);

  const style: CSS.Properties = {
    '--block-column': blockColumn,
    '--block-row': blockRow,
  };

  return (
    <div className={'block'} style={style}>
      {block.map((cell, blockCellIndex) =>
        <MemoCell
          key={blockCellIndex}
          blockCellIndex={blockCellIndex}
          cell={cell}
          base={base}
          selected={isEqual(selectedPos, cell.position)}
          setSelectedPos={setSelectedPos}
        />
      )}
    </div>
  )
};

