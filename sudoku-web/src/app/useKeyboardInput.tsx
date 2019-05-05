import {WasmSudokuController} from "./wasmSudokuController";
import * as React from "react";
import {useEffect} from "react";
import {clamp} from "lodash";

function keyToValue(key: string): number | undefined {
  const value = parseInt(key);

  if (Number.isInteger(value)) {
    return value
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

  return {row, column};
}

export function useKeyboardInput(
  sudokuController: WasmSudokuController,
  selectedPos: CellPosition,
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>,
  sideLength: TransportSudoku["sideLength"]
) {
  useEffect(() => {
    const keyDownListener = (ev: KeyboardEvent) => {
      const {key} = ev;
      const value = keyToValue(key);

      if (value !== undefined) {
        return sudokuController.handleValue(value);
      }

      const newPos = keyToNewPos(key, selectedPos, sideLength);

      if (newPos !== undefined) {
        setSelectedPos(newPos);
      }
    };

    document.addEventListener('keydown', keyDownListener);

    return () => {
      document.removeEventListener('keydown', keyDownListener);
    };
  }, [sudokuController, selectedPos, setSelectedPos, sideLength]);
}
