import * as Comlink from "comlink";
import { atom } from "jotai";
import { atomWithDefault, atomWithRefresh } from "jotai/utils";
import type { DynamicCells, WasmCellWorld, WasmSudoku } from "../../../types";
import { loadCells } from "../cellsPersistence";
import type { WorkerApi } from "./bg/worker";
import { fixupComlinkProxy } from "./comlinkProxyWrapper";
import { spawnWorker } from "./spawn";

export const workerState = atomWithRefresh<Promise<Worker>>(async () => await spawnWorker());

export type RemoteWorkerApi = Comlink.Remote<WorkerApi>;
export type RemoteWasmSudoku = Comlink.Remote<WasmSudoku>;
export type RemoteWasmSudokuClass = Comlink.Remote<typeof WasmSudoku>;
export type RemoteWasmCellWorld = Comlink.Remote<WasmCellWorld>;
export type RemoteWasmCellWorldClass = Comlink.Remote<typeof WasmCellWorld>;

export const remoteWorkerApiState = atom<Promise<RemoteWorkerApi>>(async (get) => {
    const worker = await get(workerState);
    const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
    console.debug("Worker init");
    await remoteWorkerApi.init();
    console.debug("Worker initialized");
    return remoteWorkerApi;
});

export const isWorkerReadyState = atom<Promise<boolean>>(async (get) => {
    await get(remoteWorkerApiState);
    return true;
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
    return await RemoteWasmSudoku.new();
}

export const remoteWasmSudokuClassState = atom<Promise<RemoteWasmSudokuClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkProxy(remoteWorkerApi.WasmSudoku);
});

export const remoteWasmSudokuState = atomWithDefault<RemoteWasmSudoku | Promise<RemoteWasmSudoku>>(async (get) => {
    const RemoteWasmSudoku = await get(remoteWasmSudokuClassState);
    const cells = loadCells();
    return fixupComlinkProxy(await createRemoteWasmSudoku(RemoteWasmSudoku, cells));
});
export const remoteWasmCellWorldClassState = atom<Promise<RemoteWasmCellWorldClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkProxy(remoteWorkerApi.WasmCellWorld);
});
