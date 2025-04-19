import type * as CSS from "csstype";
import { Suspense } from "react";
import { useResizeDetector } from "react-resize-detector";
import { useRecoilValue } from "recoil";
import SudokuAppBar from "./appBar/sudokuAppBar";
import { ThemeErrorBoundary } from "./components/ErrorFallback";
import { FullScreenSpinner } from "./components/FullScreenSpinner";
import { WorldMap } from "./components/world/WorldMap";
import { ControlPanel } from "./controlPanel/controlPanel";
import { Grid } from "./grid/grid";
import { sudokuBaseState, sudokuSideLengthState } from "./state/sudoku";
import { gameModeState } from "./state/world";
import { SudokuEffects } from "./sudokuEffects";
import { useKeyboardInput } from "./useKeyboardInput";

const SudokuGame = () => {
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);

    // Responsive Grid
    const { width: gridWidth, height: gridHeight, ref: gridRef } = useResizeDetector<HTMLDivElement>({});

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
        "--grid-size": gridWidth && gridHeight ? `${Math.min(gridWidth, gridHeight)}px` : "0",
    };

    return (
        <div className="sudoku-game" style={cssVariables}>
            <Grid gridRef={gridRef} />
            <ControlPanel />
        </div>
    );
};

const SudokuContent = () => {
    const gameMode = useRecoilValue(gameModeState);
    return (
        <div className="app-content">
            <ThemeErrorBoundary>
                <Suspense fallback={<FullScreenSpinner />}>
                    {gameMode.mode === "world" && gameMode.view === "map" ? <WorldMap /> : <SudokuGame />}
                </Suspense>
            </ThemeErrorBoundary>
        </div>
    );
};

export const Sudoku = () => {
    const { onKeyDown } = useKeyboardInput();

    return (
        <div
            className="app"
            onKeyDown={onKeyDown}
            // Enable keyboard events
            tabIndex={0}
        >
            <SudokuAppBar />
            <SudokuContent />
            <SudokuEffects />
        </div>
    );
};
