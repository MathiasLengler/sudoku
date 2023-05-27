import { atom } from "recoil";
import type { DynamicStrategy, TransportDeductions } from "../../types";

type Hint =
    | ({
          enabled: true;
          strategy: DynamicStrategy;
      } & TransportDeductions)
    | {
          enabled: false;
      };

export const hintState = atom<Hint>({
    key: "Hint",
    default: {
        enabled: false,
    },
});
