import { Snapshot, atom } from "recoil";
import type { StrategyEnum, TransportDeductions } from "../../types";

export async function getHint(snapshot: Snapshot) {
    return await snapshot.getPromise(hintState);
}

export type Hint = {
    strategy: StrategyEnum;
} & TransportDeductions;

export type OptionalHint = Hint | undefined;

export const hintState = atom<OptionalHint>({
    key: "Hint",
    default: undefined,
    effects: [],
});
