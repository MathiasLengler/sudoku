import { type Snapshot, useRecoilCallback } from "recoil";
import { type Input, inputState } from "../state/input";

export async function getInput(snapshot: Snapshot) {
    return await snapshot.getPromise(inputState);
}

export function useToggleCandidateMode() {
    return useRecoilCallback(
        ({ set }) =>
            () => {
                set(inputState, input => ({ ...input, candidateMode: !input.candidateMode }));
            },
        []
    );
}

export function useToggleStickyMode() {
    return useRecoilCallback(
        ({ set }) =>
            () => {
                set(inputState, (input): Input => {
                    if (input.stickyMode) {
                        return {
                            stickyMode: false,
                            candidateMode: input.candidateMode,
                            selectedPos: input.previouslySelectedPos,
                            previouslySelectedValue: input.selectedValue,
                        };
                    } else {
                        return {
                            stickyMode: true,
                            candidateMode: input.candidateMode,
                            selectedValue: input.previouslySelectedValue,
                            previouslySelectedPos: input.selectedPos,
                            stickyChain: undefined,
                        };
                    }
                });
            },
        []
    );
}

export function useEndStickyChain() {
    return useRecoilCallback(
        ({ snapshot, set }) =>
            async () => {
                const input = await getInput(snapshot);
                if (input.stickyMode && input.stickyChain) {
                    set(inputState, { ...input, stickyChain: undefined });
                }
            },
        []
    );
}
