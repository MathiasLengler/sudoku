import { useExportSudokuString } from "../../sudokuActions";
import { MyMenu } from "../../components/MyMenu";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import ShareIcon from "@mui/icons-material/Share";
import React from "react";
import { MyDialog } from "../../components/MyDialog";
import { ExportToClipboardDialog } from "./ExportToClipboardDialog";
import MyIconButton from "../../components/MyIconButton";

export function ShareMenu() {
    const exportSudokuString = useExportSudokuString();
    const [isExportToClipboardDialogOpen, setIsExportToClipboardDialogOpen] = React.useState(false);

    return (
        <>
            <MyMenu
                menuItems={[
                    {
                        label: "SudokuWiki",
                        icon: <OpenInNewIcon />,
                        onClick: async () => {
                            const binaryFixedCandidatesLine = await exportSudokuString("BinaryFixedCandidatesLine");
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
                        },
                    },
                ]}
            >
                {({ onMenuOpen }) => (
                    <MyIconButton tooltip="Share" icon={ShareIcon} color="inherit" size="large" onClick={onMenuOpen} />
                )}
            </MyMenu>
            <MyDialog open={isExportToClipboardDialogOpen} onClose={() => setIsExportToClipboardDialogOpen(false)}>
                {onClose => <ExportToClipboardDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
