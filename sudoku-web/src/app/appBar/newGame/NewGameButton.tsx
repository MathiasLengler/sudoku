import AddCircleIcon from "@mui/icons-material/AddCircle";
import { useState } from "react";

import { MyDialog } from "../../components/MyDialog";
import MyIconButton from "../../components/MyIconButton";
import { NewGameDialog } from "./NewGameDialog";

export function NewGameButton() {
    const [isNewGameDialogOpen, setIsNewGameDialogOpen] = useState(false);

    return (
        <>
            <MyIconButton
                icon={AddCircleIcon}
                size="large"
                label="New game"
                color="inherit"
                onClick={() => {
                    setIsNewGameDialogOpen(true);
                }}
            />
            <MyDialog
                open={isNewGameDialogOpen}
                onClose={() => {
                    setIsNewGameDialogOpen(false);
                }}
            >
                {(onClose) => <NewGameDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
