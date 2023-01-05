import type { DynamicStrategy } from "./types";

export const WORKER_BOOT_UP_MESSAGE = "Worker loaded";
const arrayOfAll =
    <T>() =>
    <U extends T[]>(array: U & ([T] extends [U[number]] ? unknown : "Invalid")) =>
        array;
// Copy of sudokuController.allStrategies
export const ALL_STRATEGIES = arrayOfAll<DynamicStrategy>()([
    "SingleCandidate",
    "HiddenSingles",
    "GroupReduction",
    "Backtracking",
]);
