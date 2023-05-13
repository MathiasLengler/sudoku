import type { DynamicStrategy, DynamicGridFormat } from "./types";

export const WORKER_BOOT_UP_MESSAGE = "Worker loaded";
const arrayOfAll =
    <T>() =>
    <U extends T[]>(array: U & ([T] extends [U[number]] ? unknown : "Invalid")) =>
        array;
// Copy of sudokuController.allStrategies
export const ALL_STRATEGIES = arrayOfAll<DynamicStrategy>()([
    "NakedSingles",
    "HiddenSingles",
    "NakedPairs",
    "GroupReduction",
    "Backtracking",
]);

export const ALL_GRID_FORMATS = arrayOfAll<DynamicGridFormat>()([
    "CandidatesGridPlain",
    "CandidatesGridCompact",
    "CandidatesGridANSIStyled",
    "GivensLine",
    "GivensGrid",
    "BinaryCandidatesLine",
    "BinaryFixedCandidatesLine",
]);
