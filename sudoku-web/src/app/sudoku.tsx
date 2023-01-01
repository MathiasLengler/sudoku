import type React from "react";
import type * as CSS from "csstype";
import { useKeyboardInput } from "./useKeyboardInput";
import { Grid } from "./grid/grid";
import { ControlPanel } from "./controlPanel/controlPanel";
import { useResizeDetector } from "react-resize-detector";
import SudokuAppBar from "./appBar/sudokuAppBar";
import { useRecoilValue } from "recoil";
import { sudokuBaseState, sudokuSideLengthState } from "./state/sudoku";
import { SudokuEffects } from "./sudokuEffects";

interface SudokuContentProps {
    gridRef: React.MutableRefObject<HTMLDivElement>;
}

const SudokuContent = ({ gridRef }: SudokuContentProps) => {
    return (
        <div className="app-content">
            <div className="sudoku">
                <Grid gridRef={gridRef} />
                <ControlPanel />
            </div>
        </div>
    );
};

export const Sudoku = () => {
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);

    // Responsive Grid
    const { width: gridWidth, height: gridHeight, ref: gridRef } = useResizeDetector({});

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--side-length-fr": `${sideLength}fr`,
        "--base": base,
        "--grid-size": gridWidth && gridHeight ? `${Math.min(gridWidth, gridHeight)}px` : "0",
    };

    const { onKeyDown } = useKeyboardInput();

    return (
        <div
            className="app"
            style={cssVariables}
            onKeyDown={onKeyDown}
            // Enable keyboard events
            tabIndex={0}
        >
            <SudokuAppBar />
            <SudokuContent gridRef={gridRef} />
            <SudokuEffects />
        </div>
    );
};
