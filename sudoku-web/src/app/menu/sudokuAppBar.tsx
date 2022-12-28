import Box from "@mui/material/Box";
import React from "react";
import AppBar from "@mui/material/AppBar";
import Typography from "@mui/material/Typography";
import Toolbar from "@mui/material/Toolbar";
import type { WasmSudokuController } from "../wasmSudokuController";
import { ALL_STRATEGIES } from "../../constants";
import { CustomMenu } from "./customMenu";
import { NewGameDialog } from "../controlPanel/newGame/newGameDialog";
import type { DynamicStrategy } from "../../types";
import AddCircleIcon from "@mui/icons-material/AddCircle";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import { IconButton } from "@mui/material";
import LightbulbIcon from "@mui/icons-material/Lightbulb";
import ShareIcon from "@mui/icons-material/Share";

interface NewGameButtonProps {
    sudokuController: WasmSudokuController;
}

function NewGameButton({ sudokuController }: NewGameButtonProps) {
    const [isNewGameDialogOpen, setIsNewGameDialogOpen] = React.useState(false);

    return (
        <>
            <IconButton size="large" aria-label="Create new game" onClick={() => setIsNewGameDialogOpen(true)}>
                <AddCircleIcon fontSize="large" />
            </IconButton>
            <NewGameDialog
                open={isNewGameDialogOpen}
                onClose={() => setIsNewGameDialogOpen(false)}
                sudokuController={sudokuController}
            />
        </>
    );
}
interface SolverMenuProps {
    sudokuController: WasmSudokuController;
}

function SolverMenu({ sudokuController }: SolverMenuProps) {
    return (
        <CustomMenu
            menuItems={[
                ...ALL_STRATEGIES.map(strategy => ({
                    label: strategy as string,
                    onClick: async () => {
                        await sudokuController.tryStrategy(strategy);
                    },
                })),
                {
                    label: "Debug Iterative",
                    onClick: async () => {
                        const strategies: DynamicStrategy[] = [
                            "SingleCandidate",
                            "HiddenSingles",
                            // "GroupReduction",
                            // "Backtracking",
                        ];
                        outer: while (true) {
                            for (const strategy of strategies) {
                                console.info("Trying strategy:", strategy);
                                if (await sudokuController.tryStrategy(strategy)) {
                                    console.info("Made progress with:", strategy);
                                    continue outer;
                                }
                            }
                            break;
                        }
                        console.info("All strategies failed to make progress");
                    },
                },
            ]}
        >
            {({ onMenuOpen }) => (
                <IconButton size="large" aria-label="Solver" onClick={onMenuOpen}>
                    <LightbulbIcon fontSize="large" />
                </IconButton>
            )}
        </CustomMenu>
    );
}
interface ShareMenuProps {
    sudokuController: WasmSudokuController;
}

function ShareMenu({ sudokuController }: ShareMenuProps) {
    return (
        <CustomMenu
            menuItems={[
                {
                    label: "SudokuWiki",
                    icon: <OpenInNewIcon />,
                    onClick: async () => {
                        const binaryCandidatesLine = await sudokuController.export("binaryCandidatesLine");
                        window.open(
                            // Template string, since URLSearchParams encodes the reserved character ",".
                            // sudokuwiki.org expects these characters to be unencoded.
                            `https://www.sudokuwiki.org/sudoku.htm?n=${binaryCandidatesLine}`,
                            "_blank",
                            "noopener"
                        );
                    },
                },
                {
                    label: "Clipboard",
                    icon: <ContentCopyIcon />,
                    onClick: async () => {
                        const givensGrid = await sudokuController.export("givensGrid");
                        await window.navigator.clipboard.writeText(givensGrid);
                    },
                },
            ]}
        >
            {({ onMenuOpen }) => (
                <IconButton size="large" aria-label="Share" onClick={onMenuOpen}>
                    <ShareIcon fontSize="large" />
                </IconButton>
            )}
        </CustomMenu>
    );
}

interface SudokuAppBarProps {
    sudokuController: WasmSudokuController;
}

export default function SudokuAppBar({ sudokuController }: SudokuAppBarProps) {
    return (
        <Box sx={{ flexGrow: 1 }} className="app-bar">
            <AppBar position="static">
                <Toolbar>
                    <Typography variant="h6" component="div" sx={{ flexGrow: 1 }}>
                        Sudoku
                    </Typography>

                    <ShareMenu sudokuController={sudokuController} />
                    <SolverMenu sudokuController={sudokuController} />
                    <NewGameButton sudokuController={sudokuController} />
                </Toolbar>
            </AppBar>
        </Box>
    );
}
