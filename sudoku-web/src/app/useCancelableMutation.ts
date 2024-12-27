import { useCallback, useRef, useState } from "react";
import { useMutation } from "@tanstack/react-query";

export type CancelableMutationFnArgs<Variables, Progress> = {
    variables: Variables;
    signal: AbortSignal;
    abortPromise: Promise<never>;
    onProgress: (progress: Progress) => void;
};

export function useCancelableMutation<Variables, Progress>(
    cancelableMutationFn: (args: CancelableMutationFnArgs<Variables, Progress>) => Promise<void>,
) {
    const [progress, setProgress] = useState<Progress>();
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
                        setProgress(progress);
                    }
                },
            });
        },
    });
    const { reset: resetMutation } = mutation;

    const cancel = useCallback(() => {
        abortControllerRef.current?.abort();
        resetMutation();
        setProgress(undefined);
    }, [resetMutation]);

    return { mutation, progress, cancel };
}
