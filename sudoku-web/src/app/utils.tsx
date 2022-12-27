import type { Position, TransportCell, TransportSudoku, CellViewValue } from "../types";

export function valuesFromSideLength(sideLength: TransportSudoku["sideLength"]): number[] {
    return Array.from(Array(sideLength).keys()).map(value => value + 1);
}

export function indexToPosition(index: number, base: TransportSudoku["base"]): Position {
    return {
        column: index % base,
        row: Math.floor(index / base),
    };
}

export function positionToIndex(pos: Position, base: TransportSudoku["base"]): number {
    return pos.column + pos.row * base;
}

export function cellPositionToBlockPosition(cellPosition: Position, base: TransportSudoku["base"]): Position {
    return {
        column: Math.floor(cellPosition.column / base),
        row: Math.floor(cellPosition.row / base),
    };
}

export function valueToString(value: CellViewValue["value"]): string {
    return value.toString(36);
}

export function baseToSideLength(base: number): number {
    return base ** 2;
}
export function baseToCellCount(base: number): number {
    return base ** 4;
}

export function blocksToCell(
    blocks: TransportSudoku["blocks"],
    pos: Position,
    base: TransportSudoku["base"]
): TransportCell {
    const blockPosition = cellPositionToBlockPosition(pos, base);
    const blockIndex = positionToIndex(blockPosition, base);
    const block = blocks[blockIndex];
    const cellPositionInBlock = {
        column: pos.column - blockPosition.column * base,
        row: pos.row - blockPosition.row * base,
    };
    const cellIndexInBlock = positionToIndex(cellPositionInBlock, base);

    // FIXME: crash: Cannot read properties of undefined (reading '0')
    //  Reproduction: load smaller base sudoku after interacting with larger base sudoku.
    return block[cellIndexInBlock];
}
