import { describe, expect, test } from "vitest";
import { validateCellWorldPosition } from "./world";
import type { DynamicPosition } from "../../types";

describe("validateCellWorldPosition", () => {
    const cellDim = {
        rowCount: 3,
        columnCount: 3,
    };

    test.for([
        { row: 0, column: 0 },
        { row: 1, column: 2 },
        { row: 2, column: 2 },
    ] satisfies DynamicPosition[])("positive %o", (cellWorldPosition) => {
        validateCellWorldPosition({
            cellWorldPosition,
            cellDim,
        });
    });

    test.for([
        { row: 3, column: 2 },
        { row: 2, column: 3 },
        { row: -1, column: 0 },
        { row: 0, column: -1 },
    ] satisfies DynamicPosition[])("negative %o", (cellWorldPosition) => {
        expect(() =>
            validateCellWorldPosition({
                cellWorldPosition,
                cellDim: cellDim,
            }),
        ).toThrowError(/cellWorldPosition out of bounds/);
    });
});
