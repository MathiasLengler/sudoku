import { isEqual } from "lodash-es";
import { selectorFamily, useRecoilValue } from "recoil";
import type { DynamicPosition, TransportCell } from "../../types";
import type { CreateSerializableParam } from "../../typeUtils";
import { selectedBlockPositionState } from "../state/cellIndexing";
import { inputState } from "../state/input";
import { sudokuBaseState } from "../state/sudoku";
import { indexToPosition } from "../utils/sudoku";
import { Cell } from "./cell";

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
    cachePolicy_UNSTABLE: {
        eviction: "most-recent",
    },
});
export const Block = ({ blockIndex, cells }: BlockProps) => {
    const base = useRecoilValue(sudokuBaseState);
    const input = useRecoilValue(inputState);

    const blockPosition = indexToPosition({ blockIndex, base });

    const containsSelectedPos = useRecoilValue(containsSelectedPosState(blockPosition));

    return (
        <div className="block">
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
