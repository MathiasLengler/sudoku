import type * as React from "react";
import { Selector } from "./selector";
import { Toolbar } from "./toolbar";
import type { Input, WasmSudokuController } from "../wasmSudokuController";
import type { TransportSudoku } from "../../types";

interface ControlPanelProps {
    sudokuController: WasmSudokuController;
    input: Input;
    sideLength: TransportSudoku["sideLength"];
    canUndo: boolean;
    canRedo: boolean;
}

export const ControlPanel = ({ input, sideLength, sudokuController, canUndo, canRedo }: ControlPanelProps) => {
    return (
        <>
            <Toolbar sudokuController={sudokuController} input={input} canUndo={canUndo} canRedo={canRedo} />
            <Selector sudokuController={sudokuController} input={input} sideLength={sideLength} />
        </>
    );
};
