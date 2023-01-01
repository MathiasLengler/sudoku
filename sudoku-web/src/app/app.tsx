import type * as React from "react";
import { Suspense } from "react";
import { MyTheme } from "./myTheme";
import { RecoilRoot } from "recoil";
import { SudokuLoader } from "./sudokuLoader";
import { RecoilDebug } from "./RecoilDebug";
import { NoDefaultSandbox } from "./noDefaultSandbox";

export const App = () => {
    const noDefaultSandbox = false;
    return (
        <RecoilRoot>
            <RecoilDebug />
            <MyTheme>
                {noDefaultSandbox ? (
                    <Suspense fallback={"NoDefaultSandbox fallback"}>
                        <NoDefaultSandbox />
                    </Suspense>
                ) : (
                    <Suspense fallback={"App fallback"}>
                        <SudokuLoader />
                    </Suspense>
                )}
            </MyTheme>
        </RecoilRoot>
    );
};
