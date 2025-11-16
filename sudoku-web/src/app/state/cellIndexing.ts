import { atom, type Atom } from "jotai";
import { atomFamily } from "jotai/utils";
import type { DynamicPosition, TransportCell } from "../../types";
import { cellPositionToBlockPosition, positionToIndex } from "../utils/sudoku";
import { selectedPosState } from "./input";
import { sudokuBaseState, sudokuCellsState, sudokuSideLengthState } from "./sudoku";
import { isEqual } from "lodash-es";
import { eagerAtom } from "jotai-eager";

export const cellAtIndexState = atomFamily<number, Atom<Promise<TransportCell>>>((cellIndex) =>
    atom(async (get) => {
        const cells = await get(sudokuCellsState);
        const selectedCells = cells[cellIndex];
        if (!selectedCells) {
            throw new Error(`Failed to get cell at index ${cellIndex} in cells with length of ${cells.length}`);
        }
        return selectedCells;
    }),
);
export const cellAtGridPositionState = atomFamily<DynamicPosition, Atom<Promise<TransportCell>>>(
    (gridPosition) =>
        atom(async (get) => {
            const sideLength = await get(sudokuSideLengthState);
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
