import { IconTrash, IconInfoCircle, IconDotsVertical, IconArrowForwardUp } from "@tabler/icons-react";
import { useAtomValue } from "jotai";
import { useDeleteSelectedCell, useRedo, useSetAllDirectCandidates } from "../actions/sudokuActions";
import MyIconButton from "../components/MyIconButton";
import { MyMenu } from "../components/MyMenu";
import { inputStickyModeState } from "../state/input";
import { sudokuCanRedoState } from "../state/sudoku";

export function ToolbarMenu() {
    const deleteSelectedCell = useDeleteSelectedCell();
    const setAllDirectCandidates = useSetAllDirectCandidates();
    const inputStickyMode = useAtomValue(inputStickyModeState);
    const canRedo = useAtomValue(sudokuCanRedoState);
    const redo = useRedo();

    return (
        <MyMenu
            menuItems={[
                {
                    label: "Redo",
                    icon: <IconArrowForwardUp size={18} />,
                    disabled: !canRedo,
                    onClick: async () => {
                        await redo();
                    },
                },
                {
                    // TODO: show KB shortcut [Delete]
                    label: "Delete selected cell",
                    icon: <IconTrash size={18} />,
                    disabled: inputStickyMode,
                    onClick: async () => {
                        await deleteSelectedCell();
                    },
                },
                {
                    // TODO: show KB shortcut [Insert]
                    label: "Fill candidates",
                    icon: <IconInfoCircle size={18} />,
                    onClick: async () => await setAllDirectCandidates(),
                },
            ]}
        >
            {({ onMenuOpen }) => (
                <MyIconButton label="Other actions" icon={IconDotsVertical} size="lg" onClick={onMenuOpen} />
            )}
        </MyMenu>
    );
}
