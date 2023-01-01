import { useExportSudokuString } from "../sudokuActions";
import { CustomMenu } from "./customMenu";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import { IconButton } from "@mui/material";
import ShareIcon from "@mui/icons-material/Share";
import React from "react";

export function ShareMenu() {
    const exportSudokuString = useExportSudokuString();

    return (
        <CustomMenu
            menuItems={[
                {
                    label: "SudokuWiki",
                    icon: <OpenInNewIcon />,
                    onClick: async () => {
                        const binaryCandidatesLine = await exportSudokuString("binaryCandidatesLine");
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
                        const givensGrid = await exportSudokuString("givensGrid");
                        await window.navigator.clipboard.writeText(givensGrid);
                    },
                },
            ]}
        >
            {({ onMenuOpen }) => (
                <IconButton color="inherit" size="large" aria-label="Share" onClick={onMenuOpen}>
                    <ShareIcon fontSize="large" />
                </IconButton>
            )}
        </CustomMenu>
    );
}
