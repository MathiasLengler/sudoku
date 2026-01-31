import { Alert, Button, Stack } from "@mui/material";

import { useMemo, type ReactNode } from "react";
import { ErrorBoundary } from "react-error-boundary";
import { useSetAtom } from "jotai";
import * as z from "zod";
import { resetWorkerAfterPanicAction, WasmPanicError } from "../state/worker";

type FallbackProps = {
    error: unknown;
    resetErrorBoundary: () => void;
};

const errorToMessageSchema = z
    .union([z.instanceof(Error), z.string()])
    .catch("unknown")
    .transform((value) => {
        if (value instanceof Error) {
            return value.message;
        } else {
            return value;
        }
    });

/**
 * Check if an error is likely caused by a Rust panic in the WASM module.
 * Panics typically include "unreachable" (from wasm trap) or "RuntimeError" in the message.
 */
function isProbablyWasmPanic(error: unknown): boolean {
    if (error instanceof WasmPanicError) {
        return true;
    }
    if (error instanceof Error) {
        const message = error.message.toLowerCase();
        const name = error.name.toLowerCase();
        // Common patterns for WASM panics:
        // - "unreachable" - appears in wasm traps after panic
        // - "RuntimeError" - the error type for wasm traps
        // - "panicked" - may appear in panic messages
        // - "wasm" + "trap" - wasm trap indication
        return (
            message.includes("unreachable") ||
            name.includes("runtimeerror") ||
            message.includes("panicked") ||
            (message.includes("wasm") && message.includes("trap"))
        );
    }
    return false;
}

function BasicFallback({ error, resetErrorBoundary }: FallbackProps) {
    const message = errorToMessageSchema.parse(error);

    return (
        <div role="alert">
            <p>Unexpected error:</p>
            <pre>{message}</pre>
            <button onClick={resetErrorBoundary}>Try again</button>
        </div>
    );
}

export function BasicErrorBoundary({ children }: { children: ReactNode }) {
    return <ErrorBoundary FallbackComponent={BasicFallback}>{children}</ErrorBoundary>;
}

function ThemeFallback({ error, resetErrorBoundary, inline }: FallbackProps & { inline?: boolean }) {
    const message = errorToMessageSchema.parse(error);
    const isPanic = useMemo(() => isProbablyWasmPanic(error), [error]);
    const resetWorker = useSetAtom(resetWorkerAfterPanicAction);

    const handleRetry = () => {
        if (isPanic) {
            // Reset the worker before retrying to ensure clean state
            resetWorker();
        }
        resetErrorBoundary();
    };

    return (
        <Stack
            direction={inline ? "row" : "column"}
            spacing={2}
            sx={{
                width: 1,
                height: 1,
                justifyContent: "center",
                alignItems: "center",
            }}
        >
            <Alert severity="error">
                {isPanic
                    ? "A critical error occurred in the puzzle engine. The engine will be reset."
                    : `Unexpected error: ${message}`}
            </Alert>
            <Button onClick={handleRetry} variant="contained">
                {isPanic ? "Reset and continue" : "Try again"}
            </Button>
        </Stack>
    );
}

export function ThemeErrorBoundary({ children, inline }: { children: ReactNode; inline?: boolean }) {
    return (
        <ErrorBoundary
            fallbackRender={({ error, resetErrorBoundary }) => (
                <ThemeFallback error={error} resetErrorBoundary={resetErrorBoundary} inline={inline} />
            )}
        >
            {children}
        </ErrorBoundary>
    );
}
