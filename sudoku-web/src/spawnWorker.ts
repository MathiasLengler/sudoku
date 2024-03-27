import { WORKER_BOOT_UP_MESSAGE } from "./constants";
import * as Comlink from "comlink";
import type { WorkerApi } from "./worker";
import { loadCells } from "./app/state/cellsPersistence";
import type { DynamicCells, WasmCellWorld, WasmSudoku } from "./types";

export type WasmSudokuProxy = Comlink.Remote<WasmSudoku>;
export type WasmCellWorldProxy = Comlink.Remote<WasmCellWorld>;
export type RemoteWorkerApi = {
    wasmSudokuProxy: WasmSudokuProxy;
    wasmCellWorldProxy: WasmCellWorldProxy;
};

function fixupComlinkProxy<T>(comlinkProxy: Comlink.Remote<T>): Comlink.Remote<T> {
    return new Proxy(
        // Target a plain object for `typeof proxy === "object"`
        // Reference: https://stackoverflow.com/a/42493645
        {},
        {
            // eslint-disable-next-line @typescript-eslint/no-explicit-any
            get: (_target, property: string): any => {
                // Not a thenable
                // Reference: https://stackoverflow.com/a/53890904
                if (property === "then") {
                    return undefined;
                }
                // Fix error when passing proxy to `JSON.stringify`
                if (property === "toJSON") {
                    return () => {
                        console.warn("JSON.stringify called on Comlink proxy");
                        return { COMLINK_PROXY_PLACEHOLDER: true };
                    };
                }
                return (comlinkProxy as unknown as Record<string, unknown>)[property];
            },
        },
    ) as unknown as Comlink.Remote<T>;
}

export async function spawnWorker() {
    console.debug("Spawning worker");
    const worker = new Worker(new URL("./worker.tsx", import.meta.url), { name: "SudokuWasmWorker" });
    if (process.env.NODE_ENV !== "production") {
        console.debug("Attaching debug event listeners");
        worker.addEventListener("message", (ev) => {
            console.debug("Worker message TX:", ev.data);
        });
        worker.addEventListener("error", (ev) => {
            console.error("Worker error:", ev);
        });
        worker.addEventListener("messageerror", (ev) => {
            console.error("Worker messageerror:", ev);
        });
    }
    console.debug("Waiting for worker boot up message");
    const bootUpMessage = await new Promise((resolve, reject) => {
        worker.addEventListener(
            "message",
            (ev: MessageEvent) => {
                if (ev.data === WORKER_BOOT_UP_MESSAGE) {
                    resolve(ev.data);
                } else {
                    reject(new Error(`Unexpected message: ${ev.data}`));
                }
            },
            { once: true },
        );
    });
    console.debug("Received worker boot up message:", bootUpMessage);

    return worker;
}

async function createWasmSudokuProxy(
    workerApi: Comlink.Remote<WorkerApi>,
    cells?: DynamicCells,
): Promise<WasmSudokuProxy> {
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

export async function getRemoteWorkerApi(worker: Worker): Promise<RemoteWorkerApi> {
    const workerApi = Comlink.wrap<WorkerApi>(worker, {});

    const cells = loadCells();
    console.debug("Worker init");
    await workerApi.init();
    console.debug("Worker initialized");

    const wasmSudokuProxy = await createWasmSudokuProxy(workerApi, cells);
    // TODO: restore
    const wasmCellWorldProxy: WasmCellWorldProxy = await new workerApi.WasmCellWorld();

    // Important: wasmSudokuProxy is a Proxy.
    // We must be careful when setting it's state, since the Proxy gets misinterpreted as a Function or Promise.
    return {
        wasmSudokuProxy: fixupComlinkProxy(wasmSudokuProxy),
        wasmCellWorldProxy: fixupComlinkProxy(wasmCellWorldProxy),
    };
}
