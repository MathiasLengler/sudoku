import { isEqual, sortBy, zip } from "es-toolkit";
import type { Getter, Setter } from "jotai";
import { useAtomCallback } from "jotai/utils";
import { useCallback, useState } from "react";
import type { DynamicGeneratorSettings, GeneratorProgress, StrategyEnum, TransportDeduction } from "../../types";
import { ALL_STRATEGIES, selectedStrategiesSchema } from "../constants";
import { gameState } from "../state/gameMode";
import { expectedDeductionsState, getStrategyStats, puzzleStatsState, type PuzzleStats } from "../state/puzzle";
import type { GameModePuzzle, PuzzleStatus } from "../state/puzzle/schema";
import {
    mainThreadWasmSudokuClassState,
    wasmSudokuState,
    type MainThreadWasmSudoku,
} from "../state/mainThread/wasmSudoku";
import { isWorkerReadyState, workerState } from "../state/worker";
import { updateSudoku } from "./sudokuActions";
import { useCancelableMutation } from "../useCancelableMutation";
import { measure, withMeasure } from "../utils/measure";
import { parseBase } from "../utils/base";

/**
 * Get strategies that should be used before the target strategy.
 * These are all strategies that come before the target in the strategy order.
 */
function getPrerequisiteStrategies(targetStrategy: StrategyEnum): StrategyEnum[] {
    const targetIndex = ALL_STRATEGIES.indexOf(targetStrategy);
    if (targetIndex === -1) {
        return [];
    }
    return ALL_STRATEGIES.slice(0, targetIndex);
}

/**
 * Get all strategies up to and including the target strategy.
 */
function getStrategiesUpTo(targetStrategy: StrategyEnum): StrategyEnum[] {
    const prereqs = getPrerequisiteStrategies(targetStrategy);
    return [...prereqs, targetStrategy];
}

const rebootWorker = withMeasure({ name: "rebootWorker" }, async ({ get, set }: { get: Getter; set: Setter }) => {
    console.info("Rebooting worker");
    const currentWorker = get(workerState);
    console.debug("Terminating current worker");
    currentWorker.terminate();
    set(workerState);
    await get(isWorkerReadyState);
    updateSudoku({ set, wasmSudoku: await get(wasmSudokuState) }, true);
    console.info("Worker rebooted");
});

/**
 * Hook to start a new puzzle for a given strategy
 */
export function useStartPuzzle() {
    const startPuzzleImpl = useAtomCallback(
        useCallback(
            async (
                get,
                set,
                targetStrategy: StrategyEnum,
                abortPromise: Promise<never>,
                onProgress: (progress: GeneratorProgress) => void,
            ) => {
                return await measure({ name: "startPuzzle", detail: { targetStrategy } }, async () => {
                    const MainThreadWasmSudoku = await get(mainThreadWasmSudokuClassState);

                    // Generate a puzzle that requires the target strategy
                    const strategiesUpTo = getStrategiesUpTo(targetStrategy);
                    const prereqStrategies = getPrerequisiteStrategies(targetStrategy);

                    const generatorSettings: DynamicGeneratorSettings = {
                        base: parseBase(3), // Use standard 9x9 grid
                        prune: {
                            target: "minimal",
                            strategies: selectedStrategiesSchema.decode(strategiesUpTo),
                            setAllDirectCandidates: true,
                            order: "random",
                            startFromNearMinimalGrid: false,
                        },
                        solution: undefined,
                        seed: undefined, // Random each time
                    };

                    let wasmSudoku: MainThreadWasmSudoku;
                    try {
                        wasmSudoku = await Promise.race([
                            abortPromise,
                            MainThreadWasmSudoku.generate(generatorSettings, onProgress),
                        ]);
                    } catch (err) {
                        if (!(err instanceof DOMException && err.name === "AbortError")) {
                            throw err;
                        }
                        console.info("Puzzle generation was aborted.");
                        await rebootWorker({ get, set });
                        throw err;
                    }

                    // Now partially solve the puzzle using prerequisite strategies
                    // until the target strategy is required
                    if (prereqStrategies.length > 0) {
                        const prereqStrategySet = selectedStrategiesSchema.decode(prereqStrategies);
                        let madeProgress = true;
                        while (madeProgress) {
                            const solveStep = await wasmSudoku.tryStrategies(prereqStrategySet);
                            if (solveStep) {
                                // Apply the deductions from prerequisite strategies
                                wasmSudoku.applyDeductions(solveStep.deductions);
                                console.debug(`Applied ${solveStep.strategy} deductions during puzzle setup`);
                            } else {
                                madeProgress = false;
                            }
                        }
                    }

                    // Now get the expected deductions from the target strategy
                    const targetSolveStep = await wasmSudoku.tryStrategies(
                        selectedStrategiesSchema.decode([targetStrategy]),
                    );
                    if (!targetSolveStep) {
                        // The puzzle doesn't require the target strategy - try generating again
                        console.warn(`Generated puzzle doesn't require ${targetStrategy}, retrying...`);
                        throw new Error(`Failed to generate puzzle requiring ${targetStrategy}`);
                    }

                    const expectedDeductions = targetSolveStep.deductions.deductions;
                    console.info(`Puzzle requires ${targetStrategy} with ${expectedDeductions.length} deductions`);

                    // Store the expected deductions and update state
                    set(expectedDeductionsState, expectedDeductions);
                    set(wasmSudokuState, wasmSudoku);
                    updateSudoku({ set, wasmSudoku }, true);

                    // Set game mode to puzzle
                    set(gameState, {
                        mode: "puzzle",
                        targetStrategy,
                        status: "active",
                    } satisfies GameModePuzzle);

                    return expectedDeductions;
                });
            },
            [],
        ),
    );

    const [generateProgress, setGenerateProgress] = useState<GeneratorProgress>();

    const { mutation, cancel: cancelGenerate } = useCancelableMutation<StrategyEnum, GeneratorProgress>({
        cancelableMutationFn: useCallback(
            async ({ variables: targetStrategy, abortPromise, onProgress }) => {
                await startPuzzleImpl(targetStrategy, abortPromise, onProgress);
            },
            [startPuzzleImpl],
        ),
        onProgress: useCallback((progress) => {
            console.debug("Puzzle generation progress:", progress);
            setGenerateProgress(progress);
        }, []),
        onCancel: useCallback(() => {
            console.info("Puzzle generation was canceled.");
            setGenerateProgress(undefined);
        }, []),
    });

    return { startPuzzle: mutation.mutateAsync, generateProgress, cancelGenerate };
}

