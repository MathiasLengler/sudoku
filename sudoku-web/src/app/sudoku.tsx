import type * as CSS from "csstype";
import { Suspense } from "react";
import { useResizeDetector } from "react-resize-detector";
import { useAtomValue } from "jotai";
import SudokuAppBar from "./appBar/sudokuAppBar";
import { ThemeErrorBoundary } from "./components/ErrorFallback";
import { FullScreenSpinner } from "./components/FullScreenSpinner";
import { PuzzleOverlay } from "./components/puzzle/PuzzleOverlay";
import { WorldMap } from "./components/world/WorldMap";
import { Toolbar } from "./controlPanel/toolbar";
import { ValueSelector } from "./controlPanel/valueSelector";
import { Grid } from "./grid/grid";
import { sudokuBaseState, sudokuSideLengthState } from "./state/sudoku";
import { showWorldMapState } from "./state/world";
import { SudokuEffects } from "./sudokuEffects";
import { useKeyboardInput } from "./useKeyboardInput";

function SudokuGame() {
    const base = useAtomValue(sudokuBaseState);
    const sideLength = useAtomValue(sudokuSideLengthState);

    // Responsive Grid
    const {
        width: gridWidth,
        height: gridHeight,
        ref: gridRef,
    } = useResizeDetector<HTMLDivElement>({
        observerOptions: {
            box: "border-box",
        },
    });

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--base": base,
        "--grid-size": gridWidth && gridHeight ? `${Math.min(gridWidth, gridHeight)}px` : "0",
    };

    return (
        <div className="sudoku-game" style={{ ...cssVariables, position: "relative" }}>
            <PuzzleOverlay />
            <Grid gridRef={gridRef} />
            <Toolbar />
            <ValueSelector />
        </div>
    );
}

function SudokuContent() {
    const showWorldMap = useAtomValue(showWorldMapState);

    return (
        <div className="app-content">
            <ThemeErrorBoundary>
                <Suspense fallback={<FullScreenSpinner />}>{showWorldMap ? <WorldMap /> : <SudokuGame />}</Suspense>
            </ThemeErrorBoundary>
        </div>
    );
}

export function Sudoku() {
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
}
