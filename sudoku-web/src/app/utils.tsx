export function valuesFromSideLength(sideLength: TransportSudoku['sideLength']) {
  return Array.from(Array(sideLength).keys()).map(value => value + 1);
}

export function indexToPosition(index: number, base: TransportSudoku['base']): CellPosition {
  return {
    column: index % base,
    row: Math.floor(index / base),
  }
}

export function positionToIndex(pos: CellPosition, base: TransportSudoku['base']): number {
  return pos.column + pos.row * base;
}

export function cellPositionToBlockPosition(cellPosition: CellPosition, base: TransportSudoku['base']): CellPosition {
  return {
    column: Math.floor(cellPosition.column / base),
    row: Math.floor(cellPosition.row / base),
  };
}