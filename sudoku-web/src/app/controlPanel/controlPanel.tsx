import * as React from "react";
import { Selector } from "./selector";
import { Toolbar } from "./toolbar";
import { Input, WasmSudokuController } from "../wasmSudokuController";
import { TransportSudoku } from "../../types";

interface ControlPanelProps {
    sudokuController: WasmSudokuController;
    input: Input;
    sideLength: TransportSudoku["sideLength"];
}

export const ControlPanel: React.FunctionComponent<ControlPanelProps> = props => {
    const { sudokuController, sideLength, input } = props;
    return (
        <>
            <Toolbar sudokuController={sudokuController} input={input} />
            <Selector sudokuController={sudokuController} input={input} sideLength={sideLength} />
        </>
    );
};
