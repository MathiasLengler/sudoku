import type { GridFormatEnum, StrategyEnum } from "./types";
import { z } from "zod";
import type { IsEqual } from "type-fest";
import { assert } from "./typeUtils";
import { sortBy } from "lodash";

export const WORKER_BOOT_UP_MESSAGE = "Worker loaded";

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
    .transform((strategies) => sortBy(strategies, (strategy) => strategyEnumSchema.options.indexOf(strategy)));

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
