import React from "react";
import { IconButton } from "@mui/material";
import AddCircleIcon from "@mui/icons-material/AddCircle";
import { NewGameDialog } from "../controlPanel/newGame/newGameDialog";

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
            <NewGameDialog open={isNewGameDialogOpen} onClose={() => setIsNewGameDialogOpen(false)} />
        </>
    );
}
