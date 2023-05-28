import { atom, selector } from "recoil";
import type { TransportCell, TransportSudoku } from "../../../../sudoku-rs/bindings";
import type { RemoteWorkerApi } from "../../spawnWorker";
import { getRemoteWorkerApi, spawnWorker } from "../../spawnWorker";

export const workerState = atom<Worker>({
    key: "Worker",
    default: spawnWorker(),
});

export const remoteWorkerApiState = selector<RemoteWorkerApi>({
    key: "RemoteWorkerApi",
    get: async ({ get }) => {
        const worker = get(workerState);
        return await getRemoteWorkerApi(worker);
    },
});
export const sudokuState = atom<TransportSudoku>({
    key: "Sudoku",
    default: selector({
        key: "DefaultSudoku",
        get: async ({ get }) => {
            const wasmSudokuProxy = get(remoteWorkerApiState).wasmSudokuProxy;
            return await wasmSudokuProxy.getSudoku();
        },
    }),
});

export const sudokuBaseState = selector<number>({
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
export const sudokuBlocksIndicesState = selector<TransportSudoku["blocksIndices"]>({
    key: "Sudoku.blocksIndices",
    get: ({ get }) => get(sudokuState).blocksIndices,
});
export const sudokuCanUndoState = selector<TransportSudoku["canUndo"]>({
    key: "Sudoku.canUndo",
    get: ({ get }) => get(sudokuState).canUndo,
});
export const sudokuCanRedoState = selector<TransportSudoku["canRedo"]>({
    key: "Sudoku.canRedo",
    get: ({ get }) => get(sudokuState).canRedo,
});
export const sudokuIsSolvedState = selector<TransportSudoku["isSolved"]>({
    key: "Sudoku.isSolved",
    get: ({ get }) => get(sudokuState).isSolved,
});
