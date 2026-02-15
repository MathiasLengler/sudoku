import assertNever from "assert-never";
import { inRange, isEqual } from "es-toolkit";
import type { Getter, Setter } from "jotai";
import { useAtomCallback } from "jotai/utils";
import { useCallback, useRef, useState } from "react";
import type {
    DynamicGeneratorSettings,
    DynamicMultiShotGeneratorSettings,
    DynamicPosition,
    GeneratorProgress,
    GridFormatEnum,
    MultiShotGeneratorProgress,
    StrategySet,
    TransportDeductions,
} from "../../types";
import { cellAtGridPositionState } from "../state/cellIndexing";
import { hintState } from "../state/hint";
import type { CellAction } from "../state/input";
import { inputState } from "../state/input";
import {
    mainThreadWasmSudokuClassState,
    wasmSudokuState,
    type MainThreadWasmSudoku,
} from "../state/mainThread/wasmSudoku";
import { gameCounterState, sudokuSideLengthState, sudokuState } from "../state/sudoku";
import { isWorkerReadyState, workerState } from "../state/worker";
import { useCancelableMutation } from "../useCancelableMutation";
import { measure, withMeasure } from "../utils/measure";

// Validation
async function isFixedValueCell({ get, gridPosition }: { get: Getter; gridPosition: DynamicPosition }) {
    const cell = await get(cellAtGridPositionState(gridPosition));

    if (cell.kind === "value" && cell.fixed) {
        console.info("Can't modify fixed cell", cell);

        return true;
    } else {
        return false;
    }
}

async function isInvalidValue({ get, value }: { get: Getter; value: number }) {
    const sideLength = await get(sudokuSideLengthState);

    if (!inRange(value, 0, sideLength + 1)) {
        console.warn(`Skip handling of value ${value} outside range [0, ${sideLength}]`);
        return true;
    } else {
        return false;
    }
}

async function isInvalidGridPosition({ get, gridPosition }: { get: Getter; gridPosition: DynamicPosition }) {
    const sideLength = await get(sudokuSideLengthState);

    if (!inRange(gridPosition.row, 0, sideLength) || !inRange(gridPosition.column, 0, sideLength)) {
        console.warn(
            `Skip handling of grid position ${JSON.stringify(
                gridPosition,
            )} with coordinate outside range [0, ${sideLength})`,
        );
        return true;
    } else {
        return false;
    }
}

// Mutation helpers
export function updateSudoku(
    { set, wasmSudoku }: { set: Setter; wasmSudoku: MainThreadWasmSudoku },
    isNewGame = false,
) {
    const newSudoku = wasmSudoku.getTransportSudoku();
    set(sudokuState, newSudoku);
    if (isNewGame) {
        set(gameCounterState, (prev) => prev + 1);
        set(hintState, undefined);
    }
}

