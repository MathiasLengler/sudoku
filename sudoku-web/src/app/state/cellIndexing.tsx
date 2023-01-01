import type { Position, TransportCell } from "../../../../sudoku-rs/bindings";
import { selector, selectorFamily } from "recoil";
import { cellPositionToBlockPosition, positionToIndex } from "../utils";
import { sudokuBaseState, sudokuCellsState, sudokuSideLengthState } from "./sudoku";
import { selectedPosState } from "./input";
import _ from "lodash";
import type { CreateSerializableParam } from "../../typeUtils";

export interface CellSelection {
    selectedCell: TransportCell;
    selectedCellIndex: number;
    selectedBlockPosition: Position;
    selectedBlockCellIndex: number;
}

export const cellAtIndexState = selectorFamily<TransportCell, number>({
    key: "CellAtIndex",
    get:
        cellIndex =>
        ({ get }) => {
            const cells = get(sudokuCellsState);
            const selectedCells = cells[cellIndex];
            if (!selectedCells) {
                throw new Error(`Failed to get cell at index ${cellIndex} in cells with length of ${cells.length}`);
            }
            return selectedCells;
        },
});
export const cellAtGridPositionState = selectorFamily<TransportCell, CreateSerializableParam<Position>>({
    key: "CellAtGridPosition",
    get:
        gridPosition =>
        ({ get }) => {
            const sideLength = get(sudokuSideLengthState);
            return get(cellAtIndexState(positionToIndex({ gridPosition, sideLength })));
        },
});

const selectedCellIndexState = selector<number | undefined>({
    key: "CellSelection.selectedCellIndex",
    get: ({ get }) => {
        const selectedPos = get(selectedPosState);
        if (selectedPos) {
            const base = get(sudokuBaseState);
            return positionToIndex({ gridPosition: selectedPos, base });
        }
    },
});

const selectedCellState = selector<TransportCell | undefined>({
    key: "CellSelection.selectedCell",
    get: ({ get }) => {
        const selectedCellIndex = get(selectedCellIndexState);
        if (!_.isNil(selectedCellIndex)) {
            return get(cellAtIndexState(selectedCellIndex));
        }
    },
});
export const selectedBlockPositionState = selector<Position | undefined>({
    key: "CellSelection.selectedBlockPosition",
    get: ({ get }) => {
        const selectedPos = get(selectedPosState);
        if (selectedPos) {
            const base = get(sudokuBaseState);
            return cellPositionToBlockPosition(selectedPos, base);
        }
    },
});
const selectedBlockCellIndexState = selector<number | undefined>({
    key: "CellSelection.selectedBlockCellIndex",
    get: ({ get }) => {
        const selectedPos = get(selectedPosState);
        if (selectedPos) {
            const base = get(sudokuBaseState);
            return positionToIndex({
                blockPosition: {
                    row: selectedPos.row % base,
                    column: selectedPos.column % base,
                },
                base,
            });
        }
    },
});
