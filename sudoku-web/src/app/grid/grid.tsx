import * as React from "react";
import * as CSS from "csstype";
import isEqual from "lodash/isEqual";
import { MemoCell } from "./cell";
import { cellPositionToBlockPosition, indexToPosition } from "../utils";
import { Input, WasmSudokuController } from "../wasmSudokuController";

interface BlockProps {
    block: Block;
    blockIndex: number;
    base: TransportSudoku["base"];
    input: Input;
    sudokuController: WasmSudokuController;
}

const Block: React.FunctionComponent<BlockProps> = props => {
    const {
        block,
        blockIndex,
        base,
        input: { selectedPos, selectedValue, stickyMode },
        sudokuController,
    } = props;

    const blockPosition = indexToPosition(blockIndex, base);

    const style: CSS.Properties = {
        "--block-column": blockPosition.column,
        "--block-row": blockPosition.row,
    };

    const selectedBlockPosition = cellPositionToBlockPosition(selectedPos, base);

    const containsSelectedPos = isEqual(blockPosition, selectedBlockPosition);

    return (
        <div className={"block"} style={style}>
            {block.map((cell, blockCellIndex) => {
                let isSelected: boolean;
                let isGuide: boolean;

                if (stickyMode) {
                    isSelected = cell.kind === "value" && cell.value === selectedValue;
                    isGuide = !(cell.kind === "candidates" && cell.candidates.includes(selectedValue));
                } else {
                    isSelected = containsSelectedPos && isEqual(selectedPos, cell.position);
                    isGuide =
                        containsSelectedPos ||
                        selectedPos.column == cell.position.column ||
                        selectedPos.row == cell.position.row;
                }

                return (
                    <MemoCell
                        key={blockCellIndex}
                        blockCellIndex={blockCellIndex}
                        cell={cell}
                        base={base}
                        sudokuController={sudokuController}
                        isSelected={isSelected}
                        isGuide={isGuide}
                        selectedValue={selectedValue}
                        stickyMode={stickyMode}
                    />
                );
            })}
        </div>
    );
};

interface GridProps {
    sudokuController: WasmSudokuController;
    sudoku: TransportSudoku;
    input: Input;
}

export const Grid: React.FunctionComponent<GridProps> = props => {
    const {
        sudokuController,
        sudoku: { base, blocks },
        input,
    } = props;

    return (
        <div className="grid">
            {blocks.map((block, blockIndex) => (
                <Block
                    key={blockIndex}
                    block={block}
                    blockIndex={blockIndex}
                    base={base}
                    input={input}
                    sudokuController={sudokuController}
                />
            ))}
        </div>
    );
};
