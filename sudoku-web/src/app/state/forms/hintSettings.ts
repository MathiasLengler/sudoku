import { z } from "zod";
import { ALL_STRATEGIES, dynamicStrategySchema } from "../../../constants";
import { atom } from "recoil";
import { localStorageEffect } from "../localStorageEffect";

export function scaleLoopDelayIndex(loopDelayIndex: number) {
    if (loopDelayIndex === 0) {
        return 0;
    }
    if (loopDelayIndex <= 10) {
        return 2 ** (loopDelayIndex - 1);
    }
    return 1000 * 2 ** (loopDelayIndex - 11);
}

export function unscaleLoopDelayMs(loopDelayMs: number) {
    if (loopDelayMs === 0) {
        return 0;
    }
    if (loopDelayMs < 1000) {
        return Math.log2(loopDelayMs) + 1;
    }
    return Math.log2(loopDelayMs / 1000) + 11;
}

export const MAX_LOOP_DELAY_INDEX = unscaleLoopDelayMs(4000);

//TODO: color hint light bulb in toolbar based on hint state
//  - no hint available
//  - hint available
//  - hint shown
// TODO: reset hint state when it is applied manually
// TODO: hint mode "Hint, then apply":
//   handle stale deductions (user input since strategy execution)
export type HintSettings = z.infer<typeof hintSettingsSchema>;
export const hintSettingsSchema = z.object({
    strategies: z.array(dynamicStrategySchema).min(1),
    mode: z.enum(["toggleHint", "hintApply", "apply"]),
    doLoop: z.boolean(),
    loopDelayIndex: z.number().nonnegative().max(MAX_LOOP_DELAY_INDEX),
    multipleDeductions: z.boolean(),
});

export const DEFAULT_HINT_SETTINGS = {
    strategies: ALL_STRATEGIES.filter(strategy => strategy !== "Backtracking"),
    mode: "hintApply",
    doLoop: false,
    loopDelayIndex: 0,
    multipleDeductions: true,
} satisfies HintSettings;
export const hintSettingsState = atom<HintSettings>({
    key: "HintSettingsFormValues",
    default: DEFAULT_HINT_SETTINGS,
    effects: [localStorageEffect(hintSettingsSchema)],
});
