import AddCircleIcon from "@mui/icons-material/AddCircle";
import IconButton from "@mui/material/IconButton";
import Tooltip from "@mui/material/Tooltip";
import { Link } from "@tanstack/react-router";

export function NewGameButton() {
    return (
        <Tooltip title="New game">
            <IconButton
                component={Link}
                to="/new-game"
                size="large"
                color="inherit"
                aria-label="New game"
            >
                <AddCircleIcon fontSize="large" />
            </IconButton>
        </Tooltip>
    );
}