async function applyValueAtGridPosition({
    get,
    set,
    value,
    gridPosition,
}: {
    get: Getter;
    set: Setter;
    value: number;
    gridPosition: DynamicPosition;
}) {
    if (await isFixedValueCell({ get, gridPosition })) {
        return;
    }
    const wasmSudoku = await get(wasmSudokuState);
    const input = get(inputState);

    if (input.stickyMode) {
        // Behaviour of stickyMode ("value first, cell second"):
        //  in candidateMode:
        //   only modifies existing candidates (could be configurable?)
        //   first candidate cell interaction determines if the candidate is set/deleted
        //  in valueMode:
        //   first cell interaction:
        //    candidates => set value
        //    matching value => delete value
        //    different value => set value
        //   chained cell interaction based on first action:
        //    set value => set value
        //    delete value => delete value if matching
        let cellAction: CellAction;
        if (!input.stickyChain) {
            const cell = await get(cellAtGridPositionState(gridPosition));
            if (input.candidateMode) {
                if (cell.kind === "value") {
                    return; // Wait for first candidates cell interaction.
                } else {
                    if (cell.candidates.includes(value)) {
                        cellAction = "delete";
                    } else {
                        cellAction = "set";
                    }
                }
            } else {
                if (cell.kind === "value") {
                    if (cell.value === value) {
                        cellAction = "delete";
                    } else {
                        cellAction = "set";
                    }
                } else {
                    cellAction = "set";
                }
            }

            // Initialize stickyChain
            set(inputState, (input) => {
                if (!input.stickyMode) {
                    console.warn("Expected stickyMode to be active");
                    return input;
                }
                return {
                    ...input,
                    stickyChain: {
                        cellAction,
                        handledGridPositions: [],
                    },
                };
            });
        } else {
            // Active sticky "chain"
            ({ cellAction } = input.stickyChain);
        }

        if (
            input.stickyChain?.handledGridPositions?.some((handledGridPosition) =>
                isEqual(handledGridPosition, gridPosition),
            )
        ) {
            console.info(
                `Skip handling of grid position ${JSON.stringify(
                    gridPosition,
                )}, since it was already processed in the active sticky chain.`,
            );
            return;
        }

        if (input.candidateMode) {
            if (cellAction === "set") {
                wasmSudoku.setCandidate(gridPosition, value);
            } else if (cellAction === "delete") {
                wasmSudoku.deleteCandidate(gridPosition, value);
            } else {
                assertNever(cellAction);
            }
        } else {
            if (cellAction === "set") {
                wasmSudoku.setValue(gridPosition, value);
            } else if (cellAction === "delete") {
                const cell = await get(cellAtGridPositionState(gridPosition));
                // Only delete cell value if it matches the handled value
                if (cell.kind === "value" && cell.value === value) {
                    wasmSudoku.delete(gridPosition);
                }
            } else {
                assertNever(cellAction);
            }
        }

        // Add gridPosition to handledGridPositions
        set(inputState, (input) => {
            if (!input.stickyMode) {
                console.warn("Expected stickyMode to be active");
                return input;
            }
            if (!input.stickyChain) {
                console.warn("Expected stickyChain to be defined");
                return input;
            }
            if (
                input.stickyChain.handledGridPositions.some((handledGridPosition) =>
                    isEqual(handledGridPosition, gridPosition),
                )
            ) {
                console.warn(
                    "Expected handledGridPositions to not contain gridPosition",
                    gridPosition,
                    ":",
                    input.stickyChain.handledGridPositions,
                );
                return input;
            }
            return {
                ...input,
                stickyChain: {
                    ...input.stickyChain,
                    handledGridPositions: [...input.stickyChain.handledGridPositions, gridPosition],
                },
            };
        });
    } else {
        if (value === 0) {
            wasmSudoku.delete(gridPosition);
        } else {
            if (input.candidateMode) {
                wasmSudoku.toggleCandidate(gridPosition, value);
            } else {
                wasmSudoku.setOrToggleValue(gridPosition, value);
            }
        }
    }

    updateSudoku({ set, wasmSudoku });
}

// Public action hooks
export function useHandlePosition() {
    return useAtomCallback(
        useCallback(async (get, set, gridPosition: DynamicPosition) => {
            if (await isInvalidGridPosition({ get, gridPosition })) {
                return;
            }
            const input = get(inputState);
            if (input.stickyMode) {
                await applyValueAtGridPosition({ set, get, gridPosition, value: input.selectedValue });
            } else {
                set(inputState, (input) => ({ ...input, selectedPos: gridPosition }));
            }
        }, []),
    );
}

export function useHandleValue() {
    return useAtomCallback(
        useCallback(async (get, set, value: number) => {
            if (await isInvalidValue({ get, value })) {
                return;
            }

            const input = get(inputState);
            if (input.stickyMode) {
                set(inputState, (input) => ({ ...input, selectedValue: value }));
            } else {
                await applyValueAtGridPosition({ set, get, gridPosition: input.selectedPos, value });
            }
        }, []),
    );
}
export function useDeleteSelectedCell() {
    return useAtomCallback(
        useCallback(async (get, set) => {
            const input = get(inputState);
            if (input.stickyMode) {
                console.warn("Deletion of cells is unavailable in sticky mode");
            } else {
                await applyValueAtGridPosition({
                    set,
                    get,
                    gridPosition: input.selectedPos,
                    value: 0,
                });
            }
        }, []),
    );
}

