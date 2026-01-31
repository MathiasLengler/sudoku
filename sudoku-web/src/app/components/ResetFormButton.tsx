import { IconRefresh } from "@tabler/icons-react";
import MyIconButton from "./MyIconButton";

type ResetFormButtonProps = {
    disabled?: boolean;
    onClick: () => void;
};

export function ResetFormButton({ disabled, onClick }: ResetFormButtonProps) {
    return <MyIconButton icon={IconRefresh} label="Reset to default" disabled={disabled} onClick={() => onClick()} />;
}
