import assertNever from "assert-never";
import * as Comlink from "comlink";
import * as _ from "lodash-es";
import { useCallback, useState } from "react";
import type { CallbackInterface, Snapshot } from "recoil";
import { useRecoilCallback } from "recoil";
import type {
    DynamicGeneratorSettings,
    DynamicMultiShotGeneratorSettings,
    DynamicPosition,
    GeneratorProgress,
    GridFormatEnum,
    MultiShotGeneratorProgress,
    StrategyEnums,
    TransportDeductions,
} from "../../types";
import { cellAtGridPositionState } from "../state/cellIndexing";
import { getHint, hintState } from "../state/hint";
import type { CellAction } from "../state/input";
import { inputState } from "../state/input";
import { gameCounterState, sudokuSideLengthState, sudokuState } from "../state/sudoku";
import { remoteWasmSudokuState, workerState, type RemoteWasmSudoku } from "../state/worker";
import { spawnWorker } from "../state/worker/spawn";
import { useCancelableMutation } from "../useCancelableMutation";
import { measure, withMeasure } from "../utils/measure";
import { getInput } from "./inputActions";

// Snapshot accessors
async function getRemoteWasmSudoku(snapshot: Snapshot): Promise<RemoteWasmSudoku> {
    return await snapshot.getPromise(remoteWasmSudokuState);
}

// Validation
async function isFixedValueCell({
    snapshot,
    gridPosition,
}: Pick<CallbackInterface, "snapshot"> & { gridPosition: DynamicPosition }) {
    const cell = await snapshot.getPromise(cellAtGridPositionState(gridPosition));

    if (cell.kind === "value" && cell.fixed) {
        console.info("Can't modify fixed cell", cell);

        return true;
    } else {
        return false;
    }
}

async function isInvalidValue({ snapshot, value }: Pick<CallbackInterface, "snapshot"> & { value: number }) {
    const sideLength = await snapshot.getPromise(sudokuSideLengthState);

    if (!_.inRange(value, 0, sideLength + 1)) {
        console.warn(`Skip handling of value ${value} outside range [0, ${sideLength}]`);
        return true;
    } else {
        return false;
    }
}

async function isInvalidGridPosition({
    snapshot,
    gridPosition,
}: Pick<CallbackInterface, "snapshot"> & { gridPosition: DynamicPosition }) {
    const sideLength = await snapshot.getPromise(sudokuSideLengthState);

    if (!_.inRange(gridPosition.row, 0, sideLength) || !_.inRange(gridPosition.column, 0, sideLength)) {
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
export async function updateSudoku(
    { set, wasmSudokuProxy }: Pick<CallbackInterface, "set"> & { wasmSudokuProxy: RemoteWasmSudoku },
    isNewGame = false,
) {
    const newSudoku = await wasmSudokuProxy.getTransportSudoku();
    set(sudokuState, newSudoku);
    if (isNewGame) {
        set(gameCounterState, (prev) => prev + 1);
        set(hintState, undefined);
    }
}

async function applyValueAtGridPosition({
    snapshot,
    set,
    value,
    gridPosition,
}: Pick<CallbackInterface, "snapshot" | "set"> & {
    value: number;
    gridPosition: DynamicPosition;
}) {
    if (await isFixedValueCell({ snapshot, gridPosition })) {
        return;
    }
    const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
    const input = await getInput(snapshot);

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
            const cell = await snapshot.getPromise(cellAtGridPositionState(gridPosition));
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

        if (_.find(input.stickyChain?.handledGridPositions, gridPosition)) {
            console.info(
                `Skip handling of grid position ${JSON.stringify(
                    gridPosition,
                )}, since it was already processed in the active sticky chain.`,
            );
            return;
        }

        if (input.candidateMode) {
            if (cellAction === "set") {
                await wasmSudokuProxy.setCandidate(gridPosition, value);
            } else if (cellAction === "delete") {
                await wasmSudokuProxy.deleteCandidate(gridPosition, value);
            } else {
                assertNever(cellAction);
            }
        } else {
            if (cellAction === "set") {
                await wasmSudokuProxy.setValue(gridPosition, value);
            } else if (cellAction === "delete") {
                const cell = await snapshot.getPromise(cellAtGridPositionState(gridPosition));
                // Only delete cell value if it matches the handled value
                if (cell.kind === "value" && cell.value === value) {
                    await wasmSudokuProxy.delete(gridPosition);
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
            if (_.find(input.stickyChain.handledGridPositions, gridPosition)) {
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
            await wasmSudokuProxy.delete(gridPosition);
        } else {
            if (input.candidateMode) {
                await wasmSudokuProxy.toggleCandidate(gridPosition, value);
            } else {
                await wasmSudokuProxy.setOrToggleValue(gridPosition, value);
            }
        }
    }

    await updateSudoku({ set, wasmSudokuProxy });
}

// Public action hooks
export function useHandlePosition() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (gridPosition: DynamicPosition) => {
                if (await isInvalidGridPosition({ snapshot, gridPosition })) {
                    return;
                }
                const input = await getInput(snapshot);
                if (input.stickyMode) {
                    await applyValueAtGridPosition({ set, snapshot, gridPosition, value: input.selectedValue });
                } else {
                    set(inputState, (input) => ({ ...input, selectedPos: gridPosition }));
                }
            },
        [],
    );
}

export function useHandleValue() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (value: number) => {
                if (await isInvalidValue({ snapshot, value })) {
                    return;
                }

                const input = await getInput(snapshot);
                if (input.stickyMode) {
                    set(inputState, (input) => ({ ...input, selectedValue: value }));
                } else {
                    await applyValueAtGridPosition({ set, snapshot, gridPosition: input.selectedPos, value });
                }
            },
        [],
    );
}
export function useDeleteSelectedCell() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const input = await getInput(snapshot);
                if (input.stickyMode) {
                    console.warn("Deletion of cells is unavailable in sticky mode");
                } else {
                    await applyValueAtGridPosition({
                        set,
                        snapshot,
                        gridPosition: input.selectedPos,
                        value: 0,
                    });
                }
            },
        [],
    );
}

