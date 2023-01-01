import type * as React from "react";
import Dialog from "@mui/material/Dialog";
import { NewGameTabs } from "./newGameTabs";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useTheme } from "@mui/material/styles";

interface NewGameDialogProps {
    open: boolean;
    onClose: () => void;
}

export const NewGameDialog = ({ onClose, open }: NewGameDialogProps) => {
    const theme = useTheme();
    const fullScreen = useMediaQuery(theme.breakpoints.down("md"));

    return (
        <div
            onKeyDown={e => {
                // Disable global game shortcuts in dialog boxes.
                e.stopPropagation();
            }}
        >
            <Dialog open={open} onClose={onClose} fullWidth fullScreen={fullScreen}>
                <NewGameTabs onClose={onClose} />
            </Dialog>
        </div>
    );
};
