import { zodResolver } from "@hookform/resolvers/zod";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import SaveAltIcon from "@mui/icons-material/SaveAlt";
import TabPanel from "@mui/lab/TabPanel";
import { DialogContent, Typography } from "@mui/material";
import { Stack } from "@mui/material";

import Accordion from "@mui/material/Accordion";
import AccordionDetails from "@mui/material/AccordionDetails";
import AccordionSummary from "@mui/material/AccordionSummary";
import Button from "@mui/material/Button";
import DialogActions from "@mui/material/DialogActions";
import { SwitchElement, TextFieldElement, useForm } from "react-hook-form-mui";
import { z } from "zod";
import { useImportSudokuString } from "../../actions/sudokuActions";
import { Code } from "../../components/Code";
import { Fieldset } from "../../components/Fieldset";
import { ResetFormButton } from "../../components/ResetFormButton";
import type { NewGameTabValue } from "./NewGameDialog";

function SupportedFormats() {
    return (
        <Accordion>
            <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                <Typography>Supported formats</Typography>
            </AccordionSummary>
            <AccordionDetails>
                <Typography variant="h6">List of givens</Typography>
                <Code>{"6....23..1256.......47...2.73....84...........46....15.5...81.......3472..72....8"}</Code>
                <Typography variant="h6">Grid of givens</Typography>
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
                <Typography variant="h6">Grid of candidates</Typography>
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
                <Typography variant="h6">Empty cells can be expressed as</Typography>
                <Code>{". 0"}</Code>
            </AccordionDetails>
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

export const ImportForm = ({ onClose }: ImportFormProps) => {
    const importSudokuString = useImportSudokuString();

    const {
        control,
        handleSubmit,
        formState: { isSubmitting },
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
            <DialogContent>
                <TabPanel value={"import-form" satisfies NewGameTabValue} sx={{ p: 0 }}>
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
                        <Stack spacing={2}>
                            <TextFieldElement
                                control={control}
                                name="input"
                                label="Formatted Sudoku"
                                multiline
                                fullWidth
                                InputProps={{ sx: { fontFamily: "monospace" } }}
                                disabled={isSubmitting}
                            />
                            <SupportedFormats />
                            <Fieldset label="Post import">
                                <SwitchElement
                                    control={control}
                                    name="setAllDirectCandidates"
                                    label="Fill candidates"
                                />
                            </Fieldset>
                        </Stack>
                    </form>
                </TabPanel>
            </DialogContent>
            <DialogActions>
                <ResetFormButton disabled={isSubmitting} onClick={reset} />
                <Button onClick={onClose} disabled={isSubmitting}>
                    Cancel
                </Button>
                <Button
                    type="submit"
                    form="import-form"
                    color="primary"
                    variant="contained"
                    endIcon={<SaveAltIcon />}
                    loading={isSubmitting}
                    loadingPosition="end"
                >
                    <span>Import</span>
                </Button>
            </DialogActions>
        </>
    );
};
