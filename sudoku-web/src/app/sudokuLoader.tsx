import type * as React from "react";
import { Suspense } from "react";
import { Sudoku } from "./sudoku";
import { FullScreenSpinner } from "./components/FullScreenSpinner";

export const SudokuLoader: React.FunctionComponent = () => {
    return (
        <Suspense fallback={<FullScreenSpinner />}>
            <Sudoku />
        </Suspense>
    );
};