export function useSetAllDirectCandidates() {
    return useAtomCallback(
        useCallback(async (get, set) => {
            const wasmSudoku = await get(wasmSudokuState);
            wasmSudoku.setAllDirectCandidates();
            updateSudoku({ set, wasmSudoku });
        }, []),
    );
}
export function useUndo() {
    return useAtomCallback(
        useCallback(async (get, set) => {
            // Hide hint if it's visible.
            // This is somewhat of a hack:
            // the sudoku history state lives inside Rust, but not the hint.
            // As a result, hiding of the hint is not re-doable.
            const hint = get(hintState);
            if (hint) {
                set(hintState, undefined);
                return;
            }

            const wasmSudoku = await get(wasmSudokuState);
            wasmSudoku.undo();
            updateSudoku({ set, wasmSudoku });
        }, []),
    );
}
export function useRedo() {
    return useAtomCallback(
        useCallback(async (get, set) => {
            const wasmSudoku = await get(wasmSudokuState);
            wasmSudoku.redo();
            updateSudoku({ set, wasmSudoku });
        }, []),
    );
}

const rebootWorker = withMeasure({ name: "rebootWorker" }, async ({ get, set }: { get: Getter; set: Setter }) => {
    console.info("Rebooting worker");
    const currentWorker = get(workerState);
    console.debug("Terminating current worker");
    currentWorker.terminate();
    // Refresh the atom, spawning a new worker
    set(workerState);
    // Wait for the new worker to be ready
    await get(isWorkerReadyState);

    // Sync sudokuState to the new worker
    updateSudoku({ set, wasmSudoku: await get(wasmSudokuState) }, true);

    console.info("Worker rebooted");
});

export function useGenerate() {
    const generate = useAtomCallback(
        useCallback(
            async (
                get,
                set,
                settings: DynamicGeneratorSettings,
                abortPromise: Promise<never>,
                onProgress: (progress: GeneratorProgress) => void,
            ) => {
                return await measure({ name: "generate", detail: { settings } }, async () => {
                    const MainThreadWasmSudoku = await get(mainThreadWasmSudokuClassState);

                    let wasmSudoku;
                    try {
                        wasmSudoku = await Promise.race([
                            abortPromise,
                            MainThreadWasmSudoku.generate(settings, onProgress),
                        ]);
                    } catch (err) {
                        if (!(err instanceof DOMException && err.name === "AbortError")) {
                            throw err;
                        }
                        console.info("generate was aborted.");

                        await rebootWorker({ get, set });

                        throw err;
                    }

                    set(wasmSudokuState, wasmSudoku);

                    updateSudoku({ set, wasmSudoku }, true);
                });
            },
            [],
        ),
    );

    const [generateProgress, setGenerateProgress] = useState<GeneratorProgress>();

    const { mutation, cancel: cancelGenerate } = useCancelableMutation<DynamicGeneratorSettings, GeneratorProgress>({
        cancelableMutationFn: useCallback(
            async ({ variables: settings, abortPromise, onProgress }) => {
                await generate(settings, abortPromise, onProgress);
            },
            [generate],
        ),
        onProgress: useCallback((progress) => {
            console.debug("Generate progress:", progress);
            setGenerateProgress(progress);
        }, []),
        onCancel: useCallback(() => {
            console.info("Generate was canceled.");
            setGenerateProgress(undefined);
        }, []),
    });

    return { generate: mutation.mutateAsync, generateProgress, cancelGenerate };
}

export type TrackedMultiShotGeneratorProgress = {
    latestProgress: MultiShotGeneratorProgress;
    seenIterationsCount: number;
    finishedIterationsCount: number;
};

export type MultiShotGenerationResult = {
    bestEvaluatedGridMetric: bigint;
};

