import * as React from "react";
import Menu from "@mui/material/Menu";
import MenuItem from "@mui/material/MenuItem";
import { NewGameDialog } from "./newGame/newGameDialog";
import IconButton from "@mui/material/IconButton";
import MoreVertIcon from "@mui/icons-material/MoreVert";
import Tooltip from "@mui/material/Tooltip";
import { WasmSudokuController } from "../wasmSudokuController";

interface ToolbarMenuProps {
    enterDelay: number;
    leaveDelay: number;
    sudokuController: WasmSudokuController;
}

export const ToolbarMenu: React.FunctionComponent<ToolbarMenuProps> = props => {
    const { enterDelay, leaveDelay, sudokuController } = props;

    const [menuAnchorEl, setMenuAnchorEl] = React.useState<null | HTMLElement>(null);

    const [newGameOpen, setNewGameOpen] = React.useState(false);

    const makeHandleMenuClose = (action?: () => void) => () => {
        setMenuAnchorEl(null);
        action?.();
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
                <MenuItem onClick={makeHandleMenuClose(() => setNewGameOpen(true))}>New Game</MenuItem>
                <MenuItem onClick={makeHandleMenuClose(() => sudokuController.solveSingleCandidates())}>
                    Solver: single candidates
                </MenuItem>
                <MenuItem onClick={makeHandleMenuClose(() => sudokuController.groupReduction())}>
                    Solver: group reduction
                </MenuItem>
                <MenuItem
                    onClick={makeHandleMenuClose(async () => {
                        const binaryCandidatesLine = await sudokuController.export("binaryCandidatesLine");
                        window.open(
                            // Template string, since URLSearchParams encodes the reserved character ",".
                            // sudokuwiki.org expects these characters *not* to be encoded.
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
