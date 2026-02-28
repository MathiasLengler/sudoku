import { useAtomCallback } from "jotai/utils";
import { inputState } from "../state/input";
import { useCallback } from "react";

export function useToggleCandidateMode() {
    return useAtomCallback(
        useCallback((get, set) => {
            const input = get(inputState);
            set(inputState, { ...input, candidateMode: !input.candidateMode });
        }, []),
    );
}

export function useToggleStickyMode() {
    return useAtomCallback(
        useCallback((get, set) => {
            const input = get(inputState);
            if (input.stickyMode) {
                set(inputState, {
                    stickyMode: false,
                    candidateMode: input.candidateMode,
                    colorMode: input.colorMode,
                    selectedPos: input.previouslySelectedPos,
                    previouslySelectedValue: input.selectedValue,
                });
            } else {
                set(inputState, {
                    stickyMode: true,
                    candidateMode: input.candidateMode,
                    colorMode: input.colorMode,
                    selectedValue: input.previouslySelectedValue,
                    previouslySelectedPos: input.selectedPos,
                    stickyChain: undefined,
                });
            }
        }, []),
    );
}

export function useToggleColorMode() {
    return useAtomCallback(
        useCallback((get, set) => {
            const input = get(inputState);
            set(inputState, { ...input, colorMode: !input.colorMode });
        }, []),
    );
}

export function useEndStickyChain() {
    return useAtomCallback(
        useCallback((get, set) => {
            const input = get(inputState);
            if (input.stickyMode && input.stickyChain) {
                set(inputState, { ...input, stickyChain: undefined });
            }
        }, []),
    );
}
