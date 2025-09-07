import { atom } from "jotai";
import { z } from "zod";
import type { BaseEnum, TransportCell, TransportSudoku } from "../../types";
import { hintState } from "./hint";
import { remoteWasmSudokuState } from "./worker";
import { atomWithDefault } from "jotai/utils";

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

export const sudokuState = atomWithDefault<TransportSudoku | Promise<TransportSudoku>>(async (get) => {
    console.info("sudokuState: fetching initial sudoku");
    const remoteWasmSudoku = await get(remoteWasmSudokuState);
    return await remoteWasmSudoku.getTransportSudoku();
});

export const gameCounterState = atom<number>(0);

export const sudokuBaseState = atom<Promise<BaseEnum>>(async (get) => (await get(sudokuState)).base);
export const sudokuSideLengthState = atom<Promise<number>>(async (get) => (await get(sudokuState)).sideLength);
export const sudokuCellsState = atom<Promise<TransportCell[]>>(async (get) => (await get(sudokuState)).cells);
export const sudokuBlocksIndexesState = atom<Promise<TransportSudoku["blocksIndexes"]>>(
    async (get) => (await get(sudokuState)).blocksIndexes,
);
export const sudokuCanUndoState = atom<Promise<boolean>>(
    async (get) => !!get(hintState) || (await get(sudokuState)).history.canUndo,
);
export const sudokuCanRedoState = atom<Promise<boolean>>(async (get) => (await get(sudokuState)).history.canRedo);
export const sudokuIsSolvedState = atom<Promise<boolean>>(async (get) => (await get(sudokuState)).isSolved);
export const sudokuSolutionState = atom<Promise<TransportSudoku["solution"]>>(
    async (get) => (await get(sudokuState)).solution,
);
