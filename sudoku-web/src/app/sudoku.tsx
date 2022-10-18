import React, { useEffect, useMemo, useState } from "react";
import { Input, WasmSudokuController } from "./wasmSudokuController";
import { blocksToCell } from "./utils";
import * as CSS from "csstype";
import { makeKeyDownListener } from "./useKeyboardInput";
import { Grid } from "./grid/grid";
import { ControlPanel } from "./controlPanel/controlPanel";
import * as Comlink from "comlink";
import { TransportSudoku, WasmSudoku } from "../types";
import { saveCellBlocks } from "./persistence";
import debounce from "lodash/debounce";
import { useResizeDetector } from "react-resize-detector";
import SudokuAppBar from "./menu/sudokuAppBar";

interface SudokuProps {
    sudoku: TransportSudoku;
    setSudoku: React.Dispatch<TransportSudoku>;
    wasmSudokuProxy: Comlink.Remote<WasmSudoku>;
}

export const Sudoku: React.FunctionComponent<SudokuProps> = ({ sudoku, setSudoku, wasmSudokuProxy }) => {
    const { blocks, base, sideLength } = sudoku;

    const debouncedSaveCells = useMemo(
        () =>
            debounce((blocks: TransportSudoku["blocks"]) => {
                console.debug("Saving cell blocks to localStorage");
                const cellBlocks = blocks.map(block => block.map(({ position, incorrectValue, ...cell }) => cell));
                saveCellBlocks(cellBlocks);
            }, 500),
        []
    );

    useEffect(() => {
        debouncedSaveCells(blocks);
    }, [debouncedSaveCells, blocks]);

    const [inputWithoutSelectedCell, setInput] = useState<Omit<Input, "selectedCell">>(() => {
        const selectedPos = { column: 0, row: 0 };

        return {
            selectedPos,
            selectedValue: 1,
            stickyMode: false,
            candidateMode: false,
        };
    });

    const selectedCell = blocksToCell(blocks, inputWithoutSelectedCell.selectedPos, base);

    const input = { ...inputWithoutSelectedCell, selectedCell };

    const sudokuController = new WasmSudokuController(
        wasmSudokuProxy,
        sudoku,
        sudoku => setSudoku(sudoku),
        input,
        setInput,
        sideLength
    );

    // Responsive Grid
    const { width: gridWidth, height: gridHeight, ref: gridRef } = useResizeDetector({});

    console.log({ gridWidth, gridHeight });

    const cssVariables: CSS.Properties = {
        "--side-length": sideLength,
        "--side-length-fr": `${sideLength}fr`,
        "--base": base,
        "--grid-size": `${gridHeight}px`,
    };

    return (
        <div
            className="sudoku"
            style={cssVariables}
            onKeyDown={makeKeyDownListener(sudokuController, input, sideLength)}
            tabIndex={0}
        >
            <SudokuAppBar />
            <Grid sudokuController={sudokuController} input={input} sudoku={sudoku} gridRef={gridRef} />
            <ControlPanel sudokuController={sudokuController} input={input} sideLength={sideLength} />
        </div>
    );
};
