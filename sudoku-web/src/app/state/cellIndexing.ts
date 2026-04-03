import { isEqual } from "es-toolkit";
import { eagerAtom } from "jotai-eager";
import { atomFamily } from "jotai/utils";
import type { DynamicPosition, TransportCell } from "../../types";
import { cellPositionToBlockPosition, positionToIndex } from "../utils/sudoku";
import { selectedPosState } from "./input";
import { sudokuBaseState, sudokuCellsState, sudokuSideLengthState } from "./sudoku";

export const cellAtIndexState = atomFamily((cellIndex: number) =>
    eagerAtom<TransportCell>((get) => {
        const cells = get(sudokuCellsState);
        const selectedCells = cells[cellIndex];
        if (!selectedCells) {
            throw new Error(`Failed to get cell at index ${cellIndex} in cells with length of ${cells.length}`);
        }
        return selectedCells;
    }),
);
export const cellAtGridPositionState = atomFamily(
    (gridPosition: DynamicPosition) =>
        eagerAtom<TransportCell>((get) => {
            const sideLength = get(sudokuSideLengthState);
            return get(cellAtIndexState(positionToIndex({ gridPosition, sideLength })));
        }),
    isEqual,
);

export const selectedBlockPositionState = eagerAtom<DynamicPosition | undefined>((get) => {
    const selectedPos = get(selectedPosState);
    const base = get(sudokuBaseState);
    if (selectedPos) {
        return cellPositionToBlockPosition(selectedPos, base);
    }
});
