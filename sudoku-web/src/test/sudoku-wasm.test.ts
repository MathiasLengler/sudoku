import { describe, expect, test } from "vitest";
import { WasmSudoku } from "../../../sudoku-wasm/pkg/sudoku_wasm";
import { init } from "../app/state/worker/bg/init";

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
});
