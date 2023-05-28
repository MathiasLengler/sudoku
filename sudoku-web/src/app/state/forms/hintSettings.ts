import { z } from "zod";
import { ALL_STRATEGIES, dynamicStrategySchema } from "../../../constants";
import { atom } from "recoil";
import { localStorageEffect } from "../localStorageEffect";

/* TODO: Hint settings:
- Selected Strategies (try in order)

- Hint Mode
    - Toggle hint
    - Hint, then apply
    - Apply

- Run in Loop / Repeat (disabled for toggle hint)
    - Delay

- Single / Multiple deductions
*/

export const MAX_LOOP_DELAY_MS = 4000;

//TODO: color hint light bulb in toolbar based on hint state
//  - no hint available
//  - hint available
//  - hint shown
// TODO: reset hint state when it is applied manually
// TODO: hint mode "Hint, then apply":
//   handle stale deductions (user input since strategy execution)
export type HintSettings = z.infer<typeof hintSettingsSchema>;
export const hintSettingsSchema = z.object({
    strategies: z.array(dynamicStrategySchema),
    mode: z.enum(["toggleHint", "hintApply", "apply"]),
    doLoop: z.boolean(),
    loopDelayMs: z.number().nonnegative().max(MAX_LOOP_DELAY_MS),
    multipleDeductions: z.boolean(),
});

export const DEFAULT_HINT_SETTINGS = {
    strategies: ALL_STRATEGIES.filter(strategy => strategy !== "Backtracking"),
    mode: "hintApply",
    doLoop: false,
    loopDelayMs: 0,
    multipleDeductions: true,
} satisfies HintSettings;
export const hintSettingsState = atom<HintSettings>({
    key: "HintSettingsFormValues",
    default: DEFAULT_HINT_SETTINGS,
    effects: [localStorageEffect(hintSettingsSchema)],
});
