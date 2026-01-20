import { sortBy } from "es-toolkit/compat";
import type { IsEqual } from "type-fest";
import * as z from "zod";
import type { GoalOptimization, GridMetricName, StrategyEnum } from "../types";
import { assert } from "../typeUtils";

export const strategyEnumSchema = z.enum([
    "NakedSingles",
    "HiddenSingles",
    "NakedPairs",
    "LockedSets",
    "GroupIntersectionBlockToAxis",
    "GroupIntersectionAxisToBlock",
    "GroupIntersectionBoth",
    "XWing",
    "BruteForce",
]);

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
    XWing: {
        label: "X-Wing",
        description: "A candidate appears in exactly two cells in two different rows and columns, forming a rectangle.",
        link: "https://www.sudokuwiki.org/X_Wing_Strategy",
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
    .overwrite((strategies) => sortBy(strategies, (strategy) => strategyEnumSchema.options.indexOf(strategy)));

export const gridFormatSchema = z.enum([
    "BinaryCandidatesLineV0",
    "BinaryCandidatesLineV1",
    "BinaryCandidatesLineV2",
    "CandidatesGridANSIStyled",
    "CandidatesGridCompact",
    "CandidatesGridPlain",
    "Json",
    "ValuesGrid",
    "ValuesLine",
]);
export const ALL_GRID_FORMATS = gridFormatSchema.options;

const gridMetricNameWithoutStrategySchema = z.enum([
    "strategyScore",
    "strategyApplicationCountAny",
    "strategyDeductionCountAny",
    "strategyAverageOptions",
    "solveGraphAverageBranchingFactor",
    "satStepCount",
    "backtrackCount",
    "gridGivensCount",
    "gridDirectCandidatesCount",
    "gridGivensValueCountDeviation",
]);
const gridMetricNameWithStrategySchema = z.enum(["strategyApplicationCountSingle", "strategyDeductionCountSingle"]);
export const GRID_METRIC_NAMES_WITH_STRATEGY = gridMetricNameWithStrategySchema.options;
export const gridMetricNameSchema = z.enum([
    "strategyScore",
    "strategyApplicationCountAny",
    "strategyApplicationCountSingle",
    "strategyDeductionCountAny",
    "strategyDeductionCountSingle",
    "strategyAverageOptions",
    "solveGraphAverageBranchingFactor",
    "satStepCount",
    "backtrackCount",
    "gridGivensCount",
    "gridDirectCandidatesCount",
    "gridGivensValueCountDeviation",
]);
export const ALL_GRID_METRIC_NAMES = gridMetricNameSchema.options;

export const gridMetricSchema = z.discriminatedUnion("kind", [
    z.object({
        kind: gridMetricNameWithoutStrategySchema,
    }),
    z.object({
        kind: gridMetricNameWithStrategySchema,
        strategy: strategyEnumSchema,
    }),
]);

export const GRID_METRIC_OPTIONS: Record<
    GridMetricName,
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
    strategyApplicationCountAny: {
        label: "Strategy (any): application count",
        description: "The number of times any strategy was applied to the grid.",
    },
    strategyApplicationCountSingle: {
        label: "Strategy (single): application count",
        description: "The number of times a single strategy was applied to the grid.",
    },
    strategyDeductionCountAny: {
        label: "Strategy (any): deduction count",
        description: "Number of deductions used to solve the grid.",
    },
    strategyDeductionCountSingle: {
        label: "Strategy (single): deduction count",
        description: "Number of deductions by a single strategy used to solve the grid.",
    },
    strategyAverageOptions: {
        label: "Strategy: average options",
        description: "The average number of strategies available to make progress.",
    },
    solveGraphAverageBranchingFactor: {
        label: "Solve graph average branching factor",
        // TODO: remove when implemented
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
        // TODO: remove when implemented
        disabled: true,
    },
};

export const goalOptimizationSchema = z.enum(["minimize", "maximize"]);
assert<IsEqual<z.infer<typeof goalOptimizationSchema>, GoalOptimization>>();
export const ALL_GOAL_OPTIMIZATIONS = goalOptimizationSchema.options;
