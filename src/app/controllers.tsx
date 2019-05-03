import {TypedWasmSudoku} from "../index";

export type onSudokuUpdate = (this: void, sudoku: TransportSudoku) => void;

export class WasmSudokuController {
  constructor(
    private readonly wasmSudoku: TypedWasmSudoku,
    private readonly onSudokuUpdate: onSudokuUpdate
  ) {
  }

  private updateSudoku() {
    this.onSudokuUpdate(this.wasmSudoku.get_sudoku())
  }

  private withSudokuUpdate<T>(f: () => T): T {
    let ret = f();

    this.updateSudoku();

    return ret;
  }

  setValue(pos: CellPosition, value: number): number {
    console.log("WasmSudokuController", "setValue", pos, value);
    return this.withSudokuUpdate(() =>
      this.wasmSudoku.setValue(pos, value));
  }

  setCandidates(pos: CellPosition, candidates: number[]) {
    console.log("WasmSudokuController", "setCandidates", pos, candidates);
    return this.withSudokuUpdate(() =>
      this.wasmSudoku.setCandidates(pos, candidates));
  }
}