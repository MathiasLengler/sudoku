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
            const wasmCellWorld = WasmCellWorld.new(
                3,
                worldGridDimSchema.decode({ rowCount: size, columnCount: size }),
                1,
            );
            bench(`getTransportSudoku: size=${size}`, () => {
                wasmCellWorld.allWorldCells();
            });
        });
    });
});
