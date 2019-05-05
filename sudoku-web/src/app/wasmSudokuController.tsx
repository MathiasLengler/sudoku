import {TypedWasmSudoku} from "../typedWasmSudoku";

export type onSudokuUpdate = (this: void, sudoku: TransportSudoku) => void;

export class WasmSudokuController {
  public constructor(
    private readonly wasmSudoku: TypedWasmSudoku,
    private readonly onSudokuUpdate: onSudokuUpdate,
    private readonly candidateMode: boolean,
    private readonly selectedPos: CellPosition,
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

    this.withSudokuUpdate(() => {
      if (this.candidateMode) {
        this.wasmSudoku.toggleCandidate(this.selectedPos, value);
      } else {
        this.wasmSudoku.setOrToggleValue(this.selectedPos, value);
      }
    });
  }

  public delete() {
    this.withSudokuUpdate(() => {
      // TODO: remove after cell refactoring
      this.wasmSudoku.setValue(this.selectedPos, 0);
      this.wasmSudoku.setCandidates(this.selectedPos, []);
    });
  }
}