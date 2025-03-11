import { z } from "zod";
import range from "lodash/range";
import { baseToSideLength } from "../../utils/sudoku";
import { atom } from "recoil";
import { localStorageEffect } from "../localStorageEffect";
import type { ZodBigInt } from "zod";
import { selectedStrategiesSchema } from "../../../constants";

export const BASE_MIN = 2;
export const BASE_MAX = 5;
export const BASE_MARKS = range(BASE_MIN, BASE_MAX + 1).map((base) => {
    return {
        value: base,
        label: baseToLabel(base),
    };
});

export function baseToLabel(base: number): string {
    const sideLength = baseToSideLength(base);

    return `${sideLength}x${sideLength}`;
}
export const SEED_MAX = Number.MAX_SAFE_INTEGER;
// TODO: use zod pipe to simplify this
const parseBigintSchema = <T extends ZodBigInt>(bigIntSchema: T) =>
    z.preprocess((value) => {
        const safeParseResult = z
            .bigint()
            .or(z.number())
            .or(z.string())
            .transform((value) => {
                try {
                    return BigInt(value);
                } catch (_err) {
                    return value;
                }
            })
            .safeParse(value);
        return safeParseResult.success ? safeParseResult.data : value;
    }, bigIntSchema);
export type GenerateFormValues = z.infer<typeof generateFormValuesSchema>;
export const generateFormValuesSchema = z.object({
    base: z.number().int().min(BASE_MIN).max(BASE_MAX),
    minGivens: z.number().int().min(0),
    strategies: selectedStrategiesSchema,
    setAllDirectCandidates: z.boolean(),
    useSeed: z.boolean(),
    seed: z
        .string()
        .optional()
        .superRefine((value, ctx) => {
            const bigintResult = parseBigintSchema(z.bigint().min(0n).max(BigInt(SEED_MAX)))
                .optional()
                .safeParse(value);
            if (!bigintResult.success) {
                for (const issue of bigintResult.error.issues) {
                    ctx.addIssue(issue);
                }
            }
        }),
    // TODO: remove
    parallel: z.boolean().default(false),
});
export const GENERATE_FORM_DEFAULT_VALUES = {
    base: 3,
    minGivens: 0,
    strategies: ["Backtracking"],
    setAllDirectCandidates: true,
    useSeed: false,
    seed: "0",
    parallel: false,
} satisfies GenerateFormValues;
export const generateFormValuesState = atom<GenerateFormValues>({
    key: "GenerateFormValues",
    default: GENERATE_FORM_DEFAULT_VALUES,
    effects: [localStorageEffect(generateFormValuesSchema)],
});
