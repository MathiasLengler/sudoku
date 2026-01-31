import ExtensionIcon from "@mui/icons-material/Extension";
import { useState } from "react";
import MyIconButton from "../../components/MyIconButton";
import { MyDialog } from "../../components/MyDialog";
import { PuzzleDialog } from "./PuzzleDialog";

export function PuzzleButton() {
    const [isPuzzleDialogOpen, setIsPuzzleDialogOpen] = useState(false);

    return (
        <>
            <MyIconButton
                icon={ExtensionIcon}
                size="large"
                label="Challenge Mode"
                color="inherit"
                onClick={() => setIsPuzzleDialogOpen(true)}
            />
            <MyDialog open={isPuzzleDialogOpen} onClose={() => setIsPuzzleDialogOpen(false)}>
                {(onClose) => <PuzzleDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
