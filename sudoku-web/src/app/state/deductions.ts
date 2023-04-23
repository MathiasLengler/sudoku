import { atom } from "recoil";
import type { DynamicStrategy, TransportDeductions } from "../../types";

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
