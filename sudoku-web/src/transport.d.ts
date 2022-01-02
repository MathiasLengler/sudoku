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

interface TransportCellContext {
    position: CellPosition;
    incorrectValue: boolean;
}

interface ValueCell {
    kind: "value";
    fixed: boolean;
    value: number;
}

interface CandidatesCell {
    kind: "candidates";
    candidates: number[];
}

type Cell = ValueCell | CandidatesCell;

type TransportCell = TransportCellContext & Cell;

interface GeneratorSettings {
    base: number;
    target: GeneratorTarget;
}

type GeneratorTarget =
    | "minimal"
    | "filled"
    | {
          fromMinimal: {
              distance: number;
          };
      }
    | {
          fromFilled: {
              distance: number;
          };
      };

type GridFormat = "givensLine" | "givensGrid" | "candidates";
