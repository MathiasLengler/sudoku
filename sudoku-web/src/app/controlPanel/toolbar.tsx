import CreateIcon from "@mui/icons-material/Create";
import GestureIcon from "@mui/icons-material/Gesture";
import UndoIcon from "@mui/icons-material/Undo";
import { ToggleButton } from "@mui/material";
import Tooltip from "@mui/material/Tooltip";
import { useAtomValue } from "jotai";
import { useToggleCandidateMode, useToggleStickyMode } from "../actions/inputActions";
import { useUndo } from "../actions/sudokuActions";
import MyIconButton from "../components/MyIconButton";
import { inputCandidateModeState, inputStickyModeState } from "../state/input";
import { sudokuCanUndoState } from "../state/sudoku";
import { RequestHintButton } from "./RequestHintButton";
import { ToolbarMenu } from "./toolbarMenu";

export function Toolbar() {
    const inputCandidateMode = useAtomValue(inputCandidateModeState);
    const inputStickyMode = useAtomValue(inputStickyModeState);
    const canUndo = useAtomValue(sudokuCanUndoState);

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
}
