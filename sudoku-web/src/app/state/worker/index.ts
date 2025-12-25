import * as Comlink from "comlink";
import { atom } from "jotai";
import { atomWithDefault, atomWithRefresh } from "jotai/utils";
import type { DynamicGrid, WasmCellWorld, WasmSudoku } from "../../../types";
import { loadCells } from "../cellsPersistence";
import { GENERATE_FORM_DEFAULT_VALUES } from "../forms/generate";
import type { WorkerApi } from "./bg/worker";
import { fixupComlinkRemote, type SaveComlinkRemote } from "./comlinkProxyWrapper";
import { spawnWorker } from "./spawn";

export const workerState = atomWithRefresh<Worker>(() => spawnWorker());

export type RemoteWorkerApi = Comlink.Remote<WorkerApi>;
export type UnsafeRemoteWasmSudoku = Comlink.Remote<WasmSudoku>;
export type RemoteWasmSudoku = SaveComlinkRemote<WasmSudoku>;
export type RemoteWasmSudokuClass = SaveComlinkRemote<typeof WasmSudoku>;
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

async function createRemoteWasmSudoku(
    RemoteWasmSudoku: RemoteWasmSudokuClass,
    dynamicGrid?: DynamicGrid,
): Promise<UnsafeRemoteWasmSudoku> {
    if (dynamicGrid) {
        console.debug("Restoring sudoku");
        try {
            return await RemoteWasmSudoku.fromDynamicGrid(dynamicGrid);
        } catch (err) {
            console.error("Failed to restore persisted grid:", err);
        }
    }
    console.debug("Generating initial sudoku");
    return await RemoteWasmSudoku.generate(
        {
            base: GENERATE_FORM_DEFAULT_VALUES.base,
            prune: {
                target: "minimal",
                strategies: GENERATE_FORM_DEFAULT_VALUES.strategies,
                setAllDirectCandidates: GENERATE_FORM_DEFAULT_VALUES.setAllDirectCandidates,
                order: "random",
                startFromNearMinimalGrid: false,
            },
        },
        Comlink.proxy((progress) => {
            console.debug("Initial sudoku generation progress:", progress);
        }),
    );
}

export const remoteWasmSudokuClassState = atom<Promise<RemoteWasmSudokuClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkRemote(remoteWorkerApi.WasmSudoku);
});

export const remoteWasmSudokuState = atomWithDefault<RemoteWasmSudoku | Promise<RemoteWasmSudoku>>(async (get) => {
    const RemoteWasmSudoku = await get(remoteWasmSudokuClassState);
    const cells = loadCells();
    return fixupComlinkRemote(await createRemoteWasmSudoku(RemoteWasmSudoku, cells));
});
export const remoteWasmCellWorldClassState = atom<Promise<RemoteWasmCellWorldClass>>(async (get) => {
    const remoteWorkerApi = await get(remoteWorkerApiState);
    return fixupComlinkRemote(remoteWorkerApi.WasmCellWorld);
});
