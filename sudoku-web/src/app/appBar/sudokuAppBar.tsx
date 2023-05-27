import Box from "@mui/material/Box";
import React from "react";
import AppBar from "@mui/material/AppBar";
import Typography from "@mui/material/Typography";
import Toolbar from "@mui/material/Toolbar";
import { NewGameButton } from "./newGame/NewGameButton";
import { SolverMenu } from "./solverMenu";
import { ShareMenu } from "./share/ShareMenu";
import { HintSettingsButton } from "./hintSettings/HintSettingsButton";

export default function SudokuAppBar() {
    return (
        <Box sx={{ flexGrow: 1 }} className="app-bar">
            <AppBar position="static">
                <Toolbar>
                    <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                        Sudoku
                    </Typography>
                    <ShareMenu />
                    <SolverMenu />
                    <HintSettingsButton />
                    <NewGameButton />
                </Toolbar>
            </AppBar>
        </Box>
    );
}
