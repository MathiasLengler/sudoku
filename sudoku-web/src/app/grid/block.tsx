import type { DynamicPosition, TransportCell } from "../../types";
import { selectorFamily, useRecoilValue } from "recoil";
import { sudokuBaseState } from "../state/sudoku";
import { inputState } from "../state/input";
import { indexToPosition } from "../utils/sudoku";
import type * as CSS from "csstype";
import isEqual from "lodash/isEqual";
import { Cell } from "./cell";
import * as React from "react";
import type { CreateSerializableParam } from "../../typeUtils";
import { selectedBlockPositionState } from "../state/cellIndexing";

type BlockProps = {
    cells: TransportCell[];
    blockIndex: number;
};

const containsSelectedPosState = selectorFamily<boolean, CreateSerializableParam<DynamicPosition>>({
    key: "Block.containsSelectedPos",
    get:
        (blockPosition) =>
        ({ get }) => {
            const selectedBlockPosition = get(selectedBlockPositionState);
            return !!selectedBlockPosition && isEqual(blockPosition, selectedBlockPosition);
        },
});
export const Block = ({ blockIndex, cells }: BlockProps) => {
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
                    isSelected = containsSelectedPos && isEqual(selectedPos, cell.position);
                    isGuide =
                        containsSelectedPos ||
                        selectedPos.column == cell.position.column ||
                        selectedPos.row == cell.position.row;
                }

                return <Cell key={blockCellIndex} cell={cell} isSelected={isSelected} isGuide={isGuide} />;
            })}
        </div>
    );
};
