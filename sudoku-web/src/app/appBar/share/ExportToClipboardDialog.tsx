import { Button, DialogActions, DialogContent, DialogTitle, LinearProgress, Stack } from "@mui/material";
import React, { Suspense, useEffect } from "react";
import { SelectElement, useForm } from "react-hook-form-mui";
import { ALL_GRID_FORMATS } from "../../../constants";
import { Code } from "../../components/Code";
import { type Loadable, useRecoilState, useRecoilValueLoadable } from "recoil";
import { zodResolver } from "@hookform/resolvers/zod";
import {
    EXPORT_TO_CLIPBOARD_FORM_DEFAULT_VALUES,
    exportedGridStringState,
    type ExportToClipboardFormValues,
    exportToClipboardFormValuesSchema,
    exportToClipboardFormValuesState,
} from "../../state/forms/exportToClipboard";
import { LoadingButton } from "@mui/lab";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import { ResetFormButton } from "../../components/ResetFormButton";

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
        reset,
    } = useForm<ExportToClipboardFormValues>({
        values: exportToClipboardFormValues,
        defaultValues: EXPORT_TO_CLIPBOARD_FORM_DEFAULT_VALUES,
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
            <DialogContent>
                <form
                    id="export-to-clipboard-form"
                    noValidate
                    onSubmit={handleSubmit(async () => {
                        const exportedGridString = await exportedGridStringLoadable.toPromise();
                        await window.navigator.clipboard.writeText(exportedGridString);
                        onClose();
                    })}
                    style={{ display: "sticky" }}
                >
                    <Stack spacing={2}>
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
                        <Suspense fallback={<LinearProgress variant="indeterminate" />}>
                            <DisplayExportedGridString
                                gridFormat={gridFormat}
                                exportedGridStringLoadable={exportedGridStringLoadable}
                            />
                        </Suspense>
                    </Stack>
                </form>
            </DialogContent>
            <DialogActions sx={{ justifyContent: "space-between" }}>
                <ResetFormButton disabled={isSubmitting} onClick={() => reset()} />
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <LoadingButton
                    type="submit"
                    form="export-to-clipboard-form"
                    color="primary"
                    variant="contained"
                    endIcon={<ContentCopyIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    Copy to Clipboard
                </LoadingButton>
            </DialogActions>
        </>
        // </form>
    );
}