export function useGenerateMultiShot() {
    const generateImpl = useAtomCallback(
        useCallback(
            async (
                get,
                set,
                settings: DynamicMultiShotGeneratorSettings,
                abortPromise: Promise<never>,
                onProgress: (progress: MultiShotGeneratorProgress) => void,
            ) => {
                return await measure({ name: "generateMultiShot", detail: { settings } }, async () => {
                    const MainThreadWasmSudoku = await get(mainThreadWasmSudokuClassState);

                    let wasmSudoku;
                    try {
                        wasmSudoku = await Promise.race([
                            abortPromise,
                            MainThreadWasmSudoku.generateMultiShot(settings, onProgress),
                        ]);
                    } catch (err) {
                        if (!(err instanceof DOMException && err.name === "AbortError")) {
                            throw err;
                        }
                        console.info("generateMultiShot was aborted.");

                        await rebootWorker({ get, set });

                        throw err;
                    }

                    set(wasmSudokuState, wasmSudoku);

                    updateSudoku({ set, wasmSudoku }, true);
                });
            },
            [],
        ),
    );

    const [trackedMultiShotGeneratorProgress, setTrackedMultiShotGeneratorProgress] =
        useState<TrackedMultiShotGeneratorProgress>();

    // Ref to capture the best evaluated grid metric from the last "finished" progress event.
    // This is safe because progress events are fired during WASM execution,
    // before the mutation promise resolves, ensuring the value is set when we read it.
    const bestEvaluatedGridMetricRef = useRef<bigint | undefined>(undefined);

    const { mutation, cancel: cancelGenerateMultiShot } = useCancelableMutation<
        DynamicMultiShotGeneratorSettings,
        MultiShotGeneratorProgress
    >({
        cancelableMutationFn: useCallback(
            async ({ variables: settings, abortPromise, onProgress }) => {
                // Reset the metric ref before starting new generation
                bestEvaluatedGridMetricRef.current = undefined;
                await generateImpl(settings, abortPromise, onProgress);
            },
            [generateImpl],
        ),
        onProgress: useCallback((progress: MultiShotGeneratorProgress) => {
            console.debug("MultiShot progress:", progress);
            let isFinished;
            if (progress.kind === "started") {
                isFinished = false;
            } else if (progress.kind === "finished") {
                isFinished = true;
                // Capture the best evaluated grid metric from the finished progress
                bestEvaluatedGridMetricRef.current = progress.bestEvaluatedGridMetric;
            } else {
                assertNever(progress);
            }
            setTrackedMultiShotGeneratorProgress((prev) => {
                const prevSeenIterationsCount = prev?.seenIterationsCount ?? 0;
                const prevFinishedIterationsCount = prev?.finishedIterationsCount ?? 0;
                return {
                    latestProgress: progress,
                    seenIterationsCount: isFinished ? prevSeenIterationsCount : prevSeenIterationsCount + 1,
                    finishedIterationsCount: isFinished ? prevFinishedIterationsCount + 1 : prevFinishedIterationsCount,
                };
            });
        }, []),
        onCancel: useCallback(() => {
            console.info("MultiShot generation was canceled.");
            setTrackedMultiShotGeneratorProgress(undefined);
            bestEvaluatedGridMetricRef.current = undefined;
        }, []),
    });

    const generateMultiShot = useCallback(
        async (settings: DynamicMultiShotGeneratorSettings): Promise<MultiShotGenerationResult | undefined> => {
            await mutation.mutateAsync(settings);
            // Return the result with the final best metric.
            // This is safe because progress events are fired before the mutation resolves.
            if (bestEvaluatedGridMetricRef.current !== undefined) {
                return { bestEvaluatedGridMetric: bestEvaluatedGridMetricRef.current };
            }
            return undefined;
        },
        [mutation],
    );

    return {
        generateMultiShot,
        trackedMultiShotGeneratorProgress,
        cancelGenerateMultiShot,
    };
}

export function useImportSudokuString() {
    return useAtomCallback(
        useCallback(async (get, set, input: string, setAllDirectCandidates: boolean) => {
            const MainThreadWasmSudoku = await get(mainThreadWasmSudokuClassState);
            const wasmSudoku = await MainThreadWasmSudoku.import(input);
            set(wasmSudokuState, wasmSudoku);

            if (setAllDirectCandidates) {
                wasmSudoku.setAllDirectCandidates();
            }
            updateSudoku({ set, wasmSudoku }, true);
        }, []),
    );
}
export function useExportSudokuString() {
    return useAtomCallback(
        useCallback(async (get, _set, format: GridFormatEnum) => {
            const wasmSudoku = await get(wasmSudokuState);
            return wasmSudoku.export(format);
        }, []),
    );
}

export function useTryStrategies() {
    return useAtomCallback(
        useCallback(async (get, set, strategies: StrategySet) => {
            const wasmSudoku = await get(wasmSudokuState);
            const res = await wasmSudoku.tryStrategies(strategies);
            updateSudoku({ set, wasmSudoku });
            return res;
        }, []),
    );
}

export function useApplyDeductions() {
    return useAtomCallback(
        useCallback(async (get, set, deductions: TransportDeductions) => {
            const wasmSudoku = await get(wasmSudokuState);
            wasmSudoku.applyDeductions(deductions);
            updateSudoku({ set, wasmSudoku });
        }, []),
    );
}
