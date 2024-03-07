import { Snapshot, atom } from "recoil";
import type { DynamicStrategy, TransportDeductions } from "../../types";

export async function getHint(snapshot: Snapshot) {
    return await snapshot.getPromise(hintState);
}

export type Hint = {
    strategy: DynamicStrategy;
} & TransportDeductions;

export type OptionalHint = Hint | undefined;

export const hintState = atom<OptionalHint>({
    key: "Hint",
    default: undefined,
    effects: [],
});
