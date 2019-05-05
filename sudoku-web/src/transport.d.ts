interface TransportSudoku {
  cells: TransportCell[];
  base: number;
  sideLength: number;
  cellCount: number;
}

interface CellPosition {
  column: number;
  row: number;
}

interface BaseCell {
  position: CellPosition;
}

interface ValueCell extends BaseCell {
  kind: "Value";
  value: number;
}

interface CandidatesCell extends BaseCell {
  kind: "Candidates";
  candidates: number[];
}

type TransportCell = ValueCell | CandidatesCell;