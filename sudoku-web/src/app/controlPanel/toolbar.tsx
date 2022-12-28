import type * as React from "react";
import IconButton from "@mui/material/IconButton";
import DeleteIcon from "@mui/icons-material/Delete";
import InfoIcon from "@mui/icons-material/Info";
import CreateIcon from "@mui/icons-material/Create";
import GestureIcon from "@mui/icons-material/Gesture";
import UndoIcon from "@mui/icons-material/Undo";
import type { Input, WasmSudokuController } from "../wasmSudokuController";
import Tooltip from "@mui/material/Tooltip";
import { ToolbarMenu } from "./toolbarMenu";

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
                <IconButton
                    color={candidateMode ? "primary" : "default"}
                    onClick={() => sudokuController.toggleCandidateMode()}
                    size="large"
                >
                    <CreateIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Toggle sticky mode [+]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <IconButton
                    color={stickyMode ? "primary" : "default"}
                    onClick={() => sudokuController.toggleStickyMode()}
                    size="large"
                >
                    <GestureIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Delete selected cell [delete]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <IconButton onClick={() => sudokuController.delete()} size="large">
                    <DeleteIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Set all direct candidates [insert]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <IconButton onClick={() => sudokuController.setAllDirectCandidates()} size="large">
                    <InfoIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <Tooltip title="Undo [backspace]" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <IconButton onClick={() => sudokuController.undo()} size="large">
                    <UndoIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <ToolbarMenu enterDelay={enterDelay} leaveDelay={leaveDelay} sudokuController={sudokuController} />
        </div>
    );
};
