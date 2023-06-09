import type * as React from "react";
import { Suspense } from "react";
import { MyTheme } from "./theme/myTheme";
import { RecoilRoot } from "recoil";
import { SudokuLoader } from "./sudokuLoader";
import { RecoilDebug } from "./RecoilDebug";
import { WorkboxManager } from "./workboxManager";
import { MySnackbarProvider } from "./MySnackbarProvider";

export const App = () => {
    return (
        <RecoilRoot>
            {process.env.NODE_ENV !== "production" && <RecoilDebug />}
            <MyTheme>
                <MySnackbarProvider>
                    <Suspense fallback={"App fallback"}>
                        <SudokuLoader />
                    </Suspense>
                    <WorkboxManager />
                </MySnackbarProvider>
            </MyTheme>
        </RecoilRoot>
    );
};
