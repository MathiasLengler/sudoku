import { zodResolver } from "@hookform/resolvers/zod";
import { IconChevronDown, IconDownload } from "@tabler/icons-react";
import { Accordion, Button, Group, Stack, Switch, Text, Textarea } from "@mantine/core";
import { Controller, useForm } from "react-hook-form";
import * as z from "zod";
import { useImportSudokuString } from "../../actions/sudokuActions";
import { Code } from "../../components/Code";
import { Fieldset } from "../../components/Fieldset";
import { ResetFormButton } from "../../components/ResetFormButton";

function SupportedFormats() {
    return (
        <Accordion chevron={<IconChevronDown size={16} />}>
            <Accordion.Item value="formats">
                <Accordion.Control>
                    <Text>Supported formats</Text>
                </Accordion.Control>
                <Accordion.Panel>
                    <Text size="lg" fw={500}>List of givens</Text>
                    <Code>{"6....23..1256.......47...2.73....84...........46....15.5...81.......3472..72....8"}</Code>
                    <Text size="lg" fw={500} mt="md">Grid of givens</Text>
                    <Code>{`*-----------*
|.8.|5.3|.7.|
|.27|...|38.|
|...|...|...|
|---+---+---|
|..5|.9.|6..|
|...|1.2|...|
|..4|.6.|9..|
|---+---+---|
|...|...|...|
|.32|...|45.|
|.5.|9.7|.2.|
*-----------*`}</Code>
                    <Text size="lg" fw={500} mt="md">Grid of candidates</Text>
                    <Code>
                        {`.--------------.----------------.------------.
| 6   7    89  | 189  19   2    | 3   5   4  |
| 1   2    5   | .    3    4    | 9   8   7  |
| 3   89   4   | 7    58   59   | 6   2   1  |
:--------------+----------------+------------:
| 7   3    29  | 19   25   1569 | 8   4   69 |
| 5   1    289 | 89   0    679  | 27  69  3  |
| 89  4    6   | 3    28   79   | 27  1   5  |
:--------------+----------------+------------:
| 2   5    3   | 4    7    8    | 0   69  69 |
| 89  689  1   | 5    69   3    | 4   0   2  |
| 4   69   7   | 2    169  169  | 5   3   8  |
'--------------'----------------'------------'`}
                    </Code>
                    <Text size="lg" fw={500} mt="md">Empty cells can be expressed as</Text>
                    <Code>{". 0"}</Code>
                </Accordion.Panel>
            </Accordion.Item>
        </Accordion>
    );
}

type ImportFormValues = z.infer<typeof importFormValuesSchema>;
const importFormValuesSchema = z.object({
    input: z.string().min(1),
    setAllDirectCandidates: z.boolean(),
});

type ImportFormProps = {
    onClose: () => void;
};

export function ImportForm({ onClose }: ImportFormProps) {
    const importSudokuString = useImportSudokuString();

    const {
        control,
        handleSubmit,
        formState: { isSubmitting, errors },
        reset,
        setError,
    } = useForm<ImportFormValues>({
        defaultValues: {
            input: "",
            setAllDirectCandidates: false,
        },
        resolver: zodResolver(importFormValuesSchema),
    });

    return (
        <>
            <form
                id="import-form"
                onSubmit={handleSubmit(async ({ input, setAllDirectCandidates }) => {
                    try {
                        await importSudokuString(input, setAllDirectCandidates);
                        onClose();
                    } catch (err) {
                        console.error("Unable to parse input sudoku string:", input, err);
                        if (err instanceof Error) {
                            setError("input", { type: "custom", message: err.message });
                        }
                    }
                })}
            >
                <Stack gap="md">
                    <Controller
                        name="input"
                        control={control}
                        render={({ field }) => (
                            <Textarea
                                {...field}
                                label="Formatted Sudoku"
                                autosize
                                minRows={4}
                                error={errors.input?.message}
                                styles={{ input: { fontFamily: "monospace" } }}
                                readOnly={isSubmitting}
                            />
                        )}
                    />
                    <SupportedFormats />
                    <Fieldset label="Post import">
                        <Controller
                            name="setAllDirectCandidates"
                            control={control}
                            render={({ field: { value, onChange, ...field } }) => (
                                <Switch
                                    {...field}
                                    checked={value}
                                    onChange={(e) => onChange(e.currentTarget.checked)}
                                    label="Fill candidates"
                                />
                            )}
                        />
                    </Fieldset>
                </Stack>
            </form>
            <Group justify="space-between" mt="md">
                <ResetFormButton disabled={isSubmitting} onClick={reset} />
                <Button onClick={onClose} disabled={isSubmitting} variant="subtle">
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="import-form"
                    rightSection={<IconDownload size={18} />}
                    loading={isSubmitting}
                >
                    Import
                </Button>
            </Group>
        </>
    );
}
