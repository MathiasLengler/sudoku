import * as Comlink from "comlink";
import { test as baseTest } from "vitest";
import type { WorkerApi } from "../../app/state/worker/bg/worker";
import { spawnWorker } from "../../app/state/worker/spawn";

export const test = baseTest
    // Rayon threads for the worker's wasm-bindgen-rayon pool. Defaults to 1; override per-suite
    // via `test.override("threadCount", n)` for tests that need real multi-threading.
    .extend("threadCount", 1)
    .extend("remoteWorkerApi", async ({ threadCount }, { onCleanup }) => {
        const worker = spawnWorker();
        const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
        await remoteWorkerApi.init(threadCount);
        onCleanup(() => worker.terminate());
        return remoteWorkerApi;
    });
