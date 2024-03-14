import type * as CSS from "csstype";
import { useResizeDetector } from "react-resize-detector";
import type { OnRefChangeType } from "react-resize-detector/build/types/types";
import { useRecoilValue } from "recoil";
import SudokuAppBar from "./appBar/sudokuAppBar";
import { ControlPanel } from "./controlPanel/controlPanel";
import { Grid } from "./grid/grid";
import { sudokuBaseState, sudokuSideLengthState } from "./state/sudoku";
import { SudokuEffects } from "./sudokuEffects";
import { useKeyboardInput } from "./useKeyboardInput";
import { gameModeState } from "./state/world";
import { WorldMap } from "./components/world/WorldMap";

interface SudokuContentProps {
    gridRef: OnRefChangeType<HTMLDivElement>;
}

const SudokuContent = ({ gridRef }: SudokuContentProps) => {
    const gameMode = useRecoilValue(gameModeState);
    return (
        <div className="app-content">
            {gameMode.mode === "world" && gameMode.view === "map" ? (
                <WorldMap />
            ) : (
                <div className="sudoku">
                    <Grid gridRef={gridRef} />
                    <ControlPanel />
                </div>
            )}
        </div>
    );
};

export const Sudoku = () => {
    const base = useRecoilValue(sudokuBaseState);
    const sideLength = useRecoilValue(sudokuSideLengthState);

    // Responsive Grid
    const { width: gridWidth, height: gridHeight, ref: gridRef } = useResizeDetector<HTMLDivElement>({});

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
