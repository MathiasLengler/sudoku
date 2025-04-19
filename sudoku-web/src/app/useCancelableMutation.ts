import { useMutation } from "@tanstack/react-query";
import { useCallback, useRef } from "react";

export type CancelableMutationFnArgs<Variables, Progress> = {
    variables: Variables;
    signal: AbortSignal;
    abortPromise: Promise<never>;
    onProgress: (progress: Progress) => void;
};

export function useCancelableMutation<Variables, Progress>({
    cancelableMutationFn,
    onProgress,
    onCancel,
}: {
    cancelableMutationFn: (args: CancelableMutationFnArgs<Variables, Progress>) => Promise<void>;
    onProgress: (progress: Progress) => void;
    onCancel: () => void;
}) {
    const abortControllerRef = useRef<AbortController | null>(null);

    const mutation = useMutation({
        mutationFn: async (variables: Variables) => {
            abortControllerRef.current = new AbortController();
            const signal = abortControllerRef.current.signal;
            const abortPromise = new Promise<never>((_resolve, reject) => {
                signal.addEventListener(
                    "abort",
                    () => {
                        reject(
                            signal.reason instanceof Error
                                ? signal.reason
                                : new Error("Unexpected abort reason", { cause: signal.reason }),
                        );
                    },
                    { once: true },
                );
            });
            await cancelableMutationFn({
                variables,
                signal,
                abortPromise,
                onProgress: (progress: Progress) => {
                    if (!signal.aborted) {
                        onProgress(progress);
                    } else {
                        console.warn("Progress update after abort:", progress);
                    }
                },
            });
        },
    });
    const { reset: resetMutation } = mutation;

    const cancel = useCallback(() => {
        abortControllerRef.current?.abort();
        resetMutation();
        onCancel();
    }, [onCancel, resetMutation]);

    return { mutation, cancel };
}
