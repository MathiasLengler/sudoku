import * as React from 'react';
import * as CSS from 'csstype';
import isEqual from "lodash/isEqual";
import {MemoCell} from "./cell";
import {cellPositionToBlockPosition, indexToPosition} from "../utils";

interface GridProps {
  sudoku: TransportSudoku;
  selectedPos: CellPosition;
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>;
  selectedCell: TransportCell;
}

export const Grid: React.FunctionComponent<GridProps> = (props) => {
  const {
    sudoku: {base, blocks},
    selectedPos,
    setSelectedPos,
    selectedCell,
  } = props;

  let selectedValuePositions: CellPosition[];
  if (selectedCell.kind === "value") {
    selectedValuePositions = blocks.flatMap((block) =>
      block.filter((cell) =>
        cell.kind === "value" && cell.value === selectedCell.value
      ).map((cell) => cell.position)
    );
  } else {
    selectedValuePositions = [];
  }
  console.log(selectedValuePositions);

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
            selectedValuePositions={selectedValuePositions}
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
  selectedValuePositions: CellPosition[];
}

const Block: React.FunctionComponent<BlockProps> = (props) => {
  const {
    block,
    blockIndex,
    base,
    selectedPos,
    setSelectedPos,
    selectedCell,
    selectedValuePositions
  } = props;

  const blockPosition = indexToPosition(blockIndex, base);

  const style: CSS.Properties = {
    '--block-column': blockPosition.column,
    '--block-row': blockPosition.row,
  };

  const selectedBlockPosition = cellPositionToBlockPosition(selectedPos, base);

  const containsSelectedPos = isEqual(blockPosition, selectedBlockPosition);

  const containsSelectedValue = selectedValuePositions.some(pos => {
    const selectedValueBlockPosition = cellPositionToBlockPosition(pos, base);
    return isEqual(selectedValueBlockPosition, blockPosition);
  });

  return <div className={'block'} style={style}>
    {block.map((cell, blockCellIndex) => {
      const selected = containsSelectedPos && isEqual(selectedPos, cell.position);

      const guideGroup = containsSelectedPos
        || selectedPos.column == cell.position.column
        || selectedPos.row == cell.position.row;

      const guideValue = selectedCell.kind === "value"
        && cell.kind === "value"
        && selectedCell.value === cell.value;

      const guideValueGroup = containsSelectedValue
        || selectedValuePositions.some(pos => pos.column === cell.position.column || pos.row === cell.position.row);

      return <MemoCell
        key={blockCellIndex}
        blockCellIndex={blockCellIndex}
        cell={cell}
        base={base}
        selected={selected}
        setSelectedPos={setSelectedPos}
        guideValue={guideValue}
        guideGroup={guideGroup}
        guideValueGroup={guideValueGroup}/>
    })}
  </div>
};

