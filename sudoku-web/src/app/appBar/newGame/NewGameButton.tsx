import React from "react";
import AddCircleIcon from "@mui/icons-material/AddCircle";
import { MyDialog } from "../../components/MyDialog";
import { NewGameDialog } from "./NewGameDialog";
import MyIconButton from "../../components/MyIconButton";

export function NewGameButton() {
    const [isNewGameDialogOpen, setIsNewGameDialogOpen] = React.useState(false);

    return (
        <>
            <MyIconButton
                icon={AddCircleIcon}
                size="large"
                label="New game"
                color="inherit"
                onClick={() => setIsNewGameDialogOpen(true)}
            />
            <MyDialog open={isNewGameDialogOpen} onClose={() => setIsNewGameDialogOpen(false)}>
                {onClose => <NewGameDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
