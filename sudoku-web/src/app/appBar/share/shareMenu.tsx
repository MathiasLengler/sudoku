import { useExportSudokuString } from "../../sudokuActions";
import { CustomMenu } from "../../components/CustomMenu";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import { IconButton } from "@mui/material";
import ShareIcon from "@mui/icons-material/Share";
import React from "react";
import { MyDialog } from "../../components/MyDialog";
import { ExportToClipboardDialog } from "./exportToClipboardDialog";

export function ShareMenu() {
    const exportSudokuString = useExportSudokuString();
    const [isExportToClipboardDialogOpen, setIsExportToClipboardDialogOpen] = React.useState(false);

    return (
        <>
            <CustomMenu
                menuItems={[
                    {
                        label: "SudokuWiki",
                        icon: <OpenInNewIcon />,
                        onClick: async () => {
                            const binaryFixedCandidatesLine = await exportSudokuString("binaryFixedCandidatesLine");
                            window.open(
                                // Template string, since URLSearchParams encodes the reserved character ",".
                                // sudokuwiki.org expects these characters to be unencoded.
                                `https://www.sudokuwiki.org/sudoku.htm?bd=${binaryFixedCandidatesLine}`,
                                "_blank",
                                "noopener"
                            );
                        },
                    },
                    {
                        label: "Clipboard",
                        icon: <ContentCopyIcon />,
                        onClick: () => {
                            setIsExportToClipboardDialogOpen(true);
                            // const givensGrid = await exportSudokuString("givensGrid");
                            // await window.navigator.clipboard.writeText(givensGrid);
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
            <MyDialog open={isExportToClipboardDialogOpen} onClose={() => setIsExportToClipboardDialogOpen(false)}>
                {onClose => <ExportToClipboardDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
