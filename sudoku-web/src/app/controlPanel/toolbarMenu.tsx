import DeleteIcon from "@mui/icons-material/Delete";
import InfoIcon from "@mui/icons-material/Info";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import RedoIcon from "@mui/icons-material/Redo";
import { useRecoilValue } from "recoil";
import { useDeleteSelectedCell, useRedo, useSetAllDirectCandidates } from "../actions/sudokuActions";
import MyIconButton from "../components/MyIconButton";
import { MyMenu } from "../components/MyMenu";
import { inputStickyModeState } from "../state/input";
import { sudokuCanRedoState } from "../state/sudoku";

export function ToolbarMenu() {
    const deleteSelectedCell = useDeleteSelectedCell();
    const setAllDirectCandidates = useSetAllDirectCandidates();
    const inputStickyMode = useRecoilValue(inputStickyModeState);
    const canRedo = useRecoilValue(sudokuCanRedoState);
    const redo = useRedo();

    return (
        <MyMenu
            menuItems={[
                {
                    label: "Redo",
                    icon: <RedoIcon />,
                    disabled: !canRedo,
                    onClick: async () => {
                        await redo();
                    },
                },
                {
                    // TODO: show KB shortcut [Delete]
                    label: "Delete selected cell",
                    icon: <DeleteIcon />,
                    disabled: inputStickyMode,
                    onClick: async () => {
                        await deleteSelectedCell();
                    },
                },
                {
                    // TODO: show KB shortcut [Insert]
                    label: "Fill candidates",
                    icon: <InfoIcon />,
                    onClick: async () => await setAllDirectCandidates(),
                },
            ]}
        >
            {({ onMenuOpen }) => (
                <MyIconButton label="Other actions" icon={MoreVertIcon} size="large" onClick={onMenuOpen} />
            )}
        </MyMenu>
    );
}
