import { WORKER_BOOT_UP_MESSAGE } from "./constants";
import * as Comlink from "comlink";
import type { WorkerApi } from "./worker";
import { loadCellViews } from "./app/persistence";
import type { WasmSudoku } from "./types";

export type RemoteWorkerApi = {
    wasmSudokuProxy: WasmSudokuProxy;
};
export type WasmSudokuProxy = Comlink.Remote<WasmSudoku>;

function fixupWasmSudokuProxy(wasmSudokuProxy: WasmSudokuProxy): WasmSudokuProxy {
    return new Proxy(
        // Target a plain object for `typeof wasmSudokuProxy === "object"`
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
                return (wasmSudokuProxy as unknown as Record<string, unknown>)[property];
            },
        }
    ) as unknown as WasmSudokuProxy;
}

export async function spawnWorker() {
    console.debug("Spawning worker");
    const worker = new Worker(new URL("./worker.tsx", import.meta.url));
    if (process.env.NODE_ENV !== "production") {
        console.debug("Attaching debug event listeners");
        worker.addEventListener("message", ev => {
            console.debug("Worker message TX:", ev.data);
        });
        worker.addEventListener("error", ev => {
            console.error("Worker error:", ev);
        });
        worker.addEventListener("messageerror", ev => {
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
            { once: true }
        );
    });
    console.debug("Received worker boot up message:", bootUpMessage);

    return worker;
}

export async function getRemoteWorkerApi(worker: Worker): Promise<RemoteWorkerApi> {
    const workerApi = Comlink.wrap<WorkerApi>(worker, {});

    const cellViews = loadCellViews();
    console.debug("Initializing worker with cells", cellViews);
    await workerApi.init(cellViews);
    console.debug("Worker initialized");

    // Incorrect type: `workerApi.typedWasmSudoku` is not wrapped in a Promise.
    const wasmSudokuProxy = workerApi.typedWasmSudoku as unknown as WasmSudokuProxy;

    // Important: wasmSudokuProxy is a Proxy.
    // We must be careful when setting it's state, since the Proxy gets misinterpreted as a Function or Promise.
    return {
        wasmSudokuProxy: fixupWasmSudokuProxy(wasmSudokuProxy),
    };
}
