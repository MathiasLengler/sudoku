// TODO: expose rust utility functions

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

export function valueToString(value: ValueCell['value']): string {
  return value.toString(36);
}

export function cellFromBlocks(blocks: TransportSudoku['blocks'], pos: CellPosition, base: TransportSudoku['base']): TransportCell {
  const blockPosition = cellPositionToBlockPosition(pos, base);
  const blockIndex = positionToIndex(blockPosition, base);
  const block = blocks[blockIndex];
  const cellPositionInBlock = {
    column: pos.column - blockPosition.column * base,
    row: pos.row - blockPosition.row * base
  };
  const cellIndexInBlock = positionToIndex(cellPositionInBlock, base);
  return block[cellIndexInBlock];
}

// noinspection JSUnusedLocalSymbols
export function assertNever(param: never) {
}