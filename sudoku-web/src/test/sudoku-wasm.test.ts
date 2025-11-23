import { describe, expect, test } from "vitest";
import { WasmCellWorld, WasmSudoku } from "../../../sudoku-wasm/pkg/sudoku_wasm";
import { init } from "../app/state/worker/bg/init";
import { worldGridDimSchema } from "../app/state/world/schema";

describe("sudoku-wasm", async () => {
    await init(1);
    describe("WasmSudoku", () => {
        test("default base", () => {
            const wasmSudoku = WasmSudoku.new();
            expect(wasmSudoku).toBeInstanceOf(WasmSudoku);
            const transportSudoku = wasmSudoku.getTransportSudoku();
            expect(transportSudoku.base).toBe(3);
        });
    });

    describe("WasmCellWorld", () => {
        const size = 3;
        // TODO: more representative cell world sample
        //  the default cell world only contains empty cells
        const wasmCellWorld = WasmCellWorld.new(3, worldGridDimSchema.decode({ rowCount: size, columnCount: size }), 1);

        describe("deserialization", () => {
            test(`serde_wasm_bindgen Vec<DynamicCell> => DynamicCell[]`, () => {
                const cells = wasmCellWorld.allWorldCells();
                expect(cells.length).toBe(625);
                expect(JSON.stringify(cells).length).toBe(23751);
            });

            // TODO: evaluate why postcard bytes contain field name strings
            //  the format shouldn't be self-describing:
            //  https://postcard.jamesmunns.com/wire-format#non-self-describing-format
            test(`postcard Vec<DynamicCell> => Vec<u8> => Uint8Array`, () => {
                const bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();
                expect(bytes.length).toBe(7502);
            });
            test(`postcard Vec<DynamicCell> => Box<[u8]> => Uint8Array`, () => {
                const bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                expect(bytes.length).toBe(7502);
            });
            test(`postcard Vec<DynamicCell> => Vec<u8> == Box<[u8]>`, () => {
                const vec_bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();
                const boxed_slice_bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                expect(vec_bytes).toEqual(boxed_slice_bytes);
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
            // TODO: test other serialization roundtrips
        });
    });
});
