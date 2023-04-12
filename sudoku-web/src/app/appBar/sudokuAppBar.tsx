import Box from "@mui/material/Box";
import React from "react";
import AppBar from "@mui/material/AppBar";
import Typography from "@mui/material/Typography";
import Toolbar from "@mui/material/Toolbar";
import { NewGameButton } from "./newGame/newGameButton";
import { SolverMenu } from "./solverMenu";
import { ShareMenu } from "./share/shareMenu";

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
                    <NewGameButton />
                </Toolbar>
            </AppBar>
        </Box>
    );
}
