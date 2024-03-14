import { atom, selector } from "recoil";
import { spawnWorker, type RemoteWorkerApi, getRemoteWorkerApi } from "../../spawnWorker";

export const workerState = atom<Worker>({
    key: "Worker",
    default: spawnWorker(),
});

export const remoteWorkerApiState = selector<RemoteWorkerApi>({
    key: "RemoteWorkerApi",
    get: async ({ get }) => {
        const worker = get(workerState);
        return await getRemoteWorkerApi(worker);
    },
});
