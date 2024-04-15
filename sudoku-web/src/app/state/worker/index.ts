import { atom, selector } from "recoil";
import type { WorkerApi } from "./bg/worker";
import * as Comlink from "comlink";
import { loadCells } from "../cellsPersistence";
import { spawnWorker } from "./spawn";
import { fixupComlinkProxy } from "./comlinkProxyWrapper";
import type { DynamicCells, WasmCellWorld, WasmSudoku } from "../../../types";

export const workerState = atom<Worker>({
    key: "Worker",
    default: spawnWorker(),
});

export type RemoteWorkerApi = Comlink.Remote<WorkerApi>;
export type RemoteWasmSudoku = Comlink.Remote<WasmSudoku>;
export type RemoteWasmSudokuClass = Comlink.Remote<typeof WasmSudoku>;
export type RemoteWasmCellWorld = Comlink.Remote<WasmCellWorld>;
export type RemoteWasmCellWorldClass = Comlink.Remote<typeof WasmCellWorld>;

export const remoteWorkerApiState = selector<RemoteWorkerApi>({
    key: "RemoteWorkerApi",
    get: async ({ get }) => {
        const worker = get(workerState);
        const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
        console.debug("Worker init");
        await remoteWorkerApi.init();
        console.debug("Worker initialized");
        return remoteWorkerApi;
    },
});

export const isWorkerReadyState = selector<boolean>({
    key: "isWorkerReadyState",
    get: ({ get }) => {
        // remoteWorkerApiState initializes worker before returning
        const _remoteWorkerApi = get(remoteWorkerApiState);
        return true;
    },
});

export async function createRemoteWasmSudoku(
    RemoteWasmSudoku: RemoteWasmSudokuClass,
    cells?: DynamicCells,
): Promise<RemoteWasmSudoku> {
    if (cells) {
        console.debug("Restoring sudoku from cells");
        try {
            return await RemoteWasmSudoku.from_dynamic_cells(cells);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    console.debug("Generating initial sudoku");
    return await new RemoteWasmSudoku();
}

export const remoteWasmSudokuClassState = selector<RemoteWasmSudokuClass>({
    key: "remoteWasmSudokuClassState",
    get: ({ get }) => {
        const remoteWorkerApi = get(remoteWorkerApiState);
        return fixupComlinkProxy(remoteWorkerApi.WasmSudoku);
    },
});

export const remoteWasmSudokuState = atom<RemoteWasmSudoku>({
    key: "remoteWasmSudokuState",
    default: selector({
        key: "default-remoteWasmSudokuState",
        get: async ({ get }) => {
            const RemoteWasmSudoku = get(remoteWasmSudokuClassState);
            const cells = loadCells();
            return fixupComlinkProxy(await createRemoteWasmSudoku(RemoteWasmSudoku, cells));
        },
    }),
});
export const remoteWasmCellWorldClassState = selector<RemoteWasmCellWorldClass>({
    key: "remoteWasmCellWorldClassState",
    get: ({ get }) => {
        const remoteWorkerApi = get(remoteWorkerApiState);
        return fixupComlinkProxy(remoteWorkerApi.WasmCellWorld);
    },
});
