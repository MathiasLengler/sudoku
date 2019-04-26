interface TransportSudoku {
  cells: Array<TransportCell>,
  base: number,
  side_length: number,
  cell_count: number,
}

interface TransportCell {
  value?: number,
  candidates: Array<number>,
  position: Position,
}

interface Position {
  column: number,
  row: number,
}
