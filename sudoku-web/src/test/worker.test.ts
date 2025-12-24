import * as Comlink from "comlink";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import { describe, expect } from "vitest";
import { init } from "../app/state/worker/bg/init";
import { getWasmCellWorldSamples } from "./util/cellWorld";
import { getWasmSudokuSamples } from "./util/sudoku";
import { test } from "./util/fixtures";

describe("worker communication", async () => {
    // Init foreground WASM.
    await init(1);

    const base = 3;
    const seed = 42n;

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
                    test("roundtrip DynamicGrid", async ({ remoteWorkerApi }) => {
                        const dynamicGrid = wasmSudoku.toDynamicGrid();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.fromDynamicGrid(dynamicGrid);
                        const roundTrippedDynamicGrid = await remoteWasmSudoku.toDynamicGrid();
                        expect(roundTrippedDynamicGrid).toStrictEqual(dynamicGrid);
                    });
                    test("roundtrip SerializedDynamicSudoku", async ({ remoteWorkerApi }) => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        const remoteWasmSudoku = await remoteWorkerApi.WasmSudoku.deserialize(serializedDynamicSudoku);
                        // Copied
                        expect(serializedDynamicSudoku.length).toBeGreaterThan(0);
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
                        // Transferred
                        expect(serializedDynamicSudoku.length).toBe(0);
                        const roundTrippedSerializedDynamicSudoku = await remoteWasmSudoku.serializeWithTransfer();
                        const roundTrippedWasmSudoku = WasmSudoku.deserialize(roundTrippedSerializedDynamicSudoku);
                        expect(wasmSudoku.equals(roundTrippedWasmSudoku)).toBe(true);
                    });
                });
            });
        }
    });

    describe("WasmCellWorld", () => {
        const size = 3;

        getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
            describe(name, () => {
                describe("copied", () => {
                    test("roundtrip DynamicCells", async ({ remoteWorkerApi }) => {
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
                        const roundTrippedWasmCellWorld = WasmCellWorld.with(
                            roundTrippedBase,
                            roundTrippedDimensions.gridDim,
                            roundTrippedDimensions.overlap,
                            roundTrippedCells,
                        );
                        expect(wasmCellWorld.equals(roundTrippedWasmCellWorld)).toBe(true);
                    });
                    test("roundtrip SerializedDynamicCellWorld", async ({ remoteWorkerApi }) => {
                        const serializedDynamicCellWorld = wasmCellWorld.serialize();
                        const remoteWasmCellWorld =
                            await remoteWorkerApi.WasmCellWorld.deserialize(serializedDynamicCellWorld);
                        // Copied
                        expect(serializedDynamicCellWorld.length).toBeGreaterThan(0);
                        const roundTrippedSerializedDynamicCellWorld = await remoteWasmCellWorld.serialize();
                        const roundTrippedWasmCellWorld = WasmCellWorld.deserialize(
                            roundTrippedSerializedDynamicCellWorld,
                        );
                        expect(wasmCellWorld.equals(roundTrippedWasmCellWorld)).toBe(true);
                    });
                });
                describe("transferred", () => {
                    test("roundtrip SerializedDynamicCellWorld", async ({ remoteWorkerApi }) => {
                        const serializedDynamicCellWorld = wasmCellWorld.serialize();
                        const remoteWasmCellWorld = await remoteWorkerApi.WasmCellWorldWithTransfer.deserialize(
                            Comlink.transfer(serializedDynamicCellWorld, [serializedDynamicCellWorld.buffer]),
                        );
                        // Transferred
                        expect(serializedDynamicCellWorld.length).toBe(0);
                        const roundTrippedSerializedDynamicCellWorld =
                            await remoteWasmCellWorld.serializeWithTransfer();
                        const roundTrippedWasmCellWorld = WasmCellWorld.deserialize(
                            roundTrippedSerializedDynamicCellWorld,
                        );
                        expect(wasmCellWorld.equals(roundTrippedWasmCellWorld)).toBe(true);
                    });
                });
            });
        });
    });
});
