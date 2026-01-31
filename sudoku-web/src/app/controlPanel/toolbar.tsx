import { IconPencil, IconScribble, IconArrowBackUp } from "@tabler/icons-react";
import { ActionIcon, Tooltip } from "@mantine/core";
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
            <Tooltip label="Toggle candidate mode [space bar]">
                <ActionIcon
                    variant={inputCandidateMode ? "filled" : "subtle"}
                    color="blue"
                    size="lg"
                    onClick={() => toggleCandidateMode()}
                    aria-label="Toggle candidate mode"
                >
                    <IconPencil size={26} />
                </ActionIcon>
            </Tooltip>
            <Tooltip label="Toggle sticky mode [+]">
                <ActionIcon
                    variant={inputStickyMode ? "filled" : "subtle"}
                    color="blue"
                    size="lg"
                    onClick={() => toggleStickyMode()}
                    aria-label="Toggle sticky mode"
                >
                    <IconScribble size={26} />
                </ActionIcon>
            </Tooltip>
            <MyIconButton
                label="Undo [backspace]"
                icon={IconArrowBackUp}
                size="lg"
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
