import { atom, type Atom } from "jotai";
import { atomFamily } from "jotai/utils";
import type { DynamicPosition, TransportCell } from "../../types";
import { cellPositionToBlockPosition, positionToIndex } from "../utils/sudoku";
import { selectedPosState } from "./input";
import { sudokuBaseState, sudokuCellsState, sudokuSideLengthState } from "./sudoku";

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
export const cellAtGridPositionState = atomFamily<DynamicPosition, Atom<Promise<TransportCell>>>((gridPosition) =>
    atom(async (get) => {
        const sideLength = await get(sudokuSideLengthState);
        return get(cellAtIndexState(positionToIndex({ gridPosition, sideLength })));
    }),
);

export const selectedBlockPositionState = atom<Promise<DynamicPosition | undefined>>(async (get) => {
    const selectedPos = get(selectedPosState);
    if (selectedPos) {
        const base = await get(sudokuBaseState);
        return cellPositionToBlockPosition(selectedPos, base);
    }
});
