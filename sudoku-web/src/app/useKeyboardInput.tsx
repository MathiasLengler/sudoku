import {WasmSudokuController} from "./wasmSudokuController";
import * as React from "react";
import {useEffect} from "react";
import clamp from "lodash/clamp";

// TODO: Backspace/Delete key should delete cell
// TODO: key for candidate mode toggle

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

export function useKeyboardInput(
  sudokuController: WasmSudokuController,
  selectedPos: CellPosition,
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>,
  sideLength: TransportSudoku["sideLength"]
) {
  useEffect(() => {
    const keyDownListener = (ev: KeyboardEvent) => {
      const {key, altKey, ctrlKey, metaKey, shiftKey} = ev;

      if (altKey || ctrlKey || metaKey || shiftKey) {
        return;
      }

      const value = keyToValue(key);

      if (value !== undefined) {
        ev.preventDefault();
        return sudokuController.handleValue(value);
      }

      const newPos = keyToNewPos(key, selectedPos, sideLength);

      if (newPos !== undefined) {
        ev.preventDefault();
        return setSelectedPos(newPos);
      }
    };

    document.addEventListener('keydown', keyDownListener);

    return () => {
      document.removeEventListener('keydown', keyDownListener);
    };
  }, [sudokuController, selectedPos, setSelectedPos, sideLength]);
}
