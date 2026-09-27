// Pins the runtime shape of values crossing the WASM boundary (serde-wasm-bindgen serializer config),
// independent of the TS type generator.

import { WasmCellWorld, WasmSudoku } from "sudoku-wasm";
import { describe, expect, expectTypeOf, test } from "vitest";

import type { DynamicGeneratorSettings, EvaluatedGridMetric } from "../types";

import { ALL_STRATEGIES, selectedStrategiesSchema } from "../app/constants";
import { initWasm } from "../app/state/wasm/init";
import { worldGridDimSchema } from "../app/state/world/schema";

function noopProgress(): void {
    // noop
}

function expectPlainObject(value: unknown): void {
    expect(value).toBeTypeOf("object");
    expect(Object.getPrototypeOf(value)).toBe(Object.prototype);
}

describe("serializer parity", async () => {
    await initWasm();

    const base = 3;
    const seed = 42n;

    describe("export", () => {
        // `#[serde(flatten)]` serializes through the map path: must still yield plain objects.
        test("flattened struct TransportCell is a plain object", () => {
            const [cell] = WasmSudoku.generate({ base, seed }, noopProgress).getTransportSudoku().cells;
            expectPlainObject(cell);
            expect(cell).toHaveProperty("kind");
            expect(cell).toHaveProperty("position");
        });

        test("flattened structs PositionedTransportReason/Action are plain objects", () => {
            const wasmSudoku = WasmSudoku.generate(
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
                noopProgress,
            );
            const solveStep = wasmSudoku.tryStrategies(selectedStrategiesSchema.decode(ALL_STRATEGIES));
            if (!solveStep) {
                throw new Error("expected a solve step for a pruned sudoku");
            }
            expectPlainObject(solveStep);
            const [deduction] = solveStep.deductions.deductions;
            if (!deduction) {
                throw new Error("expected at least one deduction");
            }
            expect(deduction.actions.length).toBeGreaterThan(0);
            for (const positioned of [...deduction.actions, ...deduction.reasons]) {
                expectPlainObject(positioned);
                expect(positioned).toHaveProperty("position");
            }
        });

        test("Option::None return is undefined", () => {
            const solved = WasmSudoku.generate({ base, seed }, noopProgress);
            expect(solved.tryStrategies(selectedStrategiesSchema.decode(ALL_STRATEGIES))).toBeUndefined();
        });

        test("tagged enum DynamicCell has kind discriminator", () => {
            const [cell] = WasmSudoku.generate({ base, seed }, noopProgress).toDynamicGrid();
            expectPlainObject(cell);
            expect(cell?.kind).toBeOneOf(["value", "candidates"]);
        });

        test("nested struct fields are plain objects", () => {
            const gridDim = worldGridDimSchema.decode({ rowCount: 2, columnCount: 2 });
            const dimensions = WasmCellWorld.new(base, gridDim, 1).dimensions();
            expectPlainObject(dimensions);
            expectPlainObject(dimensions.gridDim);
            expect(dimensions.gridDim).toEqual({ rowCount: 2, columnCount: 2 });
        });

        // u64 => bigint: runtime pinned by multiShotGeneration.test.ts (needs the worker's rayon pool).
        test("u64 alias is typed bigint", () => {
            expectTypeOf<EvaluatedGridMetric>().toEqualTypeOf<bigint>();
        });
    });

    describe("import", () => {
        test("optional fields accept omitted and undefined", () => {
            for (const settings of [
                { base },
                { base, seed: undefined, prune: undefined },
            ] satisfies DynamicGeneratorSettings[]) {
                expect(() => WasmSudoku.generate(settings, noopProgress)).not.toThrow();
            }
        });

        test("optional fields also accept null", () => {
            // @ts-expect-error pins the deserializer's runtime leniency, which the types forbid
            expect(() => WasmSudoku.generate({ base, seed: null, prune: null }, noopProgress)).not.toThrow();
        });

        test("u64 accepts bigint", () => {
            const a = WasmSudoku.generate({ base, seed }, noopProgress);
            const b = WasmSudoku.generate({ base, seed }, noopProgress);
            expect(a.equals(b)).toBe(true);
        });

        test("u64 also accepts number", () => {
            // @ts-expect-error pins the deserializer's runtime leniency, which the types forbid
            const a = WasmSudoku.generate({ base, seed: 42 }, noopProgress);
            const b = WasmSudoku.generate({ base, seed }, noopProgress);
            expect(a.equals(b)).toBe(true);
        });
    });
});
