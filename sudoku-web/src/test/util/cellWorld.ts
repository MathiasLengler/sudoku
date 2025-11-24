import { WasmCellWorld } from "../../../../sudoku-wasm/pkg/sudoku_wasm";
import { worldGridDimSchema } from "../../app/state/world/schema";
import type { BaseEnum } from "../../types";

export function getWasmCellWorldSamples(base: BaseEnum, size: number, seed: bigint) {
    return [
        {
            name: "empty",
            wasmCellWorld: WasmCellWorld.new(base, worldGridDimSchema.decode({ rowCount: size, columnCount: size }), 1),
        },
        {
            name: "solved",
            wasmCellWorld: (() => {
                const wasmCellWorld = WasmCellWorld.new(
                    base,
                    worldGridDimSchema.decode({ rowCount: size, columnCount: size }),
                    1,
                );
                wasmCellWorld.generateSolved(seed);
                return wasmCellWorld;
            })(),
        },
        {
            name: "pruned",
            wasmCellWorld: WasmCellWorld.generate(
                base,
                worldGridDimSchema.decode({ rowCount: size, columnCount: size }),
                1,
                seed,
            ),
        },
    ];
}
