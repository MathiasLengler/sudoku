import { IconCirclePlus } from "@tabler/icons-react";
import { MyDialog } from "../../components/MyDialog";
import { NewGameDialog } from "./NewGameDialog";
import MyIconButton from "../../components/MyIconButton";
import { useState } from "react";

export function NewGameButton() {
    const [isNewGameDialogOpen, setIsNewGameDialogOpen] = useState(false);

    return (
        <>
            <MyIconButton
                icon={IconCirclePlus}
                size="lg"
                label="New game"
                onClick={() => setIsNewGameDialogOpen(true)}
            />
            <MyDialog open={isNewGameDialogOpen} onClose={() => setIsNewGameDialogOpen(false)}>
                {(onClose) => <NewGameDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
