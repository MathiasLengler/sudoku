import { atom } from "jotai";
import { atomWithStorage } from "jotai/utils";
import type { StrategyEnum, TransportDeduction } from "../../../types";
import { strategyEnumSchema } from "../../constants";
import { getZodLocalStorage } from "../localStorageEffect";
import { gameState, type Game } from "../gameMode";
import {
    puzzleStatsSchema,
    puzzleStrategyStatsSchema,
    type GameModePuzzle,
    type PuzzleStats,
    type PuzzleStrategyStats,
    type PuzzleStatus,
} from "./schema";

/**
 * The expected deductions that the player should make to solve the puzzle.
 * This is set when a puzzle is generated and used to validate player moves.
 */
export const expectedDeductionsState = atom<TransportDeduction[] | undefined>(undefined);

/**
 * Persisted puzzle stats across all strategies
 */
export const puzzleStatsState = atomWithStorage<PuzzleStats>(
    "puzzleStats",
    {},
    getZodLocalStorage(puzzleStatsSchema),
);

/**
 * Helper to get stats for a specific strategy
 */
export function getStrategyStats(stats: PuzzleStats, strategy: StrategyEnum): PuzzleStrategyStats {
    return stats[strategy] ?? puzzleStrategyStatsSchema.parse({ solved: 0, failed: 0 });
}

/**
 * Check if current game mode is puzzle mode
 */
export const isPuzzleModeState = atom<boolean>((get) => {
    const game = get(gameState);
    return game.mode === "puzzle";
});

/**
 * Assert that the current game mode is puzzle mode and return it
 */
export function assertGameModePuzzle(gameMode: Game): GameModePuzzle {
    if (gameMode.mode !== "puzzle") {
        throw new Error(`Expected game mode 'puzzle', instead got: ${gameMode.mode}`);
    }
    return gameMode;
}

/**
 * Get the target strategy for the current puzzle (if in puzzle mode)
 */
export const puzzleTargetStrategyState = atom<StrategyEnum | undefined>((get) => {
    const game = get(gameState);
    if (game.mode === "puzzle") {
        return game.targetStrategy;
    }
    return undefined;
});

/**
 * Get the current puzzle status (if in puzzle mode)
 */
export const puzzleStatusState = atom<PuzzleStatus | undefined>((get) => {
    const game = get(gameState);
    if (game.mode === "puzzle") {
        return game.status;
    }
    return undefined;
});

/**
 * Strategies available for puzzle mode (excluding BruteForce which is not a real strategy)
 */
export const PUZZLE_STRATEGIES = strategyEnumSchema.options.filter((s) => s !== "BruteForce");
