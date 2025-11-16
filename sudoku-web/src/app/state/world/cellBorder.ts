import classNames from "classnames";
import { inRange, isEqual } from "lodash-es";
import { atomFamily } from "jotai/utils";
import { cellDimState, cellWorldDimensionsState, selectedGridPositionState, type WorldCellPosition } from ".";
import { getAxisBorders, validateCellWorldPosition } from "../../utils/world";
import { sudokuBaseState, sudokuSideLengthState } from "../sudoku";
import { atom, type Atom } from "jotai";

export const worldCellBorderClassesState = atomFamily<WorldCellPosition, Atom<Promise<string>>>(
    (cellWorldPosition) =>
        atom(async (get) => {
            const base = await get(sudokuBaseState);
            const gridSideLength = await get(sudokuSideLengthState);
            const { overlap, cellDim } = await get(cellWorldDimensionsState);
            const { rowCount: cellRowCount, columnCount: cellColumnCount } = await get(cellDimState);
            const { row: selectedGridRowIndex, column: selectedGridColumnIndex } = get(selectedGridPositionState);
            const { row: cellRowIndex, column: cellColumnIndex } = cellWorldPosition;

            validateCellWorldPosition({ cellWorldPosition: cellWorldPosition, cellDim });

            const gridStride = gridSideLength - overlap;
            const selectedGridBaseCellRowIndex = selectedGridRowIndex * gridStride;
            const selectedGridBaseCellColumnIndex = selectedGridColumnIndex * gridStride;
            const isCellInSelectedGrid =
                inRange(cellRowIndex, selectedGridBaseCellRowIndex, selectedGridBaseCellRowIndex + gridSideLength) &&
                inRange(
                    cellColumnIndex,
                    selectedGridBaseCellColumnIndex,
                    selectedGridBaseCellColumnIndex + gridSideLength,
                );

            const rowBorders = getAxisBorders(
                cellRowIndex,
                cellRowCount,
                overlap,
                base,
                gridSideLength,
                isCellInSelectedGrid,
                selectedGridRowIndex,
            );
            const columnBorders = getAxisBorders(
                cellColumnIndex,
                cellColumnCount,
                overlap,
                base,
                gridSideLength,
                isCellInSelectedGrid,
                selectedGridColumnIndex,
            );

            return classNames({
                [`${rowBorders.start}-border-top`]: !!rowBorders.start,
                [`${columnBorders.end}-border-right`]: !!columnBorders.end,
                [`${rowBorders.end}-border-bottom`]: !!rowBorders.end,
                [`${columnBorders.start}-border-left`]: !!columnBorders.start,
            });
        }),
    isEqual,
);
