import { bench, describe } from "vitest";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import { init } from "../app/state/worker/bg/init";
import { getWasmCellWorldSamples } from "../test/util/cellWorld";
import { getWasmSudokuSamples } from "../test/util/sudoku";

describe("sudoku-wasm", async () => {
    await init(1);

    const base = 3;
    const seed = 42n;

    describe.only("WasmSudoku", () => {
        const wasmSudoku = WasmSudoku.new(3);
        bench("getTransportSudoku", () => {
            const _transportSudoku = wasmSudoku.getTransportSudoku();
        });

        getWasmSudokuSamples(base, seed).forEach(({ name, wasmSudoku }) => {
            describe(name, () => {
                describe("serialization", () => {
                    bench("DynamicSudoku => TransportSudoku => serde_wasm_bindgen => TransportSudoku", () => {
                        const _transportSudoku = wasmSudoku.getTransportSudoku();
                    });
                    bench("DynamicSudoku => DynamicGrid => serde_wasm_bindgen => DynamicGrid", () => {
                        const _dynamicGrid = wasmSudoku.toDynamicGrid();
                    });
                    bench("DynamicSudoku => postcard => Uint8Array", () => {
                        const _serializedDynamicSudoku = wasmSudoku.serialize();
                    });
                });
                describe("serialization roundtrip", () => {
                    bench(`serde_wasm_bindgen DynamicSudoku <=> TransportSudoku`, () => {
                        const transportSudoku = wasmSudoku.getTransportSudoku();
                        const _deserializedWasmSudoku = WasmSudoku.fromDynamicGrid(transportSudoku.cells);
                    });
                    bench(`serde_wasm_bindgen DynamicSudoku <=> DynamicGrid`, () => {
                        const dynamicGrid = wasmSudoku.toDynamicGrid();
                        const _deserializedWasmSudoku = WasmSudoku.fromDynamicGrid(dynamicGrid);
                    });
                    bench(`postcard DynamicSudoku <=> Uint8Array`, () => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        const _deserializedWasmSudoku = WasmSudoku.deserialize(serializedDynamicSudoku);
                    });
                });
            });
        });
    });
    describe("WasmCellWorld", () => {
        [2, 4, 8, 16, 32].forEach((size) => {
            describe(`size=${size}`, () => {
                getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
                    describe(name, () => {
                        describe(`serialization`, () => {
                            bench(`"Vec<DynamicCell> => serde_wasm_bindgen => DynamicCell[]`, () => {
                                const _cells = wasmCellWorld.allWorldCells();
                            });
                            bench(`DynamicCellWorld => postcard => Uint8Array`, () => {
                                const _bytes = wasmCellWorld.serialize();
                            });
                        });
                        describe("serialization roundtrip", () => {
                            bench(`serde_wasm_bindgen Vec<DynamicCell> <=> DynamicCell[] `, () => {
                                const cells = wasmCellWorld.allWorldCells();
                                const base = wasmCellWorld.base();
                                const { overlap, gridDim } = wasmCellWorld.dimensions();
                                const _deserializedWasmCellWorld = WasmCellWorld.with(base, gridDim, overlap, cells);
                            });
                            bench(`postcard DynamicCellWorld <=> Uint8Array`, () => {
                                const serializedDynamicCellWorld = wasmCellWorld.serialize();
                                const _deserializedWasmCellWorld =
                                    WasmCellWorld.deserialize(serializedDynamicCellWorld);
                            });
                        });
                    });
                });
            });
        });
    });
});
