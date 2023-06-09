import { atom } from "recoil";
import type { DynamicStrategy, TransportDeductions } from "../../types";

export type Hint = {
    strategy: DynamicStrategy;
} & TransportDeductions;

export type OptionalHint = Hint | undefined;

export const hintState = atom<OptionalHint>({
    key: "Hint",
    default: undefined,
    effects: [],
});
