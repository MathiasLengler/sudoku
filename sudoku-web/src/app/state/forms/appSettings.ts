import { atomWithStorage } from "jotai/utils";
import * as z from "zod";
import { getZodLocalStorage } from "../localStorageEffect";

export const colorModeSchema = z.enum(["auto", "light", "dark"]);
export type ColorMode = z.infer<typeof colorModeSchema>;

export type AppSettings = z.infer<typeof appSettingsSchema>;
export const appSettingsSchema = z.object({
    // Theme settings
    themeColorHue: z.int().min(0).max(360),
    colorMode: colorModeSchema,

    // Game behavior settings
    valueHintingInStickyMode: z.boolean(),
    removeCandidatesOnSetValue: z.boolean(),
    highlightConflictIncorrectValue: z.boolean(),
    highlightMissingNote: z.boolean(),
    highlightStickyCandidates: z.boolean(),
    switchStickyValueOnTapGiven: z.boolean(),

    // UI settings
    showTimer: z.boolean(),
    inputNumberBlock: z.boolean(),
});

export const DEFAULT_APP_SETTINGS: AppSettings = {
    // Theme settings - default hue 213 (blue) matches current theme
    themeColorHue: 213,
    colorMode: "auto",

    // Game behavior settings - defaults to current behavior
    valueHintingInStickyMode: true,
    removeCandidatesOnSetValue: true,
    highlightConflictIncorrectValue: true,
    highlightMissingNote: false,
    highlightStickyCandidates: true,
    switchStickyValueOnTapGiven: false,

    // UI settings
    showTimer: false,
    inputNumberBlock: false,
};

export const appSettingsState = atomWithStorage<AppSettings>(
    "AppSettings",
    DEFAULT_APP_SETTINGS,
    getZodLocalStorage(appSettingsSchema),
);
