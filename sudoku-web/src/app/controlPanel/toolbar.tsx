import type * as React from "react";
import IconButton from "@mui/material/IconButton";
import CreateIcon from "@mui/icons-material/Create";
import GestureIcon from "@mui/icons-material/Gesture";
import UndoIcon from "@mui/icons-material/Undo";
import Tooltip from "@mui/material/Tooltip";
import { ToolbarMenu } from "./toolbarMenu";
import RedoIcon from "@mui/icons-material/Redo";
import { ToggleButton } from "@mui/material";
import { inputState } from "../state/input";
import { useRecoilValue } from "recoil";
import { useRedo, useToggleCandidateMode, useToggleStickyMode, useUndo } from "../sudokuActions";
import { sudokuCanRedoState, sudokuCanUndoState } from "../state/sudoku";

export const Toolbar = () => {
    const input = useRecoilValue(inputState);
    const canUndo = useRecoilValue(sudokuCanUndoState);
    const canRedo = useRecoilValue(sudokuCanRedoState);

    const toggleCandidateMode = useToggleCandidateMode();
    const toggleStickyMode = useToggleStickyMode();

    const undo = useUndo();
    const redo = useRedo();

    return (
        <div className="toolbar">
            <Tooltip title="Toggle candidate mode [space bar]">
                <ToggleButton
                    value="candidateMode"
                    selected={input.candidateMode}
                    onChange={() => toggleCandidateMode()}
                    color="primary"
                    size="large"
                >
                    <CreateIcon fontSize="large" />
                </ToggleButton>
            </Tooltip>
            <Tooltip title="Toggle sticky mode [+]">
                <ToggleButton
                    value="stickyMode"
                    selected={input.stickyMode}
                    onChange={() => toggleStickyMode()}
                    color="primary"
                    size="large"
                >
                    <GestureIcon fontSize="large" />
                </ToggleButton>
            </Tooltip>
            <Tooltip title="Undo [backspace]">
                <span>
                    <IconButton
                        onClick={async () => {
                            await undo();
                        }}
                        size="large"
                        disabled={!canUndo}
                    >
                        <UndoIcon fontSize="large" />
                    </IconButton>
                </span>
            </Tooltip>
            <Tooltip title="Redo [shift+backspace]">
                <span>
                    <IconButton
                        onClick={async () => {
                            await redo();
                        }}
                        disabled={!canRedo}
                        size="large"
                    >
                        <RedoIcon fontSize="large" />
                    </IconButton>
                </span>
            </Tooltip>
            <ToolbarMenu />
        </div>
    );
};
