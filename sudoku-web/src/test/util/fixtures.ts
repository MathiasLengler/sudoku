/* eslint-disable no-empty-pattern */
/* eslint-disable react-hooks/rules-of-hooks */

import * as Comlink from "comlink";
import { test as baseTest } from "vitest";
import type { RemoteWorkerApi } from "../../app/state/worker";
import type { WorkerApi } from "../../app/state/worker/bg/worker";
import { spawnWorker } from "../../app/state/worker/spawn";

type WorkerFixtures = {
    remoteWorkerApi: RemoteWorkerApi;
};

export const test = baseTest.extend<WorkerFixtures>({
    remoteWorkerApi: async ({}, use) => {
        const worker = spawnWorker();

        const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
        await remoteWorkerApi.init(1);

        await use(remoteWorkerApi);

        worker.terminate();
    },
});
