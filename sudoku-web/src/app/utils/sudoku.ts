import type { DynamicCellValue, DynamicPosition, TransportSudoku } from "../../types";

export function cellColorClass(fixed: boolean, incorrectValue: boolean) {
    if (fixed) {
        return "cell--fixed";
    }
    if (incorrectValue) {
        return "cell--incorrect-value";
    } else {
        return "cell--user";
    }
}

export function getValueColorStyle(value: number, sideLength: number): string | undefined {
    if (value >= 1 && value <= sideLength) {
        // Distribute hues evenly across the color wheel (0-360 degrees)
        // Starting from red (0°) and rotating through the spectrum
        // Use (sideLength) as divisor so colors don't wrap back to start
        const hue = Math.round(((value - 1) / sideLength) * 360);
        return `hsl(${hue}, 70%, 45%)`;
    }
    return undefined;
}

export function getValueColorStyleDark(value: number, sideLength: number): string | undefined {
    if (value >= 1 && value <= sideLength) {
        // Same hue distribution but with higher lightness for dark mode
        const hue = Math.round(((value - 1) / sideLength) * 360);
        return `hsl(${hue}, 70%, 60%)`;
    }
    return undefined;
}

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
    base: TransportSudoku["base"],
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
