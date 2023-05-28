import React from "react";
import MyIconButton from "../../components/MyIconButton";
import { MyDialog } from "../../components/MyDialog";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import { HintSettingsDialog } from "./HintSettingsDialog";
import SettingsIcon from "@mui/icons-material/Settings";
export function HintSettingsButton() {
    const [isSolverConfigDialogOpen, setIsSolverConfigDialogOpen] = React.useState(false);

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
                {onClose => <HintSettingsDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
