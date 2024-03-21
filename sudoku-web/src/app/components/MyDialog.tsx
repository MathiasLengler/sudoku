import type * as React from "react";
import type { ReactNode } from "react";
import Dialog from "@mui/material/Dialog";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useTheme } from "@mui/material/styles";

interface MyDialogProps {
    open: boolean;
    onClose: () => void;
    children: (onClose: () => void) => ReactNode;
}

export const MyDialog = ({ open, onClose, children }: MyDialogProps) => {
    const theme = useTheme();
    const fullScreen = useMediaQuery(theme.breakpoints.down("md"));

    return (
        <div
            onKeyDown={(e) => {
                // Disable global game shortcuts in dialog boxes.
                e.stopPropagation();
            }}
        >
            <Dialog open={open} onClose={onClose} fullWidth fullScreen={fullScreen} scroll="paper">
                {children(onClose)}
            </Dialog>
        </div>
    );
};
