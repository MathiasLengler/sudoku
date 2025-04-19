import type { DynamicPosition, TransportCell } from "../../types";
import { selector, selectorFamily } from "recoil";
import { cellPositionToBlockPosition, positionToIndex } from "../utils";
import { sudokuBaseState, sudokuCellsState, sudokuSideLengthState } from "./sudoku";
import { selectedPosState } from "./input";
import type { CreateSerializableParam } from "../../typeUtils";

export const cellAtIndexState = selectorFamily<TransportCell, number>({
    key: "CellAtIndex",
    get:
        (cellIndex) =>
        ({ get }) => {
            const cells = get(sudokuCellsState);
            const selectedCells = cells[cellIndex];
            if (!selectedCells) {
                throw new Error(`Failed to get cell at index ${cellIndex} in cells with length of ${cells.length}`);
            }
            return selectedCells;
        },
});
export const cellAtGridPositionState = selectorFamily<TransportCell, CreateSerializableParam<DynamicPosition>>({
    key: "CellAtGridPosition",
    get:
        (gridPosition) =>
        ({ get }) => {
            const sideLength = get(sudokuSideLengthState);
            return get(cellAtIndexState(positionToIndex({ gridPosition, sideLength })));
        },
});

export const selectedBlockPositionState = selector<DynamicPosition | undefined>({
    key: "CellSelection.selectedBlockPosition",
    get: ({ get }) => {
        const selectedPos = get(selectedPosState);
        if (selectedPos) {
            const base = get(sudokuBaseState);
            return cellPositionToBlockPosition(selectedPos, base);
        }
    },
});
