import {TypedWasmSudoku} from "../typedWasmSudoku";
import * as React from "react";
import {blocksToCell} from "./utils";
import isEqual from "lodash/isEqual";

export type onSudokuUpdate = (this: void, sudoku: TransportSudoku) => void;

export interface Input {
  stickyMode: boolean;
  candidateMode: boolean;
  selectedPos: CellPosition;
  selectedCell: TransportCell;
  selectedValue: number;
}

export class WasmSudokuController {
  public constructor(
    private readonly wasmSudoku: TypedWasmSudoku,
    private readonly sudoku: TransportSudoku,
    private readonly onSudokuUpdate: onSudokuUpdate,
    private readonly input: Input,
    private readonly setInput: React.Dispatch<React.SetStateAction<Omit<Input, "selectedCell">>>,
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

  private checkFixed(): boolean {
    if (this.input.selectedCell.fixed) {
      console.warn("WasmSudokuController", "cannot modify a fixed cell", this.input.selectedCell);

      return true;
    } else {
      return false;
    }
  }

  public handlePosition(newSelectedPosition: CellPosition, move: boolean = false) {
    console.log("WasmSudokuController", "handlePosition", newSelectedPosition, move);

    const {stickyMode, selectedPos, selectedValue} = this.input;

    if (move && isEqual(selectedPos, newSelectedPosition)) {
      return;
    }

    this.setSelectedPosition(newSelectedPosition);

    if (stickyMode) {
      this.setSelectedCell(selectedValue);
    }
  }

  private setSelectedPosition(selectedPos: CellPosition) {
    const {sudoku: {base, blocks}} = this;
    const selectedCell = blocksToCell(blocks, selectedPos, base);

    this.setInput((prevInput) => ({...prevInput, selectedPos}));
    this.input.selectedPos = selectedPos;
    this.input.selectedCell = selectedCell;
  }

  public handleValue(value: number) {
    console.log("WasmSudokuController", "handleValue", value);

    const {stickyMode} = this.input;

    if (value > this.sideLength) {
      console.warn("WasmSudokuController", "tried to handle value greater than current sudoku allows:", value);

      return;
    }

    if (stickyMode) {
      this.setInput((prevInput) => ({...prevInput, selectedValue: value}));
    } else {
      this.setSelectedCell(value);
    }
  }

  private setSelectedCell(value: number) {
    const {candidateMode, selectedPos} = this.input;

    if (this.checkFixed()) {
      return;
    }

    this.withSudokuUpdate(() => {
      if (value === 0) {
        this.wasmSudoku.delete(selectedPos);
      } else {
        if (candidateMode) {
          this.wasmSudoku.toggleCandidate(selectedPos, value);
        } else {
          this.wasmSudoku.setOrToggleValue(selectedPos, value);
        }
      }
    });
  }

  public delete() {
    if (this.checkFixed()) {
      return;
    }

    this.withSudokuUpdate(() => {
      this.wasmSudoku.delete(this.input.selectedPos);
    });
  }

  public setAllDirectCandidates() {
    this.withSudokuUpdate(() => {
      this.wasmSudoku.setAllDirectCandidates();
    });
  }

  public toggleCandidateMode() {
    this.setInput((prevInput) => ({...prevInput, candidateMode: !prevInput.candidateMode}))
  }

  public toggleStickyMode() {
    this.setInput((prevInput) => ({...prevInput, stickyMode: !prevInput.stickyMode}))
  }
}