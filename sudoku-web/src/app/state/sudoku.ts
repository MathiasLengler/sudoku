// TODO: migrate to jotai
// Recoil is not compatible with react 19: https://github.com/facebookexperimental/Recoil/issues/2318
import { atom, selector } from "recoil";
import { z } from "zod";
import type { BaseEnum, TransportCell, TransportSudoku } from "../../types";
import { hintState } from "./hint";
import { remoteWasmSudokuState } from "./worker";

const valueSchema = z.number().int().positive().safe();

const DynamicCellSchema = z.discriminatedUnion("kind", [
    z.object({ kind: z.literal("value"), value: valueSchema, fixed: z.boolean() }),
    z.object({ kind: z.literal("candidates"), candidates: z.array(valueSchema) }),
]);

export const DynamicCellsSchema = z.array(DynamicCellSchema);

// TODO: evaluate IOC
//  WasmSudoku could have callback, which is called whenever the sudoku is updated
//  Observer pattern
//  could simplify the UI code.
//  More relevant for CellWorld ("patches")

export const sudokuState = atom<TransportSudoku>({
    key: "Sudoku",
    default: selector({
        key: "DefaultSudoku",
        get: async ({ get }) => {
            const remoteWasmSudoku = get(remoteWasmSudokuState);
            return await remoteWasmSudoku.getTransportSudoku();
        },
    }),
});

export const sudokuBaseState = selector<BaseEnum>({
    key: "Sudoku.base",
    get: ({ get }) => get(sudokuState).base,
});
export const sudokuSideLengthState = selector<number>({
    key: "Sudoku.sideLength",
    get: ({ get }) => get(sudokuState).sideLength,
});
export const sudokuCellsState = selector<TransportCell[]>({
    key: "Sudoku.cells",
    get: ({ get }) => get(sudokuState).cells,
});
export const sudokuBlocksIndexesState = selector<TransportSudoku["blocksIndexes"]>({
    key: "Sudoku.blocksIndexes",
    get: ({ get }) => get(sudokuState).blocksIndexes,
});
export const sudokuCanUndoState = selector<TransportSudoku["history"]["canUndo"]>({
    key: "Sudoku.canUndo",
    get: ({ get }) => {
        // showing of a hint can be undone
        return !!get(hintState) || get(sudokuState).history.canUndo;
    },
});
export const sudokuCanRedoState = selector<TransportSudoku["history"]["canRedo"]>({
    key: "Sudoku.canRedo",
    get: ({ get }) => get(sudokuState).history.canRedo,
});
export const sudokuIsSolvedState = selector<TransportSudoku["isSolved"]>({
    key: "Sudoku.isSolved",
    get: ({ get }) => get(sudokuState).isSolved,
});
export const sudokuSolutionState = selector<TransportSudoku["solution"]>({
    key: "Sudoku.solution",
    get: ({ get }) => get(sudokuState).solution,
});
