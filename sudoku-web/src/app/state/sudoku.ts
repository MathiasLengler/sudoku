import { atom } from "jotai";
import { eagerAtom } from "jotai-eager";
import { atomWithDefault } from "jotai/utils";
import type { IsEqual } from "type-fest";
import { z } from "zod";
import type { BaseEnum, DynamicCells, DynamicGrid, TransportCell, TransportSudoku } from "../../types";
import { assert } from "../../typeUtils";
import { hintState } from "./hint";
import { remoteWasmSudokuState } from "./worker";

const valueSchema = z.int().positive();

const DynamicCellSchema = z.discriminatedUnion("kind", [
    z.object({ kind: z.literal("value"), value: valueSchema, fixed: z.boolean() }),
    z.object({ kind: z.literal("candidates"), candidates: z.array(valueSchema) }),
]);

export const DynamicCellsSchema = z.array(DynamicCellSchema);

assert<IsEqual<z.infer<typeof DynamicCellsSchema>, DynamicCells>>();
assert<IsEqual<z.infer<typeof DynamicCellsSchema>, DynamicGrid>>();

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

export const sudokuBaseState = eagerAtom<BaseEnum>((get) => get(sudokuState).base);
export const sudokuSideLengthState = eagerAtom<number>((get) => get(sudokuState).sideLength);
export const sudokuCellsState = eagerAtom<TransportCell[]>((get) => get(sudokuState).cells);
export const sudokuBlocksIndexesState = eagerAtom<TransportSudoku["blocksIndexes"]>(
    (get) => get(sudokuState).blocksIndexes,
);
export const sudokuCanUndoState = eagerAtom<boolean>((get) => !!get(hintState) || get(sudokuState).history.canUndo);
export const sudokuCanRedoState = eagerAtom<boolean>((get) => get(sudokuState).history.canRedo);
export const sudokuIsSolvedState = eagerAtom<boolean>((get) => get(sudokuState).isSolved);
export const sudokuSolutionState = eagerAtom<TransportSudoku["solution"]>((get) => get(sudokuState).solution);
