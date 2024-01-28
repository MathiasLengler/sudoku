import type * as React from "react";
import { Suspense } from "react";
import { MyTheme } from "./theme/myTheme";
import { RecoilRoot } from "recoil";
import { SudokuLoader } from "./sudokuLoader";
import { RecoilDebug } from "./RecoilDebug";
import { WorkboxManager } from "./workboxManager";
import { MySnackbarProvider } from "./MySnackbarProvider";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

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
        <RecoilRoot>
            {process.env.NODE_ENV !== "production" && <RecoilDebug />}
            <QueryClientProvider client={queryClient}>
                <MyTheme>
                    <MySnackbarProvider>
                        <Suspense fallback={"App fallback"}>
                            <SudokuLoader />
                        </Suspense>
                        <WorkboxManager />
                    </MySnackbarProvider>
                </MyTheme>
            </QueryClientProvider>
        </RecoilRoot>
    );
};
