interface TransportSudoku {
  cells: TransportCell[];
  base: number;
  sideLength: number;
  cellCount: number;
}

interface TransportCell {
  value?: number;
  candidates: number[];
  position: CellPosition;
}

interface CellPosition {
  column: number;
  row: number;
}
