import { atom } from "recoil";
import type { DynamicStrategy, TransportDeductions } from "../../types";
import { z } from "zod";
import { localStorageEffect } from "./localStorageEffect";
import { ALL_STRATEGIES, dynamicStrategySchema } from "../../constants";

type SolverHint =
    | ({
          enabled: true;
          strategy: DynamicStrategy;
      } & TransportDeductions)
    | {
          enabled: false;
      };

export const solverHintState = atom<SolverHint>({
    key: "SolverHint",
    default: {
        enabled: false,
    },
});

export type SolverConfig = z.infer<typeof solverConfigSchema>;
export const solverConfigSchema = z.object({
    strategies: z.array(dynamicStrategySchema),
});

export const solverConfigState = atom<SolverConfig>({
    key: "SolverConfig",
    default: {
        strategies: ALL_STRATEGIES.filter(strategy => strategy !== "Backtracking"),
    },
    effects: [localStorageEffect(solverConfigSchema)],
});
