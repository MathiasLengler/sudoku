import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import { describe, expect, test } from "vitest";
import { init } from "../app/state/worker/bg/init";
import { getWasmCellWorldSamples } from "./util/cellWorld";
import { getWasmSudokuSamples } from "./util/sudoku";
import z from "zod";

describe("sudoku-wasm", async () => {
    await init(1);

    const base = 3;
    const seed = 42n;

    describe("WasmSudoku", () => {
        // TODO: test/bench different serialization methods for WasmSudoku:
        //  TransportSudoku
        //  DynamicGrid
        //  postcard DynamicSudoku
        // TODO: evaluate WASM based iteration instead of serializing full grid to JS
        //  Idea: return JS-compatible iterator object from WASM. This will be used by React to render the grid.
        //   https://rustwasm.github.io/docs/wasm-bindgen/reference/types/exported-rust-types.html
        //  Problem: react/jotai rely on structural equality for render optimizations.

        getWasmSudokuSamples(base, seed).forEach(({ name, wasmSudoku }) => {
            describe(name, () => {
                describe("serialization", () => {
                    test("DynamicSudoku => TransportSudoku => serde_wasm_bindgen => TransportSudoku", () => {
                        const transportCellSchema = z.object({
                            position: z.object({
                                row: z.int().nonnegative().max(8),
                                column: z.int().nonnegative().max(8),
                            }),
                            incorrectValue: z.boolean(),
                            kind: z.enum(["value", "candidates"]),
                        });
                        const transportSudoku = wasmSudoku.getTransportSudoku();
                        expect(transportSudoku.base).toBe(3);
                        expect(transportSudoku.cells).toHaveLength(81);
                        for (const cell of transportSudoku.cells) {
                            transportCellSchema.decode(cell, { reportInput: true });
                        }
                        console.log(transportSudoku.blocksIndexes);
                        for (const blocksIndex of transportSudoku.blocksIndexes) {
                            expect(blocksIndex).toHaveLength(9);
                            for (const blockIndex of blocksIndex) {
                                expect(blockIndex).toBeGreaterThanOrEqual(0);
                                expect(blockIndex).toBeLessThan(81);
                            }
                        }
                        expect(transportSudoku.sideLength).toBe(9);
                        expect(transportSudoku.cellCount).toBe(81);
                        expect(transportSudoku.history.canUndo).toBeTypeOf("boolean");
                        expect(transportSudoku.history.canRedo).toBeTypeOf("boolean");
                        expect(transportSudoku.isSolved).toBeTypeOf("boolean");
                        expect(transportSudoku.solution).toBeOneOf([
                            "noSolution",
                            "multipleSolutions",
                            "singleSolution",
                        ]);
                    });
                    test("DynamicSudoku => DynamicGrid => serde_wasm_bindgen => DynamicGrid", () => {
                        const dynamicGrid = wasmSudoku.toDynamicGrid();
                        expect(dynamicGrid).toHaveLength(81);
                    });
                    test("DynamicSudoku => postcard => Uint8Array", () => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        expect(serializedDynamicSudoku).instanceOf(Uint8Array);
                        expect(serializedDynamicSudoku.length).toBeGreaterThan(0);
                    });
                });
                describe("serialization roundtrip", () => {
                    test(`serde_wasm_bindgen DynamicSudoku <=> TransportSudoku`, () => {
                        const transportSudoku = wasmSudoku.getTransportSudoku();
                        const deserializedWasmSudoku = WasmSudoku.fromDynamicGrid(transportSudoku.cells);
                        expect(wasmSudoku.equals(deserializedWasmSudoku)).toBe(true);
                    });
                    test(`serde_wasm_bindgen DynamicSudoku <=> DynamicGrid`, () => {
                        const dynamicGrid = wasmSudoku.toDynamicGrid();
                        const deserializedWasmSudoku = WasmSudoku.fromDynamicGrid(dynamicGrid);
                        expect(wasmSudoku.equals(deserializedWasmSudoku)).toBe(true);
                    });
                    test(`postcard DynamicSudoku <=> Uint8Array`, () => {
                        const serializedDynamicSudoku = wasmSudoku.serialize();
                        const deserializedWasmSudoku = WasmSudoku.deserialize(serializedDynamicSudoku);
                        expect(wasmSudoku.equals(deserializedWasmSudoku)).toBe(true);
                    });
                });
            });
        });
    });

    describe("WasmCellWorld", () => {
        const size = 3;

        getWasmCellWorldSamples(base, size, seed).forEach(({ name, wasmCellWorld }) => {
            describe(name, () => {
                describe("serialization", () => {
                    test(`"Vec<DynamicCell> => serde_wasm_bindgen => DynamicCell[]`, () => {
                        const cells = wasmCellWorld.allWorldCells();
                        expect(cells.length).toBe(
                            // base 3, grid 3x3, overlap 1
                            625,
                        );
                        console.log({
                            jsonLength: JSON.stringify(cells).length,
                        });
                    });

                    test(`DynamicCellWorld => postcard => Uint8Array`, () => {
                        const serializedDynamicCellWorld = wasmCellWorld.serialize();
                        expect(serializedDynamicCellWorld).instanceOf(Uint8Array);
                        expect(serializedDynamicCellWorld.length).toBeGreaterThan(0);
                        console.log({
                            byteCount: serializedDynamicCellWorld.length,
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
                        const serializedDynamicCellWorld = wasmCellWorld.serialize();
                        const deserializedWasmCellWorld = WasmCellWorld.deserialize(serializedDynamicCellWorld);
                        expect(wasmCellWorld.equals(deserializedWasmCellWorld)).toBe(true);
                    });
                });
            });
        });
    });
});
