import type * as React from "react";
import { Suspense } from "react";
import { MyTheme } from "./myTheme";
import { RecoilRoot } from "recoil";
import { SudokuLoader } from "./sudokuLoader";
import { RecoilDebug } from "./RecoilDebug";

export const App = () => {
    return (
        <RecoilRoot>
            {process.env.NODE_ENV !== "production" && <RecoilDebug />}
            <MyTheme>
                <Suspense fallback={"App fallback"}>
                    <SudokuLoader />
                </Suspense>
            </MyTheme>
        </RecoilRoot>
    );
};
