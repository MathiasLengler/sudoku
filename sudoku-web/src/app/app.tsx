import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { NotificationsProvider } from "@toolpad/core/useNotifications";
import { createStore, Provider as JotaiProvider } from "jotai";
import { DevTools } from "jotai-devtools";
import { Suspense } from "react";
import { SwManager } from "./SwManager";
import { BasicErrorBoundary, ThemeErrorBoundary } from "./components/ErrorFallback";
import { FullScreenSpinner } from "./components/FullScreenSpinner";
import { Sudoku } from "./sudoku";
import { MyTheme } from "./theme/myTheme";
import { store } from "./state/store";

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

export function App() {
    return (
        <BasicErrorBoundary>
            <JotaiProvider store={store}>
                {import.meta.env.DEV && <DevTools store={store} />}
                <QueryClientProvider client={queryClient}>
                    <MyTheme>
                        <ThemeErrorBoundary>
                            <NotificationsProvider slotProps={{ snackbar: { autoHideDuration: 3000 } }}>
                                <Suspense fallback={<FullScreenSpinner />}>
                                    <Sudoku />
                                </Suspense>
                                <SwManager />
                            </NotificationsProvider>
                        </ThemeErrorBoundary>
                    </MyTheme>
                </QueryClientProvider>
            </JotaiProvider>
        </BasicErrorBoundary>
    );
}
