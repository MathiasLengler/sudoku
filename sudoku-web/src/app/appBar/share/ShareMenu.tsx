import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import OpenInNewIcon from "@mui/icons-material/OpenInNew";
import ShareIcon from "@mui/icons-material/Share";
import { useState } from "react";

import { useExportSudokuString } from "../../actions/sudokuActions";
import { MyDialog } from "../../components/MyDialog";
import MyIconButton from "../../components/MyIconButton";
import { MyMenu } from "../../components/MyMenu";
import { ExportToClipboardDialog } from "./ExportToClipboardDialog";

export function ShareMenu() {
    const exportSudokuString = useExportSudokuString();
    const [isExportToClipboardDialogOpen, setIsExportToClipboardDialogOpen] = useState(false);

    return (
        <>
            <MyMenu
                menuItems={[
                    {
                        label: "SudokuWiki",
                        icon: <OpenInNewIcon />,
                        onClick: async () => {
                            const bd = await exportSudokuString("BinaryCandidatesLineV2");
                            const url = new URL("https://www.sudokuwiki.org/sudoku.htm");
                            url.searchParams.set("bd", bd);
                            window.open(url.toString(), "_blank", "noopener");
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
                    <MyIconButton label="Share" icon={ShareIcon} color="inherit" size="large" onClick={onMenuOpen} />
                )}
            </MyMenu>
            <MyDialog
                open={isExportToClipboardDialogOpen}
                onClose={() => {
                    setIsExportToClipboardDialogOpen(false);
                }}
            >
                {(onClose) => <ExportToClipboardDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
