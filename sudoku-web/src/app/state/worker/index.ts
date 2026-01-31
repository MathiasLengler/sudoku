import * as Comlink from "comlink";
import { atom } from "jotai";
import { atomWithRefresh, RESET } from "jotai/utils";
import type { WasmCellWorld } from "../../../types";
import type { WasmSudokuWithTransfer, WorkerApi } from "./bg/worker";
import { fixupComlinkRemote, type SaveComlinkRemote } from "./comlinkProxyWrapper";
import { spawnWorker } from "./spawn";

// Keep track of the current worker instance for termination on reset
let currentWorker: Worker | null = null;

export const workerState = atomWithRefresh<Worker>(() => {
    // Terminate the previous worker if it exists
    if (currentWorker) {
        console.debug("Terminating previous worker before reset");
        currentWorker.terminate();
    }
    const worker = spawnWorker();
    currentWorker = worker;
    return worker;
});

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

/**
 * Check if the WASM module has panicked.
 * Returns true if a Rust panic has occurred and the worker should be reset.
 */
export async function checkWorkerPanic(remoteWorkerApi: RemoteWorkerApi): Promise<boolean> {
    try {
        return await remoteWorkerApi.hasPanicked();
    } catch (error) {
        // If we can't communicate with the worker, treat it as a panic
        console.error("Failed to check panic status, assuming panic:", error);
        return true;
    }
}

/**
 * Error class to represent a WASM panic.
 * Used to distinguish panics from other errors for proper handling.
 */
export class WasmPanicError extends Error {
    constructor(
        message: string,
        public readonly originalError?: unknown,
    ) {
        super(message);
        this.name = "WasmPanicError";
    }
}

export const remoteWasmSudokuClassState = atom<Promise<RemoteWasmSudokuClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkRemote(remoteWorkerApi.WasmSudoku);
});

export const remoteWasmCellWorldClassState = atom<Promise<RemoteWasmCellWorldClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkRemote(remoteWorkerApi.WasmCellWorld);
});

/**
 * Write-only atom to reset the worker after a panic.
 * This terminates the current worker, spawns a new one, and restores the Sudoku state.
 * Use by calling `set(resetWorkerAfterPanicAction)` from a component or action.
 */
export const resetWorkerAfterPanicAction = atom(null, (_get, set) => {
    console.warn("Resetting worker after WASM panic");

    // Refresh the worker state - this will terminate the old worker and spawn a new one
    set(workerState);

    // Reset the sudoku state to force re-initialization with persisted data
    set(remoteWasmSudokuState, RESET);
});
