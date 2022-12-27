import * as React from "react";
import { MyTheme } from "./myTheme";
import { SudokuLoader } from "./sudokuLoader";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient({
    defaultOptions: {
        queries: {
            suspense: true
        }
    }
});
export const App: React.FunctionComponent = () => {
    return (
        <QueryClientProvider client={queryClient}>
            <MyTheme>
                <SudokuLoader />
            </MyTheme>
        </QueryClientProvider>
    );
};
