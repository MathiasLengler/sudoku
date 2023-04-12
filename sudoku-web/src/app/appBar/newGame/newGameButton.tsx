import React from "react";
import { IconButton } from "@mui/material";
import AddCircleIcon from "@mui/icons-material/AddCircle";
import { MyDialog } from "../../components/MyDialog";
import { NewGameTabs } from "./newGameTabs";

export function NewGameButton() {
    const [isNewGameDialogOpen, setIsNewGameDialogOpen] = React.useState(false);

    return (
        <>
            <IconButton
                color="inherit"
                size="large"
                aria-label="Create new game"
                onClick={() => setIsNewGameDialogOpen(true)}
            >
                <AddCircleIcon fontSize="large" />
            </IconButton>
            <MyDialog open={isNewGameDialogOpen} onClose={() => setIsNewGameDialogOpen(false)}>
                {onClose => <NewGameTabs onClose={onClose} />}
            </MyDialog>
        </>
    );
}
