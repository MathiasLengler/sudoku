import type * as React from "react";
import type * as CSS from "csstype";
import isEqual from "lodash/isEqual";
import { Cell } from "./cell";
import { indexToPosition } from "../utils";
import type { Position, TransportCell } from "../../types";
import { inputState } from "../state/input";
import { selectedBlockPositionState } from "../state/cellIndexing";
import { selectorFamily, useRecoilValue } from "recoil";
import { sudokuBaseState, sudokuBlocksIndicesState, sudokuCellsState } from "../state/sudoku";
import type { CreateSerializableParam } from "../../typeUtils";

interface BlockProps {
    cells: TransportCell[];
    blockIndex: number;
}

const containsSelectedPosState = selectorFamily<boolean | undefined, CreateSerializableParam<Position>>({
    key: "Block.containsSelectedPos",
    get:
        blockPosition =>
        ({ get }) => {
            const selectedBlockPosition = get(selectedBlockPositionState);
            if (selectedBlockPosition) {
                return isEqual(blockPosition, selectedBlockPosition);
            }
        },
});

const Block = ({ blockIndex, cells }: BlockProps) => {
    const base = useRecoilValue(sudokuBaseState);
    const input = useRecoilValue(inputState);

    const blockPosition = indexToPosition({ blockIndex, base });

    const containsSelectedPos = useRecoilValue(containsSelectedPosState(blockPosition));

    const style: CSS.Properties = {
        "--block-column": blockPosition.column,
        "--block-row": blockPosition.row,
    };

    return (
        <div className="block" style={style}>
            {cells.map((cell, blockCellIndex) => {
                let isSelected: boolean;
                let isGuide: boolean;

                if (input.stickyMode) {
                    const { selectedValue } = input;
                    isSelected = cell.kind === "value" && cell.value === selectedValue;
                    isGuide = !(cell.kind === "candidates" && cell.candidates.includes(selectedValue));
                } else {
                    const { selectedPos } = input;
                    isSelected = !!containsSelectedPos && isEqual(selectedPos, cell.position);
                    isGuide =
                        containsSelectedPos ||
                        selectedPos.column == cell.position.column ||
                        selectedPos.row == cell.position.row;
                }

                return (
                    <Cell
                        key={blockCellIndex}
                        blockCellIndex={blockCellIndex}
                        cell={cell}
                        isSelected={isSelected}
                        isGuide={isGuide}
                    />
                );
            })}
        </div>
    );
};

interface GridProps {
    gridRef: React.MutableRefObject<HTMLDivElement>;
}

export const Grid = ({ gridRef }: GridProps) => {
    const blocksIndices = useRecoilValue(sudokuBlocksIndicesState);
    const cells = useRecoilValue(sudokuCellsState);

    return (
        <>
            <div className="grid-container">
                <div className="grid" ref={gridRef}>
                    {blocksIndices.map((cellIndices, blockIndex) => (
                        <Block
                            key={blockIndex}
                            cells={cellIndices.map(cellIndex => cells[cellIndex])}
                            blockIndex={blockIndex}
                        />
                    ))}
                </div>
            </div>
        </>
    );
};
