import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { Suspense } from "react";
import { RecoilRoot } from "recoil";
import { MyNotificationsProvider } from "./MyNotificationsProvider";
import { RecoilDebug } from "./RecoilDebug";
import { BasicErrorBoundary, ThemeErrorBoundary } from "./components/ErrorFallback";
import { FullScreenSpinner } from "./components/FullScreenSpinner";
import { Sudoku } from "./sudoku";
import { MyTheme } from "./theme/myTheme";
import { WorkboxManager } from "./workboxManager";

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

export const App = () => {
    return (
        <BasicErrorBoundary>
            <RecoilRoot>
                {import.meta.env.DEV && <RecoilDebug />}
                <QueryClientProvider client={queryClient}>
                    <MyTheme>
                        <ThemeErrorBoundary>
                            <MyNotificationsProvider>
                                <Suspense fallback={<FullScreenSpinner />}>
                                    <Sudoku />
                                </Suspense>
                                <WorkboxManager />
                            </MyNotificationsProvider>
                        </ThemeErrorBoundary>
                    </MyTheme>
                </QueryClientProvider>
            </RecoilRoot>
        </BasicErrorBoundary>
    );
};
