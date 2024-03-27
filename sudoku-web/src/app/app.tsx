import { QueryClient, QueryClientProvider } from "@tanstack/react-query";
import { RecoilRoot } from "recoil";
import { MySnackbarProvider } from "./MySnackbarProvider";
import { RecoilDebug } from "./RecoilDebug";
import { BasicErrorBoundary, ThemeErrorBoundary } from "./components/ErrorFallback";
import { SudokuLoader } from "./sudokuLoader";
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
                {process.env.NODE_ENV !== "production" && <RecoilDebug />}
                <QueryClientProvider client={queryClient}>
                    <MyTheme>
                        <ThemeErrorBoundary>
                            <MySnackbarProvider>
                                <SudokuLoader />
                                <WorkboxManager />
                            </MySnackbarProvider>
                        </ThemeErrorBoundary>
                    </MyTheme>
                </QueryClientProvider>
            </RecoilRoot>
        </BasicErrorBoundary>
    );
};
