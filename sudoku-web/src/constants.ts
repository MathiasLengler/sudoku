import type { GoalOptimization, GridFormatEnum, GridMetric, StrategyEnum } from "./types";
import { z } from "zod";
import type { IsEqual } from "type-fest";
import { assert } from "./typeUtils";
import * as _ from "lodash-es";

export const WORKER_BOOT_UP_MESSAGE = "Worker loaded";

// TODO: add labels / links
export const strategyEnumSchema = z.enum([
    "NakedSingles",
    "HiddenSingles",
    "NakedPairs",
    "LockedSets",
    "GroupIntersectionBlockToAxis",
    "GroupIntersectionAxisToBlock",
    "GroupIntersectionBoth",
    "Backtracking",
]);

export type SelectedStrategies = z.infer<typeof selectedStrategiesSchema>;
export const selectedStrategiesSchema = strategyEnumSchema
    .array()
    .min(1)
    .transform((strategies) => _.sortBy(strategies, (strategy) => strategyEnumSchema.options.indexOf(strategy)));

assert<IsEqual<z.infer<typeof strategyEnumSchema>, StrategyEnum>>();
export const ALL_STRATEGIES = strategyEnumSchema.options;

export const gridFormatSchema = z.enum([
    "CandidatesGridPlain",
    "CandidatesGridCompact",
    "CandidatesGridANSIStyled",
    "GivensLine",
    "GivensGrid",
    "BinaryCandidatesLine",
    "BinaryFixedCandidatesLine",
]);
assert<IsEqual<z.infer<typeof gridFormatSchema>, GridFormatEnum>>();
export const ALL_GRID_FORMATS = gridFormatSchema.options;

export const gridMetricSchema = z.enum([
    "strategyTotalScore",
    "strategyExecutionCount",
    "strategyApplicationCount",
    "strategyDeductionCount",
    "strategyOptionsAverage",
    "solveGraphAverageBranchingFactor",
    "satStepCount",
    "backtrackingStepCount",
    "gridGivens",
    "gridGivensValueCountDeviation",
]);
assert<IsEqual<z.infer<typeof gridMetricSchema>, GridMetric>>();
export const ALL_GRID_METRICS = gridMetricSchema.options;

export const goalOptimizationSchema = z.enum(["minimize", "maximize"]);
assert<IsEqual<z.infer<typeof goalOptimizationSchema>, GoalOptimization>>();
export const ALL_GOAL_OPTIMIZATIONS = goalOptimizationSchema.options;
