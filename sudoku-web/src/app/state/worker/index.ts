import * as Comlink from "comlink";
import { atom } from "jotai";
import { atomWithRefresh } from "jotai/utils";
import type { WasmCellWorld } from "../../../types";
import type { WasmSudokuWithTransfer, WorkerApi } from "./bg/worker";
import { fixupComlinkRemote, type SaveComlinkRemote } from "./comlinkProxyWrapper";
import { spawnWorker } from "./spawn";

export const workerState = atomWithRefresh<Worker>(() => spawnWorker());

export type RemoteWorkerApi = Comlink.Remote<WorkerApi>;
export type UnsafeRemoteWasmSudoku = Comlink.Remote<WasmSudokuWithTransfer>;
export type RemoteWasmSudoku = SaveComlinkRemote<WasmSudokuWithTransfer>;
export type RemoteWasmSudokuClass = SaveComlinkRemote<typeof WasmSudokuWithTransfer>;
export type RemoteWasmCellWorld = SaveComlinkRemote<WasmCellWorld>;
export type RemoteWasmCellWorldClass = SaveComlinkRemote<typeof WasmCellWorld>;

export const remoteWorkerApiState = atom<Promise<RemoteWorkerApi>>(async (get) => {
    const worker = get(workerState);
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

export const remoteWasmSudokuClassState = atom<Promise<RemoteWasmSudokuClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkRemote(remoteWorkerApi.WasmSudoku);
});

export const remoteWasmCellWorldClassState = atom<Promise<RemoteWasmCellWorldClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkRemote(remoteWorkerApi.WasmCellWorld);
});
