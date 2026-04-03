import * as Comlink from "comlink";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import { bench, describe } from "vitest";
import { initWasm } from "../../app/state/wasm/init";
import type { WorkerApi } from "../../app/state/worker/bg/worker";
import { spawnWorker } from "../../app/state/worker/spawn";
import { getWasmCellWorldSamples } from "../util/cellWorld";
import { getWasmSudokuSamples } from "../util/sudoku";

describe("worker", async () => {
    // Init foreground WASM.
    await initWasm();

    const base = 3;
    const seed = 42n;

    const worker = spawnWorker();

    const remoteWorkerApi = Comlink.wrap<WorkerApi>(worker, {});
    await remoteWorkerApi.init(1);

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
                    bench("copied: roundtrip DynamicGrid", async () => {
                        const dynamicGrid = wasmSudoku.toDynamicGrid();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.fromDynamicGrid(dynamicGrid);
                        const _roundTrippedDynamicGrid = await remoteWasmSudoku.toDynamicGrid();
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
                        const roundTrippedSerializedDynamicSudoku = await remoteWasmSudoku.serialize();
                        const _roundTrippedWasmSudoku = WasmSudoku.deserialize(roundTrippedSerializedDynamicSudoku);
                    });
                });
            }
        });

        describe("WasmCellWorld", () => {
            [2, 4, 8, 16, 32, 64].forEach((size) => {
                describe(`size=${size}`, () => {
                    getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
                        describe(name, () => {
                            bench("copied: roundtrip DynamicCells", async () => {
                                const cells = wasmCellWorld.allWorldCells();
                                const base = wasmCellWorld.base();
                                const { overlap, gridDim } = wasmCellWorld.dimensions();
                                const remoteWasmCellWorld = await remoteWorkerApi.WasmCellWorld.with(
                                    base,
                                    gridDim,
                                    overlap,
                                    cells,
                                );
                                const roundTrippedCells = await remoteWasmCellWorld.allWorldCells();
                                const roundTrippedBase = await remoteWasmCellWorld.base();
                                const roundTrippedDimensions = await remoteWasmCellWorld.dimensions();
                                const _roundTrippedWasmCellWorld = WasmCellWorld.with(
                                    roundTrippedBase,
                                    roundTrippedDimensions.gridDim,
                                    roundTrippedDimensions.overlap,
                                    roundTrippedCells,
                                );
                            });
                            bench("copied: roundtrip SerializedDynamicCellWorld", async () => {
                                const serializedDynamicCellWorld = wasmCellWorld.serialize();
                                const remoteWasmCellWorld =
                                    await remoteWorkerApi.WasmCellWorld.deserialize(serializedDynamicCellWorld);
                                const roundTrippedSerializedDynamicCellWorld = await remoteWasmCellWorld.serialize();
                                const _roundTrippedWasmCellWorld = WasmCellWorld.deserialize(
                                    roundTrippedSerializedDynamicCellWorld,
                                );
                            });
                            bench("transferred: roundtrip SerializedDynamicCellWorld", async () => {
                                const serializedDynamicCellWorld = wasmCellWorld.serialize();
                                const remoteWasmCellWorld = await remoteWorkerApi.WasmCellWorldWithTransfer.deserialize(
                                    Comlink.transfer(serializedDynamicCellWorld, [serializedDynamicCellWorld.buffer]),
                                );
                                const roundTrippedSerializedDynamicCellWorld = await remoteWasmCellWorld.serialize();
                                const _roundTrippedWasmCellWorld = WasmCellWorld.deserialize(
                                    roundTrippedSerializedDynamicCellWorld,
                                );
                            });
                        });
                    });
                });
            });
        });
    });
});