/**
 * Compare player deductions with expected deductions.
 * Returns true if they match (puzzle solved), false otherwise (puzzle failed).
 */
function compareDeductions(playerDeductions: TransportDeduction[], expectedDeductions: TransportDeduction[]): boolean {
    // For now, we do a simple comparison: check if the player's deductions
    // achieve the same result as the expected deductions
    // This compares the actions (what changes are made to cells)

    if (playerDeductions.length !== expectedDeductions.length) {
        return false;
    }

    // Sort deductions by position for comparison using es-toolkit sortBy
    const sortDeductions = (deductions: TransportDeduction[]) =>
        sortBy(deductions, [(d) => d.actions[0]?.position?.row ?? 0, (d) => d.actions[0]?.position?.column ?? 0]);

    const sortedPlayer = sortDeductions(playerDeductions);
    const sortedExpected = sortDeductions(expectedDeductions);

    // Compare each deduction's actions using functional style with zip
    return zip(sortedPlayer, sortedExpected).every(([player, expected]) => {
        const playerActions = player?.actions ?? [];
        const expectedActions = expected?.actions ?? [];
        return isEqual(playerActions, expectedActions);
    });
}

/**
 * Hook to validate player's move in puzzle mode
 */
export function useValidatePuzzleMove() {
    return useAtomCallback(
        useCallback(async (get, set, playerDeductions: TransportDeduction[]) => {
            const game = get(gameState);
            if (game.mode !== "puzzle") {
                console.warn("validatePuzzleMove called outside puzzle mode");
                return;
            }

            const expectedDeductions = get(expectedDeductionsState);
            if (!expectedDeductions) {
                console.warn("No expected deductions set for puzzle");
                return;
            }

            const isCorrect = compareDeductions(playerDeductions, expectedDeductions);
            const newStatus: PuzzleStatus = isCorrect ? "solved" : "failed";

            // Update puzzle status
            set(gameState, {
                mode: game.mode,
                targetStrategy: game.targetStrategy,
                status: newStatus,
            });

            // Update stats
            const currentStats = await get(puzzleStatsState);
            const strategyStats = getStrategyStats(currentStats, game.targetStrategy);
            const newStats: PuzzleStats = {
                ...currentStats,
                [game.targetStrategy]: {
                    solved: strategyStats.solved + (isCorrect ? 1 : 0),
                    failed: strategyStats.failed + (isCorrect ? 0 : 1),
                },
            };
            await set(puzzleStatsState, newStats);

            return isCorrect;
        }, []),
    );
}

/**
 * Hook to apply deductions with puzzle mode validation.
 * In puzzle mode, validates that the player's deductions match the expected ones.
 */
export function usePuzzleAwareApplyDeductions() {
    const validatePuzzleMove = useValidatePuzzleMove();

    return useAtomCallback(
        useCallback(
            async (get, set, playerDeductions: TransportDeduction[]) => {
                const game = get(gameState);
                const wasmSudoku = await get(wasmSudokuState);

                // Apply the deductions
                wasmSudoku.applyDeductions({ deductions: playerDeductions });
                updateSudoku({ set, wasmSudoku });

                // If in puzzle mode, validate the move
                if (game.mode === "puzzle" && game.status === "active") {
                    await validatePuzzleMove(playerDeductions);
                }
            },
            [validatePuzzleMove],
        ),
    );
}

/**
 * Hook to exit puzzle mode and return to normal sudoku mode
 */
export function useExitPuzzleMode() {
    return useAtomCallback(
        useCallback((_get, set) => {
            set(gameState, { mode: "sudoku" });
            set(expectedDeductionsState, undefined);
        }, []),
    );
}

/**
 * Hook to update puzzle status (e.g., when player makes a move)
 */
export function useUpdatePuzzleStatus() {
    return useAtomCallback(
        useCallback((get, set, status: PuzzleStatus) => {
            const game = get(gameState);
            if (game.mode !== "puzzle") {
                console.warn("updatePuzzleStatus called outside puzzle mode");
                return;
            }
            set(gameState, {
                mode: game.mode,
                targetStrategy: game.targetStrategy,
                status,
            });
        }, []),
    );
}
