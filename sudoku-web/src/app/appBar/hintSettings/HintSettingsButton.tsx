import MyIconButton from "../../components/MyIconButton";
import { MyDialog } from "../../components/MyDialog";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import { HintSettingsDialog } from "./HintSettingsDialog";
import SettingsIcon from "@mui/icons-material/Settings";
import { useState } from "react";

export function HintSettingsButton() {
    const [isSolverConfigDialogOpen, setIsSolverConfigDialogOpen] = useState(false);

    return (
        <>
            <MyIconButton
                label="Hint settings"
                icon={LightbulbIcon}
                size="large"
                color="inherit"
                onClick={() => setIsSolverConfigDialogOpen(true)}
                badge={<SettingsIcon fontSize="inherit" />}
            />
            <MyDialog open={isSolverConfigDialogOpen} onClose={() => setIsSolverConfigDialogOpen(false)}>
                {(onClose) => <HintSettingsDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
