import { useAtomCallback } from "jotai/utils";
import { inputState } from "../state/input";
import { useCallback } from "react";

export function useToggleCandidateMode() {
    return useAtomCallback(
        useCallback(
            (get, set) => () => {
                const input = get(inputState);
                set(inputState, { ...input, candidateMode: !input.candidateMode });
            },
            [],
        ),
    );
}

export function useToggleStickyMode() {
    return useAtomCallback(
        useCallback(
            (get, set) => () => {
                const input = get(inputState);
                if (input.stickyMode) {
                    set(inputState, {
                        stickyMode: false,
                        candidateMode: input.candidateMode,
                        selectedPos: input.previouslySelectedPos,
                        previouslySelectedValue: input.selectedValue,
                    });
                } else {
                    set(inputState, {
                        stickyMode: true,
                        candidateMode: input.candidateMode,
                        selectedValue: input.previouslySelectedValue,
                        previouslySelectedPos: input.selectedPos,
                        stickyChain: undefined,
                    });
                }
            },
            [],
        ),
    );
}

export function useEndStickyChain() {
    return useAtomCallback(
        useCallback(
            (get, set) => () => {
                const input = get(inputState);
                if (input.stickyMode && input.stickyChain) {
                    set(inputState, { ...input, stickyChain: undefined });
                }
            },
            [],
        ),
    );
}
