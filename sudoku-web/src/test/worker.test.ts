/* eslint-disable no-empty-pattern */
/* eslint-disable react-hooks/rules-of-hooks */

import * as Comlink from "comlink";
import { test as baseTest, describe, expect } from "vitest";
import { init } from "../app/state/worker/bg/init";
import type { WorkerApi } from "../app/state/worker/bg/worker";
import { spawnWorker } from "../app/state/worker/spawn";
import type { RemoteWorkerApi } from "../app/state/worker";
import { getWasmSudokuSamples } from "./util/sudoku";
import { WasmSudoku } from "sudoku-wasm";

type WorkerFixtures = {
    remoteWorkerApi: RemoteWorkerApi;
};

const test = baseTest.extend<WorkerFixtures>({
    remoteWorkerApi: async ({}, use) => {
        const worker = await spawnWorker();

        const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
        await remoteWorkerApi.init();

        await use(remoteWorkerApi);

        worker.terminate();
    },
});

describe("worker", async () => {
    // Init foreground WASM.
    await init(1);

    const base = 3;
    const seed = 42n;

    // TODO: test and bench worker communication
    // Content: Grid, TransportSudoku, CellWorld
    //  each serialized with the most efficient method
    // Channel:
    //  baseline no worker
    //  comlink proxied class return (return structured cloned)
    //  comlink proxied class return (return transferred Uint8Array)
    // Goal: find new architecture: which parts are executed into worker, which parts stay on main thread?
    //  Probably: only heavy computations in worker, light computations on main thread
    //  We will need to manage multiple class instances, transferring the state between main thread and worker depending on the operation.

    describe("communication", () => {
        describe("WasmSudoku", () => {
            for (const { name, wasmSudoku } of getWasmSudokuSamples(base, seed)) {
                describe(name, () => {
                    describe("copied", () => {
                        test("roundtrip TransportSudoku", async ({ remoteWorkerApi }) => {
                            const transportSudoku = wasmSudoku.getTransportSudoku();
                            const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.fromDynamicGrid(
                                transportSudoku.cells,
                            );
                            const roundTrippedTransportSudoku = await remoteWasmSudoku.getTransportSudoku();
                            expect(roundTrippedTransportSudoku).toStrictEqual(transportSudoku);
                        });
                        test("SerializedDynamicSudoku to worker, TransportSudoku to host", async ({
                            remoteWorkerApi,
                        }) => {
                            const transportSudoku = wasmSudoku.getTransportSudoku();
                            const serializedDynamicSudoku = wasmSudoku.serialize();
                            const remoteWasmSudoku =
                                await remoteWorkerApi.WasmSudoku.deserialize(serializedDynamicSudoku);
                            const roundTrippedTransportSudoku = await remoteWasmSudoku.getTransportSudoku();
                            expect(roundTrippedTransportSudoku.cellCount).toBe(transportSudoku.cellCount);
                        });
                        test("roundtrip SerializedDynamicSudoku", async ({ remoteWorkerApi }) => {
                            const serializedDynamicSudoku = wasmSudoku.serialize();
                            const remoteWasmSudoku =
                                await remoteWorkerApi.WasmSudoku.deserialize(serializedDynamicSudoku);
                            const roundTrippedSerializedDynamicSudoku = await remoteWasmSudoku.serialize();
                            const roundTrippedWasmSudoku = WasmSudoku.deserialize(roundTrippedSerializedDynamicSudoku);
                            expect(wasmSudoku.equals(roundTrippedWasmSudoku)).toBe(true);
                        });
                    });
                    describe("transferred", () => {
                        test("roundtrip SerializedDynamicSudoku", async ({ remoteWorkerApi }) => {
                            const serializedDynamicSudoku = wasmSudoku.serialize();
                            const remoteWasmSudoku = await remoteWorkerApi.WasmSudokuWithTransfer.deserialize(
                                Comlink.transfer(serializedDynamicSudoku, [serializedDynamicSudoku.buffer]),
                            );
                            const roundTrippedSerializedDynamicSudoku = await remoteWasmSudoku.serializeWithTransfer();
                            const roundTrippedWasmSudoku = WasmSudoku.deserialize(roundTrippedSerializedDynamicSudoku);
                            expect(wasmSudoku.equals(roundTrippedWasmSudoku)).toBe(true);
                        });
                    });
                });
            }
        });
    });
});
