import * as React from "react";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import { NewGameDialog } from "./newGame/newGameDialog";
import IconButton from "@mui/material/IconButton";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import Tooltip from "@mui/material/Tooltip";
import type { WasmSudokuController } from "../wasmSudokuController";
import type { DynamicStrategy } from "../../types";

interface ToolbarMenuProps {
    enterDelay: number;
    leaveDelay: number;
    sudokuController: WasmSudokuController;
}

export const ToolbarMenu: React.FunctionComponent<ToolbarMenuProps> = props => {
    const { enterDelay, leaveDelay, sudokuController } = props;

    const [menuAnchorEl, setMenuAnchorEl] = React.useState<null | HTMLElement>(null);

    const [newGameOpen, setNewGameOpen] = React.useState(false);

    const makeHandleMenuClose = (action?: () => Promise<void>) => async () => {
        setMenuAnchorEl(null);
        if (action) {
            try {
                await action();
            } catch (err) {
                console.error("Error while executing menu action:", err);
            }
        }
    };

    // TODO: refactor menu into AppBar
    return (
        <>
            <Tooltip title="Menu" enterDelay={enterDelay} leaveDelay={leaveDelay}>
                <IconButton onClick={e => setMenuAnchorEl(e.currentTarget)} size="large">
                    <MoreVertIcon fontSize="large" />
                </IconButton>
            </Tooltip>
            <Menu open={!!menuAnchorEl} anchorEl={menuAnchorEl} keepMounted onClose={makeHandleMenuClose()}>
                <MenuItem onClick={makeHandleMenuClose(async () => setNewGameOpen(true))}>New Game</MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        await sudokuController.tryStrategy("SingleCandidate");
                    })}
                >
                    Solver: SingleCandidate
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        await sudokuController.tryStrategy("HiddenSingles");
                    })}
                >
                    Solver: HiddenSingles
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        await sudokuController.tryStrategy("GroupReduction");
                    })}
                >
                    Solver: GroupReduction
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        await sudokuController.tryStrategy("Backtracking");
                    })}
                >
                    Solver: Backtracking
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
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
                    })}
                >
                    Solver: debug
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        const binaryCandidatesLine = await sudokuController.export("binaryCandidatesLine");
                        window.open(
                            // Template string, since URLSearchParams encodes the reserved character ",".
                            // sudokuwiki.org expects these characters to be unencoded.
                            `https://www.sudokuwiki.org/sudoku.htm?n=${binaryCandidatesLine}`,
                            "_blank",
                            "noopener"
                        );
                    })}
                >
                    Export to SudokuWiki
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        const givensGrid = await sudokuController.export("givensGrid");
                        await window.navigator.clipboard.writeText(givensGrid);
                    })}
                >
                    Export to Clipboard
                </MenuItem>
            </Menu>
            <div
                id="dialogs"
                tabIndex={0}
                onKeyDown={e => {
                    // Disable global game shortcuts in dialog boxes.
                    e.stopPropagation();
                }}
            >
                <NewGameDialog
                    open={newGameOpen}
                    onClose={() => setNewGameOpen(false)}
                    sudokuController={sudokuController}
                />
            </div>
        </>
    );
};
