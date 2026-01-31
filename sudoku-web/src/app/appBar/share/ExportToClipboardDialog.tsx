import { zodResolver } from "@hookform/resolvers/zod";
import { IconCopy } from "@tabler/icons-react";
import { Button, Group, Loader, Select, Stack, Text } from "@mantine/core";

import { useAtom, useAtomValue } from "jotai";
import { useAtomCallback } from "jotai/utils";
import { Suspense, useCallback, useEffect } from "react";
import { Controller, useForm } from "react-hook-form";
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
            <Text size="lg" fw={500} mb="md">
                Export Sudoku to Clipboard
            </Text>
            <form id="export-to-clipboard-form" noValidate onSubmit={handleSubmit(onSubmit)}>
                <Stack gap="md">
                    <Controller
                        name="gridFormat"
                        control={control}
                        render={({ field }) => (
                            <Select
                                {...field}
                                label="Format"
                                data={ALL_GRID_FORMATS.map((format) => ({ value: format, label: format }))}
                            />
                        )}
                    />
                    <Suspense fallback={<Loader size="sm" />}>
                        <DisplayExportedGridString gridFormat={gridFormat} />
                    </Suspense>
                </Stack>
            </form>
            <Group justify="space-between" mt="md">
                <ResetFormButton disabled={isSubmitting} onClick={() => reset()} />
                <Button onClick={onClose} disabled={isSubmitting} variant="subtle">
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="export-to-clipboard-form"
                    rightSection={<IconCopy size={18} />}
                    loading={isSubmitting}
                >
                    Copy to Clipboard
                </Button>
            </Group>
        </>
    );
}
