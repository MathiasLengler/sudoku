import type * as React from "react";
import { MyTheme } from "./myTheme";
import { SudokuLoader } from "./sudokuLoader";
import { QueryClient, QueryClientProvider } from "@tanstack/react-query";

const queryClient = new QueryClient();
export const App: React.FunctionComponent = () => {
    return (
        <QueryClientProvider client={queryClient}>
            <MyTheme>
                <SudokuLoader />
            </MyTheme>
        </QueryClientProvider>
    );
};
