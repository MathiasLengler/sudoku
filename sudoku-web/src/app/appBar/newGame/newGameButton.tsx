import React from "react";
import AddCircleIcon from "@mui/icons-material/AddCircle";
import { MyDialog } from "../../components/MyDialog";
import { NewGameTabs } from "./newGameTabs";
import MyIconButton from "../../components/MyIconButton";

export function NewGameButton() {
    const [isNewGameDialogOpen, setIsNewGameDialogOpen] = React.useState(false);

    return (
        <>
            <MyIconButton
                icon={AddCircleIcon}
                size="large"
                tooltip="Create new game"
                color="inherit"
                onClick={() => setIsNewGameDialogOpen(true)}
            />
            <MyDialog open={isNewGameDialogOpen} onClose={() => setIsNewGameDialogOpen(false)}>
                {onClose => <NewGameTabs onClose={onClose} />}
            </MyDialog>
        </>
    );
}
