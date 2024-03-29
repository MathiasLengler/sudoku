import { Alert, Button, Stack } from "@mui/material";
import type { ReactNode } from "react";
import { ErrorBoundary } from "react-error-boundary";
import { z } from "zod";

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

    return (
        // <Box sx={{ display: "grid", height: 1, gap: 1, placeItems: "center", alignContent: "center" }}>
        <Stack
            direction={inline ? "row" : "column"}
            width={1}
            height={1}
            spacing={2}
            justifyContent="center"
            alignItems="center"
        >
            <Alert severity="error">Unexpected error: {message}</Alert>
            <Button onClick={resetErrorBoundary} variant="contained">
                Try again
            </Button>
        </Stack>
        // </Box>
    );
}

export function ThemeErrorBoundary({ children, inline }: { children: ReactNode; inline?: boolean }) {
    return (
        <ErrorBoundary
            fallbackRender={({ error, resetErrorBoundary }) => (
                <ThemeFallback error={error as unknown} resetErrorBoundary={resetErrorBoundary} inline={inline} />
            )}
        >
            {children}
        </ErrorBoundary>
    );
}
