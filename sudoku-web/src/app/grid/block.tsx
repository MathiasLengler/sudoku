import { atom, useAtomValue, type Atom } from "jotai";
import { atomFamily } from "jotai/utils";
import { isEqual } from "lodash-es";
import type { DynamicPosition, TransportCell } from "../../types";
import { selectedBlockPositionState } from "../state/cellIndexing";
import { inputState } from "../state/input";
import { sudokuBaseState } from "../state/sudoku";
import { indexToPosition } from "../utils/sudoku";
import { Cell } from "./cell";
import { eagerAtom } from "jotai-eager";

type BlockProps = {
    cells: TransportCell[];
    blockIndex: number;
};

const containsSelectedPosState = atomFamily<DynamicPosition, Atom<Promise<boolean> | boolean>>(
    (blockPosition) =>
        eagerAtom((get) => {
            const selectedBlockPosition = get(selectedBlockPositionState);
            return !!selectedBlockPosition && isEqual(blockPosition, selectedBlockPosition);
        }),
    isEqual,
);
export function Block({ blockIndex, cells }: BlockProps) {
    const base = useAtomValue(sudokuBaseState);
    const input = useAtomValue(inputState);

    const blockPosition = indexToPosition({ blockIndex, base });

    const containsSelectedPos = useAtomValue(containsSelectedPosState(blockPosition));

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
}
