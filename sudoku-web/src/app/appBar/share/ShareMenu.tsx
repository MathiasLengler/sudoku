import { useExportSudokuString } from "../../actions/sudokuActions";
import { MyMenu } from "../../components/MyMenu";
import { IconExternalLink, IconCopy, IconShare } from "@tabler/icons-react";

import { MyDialog } from "../../components/MyDialog";
import { ExportToClipboardDialog } from "./ExportToClipboardDialog";
import MyIconButton from "../../components/MyIconButton";
import { useState } from "react";

export function ShareMenu() {
    const exportSudokuString = useExportSudokuString();
    const [isExportToClipboardDialogOpen, setIsExportToClipboardDialogOpen] = useState(false);

    return (
        <>
            <MyMenu
                menuItems={[
                    {
                        label: "SudokuWiki",
                        icon: <IconExternalLink size={18} />,
                        onClick: async () => {
                            const bd = await exportSudokuString("BinaryCandidatesLineV2");
                            const url = new URL("https://www.sudokuwiki.org/sudoku.htm");
                            url.searchParams.set("bd", bd);
                            window.open(url.toString(), "_blank", "noopener");
                        },
                    },
                    {
                        label: "Clipboard",
                        icon: <IconCopy size={18} />,
                        onClick: () => {
                            setIsExportToClipboardDialogOpen(true);
                        },
                    },
                ]}
            >
                {({ onMenuOpen }) => (
                    <MyIconButton label="Share" icon={IconShare} color="inherit" size="lg" onClick={onMenuOpen} />
                )}
            </MyMenu>
            <MyDialog open={isExportToClipboardDialogOpen} onClose={() => setIsExportToClipboardDialogOpen(false)}>
                {(onClose) => <ExportToClipboardDialog onClose={onClose} />}
            </MyDialog>
        </>
    );
}
