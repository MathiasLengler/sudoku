import SettingsIcon from "@mui/icons-material/Settings";
import { useState } from "react";
import MyIconButton from "../../components/MyIconButton";
import { MyDialog } from "../../components/MyDialog";
import { SettingsDialog } from "./SettingsDialog";

export function SettingsButton() {
    const [isSettingsDialogOpen, setIsSettingsDialogOpen] = useState(false);

    return (
        <>
            <MyIconButton
                label="Settings"
                icon={SettingsIcon}
                size="large"
                color="inherit"
                onClick={() => setIsSettingsDialogOpen(true)}
            />
            <MyDialog open={isSettingsDialogOpen} onClose={() => setIsSettingsDialogOpen(false)}>
                {(onClose) => <SettingsDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
