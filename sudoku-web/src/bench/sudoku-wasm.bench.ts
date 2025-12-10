import { bench, describe } from "vitest";
import { WasmSudoku } from "sudoku-wasm";
import { init } from "../app/state/worker/bg/init";
import { getWasmCellWorldSamples } from "../test/util/cellWorld";

describe("sudoku-wasm", async () => {
    await init(1);
    describe("WasmSudoku", () => {
        const wasmSudoku = WasmSudoku.new();
        bench("getTransportSudoku", () => {
            const _transportSudoku = wasmSudoku.getTransportSudoku();
        });
    });
    describe("WasmCellWorld", () => {
        [2, 4, 8, 16, 32].forEach((size) => {
            const base = 3;
            const seed = 42n;

            describe(`size=${size}`, () => {
                getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
                    describe(name, () => {
                        describe(`serialization`, () => {
                            describe("Vec<DynamicCell> =>", () => {
                                bench(`serde_wasm_bindgen => DynamicCell[]`, () => {
                                    const _cells = wasmCellWorld.allWorldCells();
                                });
                                bench(`postcard => Vec<u8> => Uint8Array`, () => {
                                    const _bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();
                                });
                                bench(`postcard => Box<[u8]> => Uint8Array`, () => {
                                    const _bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                                });
                            });
                            describe("DynamicCellWorld =>", () => {
                                bench(`postcard => Vec<u8> => Uint8Array`, () => {
                                    const _bytes = wasmCellWorld.serialize();
                                });
                            });
                        });
                    });
                });
            });
        });
    });
});
