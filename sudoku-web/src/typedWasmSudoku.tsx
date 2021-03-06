import {WasmSudoku} from "../../sudoku-wasm/pkg";

export class TypedWasmSudoku {
  public constructor(private wasmSudoku: WasmSudoku) {
  }

  public getSudoku(): TransportSudoku {
    return this.wasmSudoku.get_sudoku()
  }

  public setValue(pos: CellPosition, value: number): void {
    return this.wasmSudoku.set_value(pos, value);
  }

  public setOrToggleValue(pos: CellPosition, value: number): void {
    return this.wasmSudoku.set_or_toggle_value(pos, value);
  }

  public setCandidates(pos: CellPosition, candidates: number[]): void {
    return this.wasmSudoku.set_candidates(pos, candidates);
  }

  public toggleCandidate(pos: CellPosition, candidate: number): void {
    return this.wasmSudoku.toggle_candidate(pos, candidate);
  }

  public delete(pos: CellPosition): void {
    return this.wasmSudoku.delete(pos);
  }

  public setAllDirectCandidates(): void {
    return this.wasmSudoku.set_all_direct_candidates();
  }

  public undo(): void {
    return this.wasmSudoku.undo();
  }

  public generate(settings: GeneratorSettings): void {
    return this.wasmSudoku.generate(settings);
  }

  public import(input: string): void {
    return this.wasmSudoku.import(input);
  }

  public solveSingleCandidates(): void {
    return this.wasmSudoku.solve_single_candidates();
  }

  public groupReduction(): void {
    return this.wasmSudoku.group_reduction();
  }
}
