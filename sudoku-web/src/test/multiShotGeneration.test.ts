import * as Comlink from "comlink";
import { describe, expect } from "vitest";
import { selectedStrategiesSchema } from "../app/constants";
import type { RemoteWorkerApi } from "../app/state/worker";
import type { DynamicMultiShotGeneratorSettings, MultiShotGeneratorProgress, TransportSudoku } from "../types";
import { test } from "./util/fixtures";

// The multi-shot generator is the only code path using wasm-bindgen-rayon. Overriding the
// fixture's default single-threaded pool makes this a real end-to-end multi-threading signal.
const THREAD_COUNT = Math.max(2, Math.min(navigator.hardwareConcurrency, 4));

// Base 2 (4x4, 16 cells) keeps generation cheap.
const BASE = 2;
const CELL_COUNT = 16;
const SEED = 42n;
const ITERATIONS = 16;

// Deterministic given the seed: each iteration uses `seed + iteration` and the winner is an
// order-independent max-reduction (see multi_shot::generate_with_inspect). Pinned from a green run.
const EXPECTED_BEST_METRIC = 12000000n;

function makeSettings(parallel: boolean): DynamicMultiShotGeneratorSettings {
    return {
        generatorSettings: {
            base: BASE,
            seed: SEED,
            prune: {
                target: "minimal",
                strategies: selectedStrategiesSchema.decode(["BruteForce"]),
                setAllDirectCandidates: true,
                order: "random",
                startFromNearMinimalGrid: false,
            },
        },
        iterations: ITERATIONS,
        metric: { kind: "strategyScore" },
        optimize: "maximize",
        parallel,
    };
}

type RunResult = {
    bestMetric: bigint;
    startedCount: number;
    finishedCount: number;
    transportSudoku: TransportSudoku;
};

async function runMultiShot(api: RemoteWorkerApi, parallel: boolean): Promise<RunResult> {
    const progress: MultiShotGeneratorProgress[] = [];

    const result = await api.WasmSudoku.generateMultiShot(
        makeSettings(parallel),
        // Proxied because the callback runs on the main thread, not the worker.
        Comlink.proxy((p: MultiShotGeneratorProgress) => {
            progress.push(p);
        }),
    );

    const transportSudoku = await result.getTransportSudoku();

    const finished = progress.filter((p) => p.kind === "finished");
    // Aggregate across updates (parallel message order isn't fixed); the u64 arrives as a JS number.
    const bestMetric = finished.reduce<bigint>((acc, p) => {
        const value = p.bestEvaluatedGridMetric;
        return value > acc ? value : acc;
    }, 0n);

    return {
        bestMetric,
        startedCount: progress.filter((p) => p.kind === "started").length,
        finishedCount: finished.length,
        transportSudoku,
    };
}

describe("multi-shot generation", () => {
    test.override("threadCount", THREAD_COUNT);

    test.for([{ parallel: true }, { parallel: false }])(
        "generateMultiShot (parallel=$parallel)",
        async ({ parallel }, { remoteWorkerApi }) => {
            const res = await runMultiShot(remoteWorkerApi, parallel);

            expect(res.transportSudoku.cells).toHaveLength(CELL_COUNT);
            expect(res.transportSudoku.cellCount).toBe(CELL_COUNT);
            // One "started" and one "finished" per iteration.
            expect(res.startedCount).toBe(ITERATIONS);
            expect(res.finishedCount).toBe(ITERATIONS);
            expect(res.bestMetric).toBe(EXPECTED_BEST_METRIC);
        },
    );

    test("parallel and sequential produce identical results", async ({ remoteWorkerApi }) => {
        const par = await runMultiShot(remoteWorkerApi, true);
        const seq = await runMultiShot(remoteWorkerApi, false);

        expect(par.bestMetric).toBe(seq.bestMetric);
        // Mirrors the Rust `test_parallel_vs_sequential`: parallelism must not change the output.
        expect(par.transportSudoku).toStrictEqual(seq.transportSudoku);
    });
});
