interface TransportSudoku {
  cells: Array<TransportCell>,
  base: number,
  sideLength: number,
  cellCount: number,
}

interface TransportCell {
  value?: number,
  candidates: Array<number>,
  position: CellPosition,
}

interface CellPosition {
  column: number,
  row: number,
}
