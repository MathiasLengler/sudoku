import { atomWithReset } from "jotai/utils";
import type { StrategyEnum, TransportDeductions } from "../../types";

export type Hint = {
    strategy: StrategyEnum;
} & TransportDeductions;

export type OptionalHint = Hint | undefined;

export const hintState = atomWithReset<OptionalHint>(undefined);
