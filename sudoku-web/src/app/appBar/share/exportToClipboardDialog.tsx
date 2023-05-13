import { Box, Button, DialogActions, DialogContent, DialogContentText, DialogTitle, FormGroup } from "@mui/material";
import React, { useEffect, useState } from "react";
import { useExportSudokuString } from "../../sudokuActions";
import type { DynamicGridFormat } from "../../../../../sudoku-rs/bindings";
import { SelectElement, useForm } from "react-hook-form-mui";
import { ALL_GRID_FORMATS } from "../../../constants";
import CircularProgress from "@mui/material/CircularProgress";
import { Code } from "../../components/Code";

type FormData = {
    gridFormat: DynamicGridFormat;
};

let previousFormData: FormData | undefined;

interface ExportToClipboardDialogProps {
    onClose: () => void;
}

export function ExportToClipboardDialog({ onClose }: ExportToClipboardDialogProps) {
    const exportSudokuString = useExportSudokuString();

    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
        setValue,
    } = useForm<FormData>({
        defaultValues: previousFormData ?? {
            gridFormat: "CandidatesGridPlain",
        },
    });

    const gridFormat = watch("gridFormat");

    const [exportedGridString, setExportedGridString] = useState("");

    useEffect(() => {
        exportSudokuString(gridFormat)
            .then(exportedGridString => {
                setExportedGridString(exportedGridString);
            })
            .catch(console.error);
    }, [exportSudokuString, gridFormat]);

    // Always store current values a previous form data
    useEffect(() => {
        previousFormData = { gridFormat };
    }, [gridFormat]);

    return (
        <>
            <DialogTitle>Export Sudoku to Clipboard</DialogTitle>

            <DialogContent dividers>
                <form
                    id="export-to-clipboard-form"
                    noValidate
                    onSubmit={handleSubmit(async formData => {
                        const { gridFormat } = formData;
                        const exportedGridString = await exportSudokuString(gridFormat);
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
                    <Code
                        wrap={["givensLine", "binaryCandidatesLine", "binaryFixedCandidatesLine"].includes(gridFormat)}
                    >
                        {exportedGridString}
                    </Code>
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
