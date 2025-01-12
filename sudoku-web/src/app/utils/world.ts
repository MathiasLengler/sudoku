import _ from "lodash";
import type { DynamicPosition, WorldDim } from "../../types";

export function validateCellWorldPosition({
    cellWorldPosition: { row: cellRowIndex, column: cellColumnIndex },
    cellWorldPosition,
    cellDim: { rowCount: cellRowCount, columnCount: cellColumnCount },
    cellDim,
}: {
    cellWorldPosition: DynamicPosition;
    cellDim: WorldDim;
}) {
    if (!(_.inRange(cellRowIndex, 0, cellRowCount) && _.inRange(cellColumnIndex, 0, cellColumnCount))) {
        throw new Error(
            `cellWorldPosition out of bounds: ${JSON.stringify(cellWorldPosition)} for cellDim: ${JSON.stringify(cellDim)}`,
        );
    }
}

type SelectedGridMarker = "selected-grid";
type GridBorderMarker = "grid";
type BlockBorderMarker = "block";
type BorderMarker = GridBorderMarker | BlockBorderMarker | SelectedGridMarker;

export type CellBorders<T = BorderMarker> = {
    top?: T;
    right?: T;
    bottom?: T;
    left?: T;
};

export type AxisBorders<T = BorderMarker> = {
    start?: T;
    end?: T;
};

// TODO: test
export function getAxisBorders(
    // current cell
    cellAxisIndex: number,
    // world
    cellAxisCount: number,
    overlap: number,
    // grid
    base: number,
    gridSideLength: number,
    // selected grid
    inSelectedGrid: boolean,
    selectedGridAxisIndex: number,
): AxisBorders {
    const gridStride = gridSideLength - overlap;
    const gridIndex = Math.floor(cellAxisIndex / gridStride);
    const gridCellIndex = cellAxisIndex % gridStride;

    const blockStride = base;
    const blockStrideEndIndex = blockStride - 1;
    const blockIndex = Math.floor(gridCellIndex / blockStride);
    const blockCellIndex = gridCellIndex % blockStride;

    const hasGridBefore = cellAxisIndex >= gridStride;
    const hasGridAfter = cellAxisIndex < cellAxisCount - gridStride;

    const axisBorders: AxisBorders = {};
    if (gridCellIndex === 0 && hasGridAfter) {
        axisBorders.start = inSelectedGrid && gridIndex === selectedGridAxisIndex ? "selected-grid" : "grid";
    }
    if (gridCellIndex === overlap - 1 && hasGridBefore) {
        axisBorders.end = inSelectedGrid && gridIndex === selectedGridAxisIndex + 1 ? "selected-grid" : "grid";
    }
    if (blockCellIndex === 0 && blockIndex > 0) {
        axisBorders.start ??= "block";
    } else if (blockCellIndex === blockStrideEndIndex) {
        axisBorders.end ??= "block";
    }

    return axisBorders;
}
