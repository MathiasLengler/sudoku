import classNames from "classnames";
import _ from "lodash";
import { selectorFamily } from "recoil";
import { cellDimState, cellWorldDimensionsState, selectedGridIndexState, type WorldCellPosition } from ".";
import { getAxisBorders, validateCellWorldPosition } from "../../utils/world";
import { sudokuBaseState, sudokuSideLengthState } from "../sudoku";

export const worldCellBorderClassesState = selectorFamily<string, WorldCellPosition>({
    key: "worldCellBorder",
    get:
        (cellWorldPosition) =>
        ({ get }) => {
            const base = get(sudokuBaseState);
            const gridSideLength = get(sudokuSideLengthState);
            const { overlap, cellDim } = get(cellWorldDimensionsState);
            const { rowCount: cellRowCount, columnCount: cellColumnCount } = get(cellDimState);
            const { row: selectedGridRowIndex, column: selectedGridColumnIndex } = get(selectedGridIndexState);
            const { row: cellRowIndex, column: cellColumnIndex } = cellWorldPosition;

            validateCellWorldPosition({ cellWorldPosition: cellWorldPosition, cellDim });

            const gridStride = gridSideLength - overlap;
            const selectedGridBaseCellRowIndex = selectedGridRowIndex * gridStride;
            const selectedGridBaseCellColumnIndex = selectedGridColumnIndex * gridStride;
            const isCellInSelectedGrid =
                _.inRange(cellRowIndex, selectedGridBaseCellRowIndex, selectedGridBaseCellRowIndex + gridSideLength) &&
                _.inRange(
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
        },
    cachePolicy_UNSTABLE: {
        eviction: "most-recent",
    },
});
