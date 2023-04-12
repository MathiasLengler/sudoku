import * as React from "react";
import IconButton from "@mui/material/IconButton";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import Tooltip from "@mui/material/Tooltip";
import { CustomMenu } from "../components/CustomMenu";
import InfoIcon from "@mui/icons-material/Info";
import DeleteIcon from "@mui/icons-material/Delete";
import { useDeleteSelectedCell, useSetAllDirectCandidates } from "../sudokuActions";
import { inputStickyModeState } from "../state/input";
import { useRecoilValue } from "recoil";

export const ToolbarMenu = () => {
    const deleteSelectedCell = useDeleteSelectedCell();
    const setAllDirectCandidates = useSetAllDirectCandidates();
    const inputStickyMode = useRecoilValue(inputStickyModeState);

    return (
        <CustomMenu
            menuItems={[
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
                <Tooltip title="Other actions">
                    <IconButton onClick={onMenuOpen} size="large">
                        <MoreVertIcon fontSize="large" />
                    </IconButton>
                </Tooltip>
            )}
        </CustomMenu>
    );
};
