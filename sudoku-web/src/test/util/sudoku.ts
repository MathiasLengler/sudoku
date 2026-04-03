import { selectedStrategiesSchema } from "../../app/constants";
import { type BaseEnum, type GeneratorProgress } from "../../types";
import { WasmSudoku } from "sudoku-wasm";

function noopGenerateProgress(_progress: GeneratorProgress): void {
    // noop
}

export function getWasmSudokuSamples(base: BaseEnum, seed: bigint): { name: string; wasmSudoku: WasmSudoku }[] {
    return [
        {
            name: "empty",
            wasmSudoku: WasmSudoku.new(base),
        },
        {
            name: "solved",
            wasmSudoku: WasmSudoku.generate(
                {
                    base,
                    seed,
                },
                noopGenerateProgress,
            ),
        },
        {
            name: "minimal",
            wasmSudoku: WasmSudoku.generate(
                {
                    base,
                    seed,
                    prune: {
                        target: "minimal",
                        strategies: selectedStrategiesSchema.decode(["BruteForce"]),
                        setAllDirectCandidates: true,
                        order: "random",
                        startFromNearMinimalGrid: false,
                    },
                },
                noopGenerateProgress,
            ),
        },
    ];
}
