import AppBar from "@mui/material/AppBar";
import Box from "@mui/material/Box";
import Toolbar from "@mui/material/Toolbar";
import Typography from "@mui/material/Typography";
import { HintSettingsButton } from "./hintSettings/HintSettingsButton";
import { NewGameButton } from "./newGame/NewGameButton";
import { ShareMenu } from "./share/ShareMenu";
import { WorldSettingsButton } from "./world/WorldSettingsButton";

export default function SudokuAppBar() {
    return (
        <Box sx={{ flexGrow: 1 }} className="app-bar">
            <AppBar position="static" variant="outlined" color="default">
                <Toolbar>
                    <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                        Sudoku
                    </Typography>
                    <WorldSettingsButton />
                    <ShareMenu />
                    <HintSettingsButton />
                    <NewGameButton />
                </Toolbar>
            </AppBar>
        </Box>
    );
}
