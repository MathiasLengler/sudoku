import {WasmSudoku} from "../../sudoku-wasm/pkg";

export class TypedWasmSudoku {
  public constructor(private rustSudoku: WasmSudoku) {
  }

  public sayHello(): void {
    return this.rustSudoku.say_hello()
  }

  public getSudoku(): TransportSudoku {
    return this.rustSudoku.get_sudoku()
  }

  public setValue(pos: CellPosition, value: number): number {
    return this.rustSudoku.set_value(pos, value);
  }

  public setCandidates(pos: CellPosition, candidates: number[]) {
    return this.rustSudoku.set_candidates(pos, candidates);
  }

  public toggleCandidate(pos: CellPosition, candidate: number): boolean {
    return this.rustSudoku.toggle_candidate(pos, candidate);
  }
}