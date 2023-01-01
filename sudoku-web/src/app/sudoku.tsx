import React, { useEffect, useMemo } from "react";
import type * as CSS from "csstype";
import { useKeyboardInput } from "./useKeyboardInput";
import { Grid } from "./grid/grid";
import { ControlPanel } from "./controlPanel/controlPanel";
import type { CellViews, TransportSudoku } from "../types";
import { saveCellViews } from "./persistence";
import debounce from "lodash/debounce";
import { useResizeDetector } from "react-resize-detector";
import SudokuAppBar from "./menu/sudokuAppBar";
import { useRecoilValue } from "recoil";
import { sudokuState } from "./state/sudoku";

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
    const sudoku = useRecoilValue(sudokuState);

    const { cells, base, sideLength, blocksIndices, isSolved } = sudoku;

    // TODO: refactor using recoil
    //  use Recoil Sync?
    //  replace with atom effect?
    //  move inside side effect component?
    const debouncedSaveCells = useMemo(
        () =>
            debounce((cells: TransportSudoku["cells"]) => {
                console.debug("Saving sudoku cells to localStorage");
                const cellViews: CellViews = cells.map(({ position, incorrectValue, ...cell }) => cell);
                saveCellViews(cellViews);
            }, 500),
        []
    );

    useEffect(() => {
        debouncedSaveCells(cells);
    }, [debouncedSaveCells, cells]);

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
        </div>
    );
};
