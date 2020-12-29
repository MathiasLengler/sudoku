import * as React from 'react';
import * as CSS from 'csstype';
import isEqual from "lodash/isEqual";
import {MemoCell} from "./cell";
import {cellPositionToBlockPosition, indexToPosition} from "../utils";
import {Input, WasmSudokuController} from "../wasmSudokuController";


interface BlockProps {
  block: Block;
  blockIndex: number;
  base: TransportSudoku['base'];
  input: Input;
  sudokuController: WasmSudokuController;
  selectedValuePositions: CellPosition[];
}

const Block: React.FunctionComponent<BlockProps> = (props) => {
  const {
    block,
    blockIndex,
    base,
    input: {
      selectedPos,
      selectedCell,
      selectedValue,
      stickyMode
    },
    sudokuController,
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
      const selected = !stickyMode && containsSelectedPos && isEqual(selectedPos, cell.position);

      const guideGroup = !stickyMode && (
        containsSelectedPos
        || selectedPos.column == cell.position.column
        || selectedPos.row == cell.position.row
      );

      const guideValue = cell.kind === "value" && (
        stickyMode
          ? cell.value === selectedValue
          : selectedCell.kind === "value" && selectedCell.value === cell.value
      );

      const guideValueGroup = containsSelectedValue
        || selectedValuePositions.some(pos => pos.column === cell.position.column || pos.row === cell.position.row);

      return <MemoCell
        key={blockCellIndex}
        blockCellIndex={blockCellIndex}
        cell={cell}
        base={base}
        selected={selected}
        sudokuController={sudokuController}
        guideValue={guideValue}
        guideGroup={guideGroup}
        guideValueGroup={guideValueGroup}/>
    })}
  </div>
};


interface GridProps {
  sudokuController: WasmSudokuController;
  sudoku: TransportSudoku;
  input: Input;
}

export const Grid: React.FunctionComponent<GridProps> = (props) => {
  const {
    sudokuController,
    sudoku: {base, blocks},
    input,
  } = props;

  const {
    selectedCell,
    selectedValue,
    stickyMode
  } = input;

  const guideValue = stickyMode ? selectedValue : selectedCell.kind === "value" ? selectedCell.value : undefined;

  let selectedValuePositions: CellPosition[];
  if (guideValue) {
    selectedValuePositions = blocks.flatMap((block) =>
      block.filter((cell) =>
        cell.kind === "value" && cell.value === guideValue
      ).map((cell) => cell.position)
    );
  } else {
    selectedValuePositions = [];
  }

  return <div className='grid'>
    {
      blocks
        .map((block, blockIndex) =>
          <Block
            key={blockIndex}
            block={block}
            blockIndex={blockIndex}
            base={base}
            selectedValuePositions={selectedValuePositions}
            input={input}
            sudokuController={sudokuController}/>
        )
    }
  </div>
};

