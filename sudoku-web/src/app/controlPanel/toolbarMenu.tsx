import * as React from "react";
import IconButton from "@mui/material/IconButton";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import Tooltip from "@mui/material/Tooltip";
import type { WasmSudokuController } from "../wasmSudokuController";
import { CustomMenu } from "../menu/customMenu";
import InfoIcon from "@mui/icons-material/Info";
import DeleteIcon from "@mui/icons-material/Delete";

interface ToolbarMenuProps {
    enterDelay: number;
    leaveDelay: number;
    sudokuController: WasmSudokuController;
}

export const ToolbarMenu = ({ enterDelay, leaveDelay, sudokuController }: ToolbarMenuProps) => {
    return (
        <CustomMenu
            menuItems={[
                {
                    // TODO: show KB shortcut [Delete]
                    // TODO: disable in sticky mode
                    label: "Delete selected cell",
                    icon: <DeleteIcon />,
                    onClick: async () => {
                        await sudokuController.delete();
                    },
                },
                {
                    // TODO: show KB shortcut [Insert]
                    label: "Fill candidates",
                    icon: <InfoIcon />,
                    onClick: async () => await sudokuController.setAllDirectCandidates(),
                },
            ]}
        >
            {({ onMenuOpen }) => (
                <Tooltip title="Other actions" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                    <IconButton onClick={onMenuOpen} size="large">
                        <MoreVertIcon fontSize="large" />
                    </IconButton>
                </Tooltip>
            )}
        </CustomMenu>
    );
};
