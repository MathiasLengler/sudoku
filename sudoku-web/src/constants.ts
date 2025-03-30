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
    "BruteForce",
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
    "strategyScore",
    "strategyApplicationCount",
    "strategyDeductionCount",
    "strategyAverageOptions",
    "solveGraphAverageBranchingFactor",
    "satStepCount",
    "backtrackingStepCount",
    "gridGivensCount",
    "gridGivensValueCountDeviation",
]);
assert<IsEqual<z.infer<typeof gridMetricSchema>, GridMetric>>();
export const ALL_GRID_METRICS = gridMetricSchema.options;

// TODO: remove disabled when implemented
export const GRID_METRIC_OPTIONS: Record<
    GridMetric,
    {
        label: string;
        description?: string;
        disabled?: boolean;
    }
> = {
    strategyScore: {
        label: "Strategy: score",
        description:
            "Weighted sum of all strategy scores used to solve the grid. Equals: (strategy score) * (number of deductions made by the strategy).",
    },
    strategyApplicationCount: {
        label: "Strategy: application count",
        description: "The number of times a strategy was applied to the grid.",
    },
    strategyDeductionCount: {
        label: "Strategy: deduction count",
        description: "Number of deductions used to solve the grid.",
    },
    strategyAverageOptions: {
        label: "Strategy: average options",
        description: "The average number of strategies available to make progress.",
    },
    solveGraphAverageBranchingFactor: {
        label: "Solve graph average branching factor",
        disabled: true,
    },
    satStepCount: {
        label: "SAT solver: step count",
        disabled: true,
    },
    backtrackingStepCount: {
        label: "Backtracking solver: step count",
        disabled: true,
    },
    gridGivensCount: {
        label: "Grid givens: count",
        disabled: true,
    },
    gridGivensValueCountDeviation: {
        label: "Grid givens: value count deviation",
        disabled: true,
    },
};

export const goalOptimizationSchema = z.enum(["minimize", "maximize"]);
assert<IsEqual<z.infer<typeof goalOptimizationSchema>, GoalOptimization>>();
export const ALL_GOAL_OPTIMIZATIONS = goalOptimizationSchema.options;
