import type * as React from "react";
import Dialog from "@mui/material/Dialog";
import { NewGameTabs } from "./newGameTabs";
import useMediaQuery from "@mui/material/useMediaQuery";
import { useTheme } from "@mui/material/styles";
import type { WasmSudokuController } from "../../wasmSudokuController";

interface NewGameDialogProps {
    open: boolean;
    sudokuController: WasmSudokuController;
    onClose: () => void;
}

export const NewGameDialog: React.FunctionComponent<NewGameDialogProps> = props => {
    const { open, onClose, sudokuController } = props;

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
                <NewGameTabs sudokuController={sudokuController} onClose={onClose} />
            </Dialog>
        </div>
    );
};
