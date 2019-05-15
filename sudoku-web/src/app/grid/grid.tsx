import * as React from 'react';
import * as CSS from 'csstype';
import isEqual from "lodash/isEqual";
import {MemoCell} from "./cell";
import {cellFromBlocks, cellPositionToBlockPosition, indexToPosition} from "../utils";

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

  const selectedCell = cellFromBlocks(blocks, selectedPos, base);

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
            selectedCell={selectedCell}
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
  selectedCell: TransportCell;
}

const Block: React.FunctionComponent<BlockProps> = (props) => {
  const {
    block,
    blockIndex,
    base,
    selectedPos,
    setSelectedPos,
    selectedCell
  } = props;

  const blockPosition = indexToPosition(blockIndex, base);

  const style: CSS.Properties = {
    '--block-column': blockPosition.column,
    '--block-row': blockPosition.row,
  };

  const selectedBlockPosition = cellPositionToBlockPosition(selectedPos, base);

  const containsSelectedPos = isEqual(blockPosition, selectedBlockPosition);

  return <div className={'block'} style={style}>
    {block.map((cell, blockCellIndex) => {
      const selected = containsSelectedPos && isEqual(selectedPos, cell.position);

      const guideGroup = containsSelectedPos
        || selectedPos.column == cell.position.column
        || selectedPos.row == cell.position.row;

      const guideValue = selectedCell.kind === "value"
        && cell.kind === "value"
        && selectedCell.value === cell.value;

      return <MemoCell
        key={blockCellIndex}
        blockCellIndex={blockCellIndex}
        cell={cell}
        base={base}
        selected={selected}
        setSelectedPos={setSelectedPos}
        guideValue={guideValue}
        guideGroup={guideGroup}
      />
    })}
  </div>
};

