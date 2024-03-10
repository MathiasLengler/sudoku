import type { GridFormatEnum, StrategyEnum } from "./types";
import { z } from "zod";
import type { IsEqual } from "type-fest";
import { assert } from "./typeUtils";

export const WORKER_BOOT_UP_MESSAGE = "Worker loaded";

export const strategyEnumSchema = z.enum([
    "NakedSingles",
    "HiddenSingles",
    "NakedPairs",
    "GroupReduction",
    "GroupIntersectionBlockToAxis",
    "GroupIntersectionAxisToBlock",
    "GroupIntersectionBoth",
    "Backtracking",
]);

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
