// FIXME: introduce worker fixture

import * as Comlink from "comlink";
import { afterAll, beforeAll, describe, expect, test } from "vitest";
import { selectedStrategiesSchema } from "../app/constants";
import type { RemoteWorkerApi } from "../app/state/worker";
import type { WorkerApi } from "../app/state/worker/bg/worker";
import { spawnWorker } from "../app/state/worker/spawn";
import type { DynamicMultiShotGeneratorSettings, MultiShotGeneratorProgress, TransportSudoku } from "../types";

// This test exercises the multi-threaded multi-shot generator, which is the *only* code path
// that uses `wasm-bindgen-rayon`. The shared worker fixture initializes the rayon pool with a
// single thread (`init(1)`), so it does not exercise real multi-threading. Here we initialize a
// pool with more than one thread and run `parallel: true` generation, making this a valid signal
// that wasm-bindgen-rayon works end-to-end.
const THREAD_COUNT = Math.max(2, Math.min(navigator.hardwareConcurrency, 4));

// Base 2 (4x4, 16 cells) keeps generation cheap.
const BASE = 2;
const CELL_COUNT = 16;
const SEED = 42n;
const ITERATIONS = 16;

// With a fixed seed the result is deterministic and independent of thread count:
// each iteration is seeded as `seed + iteration` and the winner is an order-independent
// max-reduction over that fixed set of grids (see sudoku-rs multi_shot::generate_with_inspect
// and the `test_parallel_vs_sequential` Rust test). Pinned by observing the first green run.
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
        // The callback runs in this (main) thread; it must be proxied across the worker boundary.
        Comlink.proxy((p: MultiShotGeneratorProgress) => {
            progress.push(p);
        }),
    );

    const transportSudoku = await result.getTransportSudoku();

    const finished = progress.filter((p) => p.kind === "finished");
    // The best metric is order-independent (max over a fixed set), so derive it from all updates
    // rather than relying on the chronological order of parallel progress messages.
    // The u64 metric is declared as bigint but arrives over comlink as a JS number, so coerce.
    const bestMetric = finished.reduce<bigint>((acc, p) => {
        const value = BigInt(p.bestEvaluatedGridMetric);
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
    let worker: Worker;
    let api: RemoteWorkerApi;

    beforeAll(async () => {
        worker = spawnWorker();
        api = Comlink.wrap<WorkerApi>(worker, {});
        // Initialize the rayon thread pool with more than one thread.
        await api.init(THREAD_COUNT);
    });

    afterAll(() => {
        worker.terminate();
    });

    test.for([{ parallel: true }, { parallel: false }])(
        "generateMultiShot (parallel=$parallel)",
        async ({ parallel }) => {
            const res = await runMultiShot(api, parallel);

            expect(res.transportSudoku.cells).toHaveLength(CELL_COUNT);
            expect(res.transportSudoku.cellCount).toBe(CELL_COUNT);
            // One "started" and one "finished" update per iteration.
            expect(res.startedCount).toBe(ITERATIONS);
            expect(res.finishedCount).toBe(ITERATIONS);
            expect(res.bestMetric).toBe(EXPECTED_BEST_METRIC);
        },
    );

    test("parallel and sequential produce identical results", async () => {
        const par = await runMultiShot(api, true);
        const seq = await runMultiShot(api, false);

        expect(par.bestMetric).toBe(seq.bestMetric);
        // Mirrors the Rust `test_parallel_vs_sequential`: parallelism must not change the output.
        expect(par.transportSudoku).toStrictEqual(seq.transportSudoku);
    });
});
