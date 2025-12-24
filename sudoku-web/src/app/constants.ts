import type { GoalOptimization, GridFormatEnum, GridMetric, StrategyEnum } from "../types";
import { z } from "zod";
import type { IsEqual } from "type-fest";
import { assert } from "../typeUtils";
import * as _ from "lodash-es";

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

assert<IsEqual<z.infer<typeof strategyEnumSchema>, StrategyEnum>>();
export const ALL_STRATEGIES = strategyEnumSchema.options;

export const STRATEGY_OPTIONS: Record<
    StrategyEnum,
    {
        label: string;
        description: string;
        link: string;
    }
> = {
    NakedSingles: {
        label: "Naked Singles",
        description: "A cell with only one remaining candidate.",
        link: "https://www.sudokuwiki.org/Getting_Started",
    },
    HiddenSingles: {
        label: "Hidden Singles",
        description: "One candidate is unique to a particular row, column and box.",
        link: "https://www.sudokuwiki.org/Getting_Started",
    },
    NakedPairs: {
        label: "Naked Pairs",
        description: "Two cells in a row, column or box contain the same two candidates exclusively.",
        link: "https://www.sudokuwiki.org/Naked_Candidates",
    },
    LockedSets: {
        label: "Locked Sets",
        description:
            "Inside a single row, column or box, are there any naked or hidden candidates of any size? (Naked/Hidden Pairs/Triples/Quads)",
        link: "https://www.sudokuwiki.org/Hidden_Candidates",
    },
    GroupIntersectionBlockToAxis: {
        label: "Pointing Pairs/Triples",
        description: "In one box the same candidate is aligned in one row or column.",
        link: "https://www.sudokuwiki.org/Intersection_Removal",
    },
    GroupIntersectionAxisToBlock: {
        label: "Box Line Reduction",
        description: "In one row or column the same candidate is aligned in one box.",
        link: "https://www.sudokuwiki.org/Intersection_Removal#LBR",
    },
    GroupIntersectionBoth: {
        label: "Intersection Removal",
        description: "A combination of pointing pairs/triples and box line reduction.",
        link: "https://www.sudokuwiki.org/Intersection_Removal",
    },
    BruteForce: {
        label: "Brute Force",
        description:
            "Solve the sudoku using brute force/trial and error. Backed by a Backtracking- or SAT-solver depending on the size of the Sudoku.",
        link: "https://t-dillon.github.io/tdoku/",
    },
};

export type SelectedStrategies = z.infer<typeof selectedStrategiesSchema>;
export const selectedStrategiesSchema = strategyEnumSchema
    .array()
    .min(1)
    .overwrite((strategies) => _.sortBy(strategies, (strategy) => strategyEnumSchema.options.indexOf(strategy)));

export const gridFormatSchema = z.enum([
    "CandidatesGridPlain",
    "CandidatesGridCompact",
    "CandidatesGridANSIStyled",
    "ValuesLine",
    "ValuesGrid",
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
    "backtrackCount",
    "gridGivensCount",
    "gridDirectCandidatesCount",
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
    },
    backtrackCount: {
        label: "Backtracking solver: step count",
    },
    gridGivensCount: {
        label: "Grid givens: count",
    },
    gridDirectCandidatesCount: {
        label: "Grid candidates: count",
    },
    gridGivensValueCountDeviation: {
        label: "Grid givens: value count deviation",
        disabled: true,
    },
};

export const goalOptimizationSchema = z.enum(["minimize", "maximize"]);
assert<IsEqual<z.infer<typeof goalOptimizationSchema>, GoalOptimization>>();
export const ALL_GOAL_OPTIMIZATIONS = goalOptimizationSchema.options;
