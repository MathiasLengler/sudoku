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
    console.log("WasmSudokuController", "handleValue", this, value);

    this.withSudokuUpdate(() => {
      if (this.candidateMode) {
        this.wasmSudoku.toggleCandidate(this.selectedPos, value);
      } else {
        this.wasmSudoku.setValue(this.selectedPos, value);
      }
    });
  }
}