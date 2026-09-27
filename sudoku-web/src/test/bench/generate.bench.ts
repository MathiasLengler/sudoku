import * as _ from "es-toolkit";
import { WasmSudoku } from "sudoku-wasm";
import { bench, describe } from "vitest";

import { GENERATE_FORM_DEFAULT_VALUES, toGeneratorSettings } from "../../app/state/forms/generate";
import { initWasm } from "../../app/state/wasm/init";

function noopProgress(): void {
    // noop
}

describe("generate", async () => {
    await initWasm();

    // Generation time varies per seed: each iteration generates the same batch.
    const seeds = _.range(8);
    const settingsBatch = seeds.map((seed) =>
        toGeneratorSettings({ ...GENERATE_FORM_DEFAULT_VALUES, useSeed: true, seed: String(seed) }),
    );

    bench(`single shot, UI defaults, ${seeds.length} seeds`, () => {
        for (const settings of settingsBatch) {
            const _wasmSudoku = WasmSudoku.generate(settings, noopProgress);
        }
    });
});
