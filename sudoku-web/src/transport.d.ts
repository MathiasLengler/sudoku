type Block = TransportCell[];

interface TransportSudoku {
  blocks: Block[];
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
  fixed: boolean;
  incorrectValue: boolean;
}

interface ValueCell extends BaseCell {
  kind: "value";
  value: number;
}

interface CandidatesCell extends BaseCell {
  kind: "candidates";
  candidates: number[];
}

type TransportCell = ValueCell | CandidatesCell;

interface GeneratorSettings {
  base: number;
  target: GeneratorTarget;
}

type GeneratorTarget = "minimal" | "filled" | {
  fromMinimal: {
    distance: number;
  };
} | {
  fromFilled: {
    distance: number;
  };
};
