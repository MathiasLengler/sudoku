import {TypedWasmSudoku} from "../typedWasmSudoku";

export type onSudokuUpdate = (this: void, sudoku: TransportSudoku) => void;

export class WasmSudokuController {
  public constructor(
    private readonly wasmSudoku: TypedWasmSudoku,
    private readonly onSudokuUpdate: onSudokuUpdate,
    private readonly candidateMode: boolean,
    private readonly selectedPos: CellPosition,
    private readonly selectedCell: TransportCell,
    private readonly sideLength: TransportSudoku['sideLength'],
  ) {
  }

  private updateSudoku() {
    this.onSudokuUpdate(this.wasmSudoku.getSudoku())
  }

  private withSudokuUpdate<T>(f: () => T): T {
    let ret = f();

    this.updateSudoku();

    return ret;
  }

  public handleValue(value: number) {
    console.log("WasmSudokuController", "handleValue", value);

    if (value > this.sideLength) {
      console.warn("WasmSudokuController", "tried to handle value greater than current sudoku allows:", value);

      return;
    }

    if (this.selectedCell.fixed) {
      console.warn("WasmSudokuController", "cannot modify a fixed cell", this.selectedCell);

      return;
    }

    this.withSudokuUpdate(() => {
      if (value === 0) {
        this.wasmSudoku.delete(this.selectedPos);
      } else {
        if (this.candidateMode) {
          this.wasmSudoku.toggleCandidate(this.selectedPos, value);
        } else {
          this.wasmSudoku.setOrToggleValue(this.selectedPos, value);
        }
      }
    });
  }

  public delete() {
    this.withSudokuUpdate(() => {
      this.wasmSudoku.delete(this.selectedPos);
    });
  }

  public setAllDirectCandidates() {
    this.withSudokuUpdate(() => {
      this.wasmSudoku.setAllDirectCandidates();
    });
  }
}