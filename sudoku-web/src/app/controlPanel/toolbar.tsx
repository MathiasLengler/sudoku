import type * as React from "react";
import IconButton from "@mui/material/IconButton";
import CreateIcon from "@mui/icons-material/Create";
import GestureIcon from "@mui/icons-material/Gesture";
import UndoIcon from "@mui/icons-material/Undo";
import type { Input, WasmSudokuController } from "../wasmSudokuController";
import Tooltip from "@mui/material/Tooltip";
import { ToolbarMenu } from "./toolbarMenu";
import RedoIcon from "@mui/icons-material/Redo";
import { ToggleButton } from "@mui/material";

interface ToolbarProps {
    sudokuController: WasmSudokuController;
    input: Input;
}

export const Toolbar: React.FunctionComponent<ToolbarProps> = props => {
    const {
        sudokuController,
        input: { candidateMode, stickyMode },
    } = props;

    const enterDelay = 500;
    const leaveDelay = 200;

    return (
        <div className="toolbar">
            <Tooltip title="Toggle candidate mode [space bar]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <ToggleButton
                    value="candidateMode"
                    selected={candidateMode}
                    onChange={() => sudokuController.toggleCandidateMode()}
                    color="primary"
                    size="large"
                >
                    <CreateIcon fontSize="large" />
                </ToggleButton>
            </Tooltip>
            <Tooltip title="Toggle sticky mode [+]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <ToggleButton
                    value="stickyMode"
                    selected={stickyMode}
                    onChange={() => sudokuController.toggleStickyMode()}
                    color="primary"
                    size="large"
                >
                    <GestureIcon fontSize="large" />
                </ToggleButton>
            </Tooltip>
            <Tooltip title="Undo [backspace]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                {/* TODO: disable if can't undo */}
                <IconButton onClick={() => sudokuController.undo()} size="large">
                    <UndoIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Redo [shift+backspace]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                {/* TODO: disable if can't redo */}
                <IconButton onClick={() => console.warn("TODO: expose redo")} size="large">
                    <RedoIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <ToolbarMenu enterDelay={enterDelay} leaveDelay={leaveDelay} sudokuController={sudokuController} />
        </div>
    );
};
