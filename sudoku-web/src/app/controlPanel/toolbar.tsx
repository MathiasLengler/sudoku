import CreateIcon from "@mui/icons-material/Create";
import GestureIcon from "@mui/icons-material/Gesture";
import PaletteIcon from "@mui/icons-material/Palette";
import UndoIcon from "@mui/icons-material/Undo";
import { ToggleButton } from "@mui/material";
import Tooltip from "@mui/material/Tooltip";
import { useAtomValue } from "jotai";
import { useToggleCandidateMode, useToggleColorMode, useToggleStickyMode } from "../actions/inputActions";
import { useUndo } from "../actions/sudokuActions";
import MyIconButton from "../components/MyIconButton";
import { inputCandidateModeState, inputColorModeState, inputStickyModeState } from "../state/input";
import { sudokuCanUndoState } from "../state/sudoku";
import { RequestHintButton } from "./RequestHintButton";
import { ToolbarMenu } from "./toolbarMenu";

export function Toolbar() {
    const inputCandidateMode = useAtomValue(inputCandidateModeState);
    const inputStickyMode = useAtomValue(inputStickyModeState);
    const inputColorMode = useAtomValue(inputColorModeState);
    const canUndo = useAtomValue(sudokuCanUndoState);

    const toggleCandidateMode = useToggleCandidateMode();
    const toggleStickyMode = useToggleStickyMode();
    const toggleColorMode = useToggleColorMode();

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
            <Tooltip title="Toggle color mode [c]">
                <ToggleButton
                    value="colorMode"
                    selected={inputColorMode}
                    onChange={() => toggleColorMode()}
                    color="primary"
                    size="large"
                >
                    <PaletteIcon fontSize="large" />
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
