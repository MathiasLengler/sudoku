import React from "react";
import MyIconButton from "../../components/MyIconButton";
import { MyDialog } from "../../components/MyDialog";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import { HintSettingsDialog } from "./HintSettingsDialog";

export function HintSettingsButton() {
    const [isSolverConfigDialogOpen, setIsSolverConfigDialogOpen] = React.useState(false);

    return (
        <>
            <MyIconButton
                tooltip="Configure Solver"
                icon={LightbulbIcon}
                size="large"
                color="inherit"
                onClick={() => setIsSolverConfigDialogOpen(true)}
            />
            <MyDialog open={isSolverConfigDialogOpen} onClose={() => setIsSolverConfigDialogOpen(false)}>
                {onClose => <HintSettingsDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
