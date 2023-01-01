import * as React from "react";
import IconButton from "@mui/material/IconButton";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import Tooltip from "@mui/material/Tooltip";
import { CustomMenu } from "../menu/customMenu";
import InfoIcon from "@mui/icons-material/Info";
import DeleteIcon from "@mui/icons-material/Delete";
import { useDeleteSelectedCell, useSetAllDirectCandidates } from "../sudokuActions";

export const ToolbarMenu = () => {
    const deleteSelectedCell = useDeleteSelectedCell();
    const setAllDirectCandidates = useSetAllDirectCandidates();

    return (
        <CustomMenu
            menuItems={[
                {
                    // TODO: show KB shortcut [Delete]
                    // TODO: disable in sticky mode
                    label: "Delete selected cell",
                    icon: <DeleteIcon />,
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
