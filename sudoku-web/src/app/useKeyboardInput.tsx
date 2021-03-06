import {Input, WasmSudokuController} from "./wasmSudokuController";
import * as React from "react";
import {KeyboardEvent} from "react";
import clamp from "lodash/clamp";
import {assertNever} from "assert-never";

function keyToValue(key: string): number | undefined {
  if (key.length === 1) {
    const value = parseInt(key, 36);

    if (Number.isInteger(value)) {
      return value
    }
  }
}

function keyToNewPos(key: string, selectedPos: CellPosition, sideLength: TransportSudoku['sideLength']): CellPosition | undefined {
  let {column, row} = selectedPos;
  switch (key) {
    case "ArrowUp":
      row -= 1;
      break;
    case "ArrowDown":
      row += 1;
      break;
    case "ArrowLeft":
      column -= 1;
      break;
    case "ArrowRight":
      column += 1;
      break;
    default:
      return;
  }

  column = clamp(column, 0, sideLength - 1);
  row = clamp(row, 0, sideLength - 1);

  return {row: row, column: column};
}

type ToolbarAction = "toggleCandidateMode" | "toggleStickyMode" | "delete" | "setAllDirectCandidates" | "undo";

function keyToToolbarAction(key: string): ToolbarAction | undefined {
  switch (key) {
    case " ":
      return "toggleCandidateMode";
    case "Delete":
      return "delete";
    case "Insert":
      return "setAllDirectCandidates";
    case "+":
      return "toggleStickyMode";
    case "Backspace":
      return "undo";
    default:
      return;
  }
}

export function makeKeyDownListener(
  sudokuController: WasmSudokuController,
  inputState: Input,
  sideLength: TransportSudoku["sideLength"],
): (ev: React.KeyboardEvent) => void {
  return (ev: KeyboardEvent): void => {
    const {key, altKey, ctrlKey, metaKey, shiftKey} = ev;

    if (altKey || ctrlKey || metaKey || shiftKey) {
      return;
    }

    const value = keyToValue(key);
    if (value !== undefined) {
      ev.preventDefault();
      return sudokuController.handleValue(value);
    }

    const {selectedPos} = inputState;

    const newPos = keyToNewPos(key, selectedPos, sideLength);
    if (newPos !== undefined) {
      ev.preventDefault();
      return sudokuController.handlePosition(newPos);
    }

    const toolbarAction = keyToToolbarAction(key);
    if (toolbarAction !== undefined) {
      ev.preventDefault();

      switch (toolbarAction) {
        case "toggleCandidateMode":
          sudokuController.toggleCandidateMode();
          break;
        case "setAllDirectCandidates":
          sudokuController.setAllDirectCandidates();
          break;
        case "delete":
          sudokuController.delete();
          break;
        case "toggleStickyMode":
          sudokuController.toggleStickyMode();
          break;
        case "undo":
          sudokuController.undo();
          break;
        default:
          assertNever(toolbarAction);
      }
    }
  }
}
