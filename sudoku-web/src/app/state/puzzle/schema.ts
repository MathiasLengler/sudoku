import * as z from "zod";
import { strategyEnumSchema } from "../../constants";

/**
 * Puzzle state when a puzzle is active
 */
export type PuzzleStatus = z.infer<typeof puzzleStatusSchema>;
export const puzzleStatusSchema = z.enum(["active", "solved", "failed"]);

/**
 * Stats for a single strategy's puzzles
 */
export type PuzzleStrategyStats = z.infer<typeof puzzleStrategyStatsSchema>;
export const puzzleStrategyStatsSchema = z.object({
    solved: z.int().nonnegative(),
    failed: z.int().nonnegative(),
});

/**
 * Stats for all strategies
 */
export type PuzzleStats = z.infer<typeof puzzleStatsSchema>;
export const puzzleStatsSchema = z.record(strategyEnumSchema, puzzleStrategyStatsSchema);

/**
 * Game mode for puzzle/challenge mode
 */
export type GameModePuzzle = z.infer<typeof gameModePuzzleSchema>;
export const gameModePuzzleSchema = z.object({
    mode: z.literal("puzzle"),
    /** The strategy that this puzzle is testing */
    targetStrategy: strategyEnumSchema,
    /** Current status of the puzzle */
    status: puzzleStatusSchema,
});
