import { describe, expect, test } from "vitest";
import { WasmCellWorld, WasmSudoku } from "../../../sudoku-wasm/pkg/sudoku_wasm";
import { init } from "../app/state/worker/bg/init";
import { getWasmCellWorldSamples } from "./util/cellWorld";

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
        const base = 3;
        const size = 3;
        const seed = 42n;

        getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
            describe(name, () => {
                describe("deserialization", () => {
                    test(`serde_wasm_bindgen Vec<DynamicCell> => DynamicCell[]`, () => {
                        const cells = wasmCellWorld.allWorldCells();
                        console.log({
                            cellCount: cells.length,
                            jsonLength: JSON.stringify(cells).length,
                        });
                    });

                    // FIXME: postcard data larger than expected.
                    // => Contains string tag names.
                    // DynamicCell has:
                    // `#[serde(rename_all = "camelCase", tag = "kind")]`
                    // Those are the tag names for `DynamicCell::Value` and `DynamicCell::Candidates`
                    // Removing `tag = "kind"` reduces size significantly:
                    // empty  7502 => 1252 bytes
                    // solved 5002 => 1877 bytes
                    // pruned 7541 => 2571 bytes
                    // Without "kind", this uses the varint(u32) representation.
                    // https://postcard.jamesmunns.com/wire-format#tagged-unions
                    //
                    // Even a bit smaller in JSON:
                    // empty  23751 => 20626
                    // solved 25626 => 22501
                    // pruned 26020 => 22895
                    // Without "kind", this uses "Externally tagged" representation.
                    // https://serde.rs/enum-representations.html#externally-tagged
                    // This saves the `"kind"` field for every cell, approx 625x4=2500 bytes saved; but with more nesting.
                    test(`postcard Vec<DynamicCell> => Vec<u8> => Uint8Array`, () => {
                        const bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();

                        console.log({
                            byteCount: bytes.length,
                        });
                    });
                    // TODO: decide on Box<[u8]> vs Vec<u8>
                    //  No perf difference in benchmark.
                    test(`postcard Vec<DynamicCell> => Box<[u8]> => Uint8Array`, () => {
                        const bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                        console.log({
                            byteCount: bytes.length,
                        });
                    });
                    test(`postcard Vec<DynamicCell> => Vec<u8> == Box<[u8]>`, () => {
                        const vec_bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();
                        const boxed_slice_bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                        expect(vec_bytes).toEqual(boxed_slice_bytes);
                    });
                    // TODO: add `postcard DynamicCellWorld => Box<[u8]> => Uint8Array`
                    //  alternative: `CellWorld<Base3>` as PoC
                    //   maybe DynamicCellWorld is easier, since the enum variant defines the base?
                    //  motivation: less conversions, more compact native representation
                    //  the postcard bytes are opaque to TS anyway, so we can choose any representation.
                    //  use-case is transfer between main thread and worker.
                    //  reuses bit packing in `compact::Cell` and `compact::Candidates`.
                    //  challenges:
                    //   `SudokuBase` generic interop with serde.
                    //   the compact::* types assume safety invariants, which could get broken by serde => UB.
                    //    custom derives necessary for validation?
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
    });
});
