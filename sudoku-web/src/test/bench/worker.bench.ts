import * as Comlink from "comlink";
import { WasmSudoku } from "sudoku-wasm";
import { bench, describe } from "vitest";
import { init } from "../../app/state/worker/bg/init";
import { getWasmSudokuSamples } from "../util/sudoku";
import { spawnWorker } from "../../app/state/worker/spawn";
import type { WorkerApi } from "../../app/state/worker/bg/worker";

describe("worker", async () => {
    // Init foreground WASM.
    await init(1);

    const base = 3;
    const seed = 42n;

    const worker = await spawnWorker();

    const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
    await remoteWorkerApi.init();

    describe("communication", () => {
        describe("WasmSudoku", () => {
            for (const { name, wasmSudoku } of getWasmSudokuSamples(base, seed)) {
                describe(name, () => {
                    bench("copied: roundtrip TransportSudoku", async () => {
                        const transportSudoku = wasmSudoku.getTransportSudoku();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.fromDynamicGrid(
                            transportSudoku.cells,
                        );
                        const _roundTrippedTransportSudoku = await remoteWasmSudoku.getTransportSudoku();
                    });
                    bench("copied: SerializedDynamicSudoku to worker, TransportSudoku to host", async () => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.deserialize(serializedDynamicSudoku);
                        const _roundTrippedTransportSudoku = await remoteWasmSudoku.getTransportSudoku();
                    });
                    bench("copied: roundtrip SerializedDynamicSudoku", async () => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.deserialize(serializedDynamicSudoku);
                        const roundTrippedSerializedDynamicSudoku = await remoteWasmSudoku.serialize();
                        const _roundTrippedWasmSudoku = WasmSudoku.deserialize(roundTrippedSerializedDynamicSudoku);
                    });
                    bench("transferred: roundtrip SerializedDynamicSudoku", async () => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudokuWithTransfer.deserialize(
                            Comlink.transfer(serializedDynamicSudoku, [serializedDynamicSudoku.buffer]),
                        );
                        const roundTrippedSerializedDynamicSudoku = await remoteWasmSudoku.serializeWithTransfer();
                        const _roundTrippedWasmSudoku = WasmSudoku.deserialize(roundTrippedSerializedDynamicSudoku);
                    });
                });
            }
        });
    });
});
