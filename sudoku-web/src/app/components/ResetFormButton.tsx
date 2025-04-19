import ReplayIcon from "@mui/icons-material/Replay";
import MyIconButton from "./MyIconButton";

type ResetFormButtonProps = {
    disabled?: boolean;
    onClick: () => void;
};

export function ResetFormButton({ disabled, onClick }: ResetFormButtonProps) {
    return <MyIconButton icon={ReplayIcon} label="Reset to default" disabled={disabled} onClick={() => onClick()} />;
}
