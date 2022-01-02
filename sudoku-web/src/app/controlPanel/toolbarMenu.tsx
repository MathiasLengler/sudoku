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
                        // TODO: switch to candidates format:
                        //  Givens format: https://www.sudokuwiki.org/sudoku.htm?bd=002105090140000670080002451063070010900000043010090520007200080026000035000409060
                        //  Candidates format: https://www.sudokuwiki.org/sudoku.htm?bd=,102,68,2,1,172,16,134,256,130,1,8,274,388,132,132,32,64,130,116,128,272,356,36,2,8,16,1,154,32,4,144,64,136,384,1,392,256,80,154,176,187,169,192,8,4,200,1,136,164,256,172,16,2,232,28,276,64,2,53,37,257,128,265,136,2,32,192,129,193,321,4,16,148,20,145,8,149,256,67,32,67
                        //   Candidates encoded in base 2, serialized in base 10.
                        const givensLine = await sudokuController.export("givensLine");
                        const url = new URL("https://www.sudokuwiki.org/sudoku.htm");
                        url.searchParams.set("bd", givensLine);
                        window.open(url.toString(), "_blank", "noopener");
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
