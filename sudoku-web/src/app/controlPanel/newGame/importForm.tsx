import {WasmSudokuController} from "../../wasmSudokuController";
import React, {useState} from "react";
import Box from "@material-ui/core/Box";
import DialogActions from "@material-ui/core/DialogActions";
import Button from "@material-ui/core/Button";
import TextField from "@material-ui/core/TextField";
import {Typography} from "@material-ui/core";
import makeStyles from "@material-ui/core/styles/makeStyles";
import ExpansionPanel from "@material-ui/core/ExpansionPanel";
import ExpansionPanelSummary from "@material-ui/core/ExpansionPanelSummary";
import ExpansionPanelDetails from "@material-ui/core/ExpansionPanelDetails";
import ExpandMoreIcon from '@material-ui/icons/ExpandMore';

interface CodeProps {
  text: string;
}

const Code: React.FunctionComponent<CodeProps> = ({text}) => {
  return <div>
    {text
      .split('\n')
      .map((line, index) =>
        <code key={index} style={{whiteSpace: "pre"}}>{line}<br key={index}/></code>
      )}
  </div>;
};

const supportedFormats = <>
  <Typography variant="h6">
    List of givens
  </Typography>
  <Code text="6....23..1256.......47...2.73....84...........46....15.5...81.......3472..72....8"/>
  <Typography variant="h6">
    Grid of givens
  </Typography>
  <Code text={`*-----------*
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
*-----------*`}/>
  <Typography variant="h6">
    Grid of candidates
  </Typography>
  <Code text={`.--------------.----------------.------------.
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
'--------------'----------------'------------'`}/>
  <Typography variant="h6">
    Empty cells can be expressed as
  </Typography>
  <Code text=". 0"/>
</>;


const useStyles = makeStyles({
  input: {
    fontFamily: "Monospace"
  },
  root: {
    flexFlow: "column",
    overflowX: "scroll"
  }
});

interface ImportFormProps {
  sudokuController: WasmSudokuController;
  onClose: () => void;
}

export const ImportForm: React.FunctionComponent<ImportFormProps> = (props) => {
  const {sudokuController, onClose} = props;

  const [input, setInput] = useState("");
  const [inputError, setInputError] = useState(false);

  const classes = useStyles();

  const supportedFormatsPanel = <ExpansionPanel>
    <ExpansionPanelSummary
      expandIcon={<ExpandMoreIcon/>}
    >
      <Typography>Supported formats</Typography>
    </ExpansionPanelSummary>
    <ExpansionPanelDetails classes={{root: classes.root}}>
      {supportedFormats}
    </ExpansionPanelDetails>
  </ExpansionPanel>;

  return <>
    <Box p={3}>
      <TextField
        label="Formatted Sudoku"
        multiline
        fullWidth
        error={inputError}
        margin="dense"
        value={input}
        onChange={e => setInput(e.target.value)}
        InputProps={{
          classes: {input: classes.input}
        }}
      />
      {supportedFormatsPanel}
    </Box>
    <DialogActions>
      <Button onClick={onClose}>
        Cancel
      </Button>
      <Button onClick={() => {
        try {
          sudokuController.import(input);
          onClose();
        } catch (e) {
          console.error("Unable to parse input sudoku string:", input, e);
          setInputError(true);
        }
      }} color="primary">
        Import
      </Button>
    </DialogActions>
  </>;
};
