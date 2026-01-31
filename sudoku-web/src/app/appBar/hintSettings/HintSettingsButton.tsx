import MyIconButton from "../../components/MyIconButton";
import { MyDialog } from "../../components/MyDialog";
import { IconBulb, IconSettings } from "@tabler/icons-react";
import { HintSettingsDialog } from "./HintSettingsDialog";
import { useState } from "react";

export function HintSettingsButton() {
    const [isSolverConfigDialogOpen, setIsSolverConfigDialogOpen] = useState(false);

    return (
        <>
            <MyIconButton
                label="Hint settings"
                icon={IconBulb}
                size="lg"
                onClick={() => setIsSolverConfigDialogOpen(true)}
                badge={<IconSettings size={10} />}
            />
            <MyDialog open={isSolverConfigDialogOpen} onClose={() => setIsSolverConfigDialogOpen(false)}>
                {(onClose) => <HintSettingsDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
