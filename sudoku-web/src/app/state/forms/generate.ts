import { atomWithStorage } from "jotai/utils";
import * as z from "zod";
import { goalOptimizationSchema, gridMetricSchema, selectedStrategiesSchema } from "../../constants";
import { baseSchema } from "../../utils/base";
import { getZodLocalStorage } from "../localStorageEffect";

export const SEED_MAX = Number.MAX_SAFE_INTEGER;

export const MIN_ITERATIONS_INDEX = 0;
export const MAX_ITERATIONS_INDEX = 16;

export function iterationsIndexToIterations(iterationsIndex: number): number {
    return 2 ** iterationsIndex;
}

// TODO: use zod pipe to simplify this
const parseBigintSchema = <T extends z.ZodBigInt>(bigIntSchema: T) =>
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
    base: baseSchema,
    minGivens: z.int().min(0),
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
                    ctx.addIssue(issue.message);
                }
            }
        }),
    multiShot: z.boolean(),
    iterationsIndex: z.int().min(MIN_ITERATIONS_INDEX).max(MAX_ITERATIONS_INDEX),
    metric: gridMetricSchema,
    optimize: goalOptimizationSchema,
    parallel: z.boolean(),
});
export const GENERATE_FORM_DEFAULT_VALUES = {
    base: 3 as const,
    minGivens: 0,
    strategies: selectedStrategiesSchema.decode([
        "NakedSingles",
        "HiddenSingles",
        "NakedPairs",
        "LockedSets",
        "GroupIntersectionBoth",
        "XWing",
    ]),
    setAllDirectCandidates: true,
    useSeed: false,
    seed: "0",
    multiShot: false,
    iterationsIndex: 8,
    metric: { kind: "strategyScore" },
    optimize: "maximize",
    parallel: true,
} satisfies GenerateFormValues;
export const generateFormValuesState = atomWithStorage<GenerateFormValues>(
    "GenerateFormValues",
    GENERATE_FORM_DEFAULT_VALUES,
    getZodLocalStorage(generateFormValuesSchema),
);