export function useSetAllDirectCandidates() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                await wasmSudokuProxy.setAllDirectCandidates();
                await updateSudoku({ set, wasmSudokuProxy });
            },
        [],
    );
}
export function useUndo() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                // Hide hint if it's visible.
                // This is somewhat of a hack:
                // the sudoku history state lives inside Rust, but not the hint.
                // As a result, hiding of the hint is not re-doable.
                const hint = await getHint(snapshot);
                if (hint) {
                    set(hintState, undefined);
                    return;
                }

                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                await wasmSudokuProxy.undo();
                await updateSudoku({ set, wasmSudokuProxy });
            },
        [],
    );
}
export function useRedo() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                await wasmSudokuProxy.redo();
                await updateSudoku({ set, wasmSudokuProxy });
            },
        [],
    );
}

const rebootWorker = withMeasure(
    { name: "rebootWorker" },
    async ({ snapshot, set }: Pick<CallbackInterface, "snapshot" | "set">) => {
        console.info("Rebooting worker");
        const currentWorker = await snapshot.getPromise(workerState);
        console.debug("Terminating current worker");
        currentWorker.terminate();
        const newWorker = await spawnWorker();
        set(workerState, newWorker);
        console.info("Worker rebooted");
    },
);

export function useGenerate() {
    const generate = useRecoilCallback(
        ({ snapshot, set }) =>
            async (
                settings: DynamicGeneratorSettings,
                abortPromise: Promise<never>,
                onProgress: (progress: GeneratorProgress) => void,
            ) => {
                return await measure({ name: "generate", detail: { settings } }, async () => {
                    const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);

                    try {
                        await Promise.race([
                            abortPromise,
                            wasmSudokuProxy.generate(settings, Comlink.proxy(onProgress)),
                        ]);
                    } catch (err) {
                        if (!(err instanceof DOMException && err.name === "AbortError")) {
                            throw err;
                        }
                        console.info("generate was aborted.");

                        await rebootWorker({ snapshot, set });

                        throw err;
                    }

                    await updateSudoku({ set, wasmSudokuProxy }, true);
                });
            },
        [],
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

export function useGenerateMultiShot() {
    const generateImpl = useRecoilCallback(
        ({ snapshot, set }) =>
            async (
                settings: DynamicMultiShotGeneratorSettings,
                abortPromise: Promise<never>,
                onProgress: (progress: MultiShotGeneratorProgress) => void,
            ) => {
                return await measure({ name: "generateMultiShot", detail: { settings } }, async () => {
                    const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);

                    try {
                        await Promise.race([
                            abortPromise,
                            wasmSudokuProxy.generateMultiShot(settings, Comlink.proxy(onProgress)),
                        ]);
                    } catch (err) {
                        if (!(err instanceof DOMException && err.name === "AbortError")) {
                            throw err;
                        }
                        console.info("generateMultiShot was aborted.");

                        await rebootWorker({ snapshot, set });

                        throw err;
                    }

                    await updateSudoku({ set, wasmSudokuProxy }, true);
                });
            },
        [],
    );

    const [trackedMultiShotGeneratorProgress, setTrackedMultiShotGeneratorProgress] =
        useState<TrackedMultiShotGeneratorProgress>();

    const { mutation, cancel: cancelGenerateMultiShot } = useCancelableMutation<
        DynamicMultiShotGeneratorSettings,
        MultiShotGeneratorProgress
    >({
        cancelableMutationFn: useCallback(
            async ({ variables: settings, abortPromise, onProgress }) => {
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
        }, []),
    });

    return {
        generateMultiShot: mutation.mutateAsync,
        trackedMultiShotGeneratorProgress,
        cancelGenerateMultiShot,
    };
}

export function useImportSudokuString() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (input: string, setAllDirectCandidates: boolean) => {
                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                await wasmSudokuProxy.import(input);
                if (setAllDirectCandidates) {
                    await wasmSudokuProxy.setAllDirectCandidates();
                }
                await updateSudoku({ set, wasmSudokuProxy }, true);
            },
        [],
    );
}
export function useExportSudokuString() {
    return useRecoilCallback(
        ({ snapshot }) =>
            async (format: GridFormatEnum) => {
                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                return await wasmSudokuProxy.export(format);
            },
        [],
    );
}

export function useTryStrategies() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (strategies: StrategyEnums) => {
                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                const res = await wasmSudokuProxy.tryStrategies(strategies);
                await updateSudoku({ set, wasmSudokuProxy });
                return res;
            },
        [],
    );
}

export function useApplyDeductions() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async (deductions: TransportDeductions) => {
                const wasmSudokuProxy = await getRemoteWasmSudoku(snapshot);
                await wasmSudokuProxy.applyDeductions(deductions);
                await updateSudoku({ set, wasmSudokuProxy });
            },
        [],
    );
}
