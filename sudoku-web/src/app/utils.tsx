export const valuesFromSideLength = (side_length: TransportSudoku['side_length']) => {
  return Array.from(Array(side_length).keys()).map(value => value + 1);
};

export const indexToPosition = (index: number, base: TransportSudoku['base']): CellPosition => {
  return {
    column: index % base,
    row: Math.floor(index / base)
  }
};