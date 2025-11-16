import { zodResolver } from "@hookform/resolvers/zod";
import ContentCopyIcon from "@mui/icons-material/ContentCopy";
import { Button, DialogActions, DialogContent, DialogTitle, LinearProgress, Stack } from "@mui/material";

import { useAtom, useAtomValue } from "jotai";
import { useAtomCallback } from "jotai/utils";
import { Suspense, useCallback, useEffect } from "react";
import { SelectElement, useForm } from "react-hook-form-mui";
import { Code } from "../../components/Code";
import { ResetFormButton } from "../../components/ResetFormButton";
import { ALL_GRID_FORMATS } from "../../constants";
import {
    exportToClipboardFormValuesSchema,
    exportToClipboardFormValuesState,
    exportedGridStringState,
    type ExportToClipboardFormValues,
} from "../../state/forms/exportToClipboard";

type DisplayExportedGridStringProps = {
    gridFormat: ExportToClipboardFormValues["gridFormat"];
};

function DisplayExportedGridString({ gridFormat }: DisplayExportedGridStringProps) {
    const exportedGridStringLoadable = useAtomValue(exportedGridStringState);

    return (
        <Code wrap={["givensLine", "binaryCandidatesLine", "binaryFixedCandidatesLine"].includes(gridFormat)}>
            {exportedGridStringLoadable}
        </Code>
    );
}

type ExportToClipboardDialogProps = {
    onClose: () => void;
};

export function ExportToClipboardDialog({ onClose }: ExportToClipboardDialogProps) {
    const [exportToClipboardFormValues, setExportToClipboardFormValues] = useAtom(exportToClipboardFormValuesState);
    const {
        control,
        handleSubmit,
        watch,
        formState: { isSubmitting },
        reset,
    } = useForm<ExportToClipboardFormValues>({
        values: exportToClipboardFormValues,
        resolver: zodResolver(exportToClipboardFormValuesSchema),
    });
    const gridFormat = watch("gridFormat");

    // Always update selected gridFormat
    useEffect(() => {
        setExportToClipboardFormValues({ gridFormat });
    }, [gridFormat, setExportToClipboardFormValues]);

    const onSubmit = useAtomCallback(
        useCallback(
            async (get) => {
                const exportedGridString = await get(exportedGridStringState);
                await window.navigator.clipboard.writeText(exportedGridString);
                onClose();
            },
            [onClose],
        ),
    );

    return (
        <>
            <DialogTitle>Export Sudoku to Clipboard</DialogTitle>
            <DialogContent>
                <form
                    id="export-to-clipboard-form"
                    noValidate
                    onSubmit={handleSubmit(onSubmit)}
                    style={{ display: "sticky" }}
                >
                    <Stack spacing={2}>
                        <SelectElement
                            control={control}
                            name="gridFormat"
                            label="Format"
                            fullWidth
                            options={ALL_GRID_FORMATS.map((gridFormat) => ({
                                id: gridFormat,
                                label: gridFormat,
                            }))}
                        />
                        <Suspense fallback={<LinearProgress variant="indeterminate" />}>
                            <DisplayExportedGridString gridFormat={gridFormat} />
                        </Suspense>
                    </Stack>
                </form>
            </DialogContent>
            <DialogActions sx={{ justifyContent: "space-between" }}>
                <ResetFormButton disabled={isSubmitting} onClick={() => reset()} />
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="export-to-clipboard-form"
                    color="primary"
                    variant="contained"
                    endIcon={<ContentCopyIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    Copy to Clipboard
                </Button>
            </DialogActions>
        </>
    );
}
