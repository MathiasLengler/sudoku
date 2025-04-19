import { atom, selector } from "recoil";
import type { WorkerApi } from "./worker";
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
export type RemoteWasmCellWorld = Comlink.Remote<WasmCellWorld>;

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

export async function createRemoteWasmSudoku(
    workerApi: Comlink.Remote<WorkerApi>,
    cells?: DynamicCells,
): Promise<RemoteWasmSudoku> {
    if (cells) {
        console.debug("Restoring sudoku from cells");
        try {
            return await workerApi.WasmSudoku.restore(cells);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    console.debug("Generating initial sudoku");
    return await new workerApi.WasmSudoku();
}

export const remoteWasmSudokuState = selector<RemoteWasmSudoku>({
    key: "RemoteWasmSudoku",
    get: async ({ get }) => {
        const remoteWorkerApi = get(remoteWorkerApiState);
        const cells = loadCells();
        return fixupComlinkProxy(await createRemoteWasmSudoku(remoteWorkerApi, cells));
    },
});
export const remoteWasmCellWorldState = selector<RemoteWasmCellWorld>({
    key: "RemoteWasmCellWorld",
    get: async ({ get }) => {
        const remoteWorkerApi = get(remoteWorkerApiState);
        return fixupComlinkProxy(await new remoteWorkerApi.WasmCellWorld());
    },
});

// export const legacyRemoteWorkerApiState = selector<LegacyRemoteWorkerApi>({
//     key: "RemoteWorkerApi",
//     get: async ({ get }) => {
//         const worker = get(workerState);
//         return await getRemoteWorkerApi(worker);
//     },
// });
