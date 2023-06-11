import type { DynamicCellValue, DynamicPosition, TransportSudoku } from "../types";

export function indexToPosition({
    blockIndex,
    base,
}: {
    blockIndex: number;
    base: TransportSudoku["base"];
}): DynamicPosition {
    return {
        row: Math.floor(blockIndex / base),
        column: blockIndex % base,
    };
}

type PositionToIndexParam =
    | {
          gridPosition: DynamicPosition;
          sideLength: TransportSudoku["sideLength"];
      }
    | {
          gridPosition: DynamicPosition;
          base: TransportSudoku["sideLength"];
      }
    | {
          blockPosition: DynamicPosition;
          base: TransportSudoku["sideLength"];
      };

export function positionToIndex(params: PositionToIndexParam): number {
    if ("gridPosition" in params) {
        // Full cell grid (4x4, 9x9, 16x16, ...)
        let sideLength;
        if ("sideLength" in params) {
            sideLength = params.sideLength;
        } else {
            sideLength = baseToSideLength(params.base);
        }

        return params.gridPosition.row * sideLength + params.gridPosition.column;
    } else {
        // Block/candidates grid (2x2, 3x3, 4x4)
        return params.blockPosition.row * params.base + params.blockPosition.column;
    }
}

export function cellPositionToBlockPosition(
    cellPosition: DynamicPosition,
    base: TransportSudoku["base"]
): DynamicPosition {
    return {
        row: Math.floor(cellPosition.row / base),
        column: Math.floor(cellPosition.column / base),
    };
}

export function valueToString(value: DynamicCellValue["value"]): string {
    return value.toString(36).toUpperCase();
}

export function baseToSideLength(base: number): number {
    return base ** 2;
}
export function baseToCellCount(base: number): number {
    return base ** 4;
}
