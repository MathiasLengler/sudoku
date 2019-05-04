export const valuesFromSideLength = (sideLength: TransportSudoku['sideLength']) => {
  return Array.from(Array(sideLength).keys()).map(value => value + 1);
};

export const indexToPosition = (index: number, base: TransportSudoku['base']): CellPosition => {
  return {
    column: index % base,
    row: Math.floor(index / base)
  }
};