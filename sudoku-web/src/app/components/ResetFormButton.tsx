import ReplayIcon from "@mui/icons-material/Replay";
import MyIconButton from "./MyIconButton";
import React from "react";

interface ResetFormButtonProps {
    disabled?: boolean;
    onClick: () => void;
}

export function ResetFormButton({ disabled, onClick }: ResetFormButtonProps) {
    return <MyIconButton icon={ReplayIcon} tooltip="Reset to default" disabled={disabled} onClick={() => onClick()} />;
}
