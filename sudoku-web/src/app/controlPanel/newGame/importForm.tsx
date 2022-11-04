import type { WasmSudokuController } from "../../wasmSudokuController";
import React, { useState } from "react";
import Box from "@mui/material/Box";
import DialogActions from "@mui/material/DialogActions";
import Button from "@mui/material/Button";
import TextField from "@mui/material/TextField";
import { Typography } from "@mui/material";
import Accordion from "@mui/material/Accordion";
import AccordionSummary from "@mui/material/AccordionSummary";
import AccordionDetails from "@mui/material/AccordionDetails";
import ExpandMoreIcon from "@mui/icons-material/ExpandMore";
import CircularProgress from "@mui/material/CircularProgress";

interface CodeProps {
    text: string;
}

const Code: React.FunctionComponent<CodeProps> = ({ text }) => {
    return (
        <div>
            {text.split("\n").map((line, index) => (
                <code key={index} style={{ whiteSpace: "pre" }}>
                    {line}
                    <br key={index} />
                </code>
            ))}
        </div>
    );
};

interface ImportFormProps {
    sudokuController: WasmSudokuController;
    onClose: () => void;
}

export const ImportForm: React.FunctionComponent<ImportFormProps> = props => {
    const { sudokuController, onClose } = props;

    const [loading, setLoading] = useState(false);
    const [input, setInput] = useState("");
    const [inputError, setInputError] = useState(false);

    const supportedFormatsPanel = (
        <Accordion>
            <AccordionSummary expandIcon={<ExpandMoreIcon />}>
                <Typography>Supported formats</Typography>
            </AccordionSummary>
            <AccordionDetails
                sx={{
                    overflowX: "scroll",
                }}
            >
                <Typography variant="h6">List of givens</Typography>
                <Code text="6....23..1256.......47...2.73....84...........46....15.5...81.......3472..72....8" />
                <Typography variant="h6">Grid of givens</Typography>
                <Code
                    text={`*-----------*
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
*-----------*`}
                />
                <Typography variant="h6">Grid of candidates</Typography>
                <Code
                    text={`.--------------.----------------.------------.
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
                />
                <Typography variant="h6">Empty cells can be expressed as</Typography>
                <Code text=". 0" />
            </AccordionDetails>
        </Accordion>
    );

    return (
        <>
            <Box p={3}>
                <TextField
                    label="Formatted Sudoku"
                    multiline
                    fullWidth
                    error={inputError}
                    margin="dense"
                    value={input}
                    onChange={e => setInput(e.target.value)}
                    sx={{ fontFamily: "Monospace" }}
                    disabled={loading}
                />
                {supportedFormatsPanel}
            </Box>
            <DialogActions>
                {loading && <CircularProgress />}
                <Button onClick={onClose} disabled={loading}>
                    Cancel
                </Button>
                <Button
                    color="primary"
                    disabled={loading}
                    onClick={async () => {
                        setLoading(true);

                        try {
                            await sudokuController.import(input);
                            onClose();
                        } catch (e) {
                            console.error("Unable to parse input sudoku string:", input, e);
                            setInputError(true);
                        } finally {
                            setLoading(false);
                        }
                    }}
                >
                    Import
                </Button>
            </DialogActions>
        </>
    );
};
