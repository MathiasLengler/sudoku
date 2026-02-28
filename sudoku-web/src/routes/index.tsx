import type * as CSS from "csstype";
import { createFileRoute } from "@tanstack/react-router";
import { useAtomValue } from "jotai";
import { useResizeDetector } from "react-resize-detector";
import { Toolbar } from "../app/controlPanel/toolbar";
import { ValueSelector } from "../app/controlPanel/valueSelector";
import { Grid } from "../app/grid/grid";
import { sudokuBaseState, sudokuSideLengthState } from "../app/state/sudoku";

export const Route = createFileRoute("/")({
    component: SudokuGame,
});

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
        <div className="sudoku-game" style={cssVariables}>
            <Grid gridRef={gridRef} />
            <Toolbar />
            <ValueSelector />
        </div>
    );
}
