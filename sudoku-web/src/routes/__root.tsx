import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { createRootRoute, Outlet } from "@tanstack/react-router";
import { TanStackRouterDevtools } from "@tanstack/react-router-devtools";
import { NotificationsProvider } from "@toolpad/core/useNotifications";
import { Provider as JotaiProvider } from "jotai";
import { DevTools } from "jotai-devtools";
import jotaiDevToolsCss from "jotai-devtools/styles.css?inline";
import { Suspense } from "react";
import "../app/theme/styles";
import SudokuAppBar from "../app/appBar/sudokuAppBar";
import { SwManager } from "../app/SwManager";
import { BasicErrorBoundary, ThemeErrorBoundary } from "../app/components/ErrorFallback";
import { FullScreenSpinner } from "../app/components/FullScreenSpinner";
import { store } from "../app/state/store";
import { SudokuEffects } from "../app/sudokuEffects";
import { useKeyboardInput } from "../app/useKeyboardInput";
import { MyTheme } from "../app/theme/myTheme";

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            // We use react-query as an async state manager when interacting with the web worker.
            // This works offline, completely independent of the network status.
            networkMode: "always",
            // The communication with the web worker is reliable, so retries don't make sense.
            retry: false,
        },
        mutations: {
            networkMode: "always",
        },
    },
});

function JotaiDevTools() {
    return import.meta.env.DEV ? (
        <>
            <style>{jotaiDevToolsCss}</style>
            <DevTools store={store} />
        </>
    ) : null;
}

function RouterDevTools() {
    return import.meta.env.DEV ? <TanStackRouterDevtools position="bottom-left" /> : null;
}

function RootLayout() {
    const { onKeyDown } = useKeyboardInput();

    return (
        <div
            className="app"
            onKeyDown={onKeyDown}
            // Enable keyboard events
            tabIndex={0}
        >
            <SudokuAppBar />
            <div className="app-content">
                <ThemeErrorBoundary>
                    <Suspense fallback={<FullScreenSpinner />}>
                        <Outlet />
                    </Suspense>
                </ThemeErrorBoundary>
            </div>
            <SudokuEffects />
        </div>
    );
}

function RootComponent() {
    return (
        <BasicErrorBoundary>
            <JotaiProvider store={store}>
                <JotaiDevTools />
                <QueryClientProvider client={queryClient}>
                    <MyTheme>
                        <ThemeErrorBoundary>
                            <NotificationsProvider slotProps={{ snackbar: { autoHideDuration: 3000 } }}>
                                <Suspense fallback={<FullScreenSpinner />}>
                                    <RootLayout />
                                </Suspense>
                                <SwManager />
                            </NotificationsProvider>
                        </ThemeErrorBoundary>
                    </MyTheme>
                </QueryClientProvider>
                <RouterDevTools />
            </JotaiProvider>
        </BasicErrorBoundary>
    );
}

export const Route = createRootRoute({
    component: RootComponent,
});
