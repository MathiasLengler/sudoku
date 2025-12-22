import { describe, expect, test } from "vitest";
import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import { init } from "../app/state/worker/bg/init";
import { getWasmCellWorldSamples } from "./util/cellWorld";

describe("sudoku-wasm", async () => {
    await init(1);

    describe("WasmSudoku", () => {
        // TODO: test/bench different serialization methods for WasmSudoku:
        //  TransportSudoku
        //  DynamicGrid
        //  postcard DynamicSudoku
        // TODO: evaluate WASM based iteration instead of serializing full grid to JS
        //  Idea: return JS-compatible iterator object from WASM. This will be used by React to render the grid.
        //   https://rustwasm.github.io/docs/wasm-bindgen/reference/types/exported-rust-types.html
        //  Problem: react/jotai rely on structural equality for render optimizations.
        test("default base", () => {
            const wasmSudoku = WasmSudoku.new(3);
            expect(wasmSudoku).toBeInstanceOf(WasmSudoku);
            const transportSudoku = wasmSudoku.getTransportSudoku();
            expect(transportSudoku.base).toBe(3);
        });
    });

    describe("WasmCellWorld", () => {
        const base = 3;
        const size = 3;
        const seed = 42n;

        getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
            describe(name, () => {
                describe("serialization", () => {
                    describe("Vec<DynamicCell> =>", () => {
                        test(`serde_wasm_bindgen => DynamicCell[]`, () => {
                            const cells = wasmCellWorld.allWorldCells();
                            console.log({
                                cellCount: cells.length,
                                jsonLength: JSON.stringify(cells).length,
                            });
                        });
                        test(`postcard => Vec<u8> => Uint8Array`, () => {
                            const bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();

                            console.log({
                                byteCount: bytes.length,
                            });
                        });
                        test(`postcard => Box<[u8]> => Uint8Array`, () => {
                            const bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                            console.log({
                                byteCount: bytes.length,
                            });
                        });
                        test(`postcard => Vec<u8> == Box<[u8]>`, () => {
                            const vec_bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();
                            const boxed_slice_bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                            expect(vec_bytes).toEqual(boxed_slice_bytes);
                        });
                    });

                    describe("DynamicCellWorld =>", () => {
                        test(`postcard => Vec<u8> => Uint8Array`, () => {
                            const bytes = wasmCellWorld.serialize();
                            console.log({
                                byteCount: bytes.length,
                            });
                        });
                    });
                });

                describe("serialization roundtrip", () => {
                    test(`serde_wasm_bindgen Vec<DynamicCell> <=> DynamicCell[] `, () => {
                        const cells = wasmCellWorld.allWorldCells();
                        const base = wasmCellWorld.base();
                        const { overlap, gridDim } = wasmCellWorld.dimensions();
                        const deserializedWasmCellWorld = WasmCellWorld.with(base, gridDim, overlap, cells);
                        expect(wasmCellWorld.equals(deserializedWasmCellWorld)).toBe(true);
                    });
                    test(`postcard DynamicCellWorld <=> Uint8Array`, () => {
                        const bytes = wasmCellWorld.serialize();
                        const deserializedWasmCellWorld = wasmCellWorld.deserialize(bytes);
                        expect(wasmCellWorld.equals(deserializedWasmCellWorld)).toBe(true);
                    });
                });
            });
        });
    });
});
