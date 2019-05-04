import {WasmSudokuController} from "./controllers";
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
  sideLength: TransportSudoku['sideLength'],
  setSelectedPos: React.Dispatch<React.SetStateAction<CellPosition>>
) {
  useEffect(() => {
    const keyDownListener = (ev: KeyboardEvent) => {
      const {key} = ev;
      const value = keyToValue(key);

      if (value !== undefined) {
        sudokuController.setValue(selectedPos, value)
      } else {
        const newPos = keyToNewPos(key, selectedPos, sideLength);

        if (newPos !== undefined) {
          setSelectedPos(newPos);
        }
      }
    };

    document.addEventListener('keydown', keyDownListener);

    return () => {
      document.removeEventListener('keydown', keyDownListener);
    };
  }, [sudokuController, selectedPos, setSelectedPos, sideLength]);
}

export function useDebugSetters(sudokuController: WasmSudokuController) {
  useEffect(
    () => {
      let timer1 = setTimeout(() =>
        sudokuController.setValue({row: 1, column: 1}, 1), 1000);
      let timer2 = setTimeout(() =>
        sudokuController.setCandidates({row: 1, column: 0}, [1, 3]), 2000);

      return () => {
        clearTimeout(timer1);
        clearTimeout(timer2);
      }
    },
    // eslint-disable-next-line react-hooks/exhaustive-deps
    []
  );
}
