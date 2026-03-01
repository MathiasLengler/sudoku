import type { IsEqual } from "type-fest";
import * as z from "zod";
import type { GoalOptimization, GridMetricName, StrategyEnum, StrategyMap } from "../types";
import { assert } from "../typeUtils";

export const STRATEGY_NAMES = [
    { strategyEnum: "NakedSingles", mapKey: "naked_singles" },
    { strategyEnum: "HiddenSingles", mapKey: "hidden_singles" },
    { strategyEnum: "NakedPairs", mapKey: "naked_pairs" },
    { strategyEnum: "LockedSets", mapKey: "locked_sets" },
    { strategyEnum: "GroupIntersectionBlockToAxis", mapKey: "group_intersection_block_to_axis" },
    { strategyEnum: "GroupIntersectionAxisToBlock", mapKey: "group_intersection_axis_to_block" },
    { strategyEnum: "GroupIntersectionBoth", mapKey: "group_intersection_both" },
    { strategyEnum: "XWing", mapKey: "x_wing" },
    { strategyEnum: "ChuteRemotePairs", mapKey: "chute_remote_pairs" },
    { strategyEnum: "BruteForce", mapKey: "brute_force" },
] satisfies { strategyEnum: StrategyEnum; mapKey: keyof StrategyMap<boolean> }[];

export const strategyEnumSchema = z.enum(STRATEGY_NAMES.map((s) => s.strategyEnum));
export const ALL_STRATEGIES = strategyEnumSchema.options;

export const strategyMapKeySchema = z.enum(STRATEGY_NAMES.map((s) => s.mapKey));

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
    ChuteRemotePairs: {
        label: "Remote Pairs",
        description:
            "Chains of bivalent cells with the same two candidates. Any cell seeing both endpoints of an even-length chain can eliminate those candidates.",
        link: "https://www.sudokuwiki.org/Remote_Pairs",
    },
    BruteForce: {
        label: "Brute Force",
        description:
            "Solve the sudoku using brute force/trial and error. Backed by a Backtracking- or SAT-solver depending on the size of the Sudoku.",
        link: "https://t-dillon.github.io/tdoku/",
    },
};

export const strategyListSchema = strategyEnumSchema.array().min(1);

export const strategySetSchema = z.record(strategyMapKeySchema, z.boolean());

export type SelectedStrategies = z.infer<typeof selectedStrategiesSchema>;
export const selectedStrategiesSchema = z.codec(strategyListSchema, strategySetSchema, {
    encode: (strategySet) => {
        return STRATEGY_NAMES.filter(({ mapKey }) => strategySet[mapKey]).map(({ strategyEnum }) => strategyEnum);
    },
    decode: (strategyList) => {
        return Object.fromEntries(
            STRATEGY_NAMES.map(({ strategyEnum, mapKey }) => {
                return [mapKey, strategyList.includes(strategyEnum)];
            }),
        ) as StrategyMap<boolean>;
    },
});

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
        label: "Grid givens: value count standard deviation",
    },
};

export const goalOptimizationSchema = z.enum(["minimize", "maximize"]);
assert<IsEqual<z.infer<typeof goalOptimizationSchema>, GoalOptimization>>();
export const ALL_GOAL_OPTIMIZATIONS = goalOptimizationSchema.options;
