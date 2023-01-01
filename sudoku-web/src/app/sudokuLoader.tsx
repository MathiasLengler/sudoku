import type * as React from "react";
import { Suspense } from "react";
import { Box, Stack } from "@mui/material";
import CircularProgress from "@mui/material/CircularProgress";
import { Sudoku } from "./sudoku";

function FullScreenSpinner() {
    return (
        <Box className="app-spinner" sx={{ height: 1 }}>
            <Stack direction="column" justifyContent="center" alignItems="center" spacing={2} sx={{ height: 1 }}>
                <CircularProgress />
            </Stack>
        </Box>
    );
}

export const SudokuLoader: React.FunctionComponent = () => {
    return (
        <Suspense fallback={<FullScreenSpinner />}>
            <Sudoku />
        </Suspense>
    );
};
