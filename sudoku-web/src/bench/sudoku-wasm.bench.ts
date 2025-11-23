import { bench, describe } from "vitest";
import { WasmCellWorld, WasmSudoku } from "../../../sudoku-wasm/pkg/sudoku_wasm";
import { init } from "../app/state/worker/bg/init";
import { worldGridDimSchema } from "../app/state/world/schema";

describe("sudoku-wasm", async () => {
    await init(1);
    describe("WasmSudoku", () => {
        const wasmSudoku = WasmSudoku.new();
        describe("getTransportSudoku", () => {
            wasmSudoku.getTransportSudoku();
        });
    });
    describe("WasmCellWorld", () => {
        [2, 4, 8, 16, 32].forEach((size) => {
            // TODO: more representative cell world sample
            //  the default cell world only contains empty cells
            const wasmCellWorld = WasmCellWorld.new(
                3,
                worldGridDimSchema.decode({ rowCount: size, columnCount: size }),
                1,
            );
            // TODO: update test names based on `sudoku-wasm.test.ts`
            describe(`allWorldCells size=${size}`, () => {
                bench(`serde_wasm_bindgen`, () => {
                    const _cells = wasmCellWorld.allWorldCells();
                });
                bench(`postcard Vec<u8>`, () => {
                    const _bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellVec();
                });
                bench(`postcard Box<[u8]>`, () => {
                    const _bytes = wasmCellWorld.allWorldCellsPostcardDynamicCellBoxedSlice();
                });
            });
        });
    });
});
