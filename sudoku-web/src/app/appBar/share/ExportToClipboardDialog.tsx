import { Box, Button, DialogActions, DialogContent, DialogTitle, LinearProgress } from "@mui/material";
import React, { Suspense, useEffect } from "react";
import { SelectElement, useForm } from "react-hook-form-mui";
import { ALL_GRID_FORMATS } from "../../../constants";
import CircularProgress from "@mui/material/CircularProgress";
import { Code } from "../../components/Code";
import { type Loadable, useRecoilState, useRecoilValueLoadable } from "recoil";
import { zodResolver } from "@hookform/resolvers/zod";
import {
    exportedGridStringState,
    type ExportToClipboardFormValues,
    exportToClipboardFormValuesSchema,
    exportToClipboardFormValuesState,
} from "../../state/forms/exportToClipboard";

interface DisplayExportedGridStringProps {
    gridFormat: ExportToClipboardFormValues["gridFormat"];
    exportedGridStringLoadable: Loadable<string>;
}

function DisplayExportedGridString({ gridFormat, exportedGridStringLoadable }: DisplayExportedGridStringProps) {
    return (
        <Code wrap={["givensLine", "binaryCandidatesLine", "binaryFixedCandidatesLine"].includes(gridFormat)}>
            {exportedGridStringLoadable.getValue()}
        </Code>
    );
}

interface ExportToClipboardDialogProps {
    onClose: () => void;
}

export function ExportToClipboardDialog({ onClose }: ExportToClipboardDialogProps) {
    const [exportToClipboardFormValues, setExportToClipboardFormValues] = useRecoilState(
        exportToClipboardFormValuesState
    );
    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
    } = useForm<ExportToClipboardFormValues>({
        values: exportToClipboardFormValues,
        resolver: zodResolver(exportToClipboardFormValuesSchema),
    });
    const gridFormat = watch("gridFormat");

    const exportedGridStringLoadable = useRecoilValueLoadable(exportedGridStringState);

    // Always update selected gridFormat
    useEffect(() => {
        setExportToClipboardFormValues({ gridFormat });
    }, [gridFormat, setExportToClipboardFormValues]);

    return (
        <>
            <DialogTitle>Export Sudoku to Clipboard</DialogTitle>
            <DialogContent dividers>
                <form
                    id="export-to-clipboard-form"
                    noValidate
                    onSubmit={handleSubmit(async () => {
                        const exportedGridString = await exportedGridStringLoadable.toPromise();
                        // const { gridFormat } = formData;
                        // const exportedGridString = await exportSudokuString(gridFormat);
                        await window.navigator.clipboard.writeText(exportedGridString);

                        onClose();
                    })}
                    style={{ display: "sticky" }}
                >
                    <SelectElement
                        control={control}
                        name="gridFormat"
                        label="Format"
                        fullWidth
                        options={ALL_GRID_FORMATS.map(gridFormat => ({
                            id: gridFormat,
                            label: gridFormat,
                        }))}
                    />
                </form>
                <Box sx={{ pt: 2 }}>
                    <Suspense fallback={<LinearProgress variant="indeterminate" />}>
                        <DisplayExportedGridString
                            gridFormat={gridFormat}
                            exportedGridStringLoadable={exportedGridStringLoadable}
                        />
                    </Suspense>
                </Box>
            </DialogContent>
            <DialogActions>
                {isSubmitting && (
                    <Box>
                        <CircularProgress />
                    </Box>
                )}
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="export-to-clipboard-form"
                    color="primary"
                    variant="contained"
                    disabled={isSubmitting}
                >
                    Copy to Clipboard
                </Button>
            </DialogActions>
        </>
        // </form>
    );
}
