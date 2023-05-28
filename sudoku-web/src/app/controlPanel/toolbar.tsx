import type * as React from "react";
import CreateIcon from "@mui/icons-material/Create";
import GestureIcon from "@mui/icons-material/Gesture";
import UndoIcon from "@mui/icons-material/Undo";
import Tooltip from "@mui/material/Tooltip";
import { ToolbarMenu } from "./toolbarMenu";
import { ToggleButton } from "@mui/material";
import { inputCandidateModeState, inputStickyModeState } from "../state/input";
import { useRecoilValue } from "recoil";
import { useUndo } from "../actions/sudokuActions";
import { sudokuCanUndoState } from "../state/sudoku";
import MyIconButton from "../components/MyIconButton";
import { RequestHintButton } from "./RequestHintButton";
import { useToggleCandidateMode, useToggleStickyMode } from "../actions/inputActions";

export const Toolbar = () => {
    const inputCandidateMode = useRecoilValue(inputCandidateModeState);
    const inputStickyMode = useRecoilValue(inputStickyModeState);
    const canUndo = useRecoilValue(sudokuCanUndoState);

    const toggleCandidateMode = useToggleCandidateMode();
    const toggleStickyMode = useToggleStickyMode();

    const undo = useUndo();

    return (
        <div className="toolbar">
            <Tooltip title="Toggle candidate mode [space bar]">
                <ToggleButton
                    value="candidateMode"
                    selected={inputCandidateMode}
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
                    selected={inputStickyMode}
                    onChange={() => toggleStickyMode()}
                    color="primary"
                    size="large"
                >
                    <GestureIcon fontSize="large" />
                </ToggleButton>
            </Tooltip>
            <MyIconButton
                label="Undo [backspace]"
                icon={UndoIcon}
                size="large"
                disabled={!canUndo}
                onClick={async () => {
                    await undo();
                }}
            />
            <RequestHintButton />
            <ToolbarMenu />
        </div>
    );
};
